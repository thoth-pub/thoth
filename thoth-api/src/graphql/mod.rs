#[cfg(feature = "backend")]
pub mod model;
pub mod utils;

#[cfg(feature = "backend")]
pub use juniper::http::GraphQLRequest;
