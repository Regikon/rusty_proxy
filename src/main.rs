use bytes::Bytes;
use dotenv::dotenv;
use dto::{prelude::Response, request::Request};
use proxy::Proxy;
use simplelog::{Config, LevelFilter, SimpleLogger};
use std::sync::{Arc, Mutex};

mod config;
mod dto;
mod proxy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();

    SimpleLogger::init(LevelFilter::Info, Config::default()).unwrap();
    let config = config::Config::from_env()?;

    let proxy = Proxy::builder()
        .with_host(config.proxy_host().clone())
        .with_port(config.proxy_port())
        .with_tls(config.ssl_certificate().clone(), config.ssl_key().clone())
        .with_callback(Arc::new(Mutex::new(remake_request_and_response)))
        .build()?;

    proxy.serve().await
}

fn remake_request_and_response(
    req: &(http::request::Parts, Bytes, bool),
    resp: &(http::response::Parts, Bytes),
) {
    let req = Request::from(req.clone());
    let resp = Response::from(resp.clone());
    println!("Here is my own request type: {:?}", req);
    println!("There is a response too: {:?}", resp);
}
