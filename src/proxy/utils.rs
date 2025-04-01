use http::HeaderName;
use http::Request;
use thiserror::Error;

pub const HEADER_PROXY_CONNECTION: HeaderName = HeaderName::from_static("proxy-connection");

// Validate if incoming request is indeed proxy request
pub fn validate_request<T>(req: &Request<T>) -> Result<(), ProxyRequestError> {
    if !req.headers().contains_key(HEADER_PROXY_CONNECTION) {
        return Err(ProxyRequestError::NoProxyConnectionHeader);
    }
    if req.uri().host().is_none() {
        return Err(ProxyRequestError::RelativeUri);
    }
    Ok(())
}

#[derive(Error, Debug)]
pub enum ProxyRequestError {
    #[error("request uri is relative")]
    RelativeUri,

    #[error("proxy-connection header is absent")]
    NoProxyConnectionHeader,
}
