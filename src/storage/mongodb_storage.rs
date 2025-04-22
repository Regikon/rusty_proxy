use crate::dto::Reqresp;

use super::storage::DynFuture;
use super::storage::ReqrespStorage;
use super::storage::StorageError;

use futures::TryStreamExt;
use mongodb::{bson::doc, Client, Collection};

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
    fn add_reqresp(&self, r: Reqresp) -> DynFuture<Result<(), StorageError>> {
        let database = self.client.database(DATABASE_NAME);
        let reqresps: Collection<Reqresp> = database.collection(COLLECTION_NAME);
        Box::pin(async move {
            reqresps.insert_one(r).await.unwrap();
            Ok(())
        })
    }

    fn get_reqresps(&self) -> DynFuture<Result<Vec<Reqresp>, StorageError>> {
        let database = self.client.database(DATABASE_NAME);
        let reqresps: Collection<Reqresp> = database.collection(COLLECTION_NAME);
        Box::pin(async move {
            let cursor = reqresps.find(doc! {}).await;
            if let Err(_) = cursor {
                return Err(StorageError::Unknown);
            }
            let mut cursor = cursor.unwrap();
            let mut result = Vec::new();
            while let Some(reqresp_doc) =
                cursor.try_next().await.map_err(|_| StorageError::Unknown)?
            {
                result.push(reqresp_doc);
            }
            Ok(result)
        })
    }
}
