use super::{
    Affiliation, AffiliationField, AffiliationHistory, AffiliationOrderBy, NewAffiliation,
    NewAffiliationHistory, PatchAffiliation,
};
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{affiliation, affiliation_history};
use crate::{crud_methods, db_insert};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
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
            AffiliationField::AffiliationId => match order.direction {
                Direction::Asc => query.order(affiliation_id.asc()),
                Direction::Desc => query.order(affiliation_id.desc()),
            },
            AffiliationField::ContributionId => match order.direction {
                Direction::Asc => query.order(contribution_id.asc()),
                Direction::Desc => query.order(contribution_id.desc()),
            },
            AffiliationField::InstitutionId => match order.direction {
                Direction::Asc => query.order(institution_id.asc()),
                Direction::Desc => query.order(institution_id.desc()),
            },
            AffiliationField::AffiliationOrdinal => match order.direction {
                Direction::Asc => query.order(affiliation_ordinal.asc()),
                Direction::Desc => query.order(affiliation_ordinal.desc()),
            },
            AffiliationField::Position => match order.direction {
                Direction::Asc => query.order(position.asc()),
                Direction::Desc => query.order(position.desc()),
            },
            AffiliationField::CreatedAt => match order.direction {
                Direction::Asc => query.order(created_at.asc()),
                Direction::Desc => query.order(created_at.desc()),
            },
            AffiliationField::UpdatedAt => match order.direction {
                Direction::Asc => query.order(updated_at.asc()),
                Direction::Desc => query.order(updated_at.desc()),
            },
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

    fn publisher_id(&self, db: &crate::db::PgPool) -> ThothResult<Uuid> {
        crate::model::contribution::Contribution::from_id(db, &self.contribution_id)?
            .publisher_id(db)
    }

    crud_methods!(affiliation::table, affiliation::dsl::affiliation);
}

impl HistoryEntry for Affiliation {
    type NewHistoryEntity = NewAffiliationHistory;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            affiliation_id: self.affiliation_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewAffiliationHistory {
    type MainEntity = AffiliationHistory;

    db_insert!(affiliation_history::table);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_affiliation_pk() {
        let affiliation: Affiliation = Default::default();
        assert_eq!(affiliation.pk(), affiliation.affiliation_id);
    }

    #[test]
    fn test_new_affiliation_history_from_affiliation() {
        let affiliation: Affiliation = Default::default();
        let account_id: Uuid = Default::default();
        let new_affiliation_history = affiliation.new_history_entry(&account_id);
        assert_eq!(
            new_affiliation_history.affiliation_id,
            affiliation.affiliation_id
        );
        assert_eq!(new_affiliation_history.account_id, account_id);
        assert_eq!(
            new_affiliation_history.data,
            serde_json::Value::String(serde_json::to_string(&affiliation).unwrap())
        );
    }
}
