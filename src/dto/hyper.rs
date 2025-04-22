use std::string::FromUtf8Error;

use super::{body::SimpleBody, Request, Response};
use bytes::Bytes;
use multimap::MultiMap;
use url_encoded_data::UrlEncodedData;

const MIME_URL_ENCODED: &str = "application/x-www-form-urlencoded";

pub type HyperBody = Bytes;
pub type HyperRequest = (http::request::Parts, HyperBody, bool);
pub type HyperResponse = (http::response::Parts, HyperBody);

impl From<HyperRequest> for Request {
    fn from(req: HyperRequest) -> Self {
        let (parts, body, is_https) = req;
        let http::request::Parts {
            method,
            uri,
            headers,
            ..
        } = parts;

        let method = method.to_string();
        let path = uri.path().to_string();
        let query_params = uri.query().map(|query| {
            UrlEncodedData::from(query)
                .as_string_pairs()
                .iter()
                .cloned()
                .collect()
        });

        let cookies = headers
            .get(http::header::COOKIE)
            .clone()
            .map(|cookie_value| {
                cookie_value
                    .clone()
                    .to_str()
                    .unwrap()
                    .split(';')
                    .map(|cookie| {
                        let cookie_pair: Vec<&str> = cookie.split('=').collect();
                        return (
                            cookie_pair[0].trim().to_string(),
                            cookie_pair.get(1).map_or("".to_string(), |s| s.to_string()),
                        );
                    })
                    .collect()
            });

        let is_urlencoded = headers
            .get(http::header::CONTENT_TYPE)
            .iter()
            .find(|&value| value.to_str().unwrap() == MIME_URL_ENCODED)
            .is_some();

        let headers = headers
            .iter()
            .filter(|(name, _)| *name != http::header::COOKIE)
            .map(|(name, value)| (name.to_string(), value.to_str().unwrap().to_string()))
            .collect();

        let body = parse_body(body, is_urlencoded);

        Request {
            is_https,
            method,
            path,
            query_params,
            headers,
            cookies,
            body,
        }
    }
}

impl From<HyperResponse> for Response {
    fn from((parts, body): HyperResponse) -> Self {
        let http::response::Parts {
            status, headers, ..
        } = parts;
        let is_urlencoded = headers
            .get(http::header::CONTENT_TYPE)
            .iter()
            .find(|&value| value.to_str().unwrap() == MIME_URL_ENCODED)
            .is_some();
        let code = status.clone().into();
        let message = status.canonical_reason().unwrap().to_string();
        let headers = headers
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_str().unwrap().to_string()))
            .collect();

        let body = parse_body(body, is_urlencoded);

        Response {
            code,
            message,
            headers,
            body,
        }
    }
}

fn parse_body(b: HyperBody, is_urlencoded: bool) -> SimpleBody {
    if is_urlencoded {
        let before_parsing = b.clone();
        match body_as_url_encoded(b) {
            Ok(parsed_body) => SimpleBody::UrlEncoded(parsed_body),
            Err(_) => SimpleBody::Blob(body_as_blob(before_parsing)),
        }
    } else {
        SimpleBody::Blob(body_as_blob(b))
    }
}

fn body_as_blob(b: HyperBody) -> Vec<u8> {
    b.to_vec()
}

fn body_as_url_encoded(b: HyperBody) -> Result<MultiMap<String, String>, FromUtf8Error> {
    let bytes = b.to_vec();
    let url_encoded = String::from_utf8(bytes)?;
    let mut map = MultiMap::new();
    UrlEncodedData::from(url_encoded.as_str())
        .as_pairs()
        .into_iter()
        .for_each(|(name, val)| {
            map.insert(name.to_string(), val.to_string());
        });
    return Ok(map);
}
