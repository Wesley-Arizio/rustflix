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

    let client_options = ClientOptions::parse(&args.mongodb_url)
        .await
        .expect("error parsing client options");

    let client = Client::with_options(client_options).expect("error initializing mongodb client");
    let collection = client
        .database(&args.database_name)
        .collection("credentials");
    let account_repository = AccountRepository::new(collection);
    let auth_service = AuthService::new(account_repository);
    let auth_service = GRPCAuthService::new(auth_service);

    Server::builder()
        .add_service(AuthServer::new(auth_service))
        .serve(address)
        .await?;

    Ok(())
}
