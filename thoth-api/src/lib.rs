#[cfg(feature = "backend")]
#[macro_use]
extern crate diesel;
#[cfg(feature = "backend")]
#[macro_use]
extern crate diesel_derive_enum;
#[cfg(feature = "backend")]
#[macro_use]
extern crate diesel_migrations;
extern crate dotenv;
#[macro_use]
extern crate juniper;

#[cfg(feature = "backend")]
pub mod db;
pub mod errors;
#[cfg(feature = "backend")]
pub mod graphql_handlers;
pub mod models;
pub mod request;
pub mod response;
#[cfg(feature = "backend")]
mod schema;

macro_rules! apis {
    ($($name:ident => $content:expr,)*) => (
        $(#[allow(missing_docs)] pub const $name: &str = $content;)*
    )
}

apis! {
    API_URL_LOGIN_CREDENTIALS => "login/credentials",
    API_URL_LOGIN_SESSION => "login/session",
    API_URL_LOGOUT => "logout",
}
