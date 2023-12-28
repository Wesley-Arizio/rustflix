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
            .create_account(user.email, user.password, user.name)
            .await
            .map_err(|e| FieldError::from(e.to_string()))?; // TODO - Map CoreError to FieldError

        Ok(response.into())
    }
}
