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

pub mod account;
pub mod contribution;
pub mod contributor;
#[cfg(feature = "backend")]
pub mod db;
pub mod errors;
pub mod funder;
pub mod funding;
#[cfg(feature = "backend")]
pub mod graphql;
pub mod imprint;
pub mod issue;
pub mod language;
pub mod price;
pub mod publication;
pub mod publisher;
#[cfg(feature = "backend")]
mod schema;
pub mod series;
pub mod subject;
pub mod work;

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
