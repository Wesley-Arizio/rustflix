use chrono::{DateTime, Utc};
use juniper::GraphQLInputObject;

#[derive(Debug)]
pub struct Birthday(pub DateTime<Utc>);

#[juniper::graphql_scalar(
// You can rename the type for GraphQL by specifying the name here.
name = "Birthday",
// You can also specify a description here.
// If present, doc comments will be ignored.
description = "String timestamp representing user's birthday")]
impl<S> GraphQLScalar for Birthday
where
    S: juniper::ScalarValue,
{
    fn resolve(&self) -> juniper::Value {
        juniper::Value::scalar(self.0.to_string())
    }

    fn from_input_value(value: &juniper::InputValue) -> Option<Birthday> {
        if let Some(date) = DateTime::from_input_value(value) {
            Some(Birthday(date))
        } else {
            None
        }
    }

    fn from_str<'a>(value: juniper::ScalarToken<'a>) -> juniper::ParseScalarResult<'a, S> {
        <String as juniper::ParseScalarValue<S>>::from_str(value)
    }
}

#[derive(GraphQLInputObject, Debug)]
#[graphql(description = "User Input")]
pub struct UserInput {
    pub email: String,
    pub name: String,
    pub password: String,
    pub birthday: Birthday,
}
