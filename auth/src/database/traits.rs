use tonic::async_trait;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Internal server error")]
    DatabaseError {
        #[source]
        source: Box<dyn std::error::Error + 'static>,
    },
}

#[async_trait]
pub trait Repository {
    type Entity;
    type CreateEntityDTO;
    async fn create(&self, entity: &Self::CreateEntityDTO)
        -> Result<Self::Entity, RepositoryError>;
    async fn exists(&self, identifier: &str) -> Result<bool, RepositoryError>;
}
