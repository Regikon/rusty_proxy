use crate::dto::Reqresp;
use thiserror::Error;

pub trait ReqrespStorage {
    async fn add_reqresp(&mut self, r: Reqresp) -> Result<(), StorageError>;
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("unknown storage error")]
    Unknown,
}
