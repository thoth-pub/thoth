use crate::model::locale::LocaleCode;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::graphql::utils::Direction;
use crate::model::HistoryEntry;
use crate::schema::title_history;
use crate::schema::work_title;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting title list")
)]
pub enum TitleField {
    TitleId,
    WorkId,
    FullTitle,
    Title,
    Subtitle,
    Canonical,
    LocaleCode,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting titles list")
)]
pub struct TitleOrderBy {
    pub field: TitleField,
    pub direction: Direction,
}

impl Default for TitleOrderBy {
    fn default() -> Self {
        Self {
            field: TitleField::Canonical,
            direction: Direction::Desc,
        }
    }
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Title {
    pub title_id: Uuid,
    pub work_id: Uuid,
    pub full_title: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub canonical: bool,
    pub locale_code: LocaleCode,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    graphql(description = "Set of values required to define a new work's title"),
    diesel(table_name = work_title)
)]
pub struct NewTitle {
    pub work_id: Uuid,
    pub locale_code: LocaleCode,
    pub full_title: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub canonical: bool,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    graphql(description = "Set of values required to update an existing work's title"),
    diesel(table_name = work_title, treat_none_as_null = true)
)]
pub struct PatchTitle {
    pub work_id: Uuid,
    pub title_id: Uuid,
    pub locale_code: LocaleCode,
    pub full_title: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub canonical: bool,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    diesel(table_name = title_history)
)]
pub struct NewTitleHistory {
    pub title_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct TitleHistory {
    pub title_history_id: Uuid,
    pub title_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl HistoryEntry for Title {
    type NewHistoryEntity = NewTitleHistory;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            title_id: self.title_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

pub trait TitleProperties {
    fn title(&self) -> &str;
    fn subtitle(&self) -> Option<&str>;
    fn locale_code(&self) -> &LocaleCode;
    fn canonical(&self) -> bool;
    fn compile_fulltitle(&self) -> String {
        self.subtitle().map_or_else(
            || self.title().to_string(),
            |_subtitle| {
                let _title = self.title();
                let _title = if _title.is_empty() {
                    "Untitled"
                } else {
                    _title
                };
                if _title.ends_with('?')
                    || _title.ends_with('!')
                    || _title.ends_with(':')
                    || _title.ends_with('.')
                {
                    format!("{} {}", _title, _subtitle)
                } else {
                    format!("{}: {}", _title, _subtitle)
                }
            },
        )
    }
}

macro_rules! title_properties {
    ($t:ty) => {
        impl TitleProperties for $t {
            fn title(&self) -> &str {
                &self.title
            }
            fn subtitle(&self) -> Option<&str> {
                self.subtitle.as_deref()
            }
            fn locale_code(&self) -> &LocaleCode {
                &self.locale_code
            }
            fn canonical(&self) -> bool {
                self.canonical
            }
        }
    };
}

title_properties!(Title);
title_properties!(NewTitle);
title_properties!(PatchTitle);

#[cfg(feature = "backend")]
pub mod crud;
