use super::{request::Request, Response};

// Request and resulted response
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Reqresp {
    pub id: String,
    pub req: Request,
    pub resp: Response,
}

impl Reqresp {
    pub fn new(req: Request, resp: Response) -> Self {
        Reqresp {
            id: String::new(),
            req,
            resp,
        }
    }
}
