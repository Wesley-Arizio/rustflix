use std::sync::Arc;

use database::{
    entities::users::{UserDAO, UserRepository},
    traits::EntityRepository,
};
use grpc_interfaces::auth::{auth_client::AuthClient, CreateCredentialsRequest};
use juniper::{graphql_object, FieldError, FieldResult, GraphQLInputObject};
use sqlx::types::{chrono::Utc, Uuid};
use std::str::FromStr;
use tokio::sync::Mutex;
use tonic::{transport::Channel, Request};

use crate::AppState;

pub struct MutationRoot {
    auth_client: Arc<Mutex<AuthClient<Channel>>>,
}

impl MutationRoot {
    pub fn new(auth_client: AuthClient<Channel>) -> Self {
        Self {
            auth_client: Arc::new(Mutex::new(auth_client)),
        }
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

impl From<UserDAO> for User {
    fn from(value: UserDAO) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
            active: value.active,
        }
    }
}

#[graphql_object(context = AppState)]
impl MutationRoot {
    async fn create_account(app_state: &AppState, user: UserInput) -> FieldResult<User> {
        let request = CreateCredentialsRequest {
            email: user.email,
            password: user.password,
        };

        let mut auth_client = self.auth_client.lock().await;

        // TODO - improve error mapping GrpcStatus -> FieldError
        let response = auth_client
            .create_credential(Request::new(request))
            .await
            .map(|r| r.into_inner())
            .map_err(|e| FieldError::from(e.message()))?;

        // TODO - improve error mapping DatabaseError -> FieldError
        // TODO - create scalar value for birthday
        let user = UserRepository::insert(
            &app_state.db,
            UserDAO {
                id: Uuid::from_str(&response.user_id).unwrap(),
                birthday: Utc::now(),
                active: true,
                name: user.name,
            },
        )
        .await
        .map_err(|e| {
            eprintln!("e: {:#?}", e);
            FieldError::from("Internal Server Error")
        })?;

        Ok(user.into())
    }
}
