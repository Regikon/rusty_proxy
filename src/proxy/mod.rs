use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use log::{error, info};
use std::net::SocketAddr;
use tokio::net::TcpListener;

use service::ProxyService;

use thiserror::Error;

mod service;
mod utils;

pub struct Proxy {
    addr: SocketAddr,
}

impl Proxy {
    pub fn builder() -> ProxyBuilder {
        ProxyBuilder::default()
    }

    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = TcpListener::bind(self.addr).await?;
        info!("Listening on http://{}", self.addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .preserve_header_case(true)
                    .title_case_headers(true)
                    .serve_connection(io, ProxyService {})
                    .await
                {
                    error!("Error serving connection: {err}");
                }
            });
        }
    }
}

#[derive(Default)]
pub struct ProxyBuilder {
    host: Option<String>,
    port: Option<u16>,
    addr: Option<SocketAddr>,
}

impl ProxyBuilder {
    pub fn new() -> ProxyBuilder {
        ProxyBuilder::default()
    }

    pub fn with_host(mut self, host: String) -> ProxyBuilder {
        self.host = Some(host);
        self
    }

    pub fn with_port(mut self, port: u16) -> ProxyBuilder {
        self.port = Some(port);
        self
    }

    pub fn with_addr(mut self, addr: SocketAddr) -> ProxyBuilder {
        self.addr = Some(addr);
        self
    }

    pub fn build(mut self) -> Result<Proxy, BuildError> {
        if None == self.addr {
            if None == self.host {
                return Err(BuildError::NoHost);
            };
            if None == self.port {
                return Err(BuildError::NoPort);
            }
            let host = match self.host.unwrap().parse() {
                Ok(ip) => ip,
                Err(_) => return Err(BuildError::InvalidHost),
            };
            let port = self.port.unwrap();
            self.addr = Some(SocketAddr::new(host, port));
        }

        Ok(Proxy {
            addr: self.addr.unwrap(),
        })
    }
}

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("host is not specified")]
    NoHost,

    #[error("given host is not a valid ip")]
    InvalidHost,

    #[error("connection port is not specified")]
    NoPort,
}
