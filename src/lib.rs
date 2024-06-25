mod error;
mod opt;

use std::collections::BTreeMap;

use error::err_map;
use napi::bindgen_prelude::*;
use napi::tokio::sync::RwLock;
use napi_derive::napi;

use once_cell::sync::Lazy;
use serde_json::Value as JsValue;
use surrealdb::dbs::Session;
use surrealdb::kvs::Datastore;
use surrealdb::rpc::format::cbor;
use surrealdb::rpc::method::Method;

use surrealdb::rpc::{Data, RpcContext};
use uuid::Uuid;

#[napi]
pub struct SurrealdbNodeEngine(RwLock<SurrealdbNodeEngineInner>);

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
                    .execute_immut(method, in_data.params)
                    .await
            }
            false => self.0.write().await.execute(method, in_data.params).await,
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
        #[napi(ts_arg_type = "ConnectionOptions")] _opts: Option<JsValue>,
    ) -> std::result::Result<SurrealdbNodeEngine, Error> {
        let endpoint = match &endpoint {
            s if s.starts_with("mem:") => "memory",
            s => s,
        };
        let kvs = Datastore::new(endpoint)
            .await
            .map_err(err_map)?
            .with_notifications();
        // let kvs = match from_value::<Option<Options>>(JsValue::from(opts))? {
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

        let session = Session::default().with_rt(true);

        let inner = SurrealdbNodeEngineInner {
            kvs,
            session,
            vars: Default::default(),
        };

        Ok(SurrealdbNodeEngine(RwLock::new(inner)))
    }

    #[napi]
    pub fn free(&self) {}

    #[napi]
    pub fn version() -> std::result::Result<String, Error> {
        Ok(SURREALDB_VERSION.clone())
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

    fn version_data(&self) -> impl Into<Data> {
        SURREALDB_VERSION.clone()
    }

    const LQ_SUPPORT: bool = true;
    fn handle_live(&self, _lqid: &Uuid) -> impl std::future::Future<Output = ()> + Send {
        async { () }
    }
    fn handle_kill(&self, _lqid: &Uuid) -> impl std::future::Future<Output = ()> + Send {
        async { () }
    }
}

static LOCK_FILE: &str = include_str!("../Cargo.lock");

pub static SURREALDB_VERSION: Lazy<String> = Lazy::new(|| {
    let lock: cargo_lock::Lockfile = LOCK_FILE.parse().expect("Failed to parse Cargo.lock");
    let package = lock
        .packages
        .iter()
        .find(|p| p.name.as_str() == "surrealdb")
        .expect("Failed to find surrealdb in Cargo.lock");

    format!(
        "{}.{}.{}",
        package.version.major, package.version.minor, package.version.patch
    )
});
