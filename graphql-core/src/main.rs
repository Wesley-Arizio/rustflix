use actix_cors::Cors;
use actix_session::config::PersistentSession;
use actix_session::{storage::RedisSessionStore, Session, SessionMiddleware};
use std::{io::Result, sync::Arc};

use actix_web::cookie::time::Duration;
use actix_web::cookie::Key;
use actix_web::{
    http, route,
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_lab::respond::Html;
use juniper::http::GraphQLRequest;
use schemas::{create_schema, Schema};

use clap::Parser;

use database::connection::PgPool;
use juniper::http::graphiql::graphiql_source;
use send_wrapper::SendWrapper;

mod input;
pub mod mutation;
mod output;
pub mod query;
pub mod schemas;

use core::service::Core;

const SECS_IN_WEEK: i64 = 60 * 60 * 24 * 7;
const SESSION_KEY: &str = "sid";

/// GraphiQL playground UI
#[route("/playground", method = "GET")]
async fn graphql_playground() -> impl Responder {
    Html(graphiql_source("/graphql", None))
}

#[route("/graphql", method = "POST", method = "GET")]
async fn graphql(
    req: HttpRequest,
    session: Session,
    schema: Data<Schema>,
    data: web::Json<GraphQLRequest>,
) -> impl Responder {
    let ctx = Context::new(req, session);
    let response = data.execute(&schema, &ctx).await;
    HttpResponse::Ok().json(response)
}

#[derive(Debug, Parser)]
struct Args {
    /// Port that the API will run at
    #[arg(env = "GRAPHQL_API_PORT")]
    api_port: u16,
    /// Database URL to connect with core database
    #[arg(env = "GRAPHQL_DATABASE_URL")]
    database_url: String,
    /// Auth microservice grpc port
    #[arg(env = "GRAPHQL_AUTH_GRPC_PORT")]
    auth_grpc_port: String,
    /// URL to connect with redis instance
    #[arg(env = "REDIS_SESSION_URL")]
    redis_session_storage_url: String,
    /// Private key for session storage
    #[arg(env = "PRIVATE_SESSION_KEY")]
    session_private_key: String,
}

#[derive(Clone)]
pub struct Context {
    pub request: Option<SendWrapper<HttpRequest>>,
    pub session: Option<SendWrapper<Session>>,
}

impl juniper::Context for Context {}

impl Context {
    pub fn new(req: HttpRequest, session: Session) -> Self {
        Self {
            request: Some(SendWrapper::new(req)),
            session: Some(SendWrapper::new(session)),
        }
    }
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv::dotenv().expect("Could not parse environment variables");
    let args = Args::parse();

    let store = RedisSessionStore::new(args.redis_session_storage_url)
        .await
        .expect("Could not initialize redis instance");

    let pool = PgPool::connect(&args.database_url)
        .await
        .expect("Could not connect to database");
    let core = Core::new(args.auth_grpc_port, pool).await;
    let schema = Arc::new(create_schema(core));
    let app = move || {
        let key = Key::derive_from(args.session_private_key.as_ref());
        let cors = Cors::default()
            .supports_credentials()
            .allowed_headers(vec![http::header::CONTENT_TYPE]);
        let session = SessionMiddleware::builder(store.clone(), key)
            .session_lifecycle(
                PersistentSession::default().session_ttl(Duration::seconds(SECS_IN_WEEK)),
            )
            .cookie_domain(Some("localhost".to_string()))
            .cookie_name(SESSION_KEY.to_string())
            .cookie_http_only(true)
            .cookie_path("/".to_string())
            .cookie_secure(true)
            .build();
        App::new()
            .wrap(session)
            .wrap(cors)
            .app_data(Data::from(Arc::clone(&schema)))
            .service(graphql)
            .service(graphql_playground)
    };

    HttpServer::new(app)
        .bind(("0.0.0.0", args.api_port))?
        .run()
        .await?;

    Ok(())
}
