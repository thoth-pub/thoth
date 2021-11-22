use super::{
    Contribution, ContributionField, ContributionHistory, ContributionType, NewContribution,
    NewContributionHistory, PatchContribution,
};
use crate::graphql::model::ContributionOrderBy;
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{contribution, contribution_history};
use crate::{crud_methods, db_insert};
use diesel::dsl::any;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

impl Crud for Contribution {
    type NewEntity = NewContribution;
    type PatchEntity = PatchContribution;
    type OrderByEntity = ContributionOrderBy;
    type FilterParameter1 = ContributionType;
    type FilterParameter2 = ();

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
        _: Option<Self::FilterParameter2>,
    ) -> ThothResult<Vec<Contribution>> {
        use crate::schema::contribution::dsl;
        let connection = db.get().unwrap();
        let mut query = dsl::contribution
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select((
                dsl::contribution_id,
                dsl::work_id,
                dsl::contributor_id,
                dsl::contribution_type,
                dsl::main_contribution,
                dsl::biography,
                dsl::institution,
                dsl::created_at,
                dsl::updated_at,
                dsl::first_name,
                dsl::last_name,
                dsl::full_name,
                dsl::contribution_ordinal,
            ))
            .into_boxed();

        match order.field {
            ContributionField::ContributionId => match order.direction {
                Direction::Asc => query = query.order(dsl::contribution_id.asc()),
                Direction::Desc => query = query.order(dsl::contribution_id.desc()),
            },
            ContributionField::WorkId => match order.direction {
                Direction::Asc => query = query.order(dsl::work_id.asc()),
                Direction::Desc => query = query.order(dsl::work_id.desc()),
            },
            ContributionField::ContributorId => match order.direction {
                Direction::Asc => query = query.order(dsl::contributor_id.asc()),
                Direction::Desc => query = query.order(dsl::contributor_id.desc()),
            },
            ContributionField::ContributionType => match order.direction {
                Direction::Asc => query = query.order(dsl::contribution_type.asc()),
                Direction::Desc => query = query.order(dsl::contribution_type.desc()),
            },
            ContributionField::MainContribution => match order.direction {
                Direction::Asc => query = query.order(dsl::main_contribution.asc()),
                Direction::Desc => query = query.order(dsl::main_contribution.desc()),
            },
            ContributionField::Biography => match order.direction {
                Direction::Asc => query = query.order(dsl::biography.asc()),
                Direction::Desc => query = query.order(dsl::biography.desc()),
            },
            ContributionField::Institution => match order.direction {
                Direction::Asc => query = query.order(dsl::institution.asc()),
                Direction::Desc => query = query.order(dsl::institution.desc()),
            },
            ContributionField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::created_at.asc()),
                Direction::Desc => query = query.order(dsl::created_at.desc()),
            },
            ContributionField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::updated_at.asc()),
                Direction::Desc => query = query.order(dsl::updated_at.desc()),
            },
            ContributionField::FirstName => match order.direction {
                Direction::Asc => query = query.order(dsl::first_name.asc()),
                Direction::Desc => query = query.order(dsl::first_name.desc()),
            },
            ContributionField::LastName => match order.direction {
                Direction::Asc => query = query.order(dsl::last_name.asc()),
                Direction::Desc => query = query.order(dsl::last_name.desc()),
            },
            ContributionField::FullName => match order.direction {
                Direction::Asc => query = query.order(dsl::full_name.asc()),
                Direction::Desc => query = query.order(dsl::full_name.desc()),
            },
            ContributionField::ContributionOrdinal => match order.direction {
                Direction::Asc => query = query.order(dsl::contribution_ordinal.asc()),
                Direction::Desc => query = query.order(dsl::contribution_ordinal.desc()),
            },
        }
        // This loop must appear before any other filter statements, as it takes advantage of
        // the behaviour of `or_filter` being equal to `filter` when no other filters are present yet.
        // Result needs to be `WHERE (x = $1 [OR x = $2...]) AND ([...])` - note bracketing.
        for pub_id in publishers {
            query = query.or_filter(crate::schema::imprint::publisher_id.eq(pub_id));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(dsl::work_id.eq(pid));
        }
        if let Some(pid) = parent_id_2 {
            query = query.filter(dsl::contributor_id.eq(pid));
        }
        if !contribution_types.is_empty() {
            query = query.filter(dsl::contribution_type.eq(any(contribution_types)));
        }
        match query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Contribution>(&connection)
        {
            Ok(t) => Ok(t),
            Err(e) => Err(ThothError::from(e)),
        }
    }

    fn count(
        db: &crate::db::PgPool,
        _: Option<String>,
        _: Vec<Uuid>,
        contribution_types: Vec<Self::FilterParameter1>,
        _: Option<Self::FilterParameter2>,
    ) -> ThothResult<i32> {
        use crate::schema::contribution::dsl;
        let connection = db.get().unwrap();
        let mut query = dsl::contribution.into_boxed();
        if !contribution_types.is_empty() {
            query = query.filter(dsl::contribution_type.eq(any(contribution_types)));
        }

        // `SELECT COUNT(*)` in postgres returns a BIGINT, which diesel parses as i64. Juniper does
        // not implement i64 yet, only i32. The only sensible way, albeit shameful, to solve this
        // is converting i64 to string and then parsing it as i32. This should work until we reach
        // 2147483647 records - if you are fixing this bug, congratulations on book number 2147483647!
        match query.count().get_result::<i64>(&connection) {
            Ok(t) => Ok(t.to_string().parse::<i32>().unwrap()),
            Err(e) => Err(ThothError::from(e)),
        }
    }

    fn publisher_id(&self, db: &crate::db::PgPool) -> ThothResult<Uuid> {
        crate::model::work::Work::from_id(db, &self.work_id)?.publisher_id(db)
    }

    crud_methods!(contribution::table, contribution::dsl::contribution);
}

impl HistoryEntry for Contribution {
    type NewHistoryEntity = NewContributionHistory;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            contribution_id: self.contribution_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewContributionHistory {
    type MainEntity = ContributionHistory;

    db_insert!(contribution_history::table);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contribution_pk() {
        let contribution: Contribution = Default::default();
        assert_eq!(contribution.pk(), contribution.contribution_id);
    }

    #[test]
    fn test_new_contribution_history_from_contribution() {
        let contribution: Contribution = Default::default();
        let account_id: Uuid = Default::default();
        let new_contribution_history = contribution.new_history_entry(&account_id);
        assert_eq!(
            new_contribution_history.contribution_id,
            contribution.contribution_id
        );
        assert_eq!(new_contribution_history.account_id, account_id);
        assert_eq!(
            new_contribution_history.data,
            serde_json::Value::String(serde_json::to_string(&contribution).unwrap())
        );
    }
}
