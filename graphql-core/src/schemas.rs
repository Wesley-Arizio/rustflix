use crate::{mutation::MutationRoot, query::QueryRoot, Context};
use juniper::{EmptySubscription, RootNode};

use core::service::Core;

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<Context>>;

pub fn create_schema(core: Core) -> Schema {
    Schema::new(
        QueryRoot::new(core.clone()),
        MutationRoot::new(core),
        EmptySubscription::new(),
    )
}
