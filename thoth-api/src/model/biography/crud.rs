use super::LocaleCode;
use super::{
    Biography, BiographyField, BiographyHistory, BiographyOrderBy, NewBiography,
    NewBiographyHistory, PatchBiography,
};
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::biography::dsl::*;
use crate::schema::{biography, biography_history};
use crate::{crud_methods, db_insert};
use diesel::{ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Crud for Biography {
    type NewEntity = NewBiography;
    type PatchEntity = PatchBiography;
    type OrderByEntity = BiographyOrderBy;
    type FilterParameter1 = LocaleCode;
    type FilterParameter2 = ();
    type FilterParameter3 = ();
    type FilterParameter4 = ();

    fn pk(&self) -> Uuid {
        self.biography_id
    }

    fn publisher_id(&self, db: &crate::db::PgPool) -> ThothResult<Uuid> {
        let contribution =
            crate::model::contribution::Contribution::from_id(db, &self.contribution_id)?;
        let work = crate::model::work::Work::from_id(db, &contribution.work_id)?;
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
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<Vec<Biography>> {
        let mut connection = db.get()?;
        let mut query = biography
            .select((
                biography_id,
                contribution_id,
                content,
                canonical,
                locale_code,
            ))
            .into_boxed();

        query = match order.field {
            BiographyField::BiographyId => match order.direction {
                Direction::Asc => query.order(biography_id.asc()),
                Direction::Desc => query.order(biography_id.desc()),
            },
            BiographyField::ContributionId => match order.direction {
                Direction::Asc => query.order(contribution_id.asc()),
                Direction::Desc => query.order(contribution_id.desc()),
            },
            BiographyField::Content => match order.direction {
                Direction::Asc => query.order(content.asc()),
                Direction::Desc => query.order(content.desc()),
            },
            BiographyField::Canonical => match order.direction {
                Direction::Asc => query.order(canonical.asc()),
                Direction::Desc => query.order(canonical.desc()),
            },
            BiographyField::LocaleCode => match order.direction {
                Direction::Asc => query.order(locale_code.asc()),
                Direction::Desc => query.order(locale_code.desc()),
            },
        };

        if let Some(filter) = filter {
            query = query.filter(content.ilike(format!("%{filter}%")));
        }

        if let Some(pid) = parent_id_1 {
            query = query.filter(contribution_id.eq(pid));
        }

        if !locale_codes.is_empty() {
            query = query.filter(locale_code.eq_any(&locale_codes));
        }

        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Biography>(&mut connection)
            .map_err(Into::into)
    }

    fn count(
        db: &crate::db::PgPool,
        filter: Option<String>,
        _: Vec<Uuid>,
        _: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<i32> {
        let mut connection = db.get()?;
        let mut query = biography.into_boxed();

        if let Some(filter) = filter {
            query = query.filter(biography::content.ilike(format!("%{filter}%")));
        }

        query
            .count()
            .get_result::<i64>(&mut connection)
            .map(|t| t.to_string().parse::<i32>().unwrap())
            .map_err(Into::into)
    }

    crud_methods!(biography::table, biography::dsl::biography);
}

impl HistoryEntry for Biography {
    type NewHistoryEntity = NewBiographyHistory;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            biography_id: self.biography_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewBiographyHistory {
    type MainEntity = BiographyHistory;

    db_insert!(biography_history::table);
}
