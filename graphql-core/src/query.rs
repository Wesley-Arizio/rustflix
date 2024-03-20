use crate::Context;
use core::service::Core;
use juniper::{graphql_object, FieldError, FieldResult, Value};

pub struct QueryRoot {
    pub core: Core,
}

impl QueryRoot {
    pub fn new(core: Core) -> Self {
        Self { core }
    }
}

#[graphql_object(context = Context)]
impl QueryRoot {
    async fn hello(&self, ctx: &Context) -> FieldResult<String> {
        if let Some(session) = &ctx.session {
            if let Ok(Some(cookie)) = session.get::<String>("sid") {
                self.core.authenticate(cookie).await?;
                Ok(String::from("Hello World!"))
            } else {
                Err(FieldError::new("Invalid Credentials", Value::Null))
            }
        } else {
            eprintln!("cannot retrieve session from context");
            Err(FieldError::new("Internal Server Error", Value::Null))
        }
    }
}
