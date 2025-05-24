use super::{NewTitle, NewTitleHistory, PatchTitle, Title, TitleField, TitleHistory, TitleOrderBy};
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use super::LocaleCode;
use crate::schema::{title, title_history};
use crate::schema::title::dsl::*;
use crate::{crud_methods, db_insert};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl,
};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Crud for Title {
    type NewEntity = NewTitle;
    type PatchEntity = PatchTitle;
    type OrderByEntity = TitleOrderBy;
    type FilterParameter1 = LocaleCode;
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
        parent_id_1: Option<Uuid>,
        _: Option<Uuid>,
        locale_codes: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
    ) -> ThothResult<Vec<Title>> {
        let mut connection = db.get()?;
        let mut query = title.select(crate::schema::title::all_columns).into_boxed();

        query = match order.field {
            TitleField::TitleId => match order.direction {
                Direction::Asc => query.order(title_id.asc()),
                Direction::Desc => query.order(title_id.desc()),
            },
            TitleField::WorkId => match order.direction {
                Direction::Asc => query.order(work_id.asc()),
                Direction::Desc => query.order(work_id.desc()),
            },
            TitleField::LocaleCode => match order.direction {
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

        if let Some(pid) = parent_id_1 {
            query = query.filter(work_id.eq(pid));
        }

        if !locale_codes.is_empty() {
            query = query.filter(locale_code.eq_any(locale_codes));
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
        let mut query = title.into_boxed();

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

impl DbInsert for NewTitleHistory {
    type MainEntity = TitleHistory;

    db_insert!(title_history::table);
}
