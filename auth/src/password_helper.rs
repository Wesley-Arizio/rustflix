use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};

use crate::auth::AuthServiceError;
use thiserror::Error;

pub struct PasswordHelper;

#[derive(Debug, Error)]
pub enum PasswordHelperError {
    #[error("Internal server error")]
    PasswordHashingError(String),
}

impl From<argon2::password_hash::Error> for PasswordHelperError {
    fn from(value: argon2::password_hash::Error) -> Self {
        PasswordHelperError::PasswordHashingError(value.to_string())
    }
}

impl From<PasswordHelperError> for AuthServiceError {
    fn from(e: PasswordHelperError) -> Self {
        eprintln!("Password helper error: {:?}", e);
        Self::InternalServerError
    }
}

impl PasswordHelper {
    pub fn hash_password(password: &str) -> Result<String, PasswordHelperError> {
        let salt = SaltString::generate(&mut OsRng);

        // Argon2 with default params (Argon2id v19)
        let argon2 = Argon2::default();

        // Hash password to PHC string ($argon2id$v=19$...)
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;

        Ok(password_hash.to_string())
    }
    #[allow(dead_code)]
    pub fn verify(hash_password: &str, password: &str) -> Result<bool, PasswordHelperError> {
        let parsed_hash = PasswordHash::new(&hash_password)?;
        let result = Argon2::default().verify_password(password.as_bytes(), &parsed_hash);

        Ok(result.is_ok())
    }
}

#[cfg(not(feature = "integration"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_helper() {
        let password = "123456";
        let hash = PasswordHelper::hash_password(password).unwrap();

        assert!(PasswordHelper::verify(&hash, password).unwrap());
        assert!(!PasswordHelper::verify(&hash, "12345").unwrap());

        // Hash generated here https://argon2.online/
        assert!(PasswordHelper::verify(
            "$argon2i$v=19$m=16,t=2,p=1$NHo1NGtha1JqV2hRMXEvOE1TQitFQQ$O6xjGjMkhshoWFMFsaB3IA",
            "any other test"
        )
        .unwrap());
        assert!(!PasswordHelper::verify(
            "$argon2i$v=19$m=16,t=2,p=1$NHo1NGtha1JqV2hRMXEvOE1TQitFQQ$O6xjGjMkhshoWFMFsaB3IA",
            "any other test2"
        )
        .unwrap());
    }
}
