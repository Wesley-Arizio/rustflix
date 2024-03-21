use crate::auth::AuthService;
use crate::grpc::GRPCAuthService;
use auth_database::connection::PgPool;
use grpc_interfaces::auth::auth_server::AuthServer;
use std::error::Error;
use std::sync::Arc;
use tonic::transport::Server;

use actix_cors::Cors;
use actix_session::config::PersistentSession;
use actix_session::{storage::RedisSessionStore, Session, SessionMiddleware};
use actix_web::cookie::time::Duration;
use actix_web::cookie::Key;
use actix_web::{
    get, http, post, web, web::Data, App, HttpRequest, HttpResponse, HttpServer, Responder,
};

use serde::Deserialize;

const SECS_IN_WEEK: i64 = 60 * 60 * 24 * 7;
const SESSION_KEY: &str = "sid";

#[derive(Clone)]
pub struct AppState {
    service: AuthService,
}

impl AppState {
    pub fn new(auth_service: AuthService) -> Self {
        Self {
            service: auth_service,
        }
    }
}

#[derive(Deserialize)]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
}

#[post("/signin")]
async fn sign_in(
    _req: HttpRequest,
    state: Data<AppState>,
    session: Session,
    payload: web::Json<SignInRequest>,
) -> impl Responder {
    let created_session = state
        .service
        .sign_in(&payload.email, &payload.password)
        .await
        .unwrap();

    if let Err(e) = session.insert(SESSION_KEY, created_session.id) {
        eprintln!("error inserting session {:?}", e);
        return HttpResponse::InternalServerError();
    };

    HttpResponse::Ok()
}

struct Api;

impl Api {
    async fn start(
        auth_service: AuthService,
        redis_session_url: String,
        session_private_key: String,
        port: u16,
        web_front_end_origin: String,
    ) -> () {
        let store = RedisSessionStore::new(redis_session_url)
            .await
            .expect("Could not connect to redis instance");
        let state = AppState::new(auth_service);
        let app = move || {
            let state = state.clone();
            let key = Key::derive_from(session_private_key.as_ref());
            let web_front_end_origin = web_front_end_origin.clone();
            let cors = Cors::default()
                .allowed_origin(&web_front_end_origin)
                .allowed_methods(vec!["GET", "POST"])
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
                .app_data(Data::new(state))
                .wrap(session)
                .wrap(cors)
                .service(sign_in)
                .service(home)
        };

        HttpServer::new(app)
            .bind(("0.0.0.0", port))
            .unwrap()
            .run()
            .await
            .unwrap();
        ()
    }
}

#[get("/home")]
async fn home(_req: HttpRequest, session: Session) -> impl Responder {
    let session = session.get::<String>(SESSION_KEY).unwrap();
    println!("session: {:?}", session);
    "Hello world".to_string()
}

pub async fn run_server(
    grpc_address: &str,
    database_url: &str,
    web_front_end_origin: &str,
    redis_session_url: &str,
    session_private_key: &str,
    auth_api_port: u16,
) -> Result<(), Box<dyn Error>> {
    let grpc_address = grpc_address.parse()?;

    let pool = Arc::new(
        PgPool::connect(&database_url)
            .await
            .expect("Could not connect to database"),
    );

    let auth_service = AuthService::new(pool);
    let web_front_end_origin = web_front_end_origin.to_owned();
    let redis_session_url = redis_session_url.to_owned();
    let session_private_key = session_private_key.to_owned();
    let service = auth_service.clone();
    let thread = tokio::spawn(async move {
        Api::start(
            service,
            redis_session_url,
            session_private_key,
            auth_api_port,
            web_front_end_origin,
        )
        .await;
        ()
    });

    let grpc_auth_service = GRPCAuthService::new(auth_service);
    Server::builder()
        .add_service(AuthServer::new(grpc_auth_service))
        .serve(grpc_address)
        .await?;
    thread.await.expect("Could not start auth api");
    Ok(())
}
