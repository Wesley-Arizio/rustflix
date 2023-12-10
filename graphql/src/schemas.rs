use crate::query::QueryRoot;
use juniper::{EmptyMutation, EmptySubscription, RootNode};

pub type Schema = RootNode<'static, QueryRoot, EmptyMutation, EmptySubscription>;

pub fn create_schema() -> Schema {
    Schema::new(
        QueryRoot {},
        EmptyMutation::default(),
        EmptySubscription::default(),
    )
}
