use juniper::{EmptySubscription, FieldResult, RootNode};

pub struct QueryRoot;

#[juniper::graphql_object]
impl QueryRoot {
    fn hello_word() -> FieldResult<String> {
        Ok(String::from("Hello World"))
    }
}

pub struct MutationRoot;

#[juniper::graphql_object]
impl MutationRoot {
    fn hello_word() -> FieldResult<String> {
        Ok(String::from("Hello World"))
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {}, EmptySubscription::new())
}
