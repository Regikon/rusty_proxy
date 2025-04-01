use bytes::Bytes;
use http::Method;
use http_body_util::{combinators::BoxBody, BodyExt, Empty};
use hyper::{body::Incoming, service::Service, Request};
use log::error;

// A TLS-connection upgrading service
#[derive(Debug, Clone)]
pub struct TlsUpgrader<S> {
    inner: S,
}

impl<S> TlsUpgrader<S> {
    pub fn new(inner: S) -> Self {
        TlsUpgrader { inner }
    }
}

impl<S> Service<Request<Incoming>> for TlsUpgrader<S>
where
    S: Service<Request<Incoming>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        if req.method() == Method::CONNECT {
            tokio::task::spawn(async move {
                match hyper::upgrade::on(req).await {
                    Ok(upgraded) => {}
                    Err(e) => error!("TLS upgrade error: {}", e),
                }
            });
        } else {
            self.inner.call(req)
        }
    }
}

fn empty_body() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
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
