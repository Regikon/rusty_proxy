use crate::dto::Reqresp;
use thiserror::Error;

use crate::DynFuture;

pub trait ReqrespStorage {
    fn add_reqresp(&self, r: Reqresp) -> DynFuture<Result<(), StorageError>>;
    fn get_reqresps(&self) -> DynFuture<Result<Vec<Reqresp>, StorageError>>;
    fn get_reqresp_by_id(&self, id: &String) -> DynFuture<Result<Option<Reqresp>, StorageError>>;
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("unknown storage error")]
    Unknown,
}
