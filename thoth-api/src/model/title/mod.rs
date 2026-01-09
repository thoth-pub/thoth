use crate::model::{convert_to_jats, locale::LocaleCode, ConversionLimit, MarkupFormat};
use serde::{Deserialize, Serialize};
use thoth_errors::ThothResult;
use uuid::Uuid;

use crate::graphql::inputs::Direction;

#[cfg(feature = "backend")]
use crate::schema::title_history;
#[cfg(feature = "backend")]
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
    derive(juniper::GraphQLInputObject, Insertable, Clone),
    graphql(description = "Set of values required to define a new work's title"),
    diesel(table_name = work_title)
)]
#[derive(Default)]
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
    derive(juniper::GraphQLInputObject, AsChangeset, Clone),
    graphql(description = "Set of values required to update an existing work's title"),
    diesel(table_name = work_title, treat_none_as_null = true)
)]
pub struct PatchTitle {
    pub title_id: Uuid,
    pub work_id: Uuid,
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
    pub user_id: String,
    pub data: serde_json::Value,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct TitleHistory {
    pub title_history_id: Uuid,
    pub title_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub trait TitleProperties {
    fn title(&self) -> &str;
    fn subtitle(&self) -> Option<&str>;
    fn full_title(&self) -> &str;
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
                    format!("{_title} {_subtitle}")
                } else {
                    format!("{_title}: {_subtitle}")
                }
            },
        )
    }
    fn set_title(&mut self, value: String);
    fn set_subtitle(&mut self, value: Option<String>);
    fn set_full_title(&mut self, value: String);
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
            fn full_title(&self) -> &str {
                &self.full_title
            }
            fn locale_code(&self) -> &LocaleCode {
                &self.locale_code
            }
            fn canonical(&self) -> bool {
                self.canonical
            }
            fn set_title(&mut self, value: String) {
                self.title = value;
            }
            fn set_subtitle(&mut self, value: Option<String>) {
                self.subtitle = value;
            }
            fn set_full_title(&mut self, value: String) {
                self.full_title = value;
            }
        }
    };
}

title_properties!(Title);
title_properties!(NewTitle);
title_properties!(PatchTitle);

pub(crate) fn convert_title_to_jats<T>(data: &mut T, format: MarkupFormat) -> ThothResult<()>
where
    T: TitleProperties,
{
    let title = convert_to_jats(data.title().to_owned(), format, ConversionLimit::Title)?;
    let subtitle = data
        .subtitle()
        .map(|s| convert_to_jats(s.to_owned(), format, ConversionLimit::Title))
        .transpose()?;
    let full_title = convert_to_jats(data.full_title().to_owned(), format, ConversionLimit::Title)?;

    data.set_title(title);
    data.set_subtitle(subtitle);
    data.set_full_title(full_title);
    Ok(())
}

#[cfg(feature = "backend")]
pub mod crud;
mod policy;
#[cfg(feature = "backend")]
pub(crate) use policy::TitlePolicy;
