use bytes::Bytes;
use multimap::MultiMap;

pub enum SimpleBody {
    Blob(Bytes),
    UrlEncoded(MultiMap<String, String>),
}
