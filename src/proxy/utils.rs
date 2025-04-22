use http::HeaderName;
use http::Request;
use hyper::Uri;
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

// Clean proxy request to make it valid non-proxy request
pub fn clean_request<T>(mut req: http::Request<T>) -> Request<T> {
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

// Get host from request
pub fn extract_host<T>(req: &http::Request<T>) -> Option<String> {
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
pub fn parse_host_header(host: &String, fallback_port: u16) -> Result<(String, u16), String> {
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
