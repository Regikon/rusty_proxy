use dotenv::dotenv;
use log::{error, info};
use rusty_proxy::dto::{Reqresp, Request, Response};
use rusty_proxy::proxy::Proxy;
use rusty_proxy::storage::storage::ReqrespStorage;
use simplelog::{Config, LevelFilter, SimpleLogger};
use std::sync::{Arc, Mutex};

use rusty_proxy::dto::hyper::{HyperRequest, HyperResponse};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();

    SimpleLogger::init(LevelFilter::Info, Config::default()).unwrap();
    let config = rusty_proxy::config::Config::from_env()?;

    info!("Conneting to storage...");
    let mongo_client = mongodb::Client::with_uri_str(config.mongodb_uri()).await?;
    let mongo_storage = rusty_proxy::storage::mongodb_storage::MongoDbStorage::new(mongo_client);

    let callback = Arc::new(Mutex::new(move |req: HyperRequest, resp: HyperResponse| {
        let mongo_storage = mongo_storage.clone();
        tokio::spawn(save_reqresp_to_storage(req, resp, mongo_storage));
    }));

    info!("Initializing proxy...");
    let proxy = Proxy::builder()
        .with_host(config.proxy_host().clone())
        .with_port(config.proxy_port())
        .with_tls(config.ssl_certificate().clone(), config.ssl_key().clone())
        .with_callback(callback)
        .build()?;

    proxy.serve().await
}

async fn save_reqresp_to_storage<T>(req: HyperRequest, resp: HyperResponse, storage: T)
where
    T: ReqrespStorage,
{
    let req = Request::from(req.clone());
    let resp = Response::from(resp.clone());
    let reqresp = Reqresp::new(req, resp);

    if let Err(e) = storage.add_reqresp(reqresp).await {
        error!("failed to write to storage: {:?}", e);
    }
}
