mod array_response;
mod error;
mod opt;

use array_response::array_response;
use error::err_map;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use opt::patch::Patch;
use opt::{auth::Credentials, endpoint::Options};
use serde_json::to_value;
use serde_json::{from_value};
use serde_json::Value as JsValue;
use std::collections::{BTreeMap, VecDeque};
use std::time::Duration;
use surrealdb::dbs::{Capabilities, Session};
use surrealdb::engine::any::Any;
use surrealdb::kvs::Datastore;
use surrealdb::opt::auth::Database;
use surrealdb::opt::auth::Namespace;
use surrealdb::sql::Value as SqlValue;
use uuid::Uuid;
use surrealdb::opt::auth::Root;
use surrealdb::opt::auth::Scope;
use surrealdb::opt::Resource;
use surrealdb::opt::{Config, PatchOp};
use surrealdb::rpc::format::cbor;
use surrealdb::rpc::method::Method;
use surrealdb::rpc::{Data, RpcContext};
use surrealdb::sql::{self, Range, json};

#[napi]
pub struct SurrealNodeEngine(SurrealNodeEngineInner);

impl SurrealNodeEngine {
    pub async fn execute(&mut self, data: Uint8Array) -> std::result::Result<Uint8Array, Error> {
        let in_data = cbor::req(data.to_vec()).map_err(|e| e.to_string())?;
        let res = self
            .0
            .execute(Method::parse(in_data.method), in_data.params)
            .await
            .map_err(|e| e.to_string())?;
        let out = cbor::res(res).map_err(|e| e.to_string())?;
        Ok(out.as_slice().into())
    }

    // pub fn notifications(&self) -> std::result::Result<sys::ReadableStream, Error> {
    //     let stream = self.0.kvs.notifications().ok_or("Notifications not enabled")?;
    //
    //
    //     let response = stream.map(|notification| {
    //         let json = json!({
	// 			"id": notification.id,
	// 			"action": notification.action.to_string(),
	// 			"result": notification.result.into_json(),
	// 		});
    //         to_value(&json).map_err(Into::into)
    //     });
    //     Ok(ReadableStream::from_stream(response).into_raw())
    // }

    pub async fn connect(
        endpoint: String,
        opts: Option<JsValue>,
    ) -> std::result::Result<SurrealNodeEngine, Error> {
        let endpoint = match &endpoint {
            s if s.starts_with("mem:") => "memory",
            s => s,
        };
        let kvs = Datastore::new(endpoint).await?.with_notifications();
        let kvs = match from_value::<Option<Options>>(JsValue::from(opts))? {
            None => kvs,
            Some(opts) => kvs
                .with_capabilities(
                    opts.capabilities.map_or(Ok(Default::default()), |a| a.try_into())?,
                )
                .with_transaction_timeout(
                    opts.transaction_timeout.map(|qt| Duration::from_secs(qt as u64)),
                )
                .with_query_timeout(opts.query_timeout.map(|qt| Duration::from_secs(qt as u64)))
                .with_strict_mode(opts.strict.map_or(Default::default(), |s| s)),
        };

        let inner = SurrealNodeEngineInner {
            kvs,
            session: Session {
                rt: true,
                ..Default::default()
            },
            vars: Default::default(),
        };

        Ok(SurrealNodeEngine(inner))
    }
}

 struct SurrealNodeEngineInner{
    pub kvs: Datastore,
    pub session: Session,
    pub vars: BTreeMap<String, surrealdb::sql::Value>,
}
impl RpcContext for SurrealNodeEngineInner {
    fn kvs(&self) -> &Datastore {
        &self.kvs
    }

    fn session(&self) -> &Session {
        &self.session
    }

    fn session_mut(&mut self) -> &mut Session {
        &mut self.session
    }

    fn vars(&self) -> &BTreeMap<String, surrealdb::sql::Value> {
        &self.vars
    }

    fn vars_mut(&mut self) -> &mut BTreeMap<String, surrealdb::sql::Value> {
        &mut self.vars
    }

    fn version_data(&self) -> impl Into<Data> {
        let val = "todo".to_string();

        val
    }

    const LQ_SUPPORT: bool = true;
    fn handle_live(&self, _lqid: &Uuid) -> impl std::future::Future<Output = ()> + Send {
        async { () }
    }
    fn handle_kill(&self, _lqid: &Uuid) -> impl std::future::Future<Output = ()> + Send {
        async { () }
    }
}

#[napi]
pub struct Surreal {
    db: surrealdb::Surreal<Any>,
}

#[napi]
impl Surreal {
    #[napi(constructor)]
    pub fn init() -> Self {
        Self {
            db: surrealdb::Surreal::init(),
        }
    }

