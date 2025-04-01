use proxy::Proxy;
use simplelog::{Config, LevelFilter, SimpleLogger};
use std::net::SocketAddr;

mod proxy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    SimpleLogger::init(LevelFilter::Info, Config::default()).unwrap();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let proxy = Proxy::builder().with_addr(addr).build()?;

    proxy.serve().await
}
