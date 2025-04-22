use multimap::MultiMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SimpleBody {
    Blob(Vec<u8>),
    UrlEncoded(MultiMap<String, String>),
}
