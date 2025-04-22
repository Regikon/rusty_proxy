use crate::dto::Reqresp;
use crate::dto::Request;
use crate::dto::SimpleBody;
use crate::proxy::client::Client;
use crate::proxy::utils;
use crate::proxy::BodyType;
use crate::DynFuture;
use http_body_util::BodyExt;
use log::debug;
use thiserror::Error;

// Url encoded string vulnerable'"><img src onerror=alert()>
// Non url-encoded crashes internal asserts of http protocol
const XSS_STRING: &str = r#"vulnerable%27%22%3E%3Cimg%20src%20onerror%3Dalert%28%29%3E"#;
const ORIGINAL_XSS_STRING: &str = r#"vulnerable'"><img src onerror=alert()>"#;

pub trait Scanner {
    fn resend_request(
        &self,
        req: Request,
    ) -> DynFuture<Result<http::Response<BodyType>, ScannerError>>;

    fn scan_xss(&self, reqresp: Reqresp) -> DynFuture<Result<Vec<String>, ScannerError>>;
}

#[derive(Clone)]
pub struct SimpleScanner {}

async fn resend_request_internal(req: Request) -> Result<http::Response<BodyType>, ScannerError> {
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
}

impl Scanner for SimpleScanner {
    fn resend_request(
        &self,
        req: Request,
    ) -> DynFuture<Result<http::Response<BodyType>, ScannerError>> {
        Box::pin(async move { resend_request_internal(req).await })
    }

    fn scan_xss(&self, reqresp: Reqresp) -> DynFuture<Result<Vec<String>, ScannerError>> {
        Box::pin(async move {
            let req = reqresp.req;
            let mut result = Vec::new();
            if let Some(query_params) = req.query_params() {
                for (key, _) in query_params.iter() {
                    let mut req = req.clone();
                    let param_value = req
                        .query_params_mut()
                        .as_mut()
                        .unwrap()
                        .get_mut(key)
                        .unwrap();
                    param_value.clear();
                    param_value.push_str(XSS_STRING);
                    debug!("Scanning with request: {:?}", req);
                    let response_body = resend_request_internal(req)
                        .await?
                        .into_body()
                        .collect()
                        .await
                        .map_err(|_| ScannerError::BodyLoadFailed)?
                        .to_bytes();
                    // Generally, the response is not a valid ascii, so
                    // we should scan with byte scanning
                    if let Some(_) =
                        response_body
                            .to_vec()
                            .windows(XSS_STRING.len())
                            .position(|window| {
                                window == XSS_STRING.as_bytes()
                                    || window == ORIGINAL_XSS_STRING.as_bytes()
                            })
                    {
                        result.push(key.clone());
                    }
                }

                if let SimpleBody::UrlEncoded(b) = req.body() {
                    for (key, _) in b {
                        let mut req = req.clone();
                        let param_value = match req.body_mut() {
                            SimpleBody::Blob(_) => break,
                            SimpleBody::UrlEncoded(copied_b) => copied_b.get_mut(key).unwrap(),
                        };
                        param_value.clear();
                        param_value.push_str(ORIGINAL_XSS_STRING);
                        debug!("Scanning with request: {:?}", req);
                        let response_body = resend_request_internal(req)
                            .await?
                            .into_body()
                            .collect()
                            .await
                            .map_err(|_| ScannerError::BodyLoadFailed)?
                            .to_bytes();
                        // Same here
                        if let Some(_) =
                            response_body
                                .to_vec()
                                .windows(XSS_STRING.len())
                                .position(|window| {
                                    window == XSS_STRING.as_bytes()
                                        || window == ORIGINAL_XSS_STRING.as_bytes()
                                })
                        {
                            result.push(key.clone());
                        }
                    }
                }
            }
            Ok(result)
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
