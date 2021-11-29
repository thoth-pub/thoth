use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::account;
#[cfg(feature = "backend")]
use crate::schema::publisher_account;
use thoth_errors::ThothError;
use thoth_errors::ThothResult;

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
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
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
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(feature = "backend", derive(Insertable))]
#[cfg_attr(feature = "backend", table_name = "publisher_account")]
pub struct NewPublisherAccount {
    pub account_id: Uuid,
    pub publisher_id: Uuid,
    pub is_admin: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountAccess {
    pub is_superuser: bool,
    pub is_bot: bool,
    pub linked_publishers: Vec<LinkedPublisher>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountDetails {
    pub account_id: Uuid,
    pub name: String,
    pub surname: String,
    pub email: String,
    pub token: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
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

#[cfg_attr(feature = "backend", derive(AsChangeset), table_name = "account")]
pub struct NewPassword {
    pub email: String,
    pub hash: Vec<u8>,
    pub salt: String,
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
    pub fn can_edit(&self, publisher_id: Uuid) -> ThothResult<()> {
        if self.is_superuser
            || self
                .linked_publishers
                .iter()
                .any(|publisher| publisher.publisher_id == publisher_id)
        {
            Ok(())
        } else {
            Err(ThothError::Unauthorised)
        }
    }

    pub fn restricted_to(&self) -> Option<Vec<String>> {
        if self.is_superuser {
            None
        } else {
            Some(
                self.linked_publishers
                    .iter()
                    .map(|publisher| publisher.publisher_id.to_string())
                    .collect(),
            )
        }
    }
}
