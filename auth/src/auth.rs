use std::error::Error;
use crate::database::entities::account::{Account, CreateAccountDAO};

use super::database::traits::{Repository, RepositoryError};

#[derive(Debug, PartialEq, Eq)]
pub enum AuthServiceError {
    InvalidInput { message: String },
    InvalidCredentials,
    InternalServerError
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
where R: Repository<Entity = Account, CreateEntityDAO = CreateAccountDAO> + Send + Sync {
    account_repository: R,
}

impl<R> AuthService<R> where R: Repository<Entity = Account, CreateEntityDAO = CreateAccountDAO> + Send + Sync {
    pub fn new(repository: R) -> Self {
        Self {
            account_repository: repository,
        }
    }

    pub async fn create_account(&self, email: &str, password: &str) -> Result<String, AuthServiceError> {
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
