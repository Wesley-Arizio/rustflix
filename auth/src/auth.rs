use crate::database::entities::account::{Account, CreateAccountDAO};
use std::error::Error;

use super::database::traits::{Repository, RepositoryError};

#[derive(Debug, PartialEq, Eq)]
pub enum AuthServiceError {
    InvalidInput { message: String },
    InvalidCredentials,
    InternalServerError,
}

impl From<RepositoryError> for AuthServiceError {
    fn from(value: RepositoryError) -> Self {
        if let Some(source) = value.source() {
            eprintln!("{:?}", source);
        }
        Self::InternalServerError
    }
}

#[derive(Debug)]
pub struct AuthService<R>
where
    R: Repository<Entity = Account, CreateEntityDAO = CreateAccountDAO> + Send + Sync,
{
    account_repository: R,
}

impl<R> AuthService<R>
where
    R: Repository<Entity = Account, CreateEntityDAO = CreateAccountDAO> + Send + Sync,
{
    pub fn new(repository: R) -> Self {
        Self {
            account_repository: repository,
        }
    }

    pub async fn create_account(
        &self,
        email: &str,
        password: &str,
    ) -> Result<String, AuthServiceError> {
        let exists = self.account_repository.exists(email).await?;

        if exists {
            return Err(AuthServiceError::InvalidCredentials);
        };

        // TODO - Hash password
        let dto = CreateAccountDAO {
            email: email.to_owned(),
            password: password.to_owned(),
        };

        let res = self.account_repository.create(&dto).await?;
        Ok(res._id)
    }
}

#[cfg(test)]
mod tests {
    use crate::auth::{AuthService, AuthServiceError};
    use crate::database::entities::account::{Account, CreateAccountDAO};
    use crate::database::traits::MockFakeRepository;
    use mockall::predicate::*;
    #[tokio::test]
    async fn test_create_account_success() {
        let mut mock = MockFakeRepository::new();

        mock.expect_exists()
            .times(1)
            .with(eq("test@gmail.com"))
            .return_once(|_| Ok(false));

        mock.expect_create()
            .times(1)
            .with(eq(CreateAccountDAO {
                email: "test@gmail.com".to_string(),
                password: "test123456".to_string(), // TODO - hash password
            }))
            .return_once(|_| {
                Ok(Account {
                    _id: String::from("1234"),
                    email: "test@gmail.com".to_string(),
                    password: "test123456".to_string(),
                })
            });

        let service = AuthService::new(mock);
        let result = service
            .create_account("test@gmail.com", "test123456")
            .await
            .unwrap();
        assert_eq!(result, "1234");
    }

    #[tokio::test]
    async fn test_create_account_invalid_credentials() {
        let mut mock = MockFakeRepository::new();
        mock.expect_exists()
            .times(1)
            .with(eq("test@gmail.com"))
            .return_once(|_| Ok(true));

        mock.expect_create().times(0);

        let service = AuthService::new(mock);
        let result = service
            .create_account("test@gmail.com", "test123456")
            .await
            .unwrap_err();
        assert_eq!(result, AuthServiceError::InvalidCredentials);
    }
}
