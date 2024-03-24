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
use mockall::mock;
use regex::Regex;
use std::str::FromStr;
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

#[derive(Debug)]
pub struct AuthService {
    db: Arc<Pool<Postgres>>,
}

#[async_trait::async_trait]
pub trait AuthServiceTrait: Send + Sync + Clone {
    async fn authenticate(&self, session_id: String) -> Result<(), AuthServiceError>;
    async fn sign_in(
        &self,
        email: String,
        password: String,
    ) -> Result<SignInResponse, AuthServiceError>;
    async fn create_account(
        &self,
        email: String,
        password: String,
    ) -> Result<String, AuthServiceError>;
}

impl AuthService {
    pub fn new(db: Arc<Pool<Postgres>>) -> Self {
        Self { db }
    }
}

impl Clone for AuthService {
    fn clone(&self) -> Self {
        Self {
            db: Arc::clone(&self.db),
        }
    }
}

#[async_trait::async_trait]
impl AuthServiceTrait for AuthService {
    async fn authenticate(&self, session_id: String) -> Result<(), AuthServiceError> {
        let uuid = Uuid::from_str(&session_id).map_err(|_| {
            eprintln!("invalid session id format: {:?}", session_id);
            AuthServiceError::InvalidInput {
                message: "invalid session id format".to_string(),
            }
        })?;
        // TODO - Create access role validation
        if let Some(session) = SessionsRepository::try_get(&self.db, SessionsBy::Id(uuid)).await? {
            if Utc::now() > session.expires_at {
                return Err(AuthServiceError::InvalidCredentials);
            }
            return Ok(());
        }
        Err(AuthServiceError::InvalidCredentials)
    }

