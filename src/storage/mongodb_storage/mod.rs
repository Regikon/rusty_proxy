use crate::dto::Reqresp;

use super::storage::ReqrespStorage;
use super::storage::StorageError;
use crate::DynFuture;

use futures::TryStreamExt;
use mongodb::{bson::doc, Client, Collection};

mod dto_bindings;

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
        let reqresps: Collection<dto_bindings::Reqresp> = database.collection(COLLECTION_NAME);
        Box::pin(async move {
            reqresps
                .insert_one(dto_bindings::Reqresp::from(r))
                .await
                .unwrap();
            Ok(())
        })
    }

    fn get_reqresps(&self) -> DynFuture<Result<Vec<Reqresp>, StorageError>> {
        let database = self.client.database(DATABASE_NAME);
        let reqresps: Collection<dto_bindings::Reqresp> = database.collection(COLLECTION_NAME);
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
                result.push(reqresp_doc.into());
            }
            Ok(result)
        })
    }

    fn get_reqresp_by_id(&self, id: &String) -> DynFuture<Result<Option<Reqresp>, StorageError>> {
        let database = self.client.database(DATABASE_NAME);
        let reqresps: Collection<dto_bindings::Reqresp> = database.collection(COLLECTION_NAME);
        let id = id.clone();
        Box::pin(async move {
            let id = bson::oid::ObjectId::parse_str(id).map_err(|_| StorageError::Unknown)?;
            let reqresp = reqresps.find_one(doc! {"_id": id}).await;
            if let Err(_) = reqresp {
                return Err(StorageError::Unknown);
            }
            Ok(reqresp.unwrap().map(|r| r.into()))
        })
    }
}
