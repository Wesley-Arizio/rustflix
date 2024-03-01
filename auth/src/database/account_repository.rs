use super::{
    entities::account::{Account, CreateAccountDAO},
    traits::{Repository, RepositoryError},
};

// use mongodb::options::FindOptions;
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

    async fn try_get(&self, identifier: &str) -> Result<Option<Account>, RepositoryError> {
        Ok(self
            .collection
            .find_one(
                doc! { "email": identifier },
                None, // Some(FindOptions { limit: Some(1i64) }),
            )
            .await?)
    }
}

#[cfg(feature = "integration")]
#[cfg(test)]
mod tests {
    use crate::database::account_repository::AccountRepository;
    use crate::database::database_setup;
    use crate::database::entities::account::CreateAccountDAO;
    use crate::database::traits::Repository;
    use dotenv;

    #[tokio::test]
    async fn test_account_repository() {
        dotenv::dotenv().ok();
        let uri =
            std::env::var("TEST_AUTH_DATABASE_URI").expect("TEST_AUTH_DATABASE_URI must be set");
        let db_name =
            std::env::var("TEST_AUTH_DATABASE_NAME").expect("TEST_AUTH_DATABASE_NAME must be set");

        let db = database_setup(&uri, &db_name).await;

        // create account
        let repo = AccountRepository::new(db.collection("credentials"));

        let res = repo
            .create(&CreateAccountDAO {
                email: "test@gmail.com".to_string(),
                password: "123456".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(res.email, "test@gmail.com");
        assert_eq!(res.password, "123456");

        // verify if it exists
        let exists = repo.exists(&res.email).await.unwrap();
        assert!(exists);

        let exists = repo.exists("any other id").await.unwrap();
        assert!(!exists);
    }
}
