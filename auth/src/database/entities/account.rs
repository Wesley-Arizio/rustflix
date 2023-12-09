use mongodb::bson::uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub _id: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountDTO {
    pub email: String,
    pub password: String,
}

impl From<&AccountDTO> for Account {
    fn from(value: &AccountDTO) -> Self {
        Self {
            _id: Uuid::new().to_string(),
            email: value.email.to_owned(),
            password: value.password.to_owned(),
        }
    }
}
