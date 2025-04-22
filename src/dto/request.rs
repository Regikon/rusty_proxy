use super::body::SimpleBody;
use multimap::MultiMap;
use std::collections::HashMap;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Request {
    pub(super) is_https: bool,
    pub(super) method: String,
    pub(super) path: String,
    pub(super) query_params: Option<MultiMap<String, String>>,
    pub(super) headers: MultiMap<String, String>,
    pub(super) cookies: Option<HashMap<String, String>>,
    pub(super) body: SimpleBody,
}

impl Request {
    pub fn is_https(&self) -> bool {
        self.is_https
    }

    pub fn method(&self) -> &String {
        &self.method
    }

    pub fn path(&self) -> &String {
        &self.path
    }

    pub fn query_params(&self) -> &Option<MultiMap<String, String>> {
        &self.query_params
    }

    pub fn headers(&self) -> &MultiMap<String, String> {
        &self.headers
    }

    pub fn cookies(&self) -> &Option<HashMap<String, String>> {
        &self.cookies
    }

    pub fn body(&self) -> &SimpleBody {
        &self.body
    }
}
