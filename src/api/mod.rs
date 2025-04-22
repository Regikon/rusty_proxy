pub mod handlers;

use crate::storage::{mongodb_storage::MongoDbStorage, storage::ReqrespStorage};
use std::sync::Arc;

use super::dto::*;

pub struct AppState {
    db: Arc<MongoDbStorage>,
}

impl AppState {
    pub fn db(&self) -> Arc<MongoDbStorage> {
        self.db.clone()
    }

    pub fn new(db: Arc<MongoDbStorage>) -> Self {
        AppState { db }
    }
}
