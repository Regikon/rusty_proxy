use proxy::Proxy;
use simplelog::{Config, LevelFilter, SimpleLogger};
use std::net::SocketAddr;

mod proxy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    SimpleLogger::init(LevelFilter::Info, Config::default()).unwrap();

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    let proxy = Proxy::builder()
        .with_addr(addr)
        .with_tls(
            String::from("./certs/ca.crt"),
            String::from("./certs/ca.key"),
        )
        .build()?;

    proxy.serve().await
}
