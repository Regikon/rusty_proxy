use std::string::FromUtf8Error;

use super::{body::SimpleBody, request::Request};
use bytes::Bytes;
use multimap::MultiMap;
use url_encoded_data::UrlEncodedData;

type HyperBody = Bytes;
type HyperRequest = (http::request::Parts, HyperBody);

impl From<HyperRequest> for Request {
    fn from(req: HyperRequest) -> Self {
        let (parts, body) = req;

        let method = parts.method.to_string();
        let path = parts.uri.path().to_string();
        let query_params = parts.uri.query().map(|query| {
            UrlEncodedData::from(query)
                .as_string_pairs()
                .iter()
                .cloned()
                .collect()
        });

        let cookies = parts
            .headers
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
                        return (cookie_pair[0].to_string(), cookie_pair[1].to_string());
                    })
                    .collect()
            });

        let headers = parts
            .headers
            .iter()
            .filter(|(name, _)| *name != http::header::COOKIE)
            .map(|(name, value)| (name.to_string(), value.to_str().unwrap().to_string()))
            .collect();

        let body = SimpleBody::Blob(body_as_blob(body));

        Request {
            method,
            path,
            query_params,
            headers,
            cookies,
            body,
        }
    }
}

fn body_as_blob(b: HyperBody) -> Bytes {
    b
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
