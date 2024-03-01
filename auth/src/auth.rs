use crate::password_helper::{PasswordHelper, PasswordHelperError};
use auth_database::entities::sessions::{CreateSessionsDAO, SessionsRepository};
use auth_database::types::Utc;
use auth_database::{
    connection::{Pool, Postgres},
    entities::credentials::{CreateCredentialsDAO, CredentialsBy, CredentialsRepository},
    traits::{DatabaseError, EntityRepository},
};
use regex::Regex;
use std::time::Duration;

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

impl From<PasswordHelperError> for AuthServiceError {
    fn from(e: PasswordHelperError) -> Self {
        eprintln!("Password helper error: {:?}", e);
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
    db: Pool<Postgres>,
}

impl AuthService {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn sign_in(&self, email: &str, password: &str) -> Result<String, AuthServiceError> {
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
                    expires_at: Utc::now() + Duration::from_secs(60 * 60),
                    credential_id: credential.id,
                },
            )
            .await?;

            Ok(session.id.to_string())
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
