use super::{
    LocaleCode, NewTitle, NewTitleHistory, PatchTitle, Title, TitleField, TitleHistory,
    TitleOrderBy,
};
use crate::model::{Crud, DbInsert, HistoryEntry, PublisherId};
use crate::schema::{title_history, work_title};
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
            TitleField::TitleId => {
                apply_directional_order!(query, order.direction, order, title_id)
            }
            TitleField::WorkId => apply_directional_order!(query, order.direction, order, work_id),
            TitleField::LocaleCode => {
                apply_directional_order!(query, order.direction, order, locale_code)
            }
            TitleField::FullTitle => {
                apply_directional_order!(query, order.direction, order, full_title)
            }
            TitleField::Title => apply_directional_order!(query, order.direction, order, title),
            TitleField::Subtitle => {
                apply_directional_order!(query, order.direction, order, subtitle)
            }
            TitleField::Canonical => {
                apply_directional_order!(query, order.direction, order, canonical)
            }
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

    crud_methods!(work_title::table, work_title::dsl::work_title);
}

publisher_id_impls!(Title, NewTitle, PatchTitle, |s, db| {
    let work = crate::model::work::Work::from_id(db, &s.work_id)?;
    <crate::model::work::Work as PublisherId>::publisher_id(&work, db)
});

impl HistoryEntry for Title {
    type NewHistoryEntity = NewTitleHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            title_id: self.title_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewTitleHistory {
    type MainEntity = TitleHistory;

    db_insert!(title_history::table);
}