	#[napi]
    pub async fn connect(&self, endpoint: String, #[napi(ts_arg_type = "Record<string, unknown>")] opts: Option<JsValue>) -> Result<()> {
        let opts: Option<Options> = match opts {
            Some(o) => serde_json::from_value(o)?,
            None => None,
        };

        let config = Config::new().capabilities(Capabilities::all());

        let connect = match opts {
            Some(opts) => {
                let connect = match opts.strict {
                    #[cfg(any(feature = "kv-indxdb", feature = "kv-mem"))]
                    Some(true) => self.db.connect((endpoint, config.strict())),
                    _ => self.db.connect((endpoint, config)),
                };
                match opts.capacity {
                    Some(capacity) => connect.with_capacity(capacity),
                    None => connect,
                }
            }
            None => self.db.connect(endpoint),
        };
        connect.await.map_err(err_map)
    }

    #[napi(js_name = use)]
    pub async fn yuse(&self, #[napi(ts_arg_type = "{ namespace?: string; database?: string }")] value:JsValue) -> Result<()> {
        let opts: opt::yuse::Use = serde_json::from_value(value)?;
        match (opts.namespace, opts.database) {
            (Some(namespace), Some(database)) => self.db.use_ns(namespace).use_db(database).await.map_err(err_map),
            (Some(namespace), None) => self.db.use_ns(namespace).await.map_err(err_map),
            (None, Some(database)) => self.db.use_db(database).await.map_err(err_map),
            (None, None) => Err(napi::Error::from_reason(
                "Select either namespace or database to use",
            )),
        }
    }

    #[napi]
    pub async fn set(&self, key: String, #[napi(ts_arg_type = "unknown")] value:JsValue) -> Result<()> {
        self.db.set(key, value).await.map_err(err_map)?;
        Ok(())
    }

    #[napi]
    pub async fn unset(&self, key: String) -> Result<()> {
        self.db.unset(key).await.map_err(err_map)?;
        Ok(())
    }

    #[napi(ts_return_type="Promise<string>")]
    pub async fn signup(&self, #[napi(ts_arg_type = "{ namespace: string; database: string; scope: string; [k: string]: unknown }")] credentials:JsValue) -> Result<JsValue> {
        match from_value::<Credentials>(credentials)? {
            Credentials::Scope {
                namespace,
                database,
                scope,
                params,
            } => {
                let response = self
                    .db
                    .signup(Scope {
                        params,
                        namespace: &namespace,
                        database: &database,
                        scope: &scope,
                    })
                    .await
                    .map_err(err_map)?;
                Ok(to_value(&response)?)
            }
            Credentials::Database { .. } => Err(napi::Error::from_reason(
                "Database users cannot signup, only scope users can",
            )),
            Credentials::Namespace { .. } => Err(napi::Error::from_reason(
                "Namespace users cannot signup, only scope users can",
            )),
            Credentials::Root { .. } => Err(napi::Error::from_reason(
                "Root users cannot signup, only scope users can",
            )),
        }
    }

    #[napi(ts_return_type="Promise<string>")]
    pub async fn signin(&self, #[napi(ts_arg_type = "{ username: string; password: string } | { namespace: string; username: string; password: string } | { namespace: string; database: string; username: string; password: string } | { namespace: string; database: string; scope: string; [k: string]: unknown }")] credentials:JsValue) -> Result<JsValue> {
        let signin = match &from_value::<Credentials>(credentials)? {
            Credentials::Scope {
                namespace,
                database,
                scope,
                params,
            } => self.db.signin(Scope {
                namespace,
                database,
                scope,
                params,
            }),
            Credentials::Database {
                namespace,
                database,
                username,
                password,
            } => self.db.signin(Database {
                namespace,
                database,
                username,
                password,
            }),
            Credentials::Namespace {
                namespace,
                username,
                password,
            } => self.db.signin(Namespace {
                namespace,
                username,
                password,
            }),
            Credentials::Root {
				username,
				password
			} => self.db.signin(Root {
				username,
				password
			}),
        };
        Ok(to_value(&signin.await.map_err(err_map)?)?)
    }

    #[napi]
    pub async fn invalidate(&self) -> Result<()> {
        self.db.invalidate().await.map_err(err_map)?;
        Ok(())
    }

    #[napi(ts_return_type="Promise<boolean>")]
    pub async fn authenticate(&self, token: String) -> Result<JsValue> {
        self.db.authenticate(token).await.map_err(err_map)?;
        Ok(Value::Bool(true))
    }

    #[napi(ts_return_type="Promise<unknown[]>")]
    pub async fn query(&self, sql: String, #[napi(ts_arg_type = "Record<string, unknown>")] bindings: Option<JsValue>) -> Result<JsValue> {
        let mut response = match bindings {
            None => self.db.query(sql).await.map_err(err_map)?,
            Some(b) => {
                let b = json(&b.to_string()).map_err(err_map)?;
                self.db.query(sql).bind(b).await.map_err(err_map)?
            },
        };

        let num_statements = response.num_statements();

        let response: sql::Value = {
            let mut output = Vec::<sql::Value>::with_capacity(num_statements);
            for index in 0..num_statements {
                output.push(response.take(index).map_err(err_map)?);
            }
            sql::Value::from(output)
        };
        Ok(to_value(&response.into_json())?)
    }

