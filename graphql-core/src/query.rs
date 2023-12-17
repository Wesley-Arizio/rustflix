use juniper::graphql_object;

pub struct QueryRoot;

#[graphql_object()]
impl QueryRoot {
    async fn hello() -> String {
        String::from("Hello World!")
    }
}
