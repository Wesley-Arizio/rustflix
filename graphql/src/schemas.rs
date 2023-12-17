use crate::{mutation::MutationRoot, query::QueryRoot};
use juniper::{EmptySubscription, RootNode};

use core::service::Core;

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<()>>;

pub fn create_schema(core: Core) -> Schema {
    Schema::new(
        QueryRoot {},
        MutationRoot::new(core),
        EmptySubscription::<()>::default(),
    )
}
