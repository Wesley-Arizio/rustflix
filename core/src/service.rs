use std::error::Error;
use crate::dto::user::UserDTO;
use database::{
    connection::{Pool, Postgres},
    entities::users::{UserDAO, UserRepository},
    traits::EntityRepository,
    types::{Utc, Uuid},
};
use grpc_interfaces::auth::auth_client::AuthClient;
use grpc_interfaces::auth::CreateCredentialsRequest;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use thiserror::Error;
use tonic::transport::Channel;
use tonic::{Code, Request, Status};
use database::traits::DatabaseError;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Internal server error")]
    InternalServerError,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("{0} not found")]
    NotFound(String)
}

impl From<Status> for CoreError {
    fn from(value: Status) -> Self {
        if let Some(s) = value.source() {
            eprintln!("core error source: {:?}", s)
        }
        match value.code() {
            Code::Unauthenticated => CoreError::InvalidCredentials,
            Code::InvalidArgument => CoreError::InvalidArgument(value.message().to_string()),
            _ => CoreError::InternalServerError
        }
    }
}

impl From<DatabaseError> for CoreError {
    fn from(value: DatabaseError) -> Self {
        match value {
            DatabaseError::NotFound(entity) => CoreError::NotFound(entity),
            _ => CoreError::InternalServerError
        }
    }
}

pub struct Core {
    auth_client: Arc<Mutex<AuthClient<Channel>>>,
    db: Pool<Postgres>
}

impl Core {
    // TODO - Refactor database to be generic
    pub async fn new(auth_grpc_port: String, db: Pool<Postgres>) -> Self {
        let auth_client = AuthClient::connect(auth_grpc_port)
            .await
            .expect("Could not connect to auth grpc client");

        Self {
            auth_client: Arc::new(Mutex::new(auth_client)),
            db,
        }
    }
}

impl Core {
    pub async fn create_account(
        &self,
        email: String,
        password: String,
        name: String,
    ) -> Result<UserDTO, CoreError> {
        let mut auth_client = self.auth_client.lock().await;

        let request = CreateCredentialsRequest { email, password };

        // TODO - create scalar value for birthday
        let response = auth_client
            .create_credential(Request::new(request))
            .await
            .map(|r| r.into_inner())
            .map_err(CoreError::from)?;

        let user = UserRepository::insert(
            &self.db,
            UserDAO {
                id: Uuid::from_str(&response.user_id).unwrap(),
                birthday: Utc::now(),
                active: true,
                name,
            },
        )
        .await
        .map_err(CoreError::from)?;

        Ok(user.into())
    }
}
