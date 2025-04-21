use dotenv::dotenv;
use proxy::Proxy;
use simplelog::{Config, LevelFilter, SimpleLogger};

mod config;
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
        .build()?;

    proxy.serve().await
}
