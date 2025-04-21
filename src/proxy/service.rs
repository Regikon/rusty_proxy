use rustls::pki_types::ServerName;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::net::TcpStream;

use super::utils::validate_request;
use bytes::Bytes;
use http::{HeaderValue, Request, Response, Uri};
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::body::Incoming;
use hyper::client;
use hyper::service::Service;
use hyper_util::rt::TokioIo;
use log::{debug, error, info};

use super::utils::HEADER_PROXY_CONNECTION;

#[derive(Debug, Clone)]
pub struct ProxyService {
    pub is_tls: bool,
}

impl Service<Request<Incoming>> for ProxyService {
    type Response = Response<BoxBody<Bytes, hyper::Error>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        info!(
            "Got request. Host: {:?}, {:?} {:?}",
            req.headers()
                .get(http::header::HOST)
                .map_or("", |host| host.to_str().unwrap_or("")),
            req.method(),
            req.uri()
        );

        if self.is_tls {
            let full_host = extract_host(&req).unwrap();
            let (host, port) = parse_host_header(&full_host, 443).unwrap();
            return Box::pin(forward_secure_request(req, host, port));
        }

        if let Err(cause) = validate_request(&req) {
            return Box::pin(async move {
                Ok(Response::builder()
                    .status(http::StatusCode::BAD_REQUEST)
                    .body(
                        Full::new(Bytes::from(format!("{}", cause)))
                            .map_err(|never| match never {})
                            .boxed(),
                    )
                    .unwrap())
            });
        }

        // Safe unwrap since validate_request covers no host situation
        let host = String::from(req.uri().host().unwrap());
        let port = req.uri().port_u16().unwrap_or(80);
        let req = clean_request(req);

        debug!("{:?}", req);

        Box::pin(forward_unsecure_request(req, host, port))
    }
}

// Get host from request
fn extract_host<T>(req: &Request<T>) -> Option<String> {
    if let Some(addr) = req.headers().get(http::header::HOST) {
        let addr = addr.to_str();
        if let Ok(addr) = addr {
            return Some(String::from(addr));
        }
    }

    if let Some(host) = req.uri().host() {
        return Some(String::from(host));
    }

    return None;
}

// Parse host header
fn parse_host_header(host: &String, fallback_port: u16) -> Result<(String, u16), String> {
    match host.find(":") {
        Some(idx) => {
            if idx == host.len() - 1 {
                return Err(String::from("unexpected eol while parsing port"));
            }
            if let Ok(port) = host[(idx + 1)..].parse::<u16>() {
                return Ok((String::from(&host[..idx]), port));
            }
            Err(String::from("invalid host"))
        }
        None => Ok((host.clone(), fallback_port)),
    }
}

// Clean proxy request to make it valid non-proxy request
fn clean_request<T>(mut req: Request<T>) -> Request<T> {
    req.headers_mut().remove(HEADER_PROXY_CONNECTION);
    let full_uri = req.uri();
    let mut clean_uri = Uri::builder();
    if let Some(p_a_q) = full_uri.path_and_query() {
        clean_uri = clean_uri.path_and_query(p_a_q.clone());
    }
    // Safe since valid Request holds valid url
    let clean_uri = clean_uri.build().unwrap();
    *req.uri_mut() = clean_uri;
    req
}

// Client side of the proxy
async fn forward_unsecure_request(
    req: Request<Incoming>,
    host: String,
    port: u16,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    debug!("Forwarding to {}:{}", host, port);
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
    debug!("Got response: {:?}", resp);

    Ok(resp.map(|b| BoxBody::new(b)))
}

async fn forward_secure_request(
    req: Request<Incoming>,
    host: String,
    port: u16,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    debug!("Forwarding to {}:{}", host, port);
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
    debug!("Got response: {:?}", resp);

    Ok(resp.map(|b| BoxBody::new(b)))
}