    async fn sign_in(
        &self,
        email: String,
        password: String,
    ) -> Result<SignInResponse, AuthServiceError> {
        valid_email(&email)?;

        if let Some(credential) =
            CredentialsRepository::try_get(&self.db, CredentialsBy::Email(email)).await?
        {
            if !PasswordHelper::verify(&credential.password, &password)? {
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

    async fn create_account(
        &self,
        email: String,
        password: String,
    ) -> Result<String, AuthServiceError> {
        valid_email(&email)?;
        let exists = CredentialsRepository::try_get(&self.db, CredentialsBy::Email(email.clone()))
            .await?
            .is_some();

        if exists {
            return Err(AuthServiceError::InvalidCredentials);
        };

        let dao = CreateCredentialsDAO {
            email,
            password: PasswordHelper::hash_password(&password)?,
        };

        let res = CredentialsRepository::insert(&self.db, dao).await?;

        Ok(res.id.to_string())
    }
}

mock! {
    pub AuthService {}

    #[async_trait::async_trait]
    impl AuthServiceTrait for AuthService {
        async fn authenticate(&self, session_id: String) -> Result<(), AuthServiceError>;
        async fn sign_in(
            &self,
            email: String,
            password: String,
        ) -> Result<SignInResponse, AuthServiceError>;
        async fn create_account(
            &self,
            email: String,
            password: String,
        ) -> Result<String, AuthServiceError>;
    }

    impl Clone for AuthService {
        fn clone(&self) -> Self;
    }
}

#[cfg(feature = "integration")]
#[cfg(test)]
mod test {
    use crate::auth::{AuthService, AuthServiceError, AuthServiceTrait};
    use crate::password_helper::PasswordHelper;
    use auth_database::connection::{PgPool, Pool, Postgres};
    use auth_database::entities::credentials::{CreateCredentialsDAO, CredentialsRepository};
    use auth_database::entities::sessions::{CreateSessionsDAO, SessionsRepository};
    use auth_database::traits::EntityRepository;
    use auth_database::types::Utc;
    use std::sync::Arc;
    use std::time::Duration;

    pub async fn setup_test() -> (AuthService, Arc<Pool<Postgres>>) {
        dotenv::dotenv().ok();
        let url =
            std::env::var("TEST_AUTH_DATABASE_URL").expect("TEST_AUTH_DATABASE_URL must be set");
        let pool = Arc::new(PgPool::connect(&url).await.unwrap());
        let auth_service = AuthService::new(pool.clone());
        (auth_service, pool)
    }
    #[tokio::test]
    async fn test_authenticate() {
        let (auth_service, pool) = setup_test().await;

        // Invalid uuid
        let result = auth_service
            .authenticate("f1ac1576-bd47-4fd3-a9af 49cd400c2cd7".to_string())
            .await
            .unwrap_err();
        assert_eq!(
            AuthServiceError::InvalidInput {
                message: "invalid session id format".to_string()
            },
            result
        );

        // session not found
        let result = auth_service
            .authenticate("f1ac1576-bd47-4fd3-a9af-49cd400c2cd7".to_string())
            .await
            .unwrap_err();
        assert_eq!(AuthServiceError::InvalidCredentials, result);

        // session_expired
        let credential = CredentialsRepository::insert(
            &pool,
            CreateCredentialsDAO {
                email: "test@gmail.com".to_string(),
                password: "123456".to_string(),
            },
        )
        .await
        .unwrap();
        let session = SessionsRepository::insert(
            &pool,
            CreateSessionsDAO {
                credential_id: credential.id,
                expires_at: Utc::now() + Duration::from_secs(10),
            },
        )
        .await
        .unwrap();

        tokio::time::sleep(Duration::from_secs(30)).await;

        let result = auth_service
            .authenticate(session.id.to_string())
            .await
            .unwrap_err();
        assert_eq!(AuthServiceError::InvalidCredentials, result);

        // success
        let session = SessionsRepository::insert(
            &pool,
            CreateSessionsDAO {
                credential_id: credential.id,
                expires_at: Utc::now() + Duration::from_secs(60),
            },
        )
        .await
        .unwrap();

        let result = auth_service.authenticate(session.id.to_string()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_signin() {
        let (auth_service, pool) = setup_test().await;

        // invalid email
        let result = auth_service
            .sign_in("test.com".to_string(), "123456".to_string())
            .await
            .unwrap_err();
        assert_eq!(
            AuthServiceError::InvalidInput {
                message: "invalid email".to_string()
            },
            result
        );

        // credential not found
        let result = auth_service
            .sign_in("t@gmail.com".to_string(), "123456".to_string())
            .await
            .unwrap_err();
        assert_eq!(AuthServiceError::InvalidCredentials, result);

        // invalid_password
        let hash = PasswordHelper::hash_password("123456").unwrap();
        let credential = CredentialsRepository::insert(
            &pool,
            CreateCredentialsDAO {
                email: "test22@gmail.com".to_string(),
                password: hash.to_string(),
            },
        )
        .await
        .unwrap();
        let result = auth_service
            .sign_in("test22@gmail.com".to_string(), "other password".to_string())
            .await
            .unwrap_err();
        assert_eq!(AuthServiceError::InvalidCredentials, result);

        // success
        let result = auth_service
            .sign_in("test22@gmail.com".to_string(), "123456".to_string())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn create_account_invalid_email() {
        let (auth_service, pool) = setup_test().await;
        // invalid email
        let result = auth_service
            .create_account("test.com".to_string(), "123456".to_string())
            .await
            .unwrap_err();
        assert_eq!(
            AuthServiceError::InvalidInput {
                message: "invalid email".to_string()
            },
            result
        );

        // create account success
        let result = auth_service
            .create_account("test3@gmail.com".to_string(), "123456".to_string())
            .await;
        assert!(result.is_ok());

        // account already exists
        let result = auth_service
            .create_account("test3@gmail.com".to_string(), "123456".to_string())
            .await
            .unwrap_err();
        assert_eq!(AuthServiceError::InvalidCredentials, result);
    }
}
