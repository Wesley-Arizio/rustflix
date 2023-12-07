use super::traits::Repository;
use mongodb::{
    bson::{doc, Uuid},
    Collection,
};
use serde::{Deserialize, Serialize};
use tonic::async_trait;

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub _id: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountDTO {
    pub email: String,
    pub password: String,
}

impl From<&AccountDTO> for Account {
    fn from(value: &AccountDTO) -> Self {
        Self {
            _id: Uuid::new().to_string(),
            email: value.email.to_owned(),
            password: value.password.to_owned(),
        }
    }
}

pub struct AccountRepository {
    collection: Collection<Account>,
}

impl AccountRepository {
    pub fn new(collection: Collection<Account>) -> Self {
        Self { collection }
    }
}

#[async_trait]
impl Repository for AccountRepository {
    type Error = mongodb::error::Error;
    type Entity = Account;
    type CreateEntityDTO = AccountDTO;

    async fn create(&self, entity: &AccountDTO) -> Result<Account, Self::Error> {
        let account = entity.into();
        let _ = self.collection.insert_one(&account, None).await?;

        Ok(account)
    }

    async fn exists(&self, identifier: &str) -> Result<bool, Self::Error> {
        let result = self
            .collection
            .count_documents(doc! { "email": identifier }, None)
            .await?;

        match result {
            0 => Ok(false),
            _ => Ok(true),
        }
    }
}
