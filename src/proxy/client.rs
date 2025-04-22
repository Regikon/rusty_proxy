use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;

use super::BodyType;
use http::{Request, Response};
use http_body_util::combinators::BoxBody;
use hyper::client;
use log::error;
use rustls::pki_types::ServerName;
use std::sync::Arc;

#[derive(Clone)]
pub struct Client {}

impl Client {
    pub async fn send_request(
        req: Request<BodyType>,
        host: String,
        port: u16,
        is_https: bool,
    ) -> Result<Response<BodyType>, hyper::Error> {
        if is_https {
            Client::send_secure_request(req, host, port).await
        } else {
            Client::send_unsecure_request(req, host, port).await
        }
    }

    async fn send_unsecure_request(
        req: Request<BodyType>,
        host: String,
        port: u16,
    ) -> Result<Response<BodyType>, hyper::Error> {
        let stream = TcpStream::connect((host.as_str(), port)).await.unwrap();
        let io = TokioIo::new(stream);

        let (mut sender, conn) = client::conn::http1::Builder::new()
            .preserve_header_case(true)
            .title_case_headers(true)
            .handshake(io)
            .await?;

        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                error!("Connection failed: {:?}", err);
            }
        });

        let resp = sender.send_request(req).await?;

        Ok(resp.map(|b| BoxBody::new(b)))
    }

    async fn send_secure_request(
        req: Request<BodyType>,
        host: String,
        port: u16,
    ) -> Result<Response<BodyType>, hyper::Error> {
        let stream = TcpStream::connect((host.as_str(), port)).await.unwrap();

        let root_store =
            rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        let rc_config = Arc::new(config);
        let conn = tokio_rustls::TlsConnector::from(rc_config);
        let server_name = ServerName::try_from(host).unwrap();
        let io = conn.connect(server_name, stream).await.unwrap();
        let io = TokioIo::new(io);

        let (mut sender, conn) = client::conn::http1::Builder::new()
            .preserve_header_case(true)
            .title_case_headers(true)
            .handshake(io)
            .await?;

        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                error!("Connection failed: {:?}", err);
            }
        });

        let resp = sender.send_request(req).await?;

        Ok(resp.map(|b| BoxBody::new(b)))
    }
}
