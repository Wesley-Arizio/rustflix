use grpc_interfaces::auth::{
    auth_server::Auth, CreateCredentialsRequest, CreateCredentialsResponse,
};

use crate::database::account_repository::{Account, AccountDTO};

use super::database::traits::Repository;

use tonic::{Code, Request, Response, Status};

#[derive(Debug)]
pub struct AuthService<R>
where
    R: Repository<Entity = Account, CreateEntityDTO = AccountDTO> + Send + Sync,
{
    account_repository: R,
}

impl<R> AuthService<R>
where
    R: Repository<Entity = Account, CreateEntityDTO = AccountDTO> + Send + Sync,
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
    R: Repository<Entity = Account, CreateEntityDTO = AccountDTO> + Send + Sync,
    R::Error: std::fmt::Display + std::fmt::Debug,
{
    async fn create_credential(
        &self,
        request: Request<CreateCredentialsRequest>,
    ) -> Result<Response<CreateCredentialsResponse>, Status> {
        let inner = request.into_inner();

        let exists = &self
            .account_repository
            .exists(&inner.email)
            .await
            .map_err(|_| Status::internal("error creating account"))?;

        if *exists {
            return Err(Status::new(Code::Unknown, "Invalid Credentials"));
        };

        let dto = AccountDTO {
            email: inner.email,
            password: inner.password,
        };

        let res = self
            .account_repository
            .create(&dto)
            .await
            .map_err(|_| Status::internal("Internal server error"))?;

        Ok(Response::new(CreateCredentialsResponse {
            user_id: res._id,
        }))
    }
}
