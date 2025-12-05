use super::{
    LocaleCode, NewTitle, NewTitleHistory, PatchTitle, Title, TitleField, TitleHistory,
    TitleOrderBy,
};
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{title_history, work_title};
use crate::{crud_methods, db_insert};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl,
};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Title {
    pub(crate) fn canonical_from_work_id(
        db: &crate::db::PgPool,
        work_id: &Uuid,
    ) -> ThothResult<Self> {
        let mut connection = db.get()?;
        work_title::table
            .filter(work_title::work_id.eq(work_id))
            .filter(work_title::canonical.eq(true))
            .first::<Title>(&mut connection)
            .map_err(Into::into)
    }
}

impl Crud for Title {
    type NewEntity = NewTitle;
    type PatchEntity = PatchTitle;
    type OrderByEntity = TitleOrderBy;
    type FilterParameter1 = LocaleCode;
    type FilterParameter2 = ();
    type FilterParameter3 = ();
    type FilterParameter4 = ();

    fn pk(&self) -> Uuid {
        self.title_id
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
    ) -> ThothResult<Vec<Title>> {
        use crate::schema::work_title::dsl::*;

        let mut connection = db.get()?;
        let mut query = work_title
            .select(crate::schema::work_title::all_columns)
            .into_boxed();

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
                Direction::Asc => query.order(title.asc()),
                Direction::Desc => query.order(title.desc()),
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
                    .or(title.ilike(format!("%{filter}%")))
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
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<i32> {
        use crate::schema::work_title::dsl::{full_title, subtitle, title, work_title};

        let mut connection = db.get()?;
        let mut query = work_title.into_boxed();

        if let Some(filter) = filter {
            query = query.filter(
                full_title
                    .ilike(format!("%{filter}%"))
                    .or(title.ilike(format!("%{filter}%")))
                    .or(subtitle.ilike(format!("%{filter}%"))),
            );
        }

        query
            .count()
            .get_result::<i64>(&mut connection)
            .map(|t| t.to_string().parse::<i32>().unwrap())
            .map_err(Into::into)
    }

    fn publisher_id(&self, db: &crate::db::PgPool) -> ThothResult<Uuid> {
        let work = crate::model::work::Work::from_id(db, &self.work_id)?;
        <crate::model::work::Work as Crud>::publisher_id(&work, db)
    }

    crud_methods!(work_title::table, work_title::dsl::work_title);
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
