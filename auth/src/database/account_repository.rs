use super::{
    entities::account::{Account, CreateAccountDAO},
    traits::{Repository, RepositoryError},
};

use mongodb::{bson::doc, Collection};

use tonic::async_trait;

impl From<mongodb::error::Error> for RepositoryError {
    fn from(value: mongodb::error::Error) -> Self {
        RepositoryError::DatabaseError { source: value.kind }
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
    type CreateEntityDAO = CreateAccountDAO;

    async fn create(&self, entity: &CreateAccountDAO) -> Result<Account, RepositoryError> {
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

#[cfg(tests)]
mod tests {
    #[test]
    fn test_account_repository() {
        // create account
        // verify if the newly created account exists
        // verify if a non-existing account exists
    }
}
