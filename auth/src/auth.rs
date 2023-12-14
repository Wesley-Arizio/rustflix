use std::error::Error;

use grpc_interfaces::auth::{
    auth_server::Auth, CreateCredentialsRequest, CreateCredentialsResponse,
};

use crate::database::entities::account::{Account, CreateAccountDAO};

use super::database::traits::{Repository, RepositoryError};

impl From<RepositoryError> for Status {
    fn from(value: RepositoryError) -> Self {
        if let Some(source) = value.source() {
            eprintln!("{:?}", source);
        }
        Self::internal("Internal Server Error")
    }
}

use tonic::{Code, Request, Response, Status};

#[derive(Debug)]
pub struct AuthService<R>
where
    R: Repository<Entity = Account, CreateEntityDAO=CreateAccountDAO> + Send + Sync,
{
    account_repository: R,
}

impl<R> AuthService<R>
where
    R: Repository<Entity = Account, CreateEntityDAO = CreateAccountDAO> + Send + Sync,
{
    pub fn new(repository: R) -> Self {
        Self {
            account_repository: repository,
        }
    }
}

#[tonic::async_trait]
impl<R> Auth for AuthService<R>
where
    R: 'static,
    R: Repository<Entity = Account, CreateEntityDAO=CreateAccountDAO> + Send + Sync,
{
    async fn create_credential(
        &self,
        request: Request<CreateCredentialsRequest>,
    ) -> Result<Response<CreateCredentialsResponse>, Status> {
        let inner = request.into_inner();

        let exists = &self.account_repository.exists(&inner.email).await?;

        if *exists {
            return Err(Status::new(Code::Unknown, "Invalid Credentials"));
        };

        let dto = CreateAccountDAO {
            email: inner.email,
            password: inner.password,
        };

        let res = self.account_repository.create(&dto).await?;

        Ok(Response::new(CreateCredentialsResponse {
            user_id: res._id,
        }))
    }
}
