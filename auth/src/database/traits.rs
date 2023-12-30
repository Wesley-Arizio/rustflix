use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Internal server error")]
    DatabaseError {
        #[source]
        source: Box<dyn std::error::Error + 'static>,
    },
}

#[async_trait::async_trait]
pub trait Repository {
    type Entity;
    type CreateEntityDAO;
    async fn create(&self, entity: &Self::CreateEntityDAO)
        -> Result<Self::Entity, RepositoryError>;
    async fn exists(&self, identifier: &str) -> Result<bool, RepositoryError>;
}

#[cfg(not(feature = "integration"))]
#[cfg(test)]
use mockall::*;

#[cfg(not(feature = "integration"))]
#[cfg(test)]
mock! {
    pub FakeRepository {}

    #[async_trait::async_trait]
    impl Repository for FakeRepository {
        type Entity = crate::database::entities::account::Account;
        type CreateEntityDAO = crate::database::entities::account::CreateAccountDAO;
        async fn create(&self, entity: &<crate::database::traits::MockFakeRepository as Repository>::CreateEntityDAO )-> Result<<crate::database::traits::MockFakeRepository as Repository>::Entity, RepositoryError>;
        async fn exists(&self, identifier: &str) -> Result<bool, RepositoryError>;
    }
}
