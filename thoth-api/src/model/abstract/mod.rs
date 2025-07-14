use crate::model::locale::LocaleCode;
use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::graphql::utils::Direction;

#[cfg(feature = "backend")]
use crate::schema::abstract_history;
#[cfg(feature = "backend")]
use crate::schema::work_abstract;

#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(description = "BCP-47 code representing locale"),
    ExistingTypePath = "crate::schema::sql_types::AbstractType"
)]
#[derive(
    Debug, Copy, Clone, Default, PartialEq, Eq, Deserialize, Serialize, EnumString, Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum AbstractType {
    #[default]
    #[cfg_attr(feature = "backend", graphql(description = "Short"))]
    Short,
    #[cfg_attr(feature = "backend", graphql(description = "Long"))]
    Long,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting abstract list")
)]
pub enum AbstractField {
    AbstractId,
    WorkId,
    Content,
    LocaleCode,
    AbstractType,
    Canonical,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting titles list")
)]
pub struct AbstractOrderBy {
    pub field: AbstractField,
    pub direction: Direction,
}

impl Default for AbstractOrderBy {
    fn default() -> Self {
        Self {
            field: AbstractField::Canonical,
            direction: Direction::Desc,
        }
    }
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Abstract {
    pub abstract_id: Uuid,
    pub work_id: Uuid,
    pub content: String,
    pub locale_code: LocaleCode,
    pub abstract_type: AbstractType,
    pub canonical: bool,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable, Clone),
    graphql(description = "Set of values required to define a new work's abstract"),
    diesel(table_name = work_abstract)
)]
pub struct NewAbstract {
    pub work_id: Uuid,
    pub content: String,
    pub locale_code: LocaleCode,
    pub abstract_type: AbstractType,
    pub canonical: bool,
}

impl Default for NewAbstract {
    fn default() -> Self {
        Self {
            work_id: Default::default(),
            content: String::new(),
            locale_code: Default::default(),
            abstract_type: AbstractType::Short,
            canonical: false,
        }
    }
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset, Clone),
    graphql(description = "Set of values required to update an existing work's abstract"),
    diesel(table_name = work_abstract)
)]
pub struct PatchAbstract {
    pub abstract_id: Uuid,
    pub work_id: Uuid,
    pub content: String,
    pub locale_code: LocaleCode,
    pub abstract_type: AbstractType,
    pub canonical: bool,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    diesel(table_name = abstract_history)
)]
pub struct NewAbstractHistory {
    pub abstract_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct AbstractHistory {
    pub abstract_history_id: Uuid,
    pub abstract_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// pub trait TitleProperties {
//     fn title(&self) -> &str;
//     fn subtitle(&self) -> Option<&str>;
//     fn locale_code(&self) -> &LocaleCode;
//     fn canonical(&self) -> bool;
//     fn compile_fulltitle(&self) -> String {
//         self.subtitle().map_or_else(
//             || self.title().to_string(),
//             |_subtitle| {
//                 let _title = self.title();
//                 let _title = if _title.is_empty() {
//                     "Untitled"
//                 } else {
//                     _title
//                 };
//                 if _title.ends_with('?')
//                     || _title.ends_with('!')
//                     || _title.ends_with(':')
//                     || _title.ends_with('.')
//                 {
//                     format!("{} {}", _title, _subtitle)
//                 } else {
//                     format!("{}: {}", _title, _subtitle)
//                 }
//             },
//         )
//     }
// }

// macro_rules! title_properties {
//     ($t:ty) => {
//         impl TitleProperties for $t {
//             fn title(&self) -> &str {
//                 &self.title
//             }
//             fn subtitle(&self) -> Option<&str> {
//                 self.subtitle.as_deref()
//             }
//             fn locale_code(&self) -> &LocaleCode {
//                 &self.locale_code
//             }
//             fn canonical(&self) -> bool {
//                 self.canonical
//             }
//         }
//     };
// }

// title_properties!(Title);
// title_properties!(NewTitle);
// title_properties!(PatchTitle);

#[cfg(feature = "backend")]
pub mod crud;
