use crate::auth::AuthService;
use crate::database::account_repository::AccountRepository;
use crate::database::database_setup;
use crate::grpc::GRPCAuthService;
use grpc_interfaces::auth::auth_server::AuthServer;
use std::error::Error;
use tonic::transport::Server;

pub async fn run_server(
    server_address: &str,
    mongodb_url: &str,
    database_name: &str,
) -> Result<(), Box<dyn Error>> {
    let address = server_address.parse()?;

    let db = database_setup(mongodb_url, database_name).await;

    let account_repository = AccountRepository::new(db.collection("credentials"));
    let auth_service = AuthService::new(account_repository);
    let auth_service = GRPCAuthService::new(auth_service);

    Server::builder()
        .add_service(AuthServer::new(auth_service))
        .serve(address)
        .await?;

    Ok(())
}
