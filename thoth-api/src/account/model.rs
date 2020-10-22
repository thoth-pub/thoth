use chrono::naive::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "backend")]
use crate::schema::account;

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Account {
    pub account_id: Uuid,
    pub name: String,
    pub surname: String,
    pub email: String,
    pub hash: Vec<u8>,
    pub salt: String,
    pub is_admin: bool,
    pub is_bot: bool,
    pub is_active: bool,
    pub registered: NaiveDateTime,
    pub token: Option<String>,
}

#[cfg_attr(feature = "backend", derive(Insertable))]
#[cfg_attr(feature = "backend", table_name = "account")]
pub struct NewAccount {
    pub name: String,
    pub surname: String,
    pub email: String,
    pub hash: Vec<u8>,
    pub salt: String,
    pub is_admin: bool,
    pub is_bot: bool,
}

#[derive(Debug)]
pub struct AccountData {
    pub name: String,
    pub surname: String,
    pub email: String,
    pub password: String,
    pub is_admin: bool,
    pub is_bot: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Token {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
}

#[derive(Clone)]
pub struct DecodedToken {
    pub jwt: Option<Token>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Session {
    pub token: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Login(pub Session);

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct LoginSession(pub Session);

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Logout(pub Session);

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct LogoutResponse;

impl Session {
    pub fn new<T>(token: T) -> Self
    where
        String: From<T>,
    {
        Self {
            token: token.into(),
        }
    }
}
