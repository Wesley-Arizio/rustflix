use crate::auth::AuthService;
use crate::grpc::GRPCAuthService;
use auth_database::connection::PgPool;
use grpc_interfaces::auth::auth_server::AuthServer;
use std::error::Error;
use tonic::transport::Server;

pub async fn run_server(server_address: &str, database_url: &str) -> Result<(), Box<dyn Error>> {
    let address = server_address.parse()?;

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Could not connect to database");

    let auth_service = AuthService::new(pool);
    let auth_service = GRPCAuthService::new(auth_service);

    Server::builder()
        .add_service(AuthServer::new(auth_service))
        .serve(address)
        .await?;

    Ok(())
}
