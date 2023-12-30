use mongodb::options::ClientOptions;
use mongodb::Client;

pub mod account_repository;
pub mod entities;
pub mod traits;

pub async fn database_setup(uri: &str, database_name: &str) -> mongodb::Database {
    let client_options = ClientOptions::parse(uri)
        .await
        .expect("error parsing client options");

    let client = Client::with_options(client_options).expect("error initializing mongodb client");

    client.database(database_name)
}
