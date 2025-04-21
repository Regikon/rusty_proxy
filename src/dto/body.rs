use bytes::Bytes;
use multimap::MultiMap;

#[derive(Debug)]
pub enum SimpleBody {
    Blob(Bytes),
    UrlEncoded(MultiMap<String, String>),
}
