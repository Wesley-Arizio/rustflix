use crate::{mutation::MutationRoot, query::QueryRoot, AppState};
use grpc_interfaces::auth::auth_client::AuthClient;
use juniper::{EmptySubscription, RootNode};
use tonic::transport::Channel;

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<AppState>>;

pub fn create_schema(auth_client: AuthClient<Channel>) -> Schema {
    Schema::new(
        QueryRoot {},
        MutationRoot::new(auth_client),
        EmptySubscription::<AppState>::default(),
    )
}
