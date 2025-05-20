use crate::model::locale::LocaleCode;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry, ThothResult};
use crate::schema::title;
use crate::schema::title::dsl::*;
use crate::schema::title_history;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleField {
    TitleId,
    WorkId,
    LocaleId,
    FullTitle,
    Title,
    Subtitle,
    Canonical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleOrderBy {
    pub field: TitleField,
    pub direction: Direction,
}

impl Default for TitleOrderBy {
    fn default() -> Self {
        Self {
            field: TitleField::TitleId,
            direction: Direction::Asc,
        }
    }
}

#[derive(Debug, Clone, Queryable, Serialize, Deserialize, PartialEq, Eq)]
#[diesel(table_name = title)]
pub struct Title {
    pub title_id: Uuid,
    pub work_id: Uuid,
    pub locale_code: LocaleCode,
    pub full_title: String,
    #[diesel(column_name = "title")]
    pub title_: String,
    pub subtitle: Option<String>,
    pub canonical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleWithRelations {
    pub title_id: Uuid,
    pub work_id: Uuid,
    pub locale_code: LocaleCode,
    pub full_title: String,
    pub title_: String,
    pub subtitle: Option<String>,
    pub canonical: bool,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    graphql(description = "Set of values required to define a new written text that can be published"),
    diesel(table_name = title)
)]
pub struct NewTitle {
    pub work_id: Uuid,
    pub locale_code: LocaleCode,
    pub full_title: String,
    pub title_: String,
    pub subtitle: Option<String>,
    pub canonical: bool,
}

#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize, PartialEq, Eq)]
#[diesel(table_name = title)]
pub struct PatchTitle {
    pub locale_code: LocaleCode,
    pub full_title: String,
    pub title_: String,
    pub subtitle: Option<String>,
    pub canonical: bool,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = title_history)]
pub struct NewTitleHistory {
    pub title_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Queryable, Serialize, Deserialize)]
#[diesel(table_name = title_history)]
pub struct TitleHistory {
    pub title_history_id: Uuid,
    pub title_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Crud for Title {
    type NewEntity = NewTitle;
    type PatchEntity = PatchTitle;
    type OrderByEntity = TitleOrderBy;
    type FilterParameter1 = ();
    type FilterParameter2 = ();
    type FilterParameter3 = ();

    fn pk(&self) -> Uuid {
        self.title_id
    }

    fn publisher_id(&self, db: &crate::db::PgPool) -> ThothResult<Uuid> {
        let work = crate::model::work::Work::from_id(db, &self.work_id)?;
        <crate::model::work::Work as Crud>::publisher_id(&work, db)
    }

    fn all(
        db: &crate::db::PgPool,
        limit: i32,
        offset: i32,
        filter: Option<String>,
        order: Self::OrderByEntity,
        _: Vec<Uuid>,
        _: Option<Uuid>,
        _: Option<Uuid>,
        _: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
    ) -> ThothResult<Vec<Title>> {
        let mut connection = db.get()?;
        let mut query = title::table.into_boxed();

        query = match order.field {
            TitleField::TitleId => match order.direction {
                Direction::Asc => query.order(title_id.asc()),
                Direction::Desc => query.order(title_id.desc()),
            },
            TitleField::WorkId => match order.direction {
                Direction::Asc => query.order(work_id.asc()),
                Direction::Desc => query.order(work_id.desc()),
            },
            TitleField::LocaleId => match order.direction {
                Direction::Asc => query.order(locale_code.asc()),
                Direction::Desc => query.order(locale_code.desc()),
            },
            TitleField::FullTitle => match order.direction {
                Direction::Asc => query.order(full_title.asc()),
                Direction::Desc => query.order(full_title.desc()),
            },
            TitleField::Title => match order.direction {
                Direction::Asc => query.order(title_.asc()),
                Direction::Desc => query.order(title_.desc()),
            },
            TitleField::Subtitle => match order.direction {
                Direction::Asc => query.order(subtitle.asc()),
                Direction::Desc => query.order(subtitle.desc()),
            },
            TitleField::Canonical => match order.direction {
                Direction::Asc => query.order(canonical.asc()),
                Direction::Desc => query.order(canonical.desc()),
            },
        };

        if let Some(filter) = filter {
            query = query.filter(
                full_title
                    .ilike(format!("%{filter}%"))
                    .or(title_.ilike(format!("%{filter}%")))
                    .or(subtitle.ilike(format!("%{filter}%"))),
            );
        }

        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Title>(&mut connection)
            .map_err(Into::into)
    }

    fn count(
        db: &crate::db::PgPool,
        filter: Option<String>,
        _: Vec<Uuid>,
        _: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
    ) -> ThothResult<i32> {
        let mut connection = db.get()?;
        let mut query = title::table.into_boxed();

        if let Some(filter) = filter {
            query = query.filter(
                full_title
                    .ilike(format!("%{filter}%"))
                    .or(title_.ilike(format!("%{filter}%")))
                    .or(subtitle.ilike(format!("%{filter}%"))),
            );
        }

        query
            .count()
            .get_result::<i64>(&mut connection)
            .map(|t| t.to_string().parse::<i32>().unwrap())
            .map_err(Into::into)
    }

    crud_methods!(title::table, title::dsl::title);
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

impl DbInsert for NewTitleHistory {
    type MainEntity = TitleHistory;

    db_insert!(title_history::table);
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
                &self.title_
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
title_properties!(TitleWithRelations);
title_properties!(NewTitle);
title_properties!(PatchTitle);
