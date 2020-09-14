use chrono::naive::NaiveDate;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[cfg(feature = "backend")]
use crate::schema::account;

#[cfg_attr(feature = "backend", derive(Insertable, Queryable))]
#[cfg_attr(feature = "backend", table_name = "account")]
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Account {
    pub account_id: Uuid,
    pub name: String,
    pub surname: String,
    pub email: String,
    pub password: String,
    pub is_admin: bool,
    pub is_bot: bool,
    pub is_active: bool,
    pub registered: NaiveDate,
    pub token: Option<String>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Session {
    pub token: String,
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

