use super::LocaleCode;
use super::{
    Abstract, AbstractField, AbstractHistory, AbstractOrderBy, AbstractType, NewAbstract,
    NewAbstractHistory, PatchAbstract,
};
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::work_abstract::dsl;
use crate::schema::{abstract_history, work_abstract};
use diesel::{ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Abstract {
    fn canonical_from_work_id_and_type(
        db: &crate::db::PgPool,
        work_id: &Uuid,
        abstract_type: AbstractType,
    ) -> ThothResult<Self> {
        let mut connection = db.get()?;
        work_abstract::table
            .filter(work_abstract::work_id.eq(work_id))
            .filter(work_abstract::canonical.eq(true))
            .filter(work_abstract::abstract_type.eq(abstract_type))
            .first::<Abstract>(&mut connection)
            .map_err(Into::into)
    }

    pub(crate) fn short_canonical_from_work_id(
        db: &crate::db::PgPool,
        work_id: &Uuid,
    ) -> ThothResult<Self> {
        Self::canonical_from_work_id_and_type(db, work_id, AbstractType::Short)
    }

    pub(crate) fn long_canonical_from_work_id(
        db: &crate::db::PgPool,
        work_id: &Uuid,
    ) -> ThothResult<Self> {
        Self::canonical_from_work_id_and_type(db, work_id, AbstractType::Long)
    }
}

impl Crud for Abstract {
    type NewEntity = NewAbstract;
    type PatchEntity = PatchAbstract;
    type OrderByEntity = AbstractOrderBy;
    type FilterParameter1 = LocaleCode;
    type FilterParameter2 = ();
    type FilterParameter3 = AbstractType;
    type FilterParameter4 = ();

    fn pk(&self) -> Uuid {
        self.abstract_id
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
        abstract_type: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<Vec<Abstract>> {
        let mut connection = db.get()?;
        let mut query = dsl::work_abstract
            .select(crate::schema::work_abstract::all_columns)
            .into_boxed();

        query = match order.field {
            AbstractField::AbstractId => match order.direction {
                Direction::Asc => query.order(dsl::abstract_id.asc()),
                Direction::Desc => query.order(dsl::abstract_id.desc()),
            },
            AbstractField::WorkId => match order.direction {
                Direction::Asc => query.order(dsl::work_id.asc()),
                Direction::Desc => query.order(dsl::work_id.desc()),
            },
            AbstractField::LocaleCode => match order.direction {
                Direction::Asc => query.order(dsl::locale_code.asc()),
                Direction::Desc => query.order(dsl::locale_code.desc()),
            },
            AbstractField::AbstractType => match order.direction {
                Direction::Asc => query.order(dsl::abstract_type.asc()),
                Direction::Desc => query.order(dsl::abstract_type.desc()),
            },
            AbstractField::Content => match order.direction {
                Direction::Asc => query.order(dsl::content.asc()),
                Direction::Desc => query.order(dsl::content.desc()),
            },
            AbstractField::Canonical => match order.direction {
                Direction::Asc => query.order(dsl::canonical.asc()),
                Direction::Desc => query.order(dsl::canonical.desc()),
            },
        };

        if let Some(filter) = filter {
            query = query.filter(dsl::content.ilike(format!("%{filter}%")));
        }

        if let Some(pid) = parent_id_1 {
            query = query.filter(dsl::work_id.eq(pid));
        }

        if !locale_codes.is_empty() {
            query = query.filter(dsl::locale_code.eq_any(locale_codes));
        }

        if let Some(at) = abstract_type {
            query = query.filter(dsl::abstract_type.eq(at));
        }

        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Abstract>(&mut connection)
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
        let mut query = dsl::work_abstract.into_boxed();

        if let Some(filter) = filter {
            query = query.filter(dsl::content.ilike(format!("%{filter}%")));
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

    crud_methods!(work_abstract::table, work_abstract::dsl::work_abstract);
}

impl HistoryEntry for Abstract {
    type NewHistoryEntity = NewAbstractHistory;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            abstract_id: self.abstract_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewAbstractHistory {
    type MainEntity = AbstractHistory;

    db_insert!(abstract_history::table);
}
