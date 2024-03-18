use crate::auth::{AuthService, AuthServiceError};
use crate::grpc::GRPCAuthService;
use auth_database::connection::PgPool;
use grpc_interfaces::auth::auth_server::AuthServer;
use std::error::Error;
use std::sync::Arc;
use tonic::transport::Server;

use tower_http::cors::CorsLayer;
use tower_sessions::{Expiry, MemoryStore, Session, SessionManagerLayer};

use axum::body::Body;
use axum::extract::State;
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};

use serde::{Deserialize, Serialize};
use tower_sessions::cookie::time::{Duration, OffsetDateTime};

#[derive(Clone)]
pub struct AppState {
    service: AuthService,
}

impl AppState {
    pub fn new(auth_service: &AuthService) -> Self {
        Self {
            service: auth_service.clone(),
        }
    }
}

#[derive(Deserialize)]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
}

impl IntoResponse for AuthServiceError {
    fn into_response(self) -> Response {
        let mut res = Response::new(Body::empty());
        let (status, maybe_body) = match self {
            AuthServiceError::InvalidInput { message } => (StatusCode::BAD_REQUEST, Some(message)),
            AuthServiceError::InvalidCredentials => (StatusCode::UNAUTHORIZED, None),
            AuthServiceError::InternalServerError => (StatusCode::BAD_REQUEST, None),
        };

        *res.status_mut() = status;
        if let Some(message) = maybe_body {
            *res.body_mut() = Body::from(message);
        }

        res
    }
}

const SESSION_KEY: &str = "sid";

#[derive(Debug, Default, Deserialize, Serialize)]
struct AuthSession(String);

async fn sign_in(
    session: Session,
    State(state): State<AppState>,
    Json(payload): Json<SignInRequest>,
) -> Result<(), AuthServiceError> {
    let created_session = state
        .service
        .sign_in(&payload.email, &payload.password)
        .await?;

    session
        .insert(SESSION_KEY, created_session.id)
        .await
        .map_err(|_| AuthServiceError::InternalServerError)?;

    let expires_at = OffsetDateTime::from_unix_timestamp(created_session.expires_at.timestamp())
        .map_err(|_| AuthServiceError::InternalServerError)?;
    session.set_expiry(Some(Expiry::AtDateTime(expires_at)));

    Ok(())
}

async fn home(session: Session) -> String {
    let session: AuthSession = session.get(SESSION_KEY).await.unwrap().unwrap_or_default();
    session.0.to_string()
}

pub async fn run_server(
    grpc_address: &str,
    database_url: &str,
    api_address: &str,
    web_front_end_origin: &str,
) -> Result<(), Box<dyn Error>> {
    let grpc_address = grpc_address.parse()?;

    let pool = Arc::new(
        PgPool::connect(&database_url)
            .await
            .expect("Could not connect to database"),
    );

    let auth_service = AuthService::new(pool);
    let state = AppState::new(&auth_service);
    let address = api_address.to_owned();
    let web_front_end_origin = web_front_end_origin.to_owned();
    let thread = tokio::spawn(async move {
        let session_store = MemoryStore::default();
        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_expiry(Expiry::OnInactivity(Duration::seconds(60)));

        let cors = CorsLayer::new()
            .allow_origin([web_front_end_origin.parse::<HeaderValue>().unwrap()])
            .allow_methods([Method::GET, Method::POST])
            .allow_credentials(true)
            .allow_headers([CONTENT_TYPE]);

        let app = Router::new()
            .route("/signin", post(sign_in))
            .route("/home", get(home))
            .layer(session_layer)
            .layer(cors)
            .with_state(state);

        let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    let grpc_auth_service = GRPCAuthService::new(auth_service);
    Server::builder()
        .add_service(AuthServer::new(grpc_auth_service))
        .serve(grpc_address)
        .await?;

    thread.await.expect("Could not start auth api");

    Ok(())
}
