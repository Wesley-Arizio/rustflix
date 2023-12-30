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

#[cfg(feature = "integration")]
#[cfg(test)]
mod tests {
    use crate::database::database_setup;
    use dotenv;

    #[tokio::test]
    async fn test_db_connection() {
        dotenv::dotenv().ok();
        let uri =
            std::env::var("TEST_AUTH_DATABASE_URI").expect("TEST_AUTH_DATABASE_URI must be set");
        let db_name =
            std::env::var("TEST_AUTH_DATABASE_NAME").expect("TEST_AUTH_DATABASE_NAME must be set");

        let db = database_setup(&uri, &db_name).await;

        assert_eq!(db.name(), db_name);
    }
}
