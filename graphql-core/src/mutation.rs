use core::{dto::user::UserDTO, service::Core};
use juniper::{graphql_object, FieldError, FieldResult, GraphQLInputObject};

pub struct MutationRoot {
    core: Core,
}

impl MutationRoot {
    pub fn new(core: Core) -> Self {
        Self { core }
    }
}

#[derive(GraphQLInputObject, Debug)]
#[graphql(description = "User Input")]
pub struct UserInput {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Debug)]
pub struct User {
    pub id: String,
    pub name: String,
    pub active: bool,
}

#[graphql_object]
impl User {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn active(&self) -> bool {
        self.active
    }
}

impl From<UserDTO> for User {
    fn from(value: UserDTO) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
            active: value.active,
        }
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
