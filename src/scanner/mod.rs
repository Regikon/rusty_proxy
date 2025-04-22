use crate::dto::Request;
use crate::proxy::client::Client;
use crate::proxy::utils;
use crate::proxy::BodyType;
use crate::DynFuture;
use thiserror::Error;

pub trait Scanner {
    fn resend_request(
        &self,
        req: Request,
    ) -> DynFuture<Result<http::Response<BodyType>, ScannerError>>;
}

#[derive(Clone)]
pub struct SimpleScanner {}

impl Scanner for SimpleScanner {
    fn resend_request(
        &self,
        req: Request,
    ) -> DynFuture<Result<http::Response<BodyType>, ScannerError>> {
        Box::pin(async move {
            let (req, is_https) = req.into();
            let full_host = utils::extract_host(&req).unwrap();
            let (host, port) = utils::parse_host_header(
                &full_host,
                match is_https {
                    false => 80,
                    true => 443,
                },
            )
            .unwrap();
            let resp = Client::send_request(req, host, port, is_https)
                .await
                .map_err(|_| ScannerError::RequestFailed)?;
            Ok(resp)
        })
    }
}

#[derive(Error, Debug)]
pub enum ScannerError {
    #[error("unknown scanner error")]
    Unknown,

    #[error("failed to send request to remote host")]
    RequestFailed,

    #[error("failed to download the response body")]
    BodyLoadFailed,
}
