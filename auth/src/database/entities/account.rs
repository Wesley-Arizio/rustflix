use mongodb::bson::uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {
    pub _id: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateAccountDAO {
    pub email: String,
    pub password: String,
}

impl From<&CreateAccountDAO> for Account {
    fn from(value: &CreateAccountDAO) -> Self {
        Self {
            _id: Uuid::new().to_string(),
            email: value.email.to_owned(),
            password: value.password.to_owned(),
        }
    }
}
