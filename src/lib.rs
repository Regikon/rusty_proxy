use std::pin::Pin;

pub mod api;
pub mod config;
pub mod dto;
pub mod proxy;
pub mod scanner;
pub mod storage;

pub type DynFuture<T> = Pin<Box<dyn futures::Future<Output = T> + Send>>;
