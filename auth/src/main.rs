use grpc_interfaces::auth::auth_server::AuthServer;
use tonic::transport::Server;

use auth::AuthService;

use clap::Parser;

mod auth;

use mongodb::{bson::doc, options::ClientOptions, Client, Collection};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize, Serialize)]
struct Account {
    id: String,
    email: String,
    password: String,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    dotenv::dotenv().unwrap();
    let args = Cli::parse();

    let mongodb_url = format!(
        "mongodb://{}:{}@{}:{}",
        args.mongodb_username, args.mongodb_password, args.mongodb_host, args.mongodb_port
    );

    let mut client_options = ClientOptions::parse(&mongodb_url)
        .await
        .map_err(|e| e.to_string())?;

    let address = args
        .grpc_port
        .parse()
        .expect("Could not parse auth gprc_port to a SocketAddr");
    let auth_service = AuthService::default();

    Server::builder()
        .add_service(AuthServer::new(auth_service))
        .serve(address)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
