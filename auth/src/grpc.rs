use crate::auth::{AuthService, AuthServiceError};
use grpc_interfaces::auth::{
    auth_server::Auth, AuthenticateRequest, CreateCredentialsRequest, CreateCredentialsResponse,
};
use tonic::{Request, Response, Status};

impl From<AuthServiceError> for Status {
    fn from(value: AuthServiceError) -> Self {
        match value {
            AuthServiceError::InvalidCredentials => Status::unauthenticated("Invalid Credentials"),
            AuthServiceError::InvalidInput { message } => Status::invalid_argument(message),
            AuthServiceError::InternalServerError => Status::unknown("Internal Server Error"),
        }
    }
}

#[derive(Debug)]
pub struct GRPCAuthService {
    service: AuthService,
}

impl GRPCAuthService {
    pub fn new(auth_service: AuthService) -> Self {
        Self {
            service: auth_service,
        }
    }
}

#[tonic::async_trait]
impl Auth for GRPCAuthService {
    async fn create_credential(
        &self,
        request: Request<CreateCredentialsRequest>,
    ) -> Result<Response<CreateCredentialsResponse>, Status> {
        let input = request.into_inner();
        let id = self
            .service
            .create_account(&input.email, &input.password)
            .await
            .map_err(Status::from)?;
        let response = CreateCredentialsResponse { user_id: id };
        Ok(Response::new(response))
    }

    async fn authenticate(
        &self,
        request: Request<AuthenticateRequest>,
    ) -> Result<Response<()>, Status> {
        self.service
            .authenticate(&request.into_inner().session_id)
            .await?;

        Ok(Response::new(()))
    }
}
