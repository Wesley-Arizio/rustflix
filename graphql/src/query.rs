use juniper::graphql_object;

pub struct QueryRoot;

#[graphql_object]
impl QueryRoot {
    fn hello() -> String {
        String::from("Hello World!")
    }
}
