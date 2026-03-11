use super::{
    Affiliation, AffiliationField, AffiliationHistory, AffiliationOrderBy, NewAffiliation,
    NewAffiliationHistory, PatchAffiliation,
};
use crate::model::{Crud, DbInsert, HistoryEntry, Reorder};
use crate::schema::{affiliation, affiliation_history};
use diesel::{BoolExpressionMethods, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Crud for Affiliation {
    type NewEntity = NewAffiliation;
    type PatchEntity = PatchAffiliation;
    type OrderByEntity = AffiliationOrderBy;
    type FilterParameter1 = ();
    type FilterParameter2 = ();
    type FilterParameter3 = ();
    type FilterParameter4 = ();

    fn pk(&self) -> Uuid {
        self.affiliation_id
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
    ) -> ThothResult<Vec<Affiliation>> {
        use crate::schema::affiliation::dsl::*;
        let mut connection = db.get()?;
        let mut query =
            affiliation
                .inner_join(crate::schema::contribution::table.inner_join(
                    crate::schema::work::table.inner_join(crate::schema::imprint::table),
                ))
                .select(crate::schema::affiliation::all_columns)
                .into_boxed();

        query = match order.field {
            AffiliationField::AffiliationId => {
                apply_directional_order!(query, order.direction, order, affiliation_id)
            }
            AffiliationField::ContributionId => {
                apply_directional_order!(query, order.direction, order, contribution_id)
            }
            AffiliationField::InstitutionId => {
                apply_directional_order!(query, order.direction, order, institution_id)
            }
            AffiliationField::AffiliationOrdinal => {
                apply_directional_order!(query, order.direction, order, affiliation_ordinal)
            }
            AffiliationField::Position => {
                apply_directional_order!(query, order.direction, order, position)
            }
            AffiliationField::CreatedAt => {
                apply_directional_order!(query, order.direction, order, created_at)
            }
            AffiliationField::UpdatedAt => {
                apply_directional_order!(query, order.direction, order, updated_at)
            }
        };
        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(institution_id.eq(pid));
        }
        if let Some(pid) = parent_id_2 {
            query = query.filter(contribution_id.eq(pid));
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Affiliation>(&mut connection)
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
        use crate::schema::affiliation::dsl::*;
        let mut connection = db.get()?;

        // `SELECT COUNT(*)` in postgres returns a BIGINT, which diesel parses as i64. Juniper does
        // not implement i64 yet, only i32. The only sensible way, albeit shameful, to solve this
        // is converting i64 to string and then parsing it as i32. This should institution until we reach
        // 2147483647 records - if you are fixing this bug, congratulations on book number 2147483647!
        affiliation
            .count()
            .get_result::<i64>(&mut connection)
            .map(|t| t.to_string().parse::<i32>().unwrap())
            .map_err(Into::into)
    }

    crud_methods!(affiliation::table, affiliation::dsl::affiliation);
}

publisher_id_impls!(Affiliation, NewAffiliation, PatchAffiliation, |s, db| {
    crate::model::contribution::Contribution::from_id(db, &s.contribution_id)?.publisher_id(db)
});

impl HistoryEntry for Affiliation {
    type NewHistoryEntity = NewAffiliationHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            affiliation_id: self.affiliation_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewAffiliationHistory {
    type MainEntity = AffiliationHistory;

    db_insert!(affiliation_history::table);
}

impl Reorder for Affiliation {
    db_change_ordinal!(
        affiliation::table,
        affiliation::affiliation_ordinal,
        "affiliation_affiliation_ordinal_contribution_id_uniq"
    );

    fn get_other_objects(
        &self,
        connection: &mut diesel::PgConnection,
    ) -> ThothResult<Vec<(Uuid, i32)>> {
        affiliation::table
            .select((
                affiliation::affiliation_id,
                affiliation::affiliation_ordinal,
            ))
            .filter(
                affiliation::contribution_id
                    .eq(self.contribution_id)
                    .and(affiliation::affiliation_id.ne(self.affiliation_id)),
            )
            .load::<(Uuid, i32)>(connection)
            .map_err(Into::into)
    }
}
