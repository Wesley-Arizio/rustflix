use database::{
    entities::users::{UserBy, UserRepository},
    traits::EntityRepository,
};
use juniper::graphql_object;
use sqlx::types::Uuid;

use crate::AppState;

pub struct QueryRoot;

#[graphql_object(context = AppState)]
impl QueryRoot {
    async fn hello(app_state: &AppState) -> String {
        let uuid = Uuid::default();
        let user = UserRepository::try_get(&app_state.db, UserBy::Id(uuid))
            .await
            .expect("Ok");
        println!("User: {:#?}", user);
        String::from("Hello World!")
    }
}
