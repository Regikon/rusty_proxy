use super::body::SimpleBody;
use multimap::MultiMap;
use std::collections::HashMap;

pub struct Request<'a> {
    pub(super) method: String,
    pub(super) path: String,
    pub(super) query_params: Option<MultiMap<String, String>>,
    // Header value might be a non valid utf-8 so we have to use byte slice
    pub(super) headers: MultiMap<String, &'a str>,
    pub(super) cookies: Option<HashMap<&'a str, &'a str>>,
    pub(super) body: SimpleBody,
}

impl Request<'_> {
    pub fn method(&self) -> &String {
        &self.method
    }

    pub fn path(&self) -> &String {
        &self.path
    }

    pub fn query_params(&self) -> &Option<MultiMap<String, String>> {
        &self.query_params
    }

    pub fn headers(&self) -> &MultiMap<String, &str> {
        &self.headers
    }

    pub fn cookies(&self) -> &Option<HashMap<&str, &str>> {
        &self.cookies
    }

    pub fn body(&self) -> &SimpleBody {
        &self.body
    }
}
