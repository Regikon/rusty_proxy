use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use log::{error, info};
use middleware::TlsUpgrader;
use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};
use std::net::SocketAddr;
use tokio::net::TcpListener;

use service::ProxyService;

use thiserror::Error;

pub mod client;
mod middleware;
mod service;
pub mod utils;

pub use service::BodyType;
pub use service::CallbackType;

pub struct Proxy {
    addr: SocketAddr,
    cert: String,
    key: String,
    callback: Option<service::CallbackType>,
}

impl Proxy {
    pub fn builder() -> ProxyBuilder {
        ProxyBuilder::default()
    }

    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = TcpListener::bind(self.addr).await?;

        let certs = CertificateDer::pem_file_iter(self.cert)
            .unwrap()
            .map(|cert| cert.unwrap())
            .collect();
        let private_key = PrivateKeyDer::from_pem_file(self.key).unwrap();
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, private_key)
            .unwrap();

        info!("Listening on address {:?}", self.addr);

        loop {
            let stream = match listener.accept().await {
                Ok((stream, _)) => stream,
                Err(e) => {
                    error!("Failed to accept connection: {:?}", e);
                    continue;
                }
            };
            let io = TokioIo::new(stream);

            let service = TlsUpgrader::new(
                ProxyService {
                    is_tls: false,
                    callback: self.callback.clone(),
                },
                ProxyService {
                    is_tls: true,
                    callback: self.callback.clone(),
                },
                config.clone(),
            );
            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .preserve_header_case(true)
                    .serve_connection(io, service)
                    .with_upgrades()
                    .await
                {
                    error!("Error serving connection: {err:?}");
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
    cert_filepath: Option<String>,
    key_filepath: Option<String>,
    callback: Option<service::CallbackType>,
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

    pub fn with_tls(mut self, cert_path: String, key_path: String) -> ProxyBuilder {
        self.cert_filepath = Some(cert_path);
        self.key_filepath = Some(key_path);
        self
    }

    pub fn with_callback(mut self, callback: service::CallbackType) -> ProxyBuilder {
        self.callback = Some(callback);
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

        if None == self.cert_filepath || None == self.key_filepath {
            return Err(BuildError::NoSSL);
        }

        Ok(Proxy {
            addr: self.addr.unwrap(),
            cert: self.cert_filepath.unwrap(),
            key: self.key_filepath.unwrap(),
            callback: self.callback,
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

    #[error("not given ssl certificates")]
    NoSSL,
}
