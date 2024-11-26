mod error;
mod opt;

use std::collections::BTreeMap;
use std::time::Duration;

use error::err_map;
use napi::bindgen_prelude::*;
use napi::tokio::sync::RwLock;
use napi_derive::napi;

use opt::endpoint::Options;
use serde_json::from_value;
use serde_json::Value as JsValue;
use surrealdb::dbs::Session;
use surrealdb::kvs::Datastore;
use surrealdb::rpc::format::cbor;
use surrealdb::rpc::method::Method;
use surrealdb::kvs::export::Config;

use surrealdb::rpc::{Data, RpcContext};
use surrealdb::sql::Value;
use uuid::Uuid;

#[napi]
pub struct SurrealdbNodeEngine(RwLock<Option<SurrealdbNodeEngineInner>>);

#[napi]
impl SurrealdbNodeEngine {
    #[napi]
    pub async fn execute(&self, data: Uint8Array) -> std::result::Result<Uint8Array, Error> {
        let in_data = cbor::req(data.to_vec()).map_err(err_map)?;
        let method = Method::parse(in_data.method);
        let res = match method.can_be_immut() {
            true => {
                self.0
                    .read()
                    .await
                    .as_ref()
                    .unwrap()
                    .execute_immut(method, in_data.params)
                    .await
            }
            false => {
                self.0
                    .write()
                    .await
                    .as_mut()
                    .unwrap()
                    .execute(method, in_data.params)
                    .await
            }
        }
        .map_err(err_map)?;

        let out = cbor::res(res).map_err(err_map)?;
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

    #[napi]
    pub async fn connect(
        endpoint: String,
        #[napi(ts_arg_type = "ConnectionOptions")] opts: Option<JsValue>,
    ) -> std::result::Result<SurrealdbNodeEngine, Error> {
        let endpoint = match &endpoint {
            s if s.starts_with("mem:") => "memory",
            s => s,
        };
        let kvs = Datastore::new(endpoint)
            .await
            .map_err(err_map)?
            .with_notifications();
        // let kvs = match opts.map(|o| o.try_into()) {
        //     None => kvs,
        //     Some(opts) => kvs
        //         .with_capabilities(
        //             opts.capabilities
        //                 .map_or(Ok(Default::default()), |a| a.try_into())?,
        //         )
        //         .with_transaction_timeout(
        //             opts.transaction_timeout
        //                 .map(|qt| Duration::from_secs(qt as u64)),
        //         )
        //         .with_query_timeout(opts.query_timeout.map(|qt| Duration::from_secs(qt as u64)))
        //         .with_strict_mode(opts.strict.map_or(Default::default(), |s| s)),
        // };

        // let kvs = if let Some(opts) = opts.map(from_value::<Option<Options>>) {
        //     kvs
        // } else {
        //     kvs
        // };

        let kvs = if let Some(o) = opts {
            let opts = from_value::<Options>(o)?;
            kvs.with_capabilities(
                opts.capabilities
                    .map_or(Ok(Default::default()), |a| a.try_into())?,
            )
            .with_transaction_timeout(
                opts.transaction_timeout
                    .map(|qt| Duration::from_secs(qt as u64)),
            )
            .with_query_timeout(opts.query_timeout.map(|qt| Duration::from_secs(qt as u64)))
            .with_strict_mode(opts.strict.map_or(Default::default(), |s| s))
        } else {
            kvs
        };

        let session = Session::default().with_rt(true);

        let inner = SurrealdbNodeEngineInner {
            kvs,
            session,
            vars: Default::default(),
        };

        Ok(SurrealdbNodeEngine(RwLock::new(Some(inner))))
    }

    #[napi]
    pub async fn free(&self) {
        let _inner_opt = self.0.write().await.take();
    }

    #[napi]
    pub fn version() -> std::result::Result<String, Error> {
        Ok(env!("SURREALDB_VERSION").into())
    }

	#[napi]
	pub async fn export(&self, config: Option<Uint8Array>) -> std::result::Result<String, Error> {
		let lock = self.0.read().await;
		let inner = lock.as_ref().unwrap();
		let (tx, rx) = channel::unbounded();

		match config {
			Some(config) => {
				let in_config = cbor::parse_value(config.to_vec()).map_err(err_map)?;
				let config = Config::try_from(&in_config).map_err(err_map)?;

				inner.kvs.export_with_config(&inner.session, tx, config).await.map_err(err_map)?.await.map_err(err_map)?;
			}
			None => {
				inner.kvs.export(&inner.session, tx).await.map_err(err_map)?.await.map_err(err_map)?;
			}
		};

		let mut buffer = Vec::new();
		while let Ok(item) = rx.try_recv() {
			buffer.push(item);
		}

		let result = String::from_utf8(buffer.concat().into()).map_err(err_map)?;

		Ok(result)
	}
}

struct SurrealdbNodeEngineInner {
    pub kvs: Datastore,
    pub session: Session,
    pub vars: BTreeMap<String, surrealdb::sql::Value>,
}
impl RpcContext for SurrealdbNodeEngineInner {
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

    fn version_data(&self) -> Data {
		Value::Strand(format!("surrealdb-{}", env!("SURREALDB_VERSION")).into()).into()
	}

    const LQ_SUPPORT: bool = true;
    fn handle_live(&self, _lqid: &Uuid) -> impl std::future::Future<Output = ()> + Send {
        async { () }
    }
    fn handle_kill(&self, _lqid: &Uuid) -> impl std::future::Future<Output = ()> + Send {
        async { () }
    }
}