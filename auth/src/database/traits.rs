use tonic::async_trait;

#[async_trait]
pub trait Repository {
    type Error;
    type Entity;
    type CreateEntityDTO;
    async fn create(&self, entity: &Self::CreateEntityDTO) -> Result<Self::Entity, Self::Error>;
    async fn exists(&self, identifier: &str) -> Result<bool, Self::Error>;
}
