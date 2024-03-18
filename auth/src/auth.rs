use crate::password_helper::PasswordHelper;
use auth_database::entities::sessions::{CreateSessionsDAO, SessionsBy, SessionsRepository};
use auth_database::types::Uuid;
use auth_database::{
    connection::{Pool, Postgres},
    entities::{
        credentials::{CreateCredentialsDAO, CredentialsBy, CredentialsRepository},
        sessions::SessionsDAO,
    },
    traits::{DatabaseError, EntityRepository},
    types::{DateTime, Utc},
};
use regex::Regex;
use std::sync::Arc;
use std::time::Duration;

const ONE_DAY_IN_SECONDS: u32 = 60 * 60 * 24;

#[derive(Debug, PartialEq, Eq)]
pub enum AuthServiceError {
    InvalidInput { message: String },
    InvalidCredentials,
    InternalServerError,
}

impl From<DatabaseError> for AuthServiceError {
    fn from(value: DatabaseError) -> Self {
        eprintln!("Database failed: {:?}", value);
        Self::InternalServerError
    }
}

fn valid_email(email: &str) -> Result<(), AuthServiceError> {
    let regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .map_err(|_| AuthServiceError::InternalServerError)?;

    if !regex.is_match(email) {
        return Err(AuthServiceError::InvalidInput {
            message: "invalid email".to_string(),
        });
    }

    Ok(())
}

#[derive(Debug)]
pub struct AuthService {
    db: Arc<Pool<Postgres>>,
}

impl Clone for AuthService {
    fn clone(&self) -> Self {
        Self {
            db: Arc::clone(&self.db),
        }
    }
}

#[derive(Debug)]
pub struct SignInResponse {
    pub id: String,
    pub expires_at: DateTime<Utc>,
}

impl From<&SessionsDAO> for SignInResponse {
    fn from(value: &SessionsDAO) -> Self {
        Self {
            id: value.id.to_string(),
            expires_at: value.expires_at,
        }
    }
}

impl AuthService {
    pub fn new(db: Arc<Pool<Postgres>>) -> Self {
        Self { db }
    }

    pub async fn authenticate(&self, session_id: &[u8]) -> Result<(), AuthServiceError> {
        let uuid = Uuid::from_slice(session_id)
                .map_err(|_| {
                    eprintln!("invalid session id format: {:?}", session_id);
                    AuthServiceError::InternalServerError
                })?;
        // TODO - Create access role validation
        if let Some(session) =
            SessionsRepository::try_get(&self.db, SessionsBy::CredentialId(uuid)).await?
        {
            if Utc::now() > session.expires_at {
                return Err(AuthServiceError::InvalidCredentials);
            }
            return Ok(());
        }

        Err(AuthServiceError::InvalidCredentials)
    }

    pub async fn sign_in(
        &self,
        email: &str,
        password: &str,
    ) -> Result<SignInResponse, AuthServiceError> {
        valid_email(email)?;

        if let Some(credential) =
            CredentialsRepository::try_get(&self.db, CredentialsBy::Email(email.to_owned())).await?
        {
            if !PasswordHelper::verify(&credential.password, password)? {
                return Err(AuthServiceError::InvalidCredentials);
            };

            let session = SessionsRepository::insert(
                &self.db,
                CreateSessionsDAO {
                    expires_at: Utc::now() + Duration::from_secs(ONE_DAY_IN_SECONDS as u64),
                    credential_id: credential.id,
                },
            )
            .await?;

            Ok((&session).into())
        } else {
            return Err(AuthServiceError::InvalidCredentials);
        }
    }

    pub async fn create_account(
        &self,
        email: &str,
        password: &str,
    ) -> Result<String, AuthServiceError> {
        valid_email(email)?;
        let exists =
            CredentialsRepository::try_get(&self.db, CredentialsBy::Email(email.to_owned()))
                .await?
                .is_some();

        if exists {
            return Err(AuthServiceError::InvalidCredentials);
        };

        let dao = CreateCredentialsDAO {
            email: email.to_owned(),
            password: PasswordHelper::hash_password(&password)?,
        };

        let res = CredentialsRepository::insert(&self.db, dao).await?;

        Ok(res.id.to_string())
    }
}
