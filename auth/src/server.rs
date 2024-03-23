use crate::auth::{AuthService, AuthServiceError, AuthServiceTrait};
use crate::grpc::GRPCAuthService;
use auth_database::connection::PgPool;
use grpc_interfaces::auth::auth_server::AuthServer;
use std::error::Error;
use std::marker::PhantomData;
use std::sync::Arc;
use tonic::transport::Server;

use actix_cors::Cors;
use actix_session::config::PersistentSession;
use actix_session::storage::SessionStore;
use actix_session::{storage::RedisSessionStore, Session, SessionMiddleware};
use actix_web::cookie::time::Duration;
use actix_web::cookie::Key;
use actix_web::{http, web, web::Data, App, HttpRequest, HttpResponse, HttpServer};

use serde::{Deserialize, Serialize};

const SECS_IN_WEEK: i64 = 60 * 60 * 24 * 7;
const SESSION_KEY: &str = "sid";

impl From<AuthServiceError> for HttpResponse {
    fn from(value: AuthServiceError) -> Self {
        match value {
            AuthServiceError::InvalidCredentials => HttpResponse::Unauthorized().finish(),
            AuthServiceError::InternalServerError => HttpResponse::InternalServerError().finish(),
            AuthServiceError::InvalidInput { message } => HttpResponse::BadRequest().body(message),
        }
    }
}

pub struct AppState<T>
where
    T: AuthServiceTrait,
{
    service: T,
}

impl<T> AppState<T>
where
    T: AuthServiceTrait,
{
    pub fn new(service: T) -> Self {
        Self { service }
    }
}

#[derive(Deserialize, Serialize)]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
}

async fn sign_in<T: AuthServiceTrait>(
    _req: HttpRequest,
    state: Data<AppState<T>>,
    session: Session,
    payload: web::Json<SignInRequest>,
) -> HttpResponse {
    let result = state
        .service
        .sign_in(payload.email.to_owned(), payload.password.to_owned())
        .await;

    match result {
        Ok(created_session) => {
            if let Err(e) = session.insert(SESSION_KEY, created_session.id) {
                eprintln!("error inserting session {:?}", e);
                return HttpResponse::InternalServerError().finish();
            };

            HttpResponse::Ok().finish()
        }
        Err(error) => HttpResponse::from(error),
    }
}

struct Api<Session, Service>
where
    Session: SessionStore + 'static,
    Service: AuthServiceTrait + 'static,
{
    session: PhantomData<Session>,
    service: PhantomData<Service>,
}

