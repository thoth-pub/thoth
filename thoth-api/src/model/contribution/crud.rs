use super::{
    Contribution, ContributionField, ContributionHistory, ContributionType, NewContribution,
    NewContributionHistory, PatchContribution,
};
use crate::graphql::types::inputs::ContributionOrderBy;
use crate::model::{Crud, DbInsert, HistoryEntry, Reorder};
use crate::schema::{contribution, contribution_history};
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, NullableExpressionMethods, QueryDsl,
    RunQueryDsl,
};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Crud for Contribution {
    type NewEntity = NewContribution;
    type PatchEntity = PatchContribution;
    type OrderByEntity = ContributionOrderBy;
    type FilterParameter1 = ContributionType;
    type FilterParameter2 = ();
    type FilterParameter3 = ();
    type FilterParameter4 = ();

    fn pk(&self) -> Uuid {
        self.contribution_id
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
        contribution_types: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<Vec<Contribution>> {
        use crate::schema::contribution::dsl::*;

        let mut connection = db.get()?;
        let mut query = contribution
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select(crate::schema::contribution::all_columns)
            .into_boxed();

        query = match order.field {
            ContributionField::ContributionId => apply_directional_order!(query, order.direction, order_by, contribution_id),
            ContributionField::WorkId => apply_directional_order!(query, order.direction, order_by, work_id, contribution_id),
            ContributionField::ContributorId => apply_directional_order!(query, order.direction, order_by, contributor_id, contribution_id),
            ContributionField::ContributionType => apply_directional_order!(query, order.direction, order_by, contribution_type, contribution_id),
            ContributionField::MainContribution => apply_directional_order!(query, order.direction, order_by, main_contribution, contribution_id),
            ContributionField::Biography => {
                let biography_content = crate::schema::biography::table
                    .select(crate::schema::biography::content.nullable())
                    .filter(crate::schema::biography::contribution_id.eq(contribution_id))
                    .order((
                        crate::schema::biography::canonical.desc(),
                        crate::schema::biography::biography_id.asc(),
                    ))
                    .limit(1)
                    .single_value();
                apply_directional_order!(query, order.direction, order_by, biography_content, contribution_id)
            }
            ContributionField::CreatedAt => apply_directional_order!(query, order.direction, order_by, created_at, contribution_id),
            ContributionField::UpdatedAt => apply_directional_order!(query, order.direction, order_by, updated_at, contribution_id),
            ContributionField::FirstName => apply_directional_order!(query, order.direction, order_by, first_name, contribution_id),
            ContributionField::LastName => apply_directional_order!(query, order.direction, order_by, last_name, contribution_id),
            ContributionField::FullName => apply_directional_order!(query, order.direction, order_by, full_name, contribution_id),
            ContributionField::ContributionOrdinal => apply_directional_order!(query, order.direction, order_by, contribution_ordinal, contribution_id),
        };
        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(work_id.eq(pid));
        }
        if let Some(pid) = parent_id_2 {
            query = query.filter(contributor_id.eq(pid));
        }
        if !contribution_types.is_empty() {
            query = query.filter(contribution_type.eq_any(contribution_types));
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Contribution>(&mut connection)
            .map_err(Into::into)
    }

    fn count(
        db: &crate::db::PgPool,
        _: Option<String>,
        _: Vec<Uuid>,
        contribution_types: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<i32> {
        use crate::schema::contribution::dsl::*;
        let mut connection = db.get()?;
        let mut query = contribution.into_boxed();
        if !contribution_types.is_empty() {
            query = query.filter(contribution_type.eq_any(contribution_types));
        }

        // `SELECT COUNT(*)` in postgres returns a BIGINT, which diesel parses as i64. Juniper does
        // not implement i64 yet, only i32. The only sensible way, albeit shameful, to solve this
        // is converting i64 to string and then parsing it as i32. This should work until we reach
        // 2147483647 records - if you are fixing this bug, congratulations on book number 2147483647!
        query
            .count()
            .get_result::<i64>(&mut connection)
            .map(|t| t.to_string().parse::<i32>().unwrap())
            .map_err(Into::into)
    }

    crud_methods!(contribution::table, contribution::dsl::contribution);
}

publisher_id_impls!(Contribution, NewContribution, PatchContribution, |s, db| {
    crate::model::work::Work::from_id(db, &s.work_id)?.publisher_id(db)
});

impl HistoryEntry for Contribution {
    type NewHistoryEntity = NewContributionHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            contribution_id: self.contribution_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewContributionHistory {
    type MainEntity = ContributionHistory;

    db_insert!(contribution_history::table);
}

impl Reorder for Contribution {
    db_change_ordinal!(
        contribution::table,
        contribution::contribution_ordinal,
        "contribution_contribution_ordinal_work_id_uniq"
    );

    fn get_other_objects(
        &self,
        connection: &mut diesel::PgConnection,
    ) -> ThothResult<Vec<(Uuid, i32)>> {
        contribution::table
            .select((
                contribution::contribution_id,
                contribution::contribution_ordinal,
            ))
            .filter(
                contribution::work_id
                    .eq(self.work_id)
                    .and(contribution::contribution_id.ne(self.contribution_id)),
            )
            .load::<(Uuid, i32)>(connection)
            .map_err(Into::into)
    }
}
