use futures;
use std::pin::Pin;

use crate::dto::Reqresp;
use thiserror::Error;

pub type DynFuture<T> = Pin<Box<dyn futures::Future<Output = T> + Send>>;

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
