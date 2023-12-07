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
    #[arg(short, env)]
    grpc_port: String,

    /// database username
    #[arg(env)]
    mongodb_username: String,

    /// database password
    #[arg(env)]
    mongodb_password: String,

    /// database host
    #[arg(env)]
    mongodb_host: String,

    #[arg(env)]
    mongodb_port: String,
}

use anyhow::{Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().unwrap();
    let args = Cli::parse();

    let mongodb_url = format!(
        "mongodb://{}:{}@{}:{}",
        args.mongodb_username, args.mongodb_password, args.mongodb_host, args.mongodb_port
    );

    let client_options = ClientOptions::parse(&mongodb_url)
        .await
        .context("error parsing client options for mongod bclient")?;

    let client = Client::with_options(client_options).context("configuring mongodb client")?;
    let collection = client.database("auth").collection("credentials");
    let account_repository = AccountRepository::new(collection);
    let auth_service = AuthService::new(account_repository);

    let address = args
        .grpc_port
        .parse()
        .expect("Could not parse auth gprc_port to a ScoketAddr");

    Server::builder()
        .add_service(AuthServer::new(auth_service))
        .serve(address)
        .await
        .context("initializing gRPC server")?;

    Ok(())
}
