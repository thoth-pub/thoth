pub mod model;
pub mod types;

mod mutation;
mod query;

pub use juniper::http::GraphQLRequest;

pub use model::Context;
pub use mutation::MutationRoot;
pub use query::QueryRoot;

use juniper::{EmptySubscription, RootNode};

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<Context>>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {}, EmptySubscription::new())
}

#[cfg(test)]
mod tests;
