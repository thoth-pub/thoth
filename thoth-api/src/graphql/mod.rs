pub mod inputs;
#[cfg(feature = "backend")]
pub mod model;

#[cfg(feature = "backend")]
pub use juniper::http::GraphQLRequest;
