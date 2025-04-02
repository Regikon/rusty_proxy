use std::{future::Future, pin::Pin, sync::Arc};

use super::ProxyService;
use bytes::Bytes;
use http::{Method, Response};
use http_body_util::{combinators::BoxBody, BodyExt, Empty};
use hyper::server::conn::http1;
use hyper::{body::Incoming, service::Service, Request};
use hyper_util::rt::TokioIo;
use log::{debug, error};
use rustls::ServerConfig;

// A TLS-connection upgrading service
#[derive(Debug, Clone)]
pub struct TlsUpgrader<S> {
    inner_tls: S,
    inner: S,
    tls_config: ServerConfig,
}

impl<S> TlsUpgrader<S> {
    pub fn new(inner: S, inner_tls: S, tls_config: ServerConfig) -> Self {
        TlsUpgrader {
            inner,
            inner_tls,
            tls_config,
        }
    }
}

impl Service<Request<Incoming>> for TlsUpgrader<ProxyService> {
    type Response = Response<BoxBody<Bytes, hyper::Error>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        if req.method() == Method::CONNECT {
            let config = Arc::new(self.tls_config.clone());
            let tls_service = self.inner_tls.clone();
            tokio::task::spawn(async move {
                match hyper::upgrade::on(req).await {
                    Ok(upgraded) => {
                        debug!("Upgrading connection to TLS");
                        let upgraded = TokioIo::new(upgraded);
                        let tls_conn = tokio_rustls::TlsAcceptor::from(config);
                        let stream = tls_conn.accept(upgraded).await.unwrap();
                        let stream = TokioIo::new(stream);

                        if let Err(err) = http1::Builder::new()
                            .preserve_header_case(true)
                            .title_case_headers(true)
                            .serve_connection(stream, tls_service)
                            .await
                        {
                            error!("Error serving connection: {err}");
                        }
                    }
                    Err(e) => {
                        error!("TLS upgrade error: {}", e);
                        panic!("TLS upgrade error");
                    }
                }
            });
            return Box::pin(async { Ok(Response::new(empty_body())) });
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
