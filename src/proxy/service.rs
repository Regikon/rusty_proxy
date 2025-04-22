use std::{future::Future, pin::Pin, sync::Arc};

use super::utils::{clean_request, extract_host, parse_host_header};
use super::{client::Client, utils::validate_request};
use bytes::Bytes;
use http::{Request, Response};
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::body::Incoming;
use hyper::service::Service;
use log::{debug, error, info};
use std::sync::Mutex;

pub type BodyType = BoxBody<Bytes, hyper::Error>;
pub type CallbackType = Arc<
    Mutex<
        dyn Fn((http::request::Parts, Bytes, bool), (http::response::Parts, Bytes)) -> ()
            + Send
            + 'static,
    >,
>;

#[derive(Clone)]
pub struct ProxyService {
    pub is_tls: bool,
    pub callback: Option<CallbackType>,
}

impl Service<Request<Incoming>> for ProxyService {
    type Response = Response<BodyType>;
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

        Box::pin(process_proxy_request(
            req,
            self.is_tls,
            self.callback.clone(),
        ))
    }
}

async fn process_proxy_request(
    req: Request<Incoming>,
    is_tls: bool,
    callback: Option<CallbackType>,
) -> Result<Response<BodyType>, hyper::Error> {
    // Downloading request body in order to use callback later
    // TODO: do not download the body if callback is not set (requires some extra magic with
    // generics)
    let (req_parts, req_body) = req.into_parts();
    let req_body_bytes = req_body.collect().await?.to_bytes();
    // we do not copy the request body because we are using Bytes, which is Arc under hood
    let collected_body =
        BodyType::new(Full::new(req_body_bytes.clone()).map_err(|never| match never {}));

    // The request is changed when proxy connection header is removed
    let mut req = Request::from_parts(req_parts.clone(), collected_body);
    let mut response: Response<BodyType>;
    let host: String;
    let port: u16;

    if is_tls {
        let full_host = extract_host(&req).unwrap();
        (host, port) = parse_host_header(&full_host, 443).unwrap();
    } else {
        if let Err(cause) = validate_request(&req) {
            return Ok(Response::builder()
                .status(http::StatusCode::BAD_REQUEST)
                .body(
                    Full::new(Bytes::from(format!("{}", cause)))
                        .map_err(|never| match never {})
                        .boxed(),
                )
                .unwrap());
        }
        // Safe unwrap since validate_request covers no host situation
        host = String::from(req.uri().host().unwrap());
        port = req.uri().port_u16().unwrap_or(80);
        req = clean_request(req);
    }

    debug!("Forwarding to {}:{}", host, port);
    response = Client::send_request(req, host, port, is_tls).await?;
    debug!("Got response: {:?}", response);

    if let Some(callback) = callback {
        let (response_parts, response_body) = response.into_parts();
        let resp_body_bytes = response_body.collect().await?.to_bytes();

        let callback = callback.lock();
        match callback {
            Ok(callback) => callback(
                (req_parts, req_body_bytes, is_tls),
                (response_parts.clone(), resp_body_bytes.clone()),
            ),
            Err(_) => error!("failed to use callback: the mutex is poisoned"),
        }
        response = Response::from_parts(
            response_parts,
            Full::new(resp_body_bytes)
                .map_err(|never| match never {})
                .boxed(),
        );
    }
    Ok(response)
}
