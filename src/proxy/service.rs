use std::{future::Future, pin::Pin};
use tokio::net::TcpStream;

use super::utils::validate_request;
use bytes::Bytes;
use http::{Request, Response, Uri};
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::body::Incoming;
use hyper::client;
use hyper::service::Service;
use hyper_util::rt::TokioIo;
use log::{debug, error, info};

use super::utils::HEADER_PROXY_CONNECTION;

#[derive(Debug, Clone)]
pub struct ProxyService {}

impl Service<Request<Incoming>> for ProxyService {
    type Response = Response<BoxBody<Bytes, hyper::Error>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        info!("Got request {:?} {:?}", req.method(), req.uri());

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

        Box::pin(forward_request(req, host, port))
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
async fn forward_request(
    req: Request<Incoming>,
    host: String,
    port: u16,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
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
    info!("Got response: {:?}", resp.status());
    debug!("{:?}", resp);

    Ok(resp.map(|b| BoxBody::new(b)))
}
