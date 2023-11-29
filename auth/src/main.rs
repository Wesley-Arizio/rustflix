use grpc_interfaces::auth::auth_server::AuthServer;
use tonic::transport::Server;

use auth::AuthService;

use clap::Parser;

mod auth;

#[derive(Parser, Debug)]
struct Cli {
    /// grpc port for Auth server
    #[arg(short, env)]
    grpc_port: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().unwrap();
    let args = Cli::parse();
    let address = args
        .grpc_port
        .parse()
        .expect("Could not parse auth gprc_port to a ScoketAddr");
    let auth_service = AuthService::default();

    Server::builder()
        .add_service(AuthServer::new(auth_service))
        .serve(address)
        .await?;

    Ok(())
}
