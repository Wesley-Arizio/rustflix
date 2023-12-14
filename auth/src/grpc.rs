use tonic::{Request, Response, Status};
use grpc_interfaces::auth::{
    auth_server::Auth, CreateCredentialsRequest, CreateCredentialsResponse,
};
use crate::auth::{AuthService, AuthServiceError};
use crate::database::entities::account::{Account, CreateAccountDAO};
use crate::database::traits::Repository;

impl From<AuthServiceError> for Status {
    fn from(value: AuthServiceError) -> Self {
        match value {
            AuthServiceError::InvalidCredentials => Status::unauthenticated("Invalid Credentials"),
            AuthServiceError::InvalidInput { message} => Status::invalid_argument(message),
            AuthServiceError::InternalServerError => Status::unknown("Internal Server Error")
        }
    }
}

#[derive(Debug)]
pub struct GRPCAuthService<R> where R: 'static, R: Repository<Entity = Account, CreateEntityDAO = CreateAccountDAO> + Send + Sync, {
    service: AuthService<R>
}

impl<R> GRPCAuthService<R> where R: 'static, R: Repository<Entity = Account, CreateEntityDAO = CreateAccountDAO> + Send + Sync {
    pub fn new(auth_service: AuthService<R>) -> Self {
        Self {
            service: auth_service,
        }
    }
}

#[tonic::async_trait]
impl<R> Auth for GRPCAuthService<R> where R: 'static, R: Repository<Entity = Account, CreateEntityDAO = CreateAccountDAO> + Send + Sync,  {
    async fn create_credential(
        &self,
        request: Request<CreateCredentialsRequest>,
    ) -> Result<Response<CreateCredentialsResponse>, Status> {
        let input = request.into_inner();
        let id = self.service.create_account(&input.email, &input.password).await.map_err(Status::from)?;
        let response = CreateCredentialsResponse {
            user_id: id,
        };
        Ok(Response::new(response))
    }
}