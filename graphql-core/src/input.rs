use database::types::{DateTime, Utc};
use juniper::GraphQLInputObject;

#[derive(GraphQLInputObject, Debug)]
#[graphql(description = "User Input")]
pub struct UserInput {
    pub email: String,
    pub name: String,
    pub password: String,
}
