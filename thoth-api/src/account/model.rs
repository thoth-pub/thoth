use chrono::naive::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::Result;
use crate::errors::ThothError;
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
    pub token: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub resource_access: AccountAccess,
}

#[derive(Debug, Clone)]
pub struct DecodedToken {
    pub jwt: Option<Token>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, Default)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
}

impl DecodedToken {
    pub fn get_user_permissions(&self) -> AccountAccess {
        if let Some(jwt) = &self.jwt {
            jwt.namespace.clone()
        } else {
            AccountAccess {
                is_superuser: false,
                is_bot: false,
                linked_publishers: vec![],
            }
        }
    }
}

impl AccountAccess {
    pub fn can_edit(&self, publisher_id: Uuid) -> Result<()> {
        if self.is_superuser {
            Ok(())
        } else if let Some(_found) = &self
            .linked_publishers
            .iter()
            .position(|publisher| publisher.publisher_id == publisher_id)
        {
            Ok(())
        } else {
            Err(ThothError::Unauthorised.into())
        }
    }
}
