use std::{io::Result, sync::Arc};

use actix_web::{
    route,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use juniper::http::GraphQLRequest;
use schemas::{create_schema, Schema};

use grpc_interfaces::auth::auth_client::AuthClient;

use clap::Parser;

pub mod mutation;
pub mod query;
pub mod schemas;

use sqlx::{PgPool, Pool, Postgres};

#[route("/graphql", method = "POST", method = "GET")]
async fn graphql(
    schema: web::Data<Schema>,
    state: web::Data<AppState>,
    data: web::Json<GraphQLRequest>,
) -> impl Responder {
    let response = data.execute(&schema, &state).await;
    HttpResponse::Ok().json(response)
}

#[derive(Debug, Parser)]
struct Args {
    /// Port that the API will run at
    #[arg(env = "GRAPHQL_API_PORT")]
    api_port: u16,

    #[arg(env = "GRAPHQL_DATABASE_URL")]
    database_url: String,
}

pub struct AppState {
    db: Pool<Postgres>,
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv::dotenv().expect("Could not parse environment variables");
    let args = Args::parse();
    let auth_client = AuthClient::connect("http://[::1]:50051")
        .await
        .expect("Could not connect to auth grpc client");

    let pool = PgPool::connect(&args.database_url)
        .await
        .expect("Could not connect to database");

    let schema = Arc::new(create_schema(auth_client));
    let app = move || {
        App::new()
            .app_data(Data::from(Arc::clone(&schema)))
            .app_data(Data::new(AppState { db: pool.clone() }))
            .service(graphql)
    };

    HttpServer::new(app)
        .bind(("0.0.0.0", args.api_port))?
        .run()
        .await?;

    Ok(())
}
