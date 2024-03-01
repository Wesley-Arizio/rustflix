use crate::input::UserInput;
use crate::output::User;
use core::service::Core;
use juniper::{graphql_object, FieldError, FieldResult};

pub struct MutationRoot {
    core: Core,
}

impl MutationRoot {
    pub fn new(core: Core) -> Self {
        Self { core }
    }
}

#[graphql_object()]
impl MutationRoot {
    async fn create_account(&self, user: UserInput) -> FieldResult<User> {
        let response = self
            .core
            .create_account(user.email, user.password, user.name, user.birthday.0)
            .await
            .map_err(|e| FieldError::new(e, juniper::Value::null()))?;

        Ok(response.into())
    }

    async fn sign_in(&self, email: String, password: String) -> FieldResult<String> {
        let response = self
            .core
            .sign_in(email, password)
            .await
            .map_err(|e| FieldError::new(e, juniper::Value::null()))?;

        Ok(response.into())
    }
}
