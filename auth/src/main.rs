use grpc_interfaces::auth::auth_server::AuthServer;
use tonic::transport::Server;

use grpc::GRPCAuthService;

use clap::Parser;

mod auth;
mod database;
mod grpc;
mod password_helper;

use database::account_repository::AccountRepository;

use crate::auth::AuthService;
use crate::database::database_setup;
use mongodb::{options::ClientOptions, Client};

#[derive(Parser, Debug)]
struct Cli {
    /// grpc port for Auth server
    #[arg(short, env = "AUTH_GRPC_PORT")]
    grpc_port: String,

    #[arg(env = "AUTH_MONGODB_URL")]
    mongodb_url: String,

    #[arg(env = "AUTH_DATABASE_NAME")]
    database_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().unwrap();
    let args = Cli::parse();

    let address = args
        .grpc_port
        .parse()
        .expect("Could not parse socket address with given grpc port");

    let db = database_setup(&args.mongodb_url, &args.database_name).await;

    let account_repository = AccountRepository::new(db.collection("credentials"));
    let auth_service = AuthService::new(account_repository);
    let auth_service = GRPCAuthService::new(auth_service);

    Server::builder()
        .add_service(AuthServer::new(auth_service))
        .serve(address)
        .await?;

    Ok(())
}
