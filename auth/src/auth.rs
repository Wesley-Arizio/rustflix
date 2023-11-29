use grpc_interfaces::auth::{
    auth_server::Auth, CreateCredentialsRequest, CreateCredentialsResponse,
};

use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct AuthService;

#[tonic::async_trait]
impl Auth for AuthService {
    async fn create_credential(
        &self,
        request: Request<CreateCredentialsRequest>,
    ) -> Result<Response<CreateCredentialsResponse>, Status> {
        let inner = request.into_inner();

        println!("email: {}, password: {}", inner.email, inner.password);
        Ok(Response::new(CreateCredentialsResponse {
            user_id: String::from("user id"),
        }))
    }
}