    #[napi(ts_return_type="Promise<{ id: string; [k: string]: unknown }[]>")]
    pub async fn select(&self, resource: String) -> Result<JsValue> {
        let response = match resource.parse::<Range>() {
            Ok(range) => self
                .db
                .select(Resource::from(range.tb))
                .range((range.beg, range.end))
                .await
                .map_err(err_map)?,
            Err(_) => self
                .db
                .select(Resource::from(resource))
                .await
                .map_err(err_map)?,
        };
		let response = array_response(response);
        Ok(to_value(&response.into_json())?)
    }

    #[napi(ts_return_type="Promise<{ id: string; [k: string]: unknown }[]>")]
    pub async fn create(&self, resource: String, #[napi(ts_arg_type = "Record<string, unknown>")] data: Option<JsValue>) -> Result<JsValue> {
        let resource = Resource::from(resource);
		let response = match data {
            None => self.db.create(resource).await.map_err(err_map)?,
            Some(d) => {
                let d = json(&d.to_string()).map_err(err_map)?;
                self.db.create(resource).content(d).await.map_err(err_map)?
            },
        };
		let response = array_response(response);
        Ok(to_value(&response.into_json())?)
    }

    #[napi(ts_return_type="Promise<{ id: string; [k: string]: unknown }[]>")]
    pub async fn update(&self, resource: String, #[napi(ts_arg_type = "Record<string, unknown>")] data: Option<JsValue>) -> Result<JsValue> {
        let update = match resource.parse::<Range>() {
            Ok(range) => self
                .db
                .update(Resource::from(range.tb))
                .range((range.beg, range.end)),
            Err(_) => self.db.update(Resource::from(resource)),
        };
		let response = match data {
            None => update.await.map_err(err_map)?,
            Some(d) => {
                let d = json(&d.to_string()).map_err(err_map)?;
                update.content(d).await.map_err(err_map)?
            },
        };
		let response = array_response(response);
        Ok(to_value(&response.into_json())?)
    }

    #[napi(ts_return_type="Promise<{ id: string; [k: string]: unknown }[]>")]
    pub async fn merge(&self, resource: String, #[napi(ts_arg_type = "Record<string, unknown>")] data:JsValue) -> Result<JsValue> {
        let update = match resource.parse::<Range>() {
            Ok(range) => self
                .db
                .update(Resource::from(range.tb))
                .range((range.beg, range.end)),
            Err(_) => self.db.update(Resource::from(resource)),
        };
		let data = json(&data.to_string()).map_err(err_map)?;
        let response = update.merge(data).await.map_err(err_map)?;
		let response = array_response(response);
        Ok(to_value(&response.into_json())?)
    }

    #[napi(ts_return_type="Promise<unknown[]>")]
    pub async fn patch(&self, resource: String, #[napi(ts_arg_type = "unknown[]")] data:JsValue) -> Result<JsValue> {
        // Prepare the update request
        let update = match resource.parse::<Range>() {
            Ok(range) => self
                .db
                .update(Resource::from(range.tb))
                .range((range.beg, range.end)),
            Err(_) => self.db.update(Resource::from(resource)),
        };
        let mut patches: VecDeque<Patch> = from_value(data)?;
        // Extract the first patch
        let mut patch = match patches.pop_front() {
            // Setup the correct update type using the first patch
            Some(p) => update.patch(match p {
                Patch::Add { path, value } => PatchOp::add(&path, value),
                Patch::Remove { path } => PatchOp::remove(&path),
                Patch::Replace { path, value } => PatchOp::replace(&path, value),
                Patch::Change { path, diff } => PatchOp::change(&path, diff),
            }),
            None => {
                return Ok(to_value(&update.await.map_err(err_map)?.into_json())?);
            }
        };
        // Loop through the rest of the patches and append them
        for p in patches {
            patch = patch.patch(match p {
                Patch::Add { path, value } => PatchOp::add(&path, value),
                Patch::Remove { path } => PatchOp::remove(&path),
                Patch::Replace { path, value } => PatchOp::replace(&path, value),
                Patch::Change { path, diff } => PatchOp::change(&path, diff),
            });
        }
        // Execute the update statement
        let response = patch.await.map_err(err_map)?;
		let response = array_response(response);
        Ok(to_value(&response.into_json())?)
    }

    #[napi(ts_return_type="Promise<{ id: string; [k: string]: unknown }[]>")]
    pub async fn delete(&self, resource: String) -> Result<JsValue> {
        let response = match resource.parse::<Range>() {
            Ok(range) => self
                .db
                .delete(Resource::from(range.tb))
                .range((range.beg, range.end))
                .await
                .map_err(err_map)?,
            Err(_) => self
                .db
                .delete(Resource::from(resource))
                .await
                .map_err(err_map)?,
        };
		let response = array_response(response);
        Ok(to_value(&response.into_json())?)
    }

    #[napi(ts_return_type="Promise<string>")]
    pub async fn version(&self) -> Result<JsValue> {
        let response = self.db.version().await.map_err(err_map)?;
        Ok(to_value(&response)?)
    }

    #[napi]
    pub async fn health(&self) -> Result<()> {
        self.db.health().await.map_err(err_map)?;
        Ok(())
    }
}
