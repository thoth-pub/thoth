#![allow(clippy::extra_unused_lifetimes)]

#[cfg(feature = "backend")]
#[macro_use]
extern crate diesel;
#[cfg(feature = "backend")]
#[macro_use]
extern crate diesel_derive_enum;
#[cfg(feature = "backend")]
#[macro_use]
extern crate diesel_derive_newtype;
#[cfg(feature = "backend")]
extern crate diesel_migrations;
extern crate dotenv;
extern crate juniper;

pub mod account;
pub mod ast;
#[cfg(feature = "backend")]
pub mod db;
pub mod graphql;
#[macro_use]
pub mod model;
#[cfg(feature = "backend")]
pub mod redis;
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
