use crate::{query::QueryRoot, AppState};
use juniper::{EmptyMutation, EmptySubscription, RootNode};

pub type Schema =
    RootNode<'static, QueryRoot, EmptyMutation<AppState>, EmptySubscription<AppState>>;

pub fn create_schema() -> Schema {
    Schema::new(
        QueryRoot {},
        EmptyMutation::<AppState>::default(),
        EmptySubscription::<AppState>::default(),
    )
}
