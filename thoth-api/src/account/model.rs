use chrono::naive::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "backend")]
use crate::schema::account;
#[cfg(feature = "backend")]
use crate::schema::publisher_account;

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Account {
    pub account_id: Uuid,
    pub name: String,
    pub surname: String,
    pub email: String,
    pub hash: Vec<u8>,
    pub salt: String,
    pub is_superuser: bool,
    pub is_bot: bool,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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
    pub is_superuser: bool,
    pub is_bot: bool,
}

#[derive(Debug)]
pub struct AccountData {
    pub name: String,
    pub surname: String,
    pub email: String,
    pub password: String,
    pub is_superuser: bool,
    pub is_bot: bool,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct PublisherAccount {
    pub account_id: Uuid,
    pub publisher_id: Uuid,
    pub is_admin: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[cfg_attr(feature = "backend", derive(Insertable))]
#[cfg_attr(feature = "backend", table_name = "publisher_account")]
pub struct NewPublisherAccount {
    pub account_id: Uuid,
    pub publisher_id: Uuid,
    pub is_admin: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountAccess {
    pub is_superuser: bool,
    pub is_bot: bool,
    pub linked_publishers: Vec<LinkedPublisher>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedPublisher {
    pub publisher_id: Uuid,
    pub is_admin: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Token {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
    #[serde(rename = "https://thoth.pub/resource_access")]
    pub namespace: AccountAccess,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDetails {
    pub account_id: Uuid,
    pub name: String,
    pub surname: String,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub resource_access: AccountAccess,
}

#[derive(Debug, Clone)]
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

#[cfg_attr(feature = "backend", derive(AsChangeset), table_name = "account")]
pub struct NewPassword {
    pub email: String,
    pub hash: Vec<u8>,
    pub salt: String,
}

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
