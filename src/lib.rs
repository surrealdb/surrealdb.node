mod error;
mod opt;

use error::err_map;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use opt::patch::Patch;
use opt::{auth::Credentials, endpoint::Options};
use serde_json::to_value;
use serde_json::{from_value, Value};
use std::collections::VecDeque;
use surrealdb::dbs::Capabilities;
use surrealdb::engine::any::Any;
use surrealdb::opt::auth::Database;
use surrealdb::opt::auth::Namespace;
use surrealdb::opt::auth::Root;
use surrealdb::opt::auth::Scope;
use surrealdb::opt::Resource;
use surrealdb::opt::{Config, PatchOp};
use surrealdb::sql::{self, Range, json};

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
    pub async fn connect(&self, endpoint: String, opts: Option<Value>) -> Result<()> {
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
    pub async fn yuse(&self, value: Value) -> Result<()> {
        let opts: opt::yuse::Use = serde_json::from_value(value)?;
        match (opts.ns, opts.db) {
            (Some(ns), Some(db)) => self.db.use_ns(ns).use_db(db).await.map_err(err_map),
            (Some(ns), None) => self.db.use_ns(ns).await.map_err(err_map),
            (None, Some(db)) => self.db.use_db(db).await.map_err(err_map),
            (None, None) => Err(napi::Error::from_reason(
                "Select either namespace or database to use",
            )),
        }
    }

    #[napi]
    pub async fn set(&self, key: String, value: Value) -> Result<()> {
        self.db.set(key, value).await.map_err(err_map)?;
        Ok(())
    }

    #[napi]
    pub async fn unset(&self, key: String) -> Result<()> {
        self.db.unset(key).await.map_err(err_map)?;
        Ok(())
    }

    #[napi]
    pub async fn signup(&self, credentials: Value) -> Result<Value> {
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

    #[napi]
    pub async fn signin(&self, credentials: Value) -> Result<Value> {
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
            Credentials::Root { username, password } => {
                self.db
                    .signin(Root { username, password })
                    .await
                    .map_err(err_map)?;
                return Ok(Value::Null);
            }
        };
        Ok(to_value(&signin.await.map_err(err_map)?)?)
    }

    #[napi]
    pub async fn invalidate(&self) -> Result<()> {
        self.db.invalidate().await.map_err(err_map)?;
        Ok(())
    }

    #[napi]
    pub async fn authenticate(&self, token: String) -> Result<()> {
        self.db.authenticate(token).await.map_err(err_map)?;
        Ok(())
    }

    #[napi]
    pub async fn query(&self, sql: String, bindings: Option<Value>) -> Result<Value> {
        let mut response = match bindings {
            None => self.db.query(sql).await.map_err(err_map)?,
            Some(b) => {
                let b = json(&b.to_string()).map_err(err_map)?;
                self.db.query(sql).bind(b).await.map_err(err_map)?
            },
        };

        let num_statements = response.num_statements();

        let response: sql::Value = if num_statements > 1 {
            let mut output = Vec::<sql::Value>::with_capacity(num_statements);
            for index in 0..num_statements {
                output.push(response.take(index).map_err(err_map)?);
            }
            sql::Value::from(output)
        } else {
            response.take(0).map_err(err_map)?
        };
        Ok(to_value(&response.into_json())?)
    }

    #[napi]
    pub async fn select(&self, resource: String) -> Result<Value> {
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
        Ok(to_value(&response.into_json())?)
    }

    #[napi]
    pub async fn create(&self, resource: String, data: Option<Value>) -> Result<Value> {
        let resource = Resource::from(resource);
		let response = match data {
            None => self.db.create(resource).await.map_err(err_map)?,
            Some(d) => {
                let d = json(&d.to_string()).map_err(err_map)?;
                self.db.create(resource).content(d).await.map_err(err_map)?
            },
        };
        Ok(to_value(&response.into_json())?)
    }

    #[napi]
    pub async fn update(&self, resource: String, data: Option<Value>) -> Result<Value> {
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
        Ok(to_value(&response.into_json())?)
    }

    #[napi]
    pub async fn merge(&self, resource: String, data: Value) -> Result<Value> {
        let update = match resource.parse::<Range>() {
            Ok(range) => self
                .db
                .update(Resource::from(range.tb))
                .range((range.beg, range.end)),
            Err(_) => self.db.update(Resource::from(resource)),
        };
		let data = json(&data.to_string()).map_err(err_map)?;
        let response = update.merge(data).await.map_err(err_map)?;
        Ok(to_value(&response.into_json())?)
    }

    #[napi]
    pub async fn patch(&self, resource: String, data: Value) -> Result<Value> {
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
        Ok(to_value(&response.into_json())?)
    }

    #[napi]
    pub async fn delete(&self, resource: String) -> Result<Value> {
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
        Ok(to_value(&response.into_json())?)
    }

    #[napi]
    pub async fn version(&self) -> Result<Value> {
        let response = self.db.version().await.map_err(err_map)?;
        Ok(to_value(&response)?)
    }

    #[napi]
    pub async fn health(&self) -> Result<()> {
        self.db.health().await.map_err(err_map)?;
        Ok(())
    }
}
