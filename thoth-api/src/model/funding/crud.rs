use super::{Funding, FundingField, FundingHistory, NewFunding, NewFundingHistory, PatchFunding};
use crate::graphql::types::inputs::FundingOrderBy;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{funding, funding_history};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Crud for Funding {
    type NewEntity = NewFunding;
    type PatchEntity = PatchFunding;
    type OrderByEntity = FundingOrderBy;
    type FilterParameter1 = ();
    type FilterParameter2 = ();
    type FilterParameter3 = ();
    type FilterParameter4 = ();

    fn pk(&self) -> Uuid {
        self.funding_id
    }

    fn all(
        db: &crate::db::PgPool,
        limit: i32,
        offset: i32,
        _: Option<String>,
        order: Self::OrderByEntity,
        publishers: Vec<Uuid>,
        parent_id_1: Option<Uuid>,
        parent_id_2: Option<Uuid>,
        _: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<Vec<Funding>> {
        use crate::schema::funding::dsl::*;
        let mut connection = db.get()?;
        let mut query = funding
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select(crate::schema::funding::all_columns)
            .into_boxed();

        query = match order.field {
            FundingField::FundingId => {
                apply_directional_order!(query, order.direction, order, funding_id)
            }
            FundingField::WorkId => {
                apply_directional_order!(query, order.direction, order, work_id)
            }
            FundingField::InstitutionId => {
                apply_directional_order!(query, order.direction, order, institution_id)
            }
            FundingField::Program => {
                apply_directional_order!(query, order.direction, order, program)
            }
            FundingField::ProjectName => {
                apply_directional_order!(query, order.direction, order, project_name)
            }
            FundingField::ProjectShortname => {
                apply_directional_order!(query, order.direction, order, project_shortname)
            }
            FundingField::GrantNumber => {
                apply_directional_order!(query, order.direction, order, grant_number)
            }
            FundingField::Jurisdiction => {
                apply_directional_order!(query, order.direction, order, jurisdiction)
            }
            FundingField::CreatedAt => {
                apply_directional_order!(query, order.direction, order, created_at)
            }
            FundingField::UpdatedAt => {
                apply_directional_order!(query, order.direction, order, updated_at)
            }
        };
        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(work_id.eq(pid));
        }
        if let Some(pid) = parent_id_2 {
            query = query.filter(institution_id.eq(pid));
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Funding>(&mut connection)
            .map_err(Into::into)
    }

    fn count(
        db: &crate::db::PgPool,
        _: Option<String>,
        _: Vec<Uuid>,
        _: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<i32> {
        use crate::schema::funding::dsl::*;
        let mut connection = db.get()?;

        // `SELECT COUNT(*)` in postgres returns a BIGINT, which diesel parses as i64. Juniper does
        // not implement i64 yet, only i32. The only sensible way, albeit shameful, to solve this
        // is converting i64 to string and then parsing it as i32. This should work until we reach
        // 2147483647 records - if you are fixing this bug, congratulations on book number 2147483647!
        funding
            .count()
            .get_result::<i64>(&mut connection)
            .map(|t| t.to_string().parse::<i32>().unwrap())
            .map_err(Into::into)
    }

    crud_methods!(funding::table, funding::dsl::funding);
}

publisher_id_impls!(Funding, NewFunding, PatchFunding, |s, db| {
    crate::model::work::Work::from_id(db, &s.work_id)?.publisher_id(db)
});

impl HistoryEntry for Funding {
    type NewHistoryEntity = NewFundingHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            funding_id: self.funding_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewFundingHistory {
    type MainEntity = FundingHistory;

    db_insert!(funding_history::table);
}
