use super::{body::SimpleBody, request::Request};
use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use url_encoded_data::UrlEncodedData;

type HyperBody = BoxBody<Bytes, hyper::Error>;
type HyperRequest = hyper::Request<HyperBody>;

impl From<&HyperRequest> for Request<'_> {
    fn from(req: &HyperRequest) -> Self {
        let method = req.method().to_string();
        let path = req.uri().path().to_string();
        let query_params = req.uri().query().map(|query| {
            UrlEncodedData::from(query)
                .as_string_pairs()
                .iter()
                .cloned()
                .collect()
        });
        let headers = req
            .headers()
            .iter()
            .filter(|(&name, _)| name != http::header::COOKIE)
            .map(|(&name, &value)| (name.clone().to_string(), value.clone().to_str().unwrap()))
            .collect();

        let cookies = req
            .headers()
            .get(http::header::COOKIE)
            .map(|&cookie_value| {
                cookie_value
                    .clone()
                    .to_str()
                    .unwrap()
                    .split(',')
                    .map(|cookie| {
                        let cookie_pair: Vec<&str> = cookie.split('=').collect();
                        return (cookie_pair[0], cookie_pair[1]);
                    })
                    .collect()
            });

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
