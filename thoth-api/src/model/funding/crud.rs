use super::{Funding, FundingField, FundingHistory, NewFunding, NewFundingHistory, PatchFunding};
use crate::graphql::model::FundingOrderBy;
use crate::graphql::utils::Direction;
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
            FundingField::FundingId => match order.direction {
                Direction::Asc => query.order(funding_id.asc()),
                Direction::Desc => query.order(funding_id.desc()),
            },
            FundingField::WorkId => match order.direction {
                Direction::Asc => query.order(work_id.asc()),
                Direction::Desc => query.order(work_id.desc()),
            },
            FundingField::InstitutionId => match order.direction {
                Direction::Asc => query.order(institution_id.asc()),
                Direction::Desc => query.order(institution_id.desc()),
            },
            FundingField::Program => match order.direction {
                Direction::Asc => query.order(program.asc()),
                Direction::Desc => query.order(program.desc()),
            },
            FundingField::ProjectName => match order.direction {
                Direction::Asc => query.order(project_name.asc()),
                Direction::Desc => query.order(project_name.desc()),
            },
            FundingField::ProjectShortname => match order.direction {
                Direction::Asc => query.order(project_shortname.asc()),
                Direction::Desc => query.order(project_shortname.desc()),
            },
            FundingField::GrantNumber => match order.direction {
                Direction::Asc => query.order(grant_number.asc()),
                Direction::Desc => query.order(grant_number.desc()),
            },
            FundingField::Jurisdiction => match order.direction {
                Direction::Asc => query.order(jurisdiction.asc()),
                Direction::Desc => query.order(jurisdiction.desc()),
            },
            FundingField::CreatedAt => match order.direction {
                Direction::Asc => query.order(created_at.asc()),
                Direction::Desc => query.order(created_at.desc()),
            },
            FundingField::UpdatedAt => match order.direction {
                Direction::Asc => query.order(updated_at.asc()),
                Direction::Desc => query.order(updated_at.desc()),
            },
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

    fn publisher_id(&self, db: &crate::db::PgPool) -> ThothResult<Uuid> {
        crate::model::work::Work::from_id(db, &self.work_id)?.publisher_id(db)
    }

    crud_methods!(funding::table, funding::dsl::funding);
}

impl HistoryEntry for Funding {
    type NewHistoryEntity = NewFundingHistory;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            funding_id: self.funding_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewFundingHistory {
    type MainEntity = FundingHistory;

    db_insert!(funding_history::table);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_funding_pk() {
        let funding: Funding = Default::default();
        assert_eq!(funding.pk(), funding.funding_id);
    }

    #[test]
    fn test_new_funding_history_from_funding() {
        let funding: Funding = Default::default();
        let account_id: Uuid = Default::default();
        let new_funding_history = funding.new_history_entry(&account_id);
        assert_eq!(new_funding_history.funding_id, funding.funding_id);
        assert_eq!(new_funding_history.account_id, account_id);
        assert_eq!(
            new_funding_history.data,
            serde_json::Value::String(serde_json::to_string(&funding).unwrap())
        );
    }
}
