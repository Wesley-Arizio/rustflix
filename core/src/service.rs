use crate::dto::movie::MovieDTO;
use crate::dto::user::UserDTO;
use core_database::entities::movies::{MovieBy, MovieRepository, MoviesWhere};
use core_database::{
    connection::{Pool, Postgres},
    entities::users::{UserDAO, UserRepository},
    traits::{DatabaseError, EntityRepository},
    types::{DateTime, Utc, Uuid},
};
use grpc_interfaces::auth::{
    auth_client::AuthClient, AuthenticateRequest, CreateCredentialsRequest,
};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tonic::{Code, Request, Status};

#[derive(Debug)]
pub enum CoreError {
    InternalServerError,

    InvalidCredentials,

    InvalidArgument(String),

    NotFound(String),
}

impl Display for CoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreError::InternalServerError => write!(f, "Internal Server Error"),
            CoreError::InvalidCredentials => write!(f, "Invalid Credentials"),
            CoreError::InvalidArgument(msg) => write!(f, "Invalid Argument: {:?}", msg),
            CoreError::NotFound(entity) => write!(f, "{:?} Not Found", entity),
        }
    }
}

impl Error for CoreError {}

impl From<Status> for CoreError {
    fn from(value: Status) -> Self {
        match value.code() {
            Code::Unauthenticated => CoreError::InvalidCredentials,
            Code::InvalidArgument => CoreError::InvalidArgument(value.message().to_string()),
            _ => CoreError::InternalServerError,
        }
    }
}

impl From<DatabaseError> for CoreError {
    fn from(value: DatabaseError) -> Self {
        match value {
            DatabaseError::NotFound(entity) => CoreError::NotFound(entity),
            _ => CoreError::InternalServerError,
        }
    }
}

#[derive(Clone)]
pub struct Core {
    auth_client: Arc<Mutex<AuthClient<Channel>>>,
    db: Pool<Postgres>,
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
    pub async fn authenticate(&self, session_id: String) -> Result<(), CoreError> {
        let mut auth_client = self.auth_client.lock().await;
        let request = AuthenticateRequest { session_id };

        auth_client
            .authenticate(Request::new(request))
            .await
            .map(|r| r.into_inner())
            .map_err(CoreError::from)?;
        Ok(())
    }

    pub async fn list_movies(
        &self,
        session_id: String,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<MovieDTO>, CoreError> {
        self.authenticate(session_id).await?;

        let movies = MovieRepository::get_all(&self.db, MoviesWhere::Page { offset, limit })
            .await?
            .into_iter()
            .map(MovieDTO::from)
            .collect::<Vec<MovieDTO>>();

        Ok(movies)
    }

    pub async fn movie(
        &self,
        session_id: String,
        movie_id: Uuid,
    ) -> Result<Option<MovieDTO>, CoreError> {
        self.authenticate(session_id).await?;
        Ok(MovieRepository::try_get(&self.db, MovieBy::Id(movie_id))
            .await?
            .map(MovieDTO::from))
    }

    pub async fn create_account(
        &self,
        email: String,
        password: String,
        name: String,
        birthday: DateTime<Utc>,
    ) -> Result<UserDTO, CoreError> {
        let mut auth_client = self.auth_client.lock().await;

        let request = CreateCredentialsRequest { email, password };

        let response = auth_client
            .create_credential(Request::new(request))
            .await
            .map(|r| r.into_inner())
            .map_err(CoreError::from)?;

        let id = Uuid::from_str(&response.user_id).map_err(|e| {
            eprintln!("{:?}", e);
            CoreError::InternalServerError
        })?;

        let user = UserRepository::insert(
            &self.db,
            UserDAO {
                id,
                birthday,
                active: true,
                name,
            },
        )
        .await
        .map_err(CoreError::from)?;

        Ok(user.into())
    }
}
