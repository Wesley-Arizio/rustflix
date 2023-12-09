use super::{
    entities::account::{Account, AccountDTO},
    traits::{Repository, RepositoryError},
};

use mongodb::{bson::doc, error::Error, Collection};

use tonic::async_trait;

impl From<Error> for RepositoryError {
    fn from(value: Error) -> Self {
        match value {
            Error { kind, .. } => RepositoryError::DatabaseError { source: kind },
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
    type Entity = Account;
    type CreateEntityDTO = AccountDTO;

    async fn create(&self, entity: &AccountDTO) -> Result<Account, RepositoryError> {
        let account = entity.into();
        let _ = self.collection.insert_one(&account, None).await?;

        Ok(account)
    }

    async fn exists(&self, identifier: &str) -> Result<bool, RepositoryError> {
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
