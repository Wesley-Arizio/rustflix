use crate::auth::{AuthServiceError, AuthServiceTrait};
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
pub struct GRPCAuthService<T: AuthServiceTrait> {
    service: T,
}

impl<T> GRPCAuthService<T>
where
    T: AuthServiceTrait + 'static,
{
    pub fn new(service: T) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl<T> Auth for GRPCAuthService<T>
where
    T: AuthServiceTrait + 'static,
{
    async fn create_credential(
        &self,
        request: Request<CreateCredentialsRequest>,
    ) -> Result<Response<CreateCredentialsResponse>, Status> {
        let input = request.into_inner();
        let id = self
            .service
            .create_account(input.email, input.password)
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
            .authenticate(request.into_inner().session_id)
            .await?;

        Ok(Response::new(()))
    }
}

#[cfg(test)]
mod test {
    use crate::auth::{AuthServiceError, MockAuthService};
    use crate::grpc::GRPCAuthService;
    use grpc_interfaces::auth::auth_server::Auth;
    use grpc_interfaces::auth::{AuthenticateRequest, CreateCredentialsRequest};
    use mockall::predicate::eq;
    use tonic::{Code, Request};

    #[tokio::test]
    async fn create_account_success() {
        let mut mock = MockAuthService::new();

        mock.expect_create_account()
            .with(eq("test@gmail.com".to_string()), eq("123456".to_string()))
            .returning(|_, _| Ok("id".to_string()))
            .times(1);

        let grpc = GRPCAuthService::new(mock);

        let request = Request::new(CreateCredentialsRequest {
            email: "test@gmail.com".to_string(),
            password: "123456".to_string(),
        });

        let response = grpc.create_credential(request).await.unwrap();
        assert_eq!(response.into_inner().user_id, "id");
    }

    #[tokio::test]
    async fn create_account_invalid_email() {
        let mut mock = MockAuthService::new();

        mock.expect_create_account()
            .with(eq("test.com".to_string()), eq("123456".to_string()))
            .returning(|_, _| {
                Err(AuthServiceError::InvalidInput {
                    message: "invalid email".to_string(),
                })
            })
            .times(1);

        let grpc = GRPCAuthService::new(mock);

        let request = Request::new(CreateCredentialsRequest {
            email: "test.com".to_string(),
            password: "123456".to_string(),
        });

        let response = grpc.create_credential(request).await.unwrap_err();
        assert_eq!(response.message(), "invalid email");
        assert_eq!(response.code(), Code::InvalidArgument);
    }

    #[tokio::test]
    async fn create_account_invalid_credentials() {
        let mut mock = MockAuthService::new();

        mock.expect_create_account()
            .with(eq("test@gmail.com".to_string()), eq("123456".to_string()))
            .returning(|_, _| Err(AuthServiceError::InvalidCredentials))
            .times(1);

        let grpc = GRPCAuthService::new(mock);

        let request = Request::new(CreateCredentialsRequest {
            email: "test@gmail.com".to_string(),
            password: "123456".to_string(),
        });

        let response = grpc.create_credential(request).await.unwrap_err();
        assert_eq!(response.message(), "Invalid Credentials");
        assert_eq!(response.code(), Code::Unauthenticated);
    }

    #[tokio::test]
    async fn create_account_internal_server_error() {
        let mut mock = MockAuthService::new();

        mock.expect_create_account()
            .with(eq("test@gmail.com".to_string()), eq("123456".to_string()))
            .returning(|_, _| Err(AuthServiceError::InternalServerError))
            .times(1);

        let grpc = GRPCAuthService::new(mock);

        let request = Request::new(CreateCredentialsRequest {
            email: "test@gmail.com".to_string(),
            password: "123456".to_string(),
        });

        let response = grpc.create_credential(request).await.unwrap_err();
        assert_eq!(response.message(), "Internal Server Error");
        assert_eq!(response.code(), Code::Unknown);
    }

    #[tokio::test]
    async fn test_authenticate_success() {
        let mut mock = MockAuthService::new();

        mock.expect_authenticate()
            .with(eq("84c36fe0-1b4d-41d1-8968-9d5af5883537".to_string()))
            .returning(|_| Ok(()))
            .times(1);

        let grpc = GRPCAuthService::new(mock);
        let request = Request::new(AuthenticateRequest {
            session_id: "84c36fe0-1b4d-41d1-8968-9d5af5883537".to_string(),
        });
        let response = grpc.authenticate(request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_authenticate_invalid_credentials() {
        let mut mock = MockAuthService::new();

        mock.expect_authenticate()
            .with(eq("84c36fe0-1b4d-41d1-8968-9d5af5883537".to_string()))
            .returning(|_| Err(AuthServiceError::InvalidCredentials))
            .times(1);

        let grpc = GRPCAuthService::new(mock);
        let request = Request::new(AuthenticateRequest {
            session_id: "84c36fe0-1b4d-41d1-8968-9d5af5883537".to_string(),
        });
        let response = grpc.authenticate(request).await.unwrap_err();
        assert_eq!(response.code(), Code::Unauthenticated);
        assert_eq!(response.message(), "Invalid Credentials");
    }

    #[tokio::test]
    async fn test_authenticate_invalid_input() {
        let mut mock = MockAuthService::new();

        mock.expect_authenticate()
            .with(eq("84c36fe0-1b4d-41d1-8968 9d5af5883537".to_string()))
            .returning(|_| {
                Err(AuthServiceError::InvalidInput {
                    message: "invalid session id format".to_string(),
                })
            })
            .times(1);

        let grpc = GRPCAuthService::new(mock);
        let request = Request::new(AuthenticateRequest {
            session_id: "84c36fe0-1b4d-41d1-8968 9d5af5883537".to_string(),
        });
        let response = grpc.authenticate(request).await.unwrap_err();
        assert_eq!(response.code(), Code::InvalidArgument);
        assert_eq!(response.message(), "invalid session id format");
    }
}
