use super::body::SimpleBody;
use multimap::MultiMap;

#[derive(Debug)]
pub struct Response {
    pub(super) code: u16,
    pub(super) message: String,
    pub(super) headers: MultiMap<String, String>,
    pub(super) body: SimpleBody,
}

impl Response {
    pub fn code(&self) -> u16 {
        self.code
    }

    pub fn message(&self) -> &String {
        &self.message
    }

    pub fn headers(&self) -> &MultiMap<String, String> {
        &self.headers
    }

    pub fn body(&self) -> &SimpleBody {
        &self.body
    }
}
