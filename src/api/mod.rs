pub mod handlers;

use crate::scanner::SimpleScanner;
use crate::storage::mongodb_storage::MongoDbStorage;
use std::sync::Arc;

use super::dto::*;

pub struct AppState {
    db: Arc<MongoDbStorage>,
    scanner: SimpleScanner,
}

impl AppState {
    pub fn db(&self) -> Arc<MongoDbStorage> {
        self.db.clone()
    }

    pub fn scanner(&self) -> SimpleScanner {
        self.scanner.clone()
    }

    pub fn new(db: Arc<MongoDbStorage>, scanner: SimpleScanner) -> Self {
        AppState { db, scanner }
    }
}
