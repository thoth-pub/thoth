#[cfg(feature = "backend")]
use diesel::prelude::*;
#[cfg(feature = "backend")]
use dotenv::dotenv;
use chrono::naive::NaiveDateTime;
use serde::{Deserialize, Serialize};
#[cfg(feature = "backend")]
use jsonwebtoken::encode;
#[cfg(feature = "backend")]
use jsonwebtoken::EncodingKey;
#[cfg(feature = "backend")]
use jsonwebtoken::Header;
#[cfg(feature = "backend")]
use std::time::SystemTime;
#[cfg(feature = "backend")]
use std::time::UNIX_EPOCH;
#[cfg(feature = "backend")]
use std::env;
use uuid::Uuid;

#[cfg(feature = "backend")]
use crate::schema::account;
#[cfg(feature = "backend")]
use crate::errors::ThothError;
#[cfg(feature = "backend")]
use crate::db::Context;
#[cfg(feature = "backend")]
use crate::account::util::make_hash;
#[cfg(feature = "backend")]
use crate::account::util::make_salt;

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

#[derive(Deserialize, Serialize)]
pub struct Token {
    sub: String,
    exp: i64,
    iat: i64,
    jti: String,
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

impl Account {
    #[cfg(feature = "backend")]
    pub fn issue_token(&self, context: &Context) -> Result<String, ThothError> {
        const DEFAULT_TOKEN_VALIDITY: i64 = 3600;
        let connection = context.db.get().unwrap();
        dotenv().ok();
        let secret_str = env::var("SECRET_KEY").expect("SECRET_KEY must be set");
        let secret: &[u8] = secret_str.as_bytes();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ThothError::InternalError("Unable to set token iat".into()))?;
        let claim = Token {
            sub: self.email.clone(),
            exp: now.as_secs() as i64 + DEFAULT_TOKEN_VALIDITY,
            iat: now.as_secs() as i64,
            jti: Uuid::new_v4().to_string(),
        };
        let token = encode(
            &Header::default(),
            &claim,
            &EncodingKey::from_secret(secret),
        )
        .map_err(|_| ThothError::InternalError("Unable to create token".into()));

        use crate::schema::account::dsl;
        let updated_account = diesel::update(dsl::account.find(self.account_id))
            .set(dsl::token.eq(token.unwrap()))
            .get_result::<Account>(&connection)
            .expect("Unable to set token");
        Ok(updated_account.token.unwrap())
    }
}

#[cfg(feature = "backend")]
impl From<AccountData> for NewAccount {
    fn from(account_data: AccountData) -> Self {
        let AccountData {
            name,
            surname,
            email,
            password,
            is_admin,
            is_bot,
            ..
        } = account_data;

        let salt = make_salt();
        let hash = make_hash(&password, &salt).to_vec();
        Self {
            name,
            surname,
            email,
            hash,
            salt,
            is_admin,
            is_bot,
        }
    }
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
