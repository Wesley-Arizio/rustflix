use grpc_interfaces::auth::auth_server::AuthServer;
use tonic::transport::Server;

use auth::AuthService;

use clap::Parser;

mod auth;
mod database;

use database::account_repository::AccountRepository;

use mongodb::{options::ClientOptions, Client};
#[derive(Parser, Debug)]
struct Cli {
    /// grpc port for Auth server
    #[arg(short, env = "AUTH_GRPC_PORT")]
    grpc_port: String,

    #[arg(env = "AUTH_MONGODB_URL")]
    mongodb_url: String,
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
    let collection = client.database("auth").collection("credentials");
    let account_repository = AccountRepository::new(collection);
    let auth_service = AuthService::new(account_repository);

    Server::builder()
        .add_service(AuthServer::new(auth_service))
        .serve(address)
        .await?;

    Ok(())
}