impl<Session, Service> Api<Session, Service>
where
    Session: SessionStore + 'static,
    Service: AuthServiceTrait + 'static,
{
    pub fn configure_app(
        cfg: &mut web::ServiceConfig,
        session: Session,
        state: AppState<Service>,
        session_private_key: &str,
        web_front_end_origin: &str,
    ) {
        let cors = Cors::default()
            .allowed_origin(web_front_end_origin)
            .allowed_methods(vec!["GET", "POST"])
            .supports_credentials()
            .allowed_headers(vec![http::header::CONTENT_TYPE]);
        let key = Key::derive_from(session_private_key.as_bytes());
        let store = SessionMiddleware::builder(session, key)
            .session_lifecycle(
                PersistentSession::default().session_ttl(Duration::seconds(SECS_IN_WEEK)),
            )
            .cookie_domain(Some("localhost".to_string()))
            .cookie_name(SESSION_KEY.to_string())
            .cookie_http_only(true)
            .cookie_path("/".to_string())
            .cookie_secure(true)
            .build();
        cfg.service(
            web::resource("/signin")
                .app_data(Data::new(state))
                .wrap(cors)
                .wrap(store)
                .route(web::post().to(sign_in::<Service>)),
        );
    }

    async fn start(
        port: u16,
        service: AuthService,
        session_private_key: &str,
        web_front_end_origin: &str,
        redis_session_url: &str,
    ) -> () {
        let store = RedisSessionStore::new(redis_session_url)
            .await
            .expect("Could not connect to redis instance");
        let session_private_key = session_private_key.to_owned();
        let web_front_end_origin = web_front_end_origin.to_owned();
        let app = move || {
            let state = AppState::new(service.clone());
            App::new().configure(|cfg| {
                Api::configure_app(
                    cfg,
                    store.clone(),
                    state,
                    &session_private_key,
                    &web_front_end_origin,
                )
            })
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
        Api::<RedisSessionStore, AuthService>::start(
            auth_api_port,
            service,
            &session_private_key,
            &web_front_end_origin,
            &redis_session_url,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{AuthServiceError, MockAuthService, SignInResponse};
    use actix_session::storage::CookieSessionStore;
    use actix_web::http::{header, StatusCode};
    use actix_web::{cookie, http::header::ContentType, test};
    use auth_database::types::Utc;
    use mockall::predicate::eq;
    use std::str::FromStr;

    #[actix_web::test]
    async fn signin_success() {
        let mut mock = MockAuthService::new();
        mock.expect_sign_in()
            .with(eq("test@gmail.com".to_string()), eq("123456".to_string()))
            .returning(move |_, _| {
                Ok(SignInResponse {
                    id: "value".to_string(),
                    expires_at: Utc::now(),
                })
            })
            .times(1);
        let state = AppState::new(mock);
        let secret = "r4RXHJ0Ec7UZIhR7MfbzVzQjv2YSxHfb3LUeW2fHf6gZL6Lb7B1zYuNOfd5q8m25";
        let app = test::init_service(App::new().configure(|cfg| {
            Api::configure_app(
                cfg,
                CookieSessionStore::default(),
                state,
                secret,
                "localhost",
            )
        }))
        .await;
        let payload = SignInRequest {
            email: "test@gmail.com".to_string(),
            password: "123456".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/signin")
            .set_json(payload)
            .insert_header(ContentType::json())
            .to_request();
        let resp = test::call_service(&app, req).await;
        let cookie = cookie::Cookie::from_str(
            resp.headers()
                .get(header::SET_COOKIE)
                .unwrap()
                .to_str()
                .unwrap(),
        )
        .unwrap();

        assert_eq!(cookie.name(), "sid");
        assert_eq!(cookie.domain().unwrap(), "localhost");
        assert!(cookie.http_only().unwrap());
        assert!(cookie.secure().unwrap());
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn signin_error_invalid_credentials() {
        let mut mock = MockAuthService::new();
        mock.expect_sign_in()
            .with(eq("test@gmail.com".to_string()), eq("123456".to_string()))
            .returning(|_, _| Err(AuthServiceError::InvalidCredentials))
            .times(1);
        let state = AppState::new(mock);
        let secret = "r4RXHJ0Ec7UZIhR7MfbzVzQjv2YSxHfb3LUeW2fHf6gZL6Lb7B1zYuNOfd5q8m25";
        let app = test::init_service(App::new().configure(|cfg| {
            Api::configure_app(
                cfg,
                CookieSessionStore::default(),
                state,
                secret,
                "localhost",
            )
        }))
        .await;
        let payload = SignInRequest {
            email: "test@gmail.com".to_string(),
            password: "123456".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/signin")
            .set_json(payload)
            .insert_header(ContentType::json())
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(!resp.headers().contains_key(header::SET_COOKIE));
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn signin_error_bad_request() {
        let mut mock = MockAuthService::new();
        mock.expect_sign_in()
            .with(eq("test".to_string()), eq("123456".to_string()))
            .returning(|_, _| {
                Err(AuthServiceError::InvalidInput {
                    message: "invalid email".to_string(),
                })
            })
            .times(1);
        let state = AppState::new(mock);
        let secret = "r4RXHJ0Ec7UZIhR7MfbzVzQjv2YSxHfb3LUeW2fHf6gZL6Lb7B1zYuNOfd5q8m25";
        let app = test::init_service(App::new().configure(|cfg| {
            Api::configure_app(
                cfg,
                CookieSessionStore::default(),
                state,
                secret,
                "localhost",
            )
        }))
        .await;
        let payload = SignInRequest {
            email: "test".to_string(),
            password: "123456".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/signin")
            .set_json(payload)
            .insert_header(ContentType::json())
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(!resp.headers().contains_key(header::SET_COOKIE));
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body = test::read_body(resp).await;
        assert_eq!(body, web::Bytes::from_static(b"invalid email"))
    }
}
