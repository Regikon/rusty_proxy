use crate::dto::Reqresp;

use super::storage::ReqrespStorage;
use super::storage::StorageError;
use std::sync::{Arc, Mutex};

use mongodb::{Client, Collection};

const DATABASE_NAME: &str = "rusty_proxy";
const COLLECTION_NAME: &str = "reqresp";

#[derive(Clone)]
pub struct MongoDbStorage {
    client: Client,
}

impl MongoDbStorage {
    pub fn new(client: Client) -> Self {
        MongoDbStorage { client }
    }
}

impl ReqrespStorage for MongoDbStorage {
    async fn add_reqresp(&mut self, r: Reqresp) -> Result<(), StorageError> {
        let database = self.client.database(DATABASE_NAME);
        let reqresps: Collection<Reqresp> = database.collection(COLLECTION_NAME);
        let _ = reqresps.insert_one(r).await;
        Ok(())
    }
}
