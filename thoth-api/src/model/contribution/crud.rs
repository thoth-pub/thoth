use super::{
    Contribution, ContributionField, ContributionHistory, ContributionType, NewContribution,
    NewContributionHistory, PatchContribution,
};
use crate::diesel::JoinOnDsl;
use crate::graphql::model::ContributionOrderBy;
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry, Reorder};
use crate::schema::{contribution, contribution_history};
use diesel::{BoolExpressionMethods, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
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
        let mut query = diesel::query_dsl::methods::DistinctOnDsl::distinct_on(
            contribution
                .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
                .left_join(
                    crate::schema::biography::table
                        .on(crate::schema::biography::contribution_id.eq(contribution_id)),
                )
                .select(crate::schema::contribution::all_columns),
            contribution_id,
        )
        .into_boxed();

        query = match order.field {
            ContributionField::ContributionId => match order.direction {
                Direction::Asc => query.order((contribution_id, contribution_id.asc())),
                Direction::Desc => query.order((contribution_id, contribution_id.desc())),
            },
            ContributionField::WorkId => match order.direction {
                Direction::Asc => query.order((contribution_id, work_id.asc())),
                Direction::Desc => query.order((contribution_id, work_id.desc())),
            },
            ContributionField::ContributorId => match order.direction {
                Direction::Asc => query.order((contribution_id, contributor_id.asc())),
                Direction::Desc => query.order((contribution_id, contributor_id.desc())),
            },
            ContributionField::ContributionType => match order.direction {
                Direction::Asc => query.order((contribution_id, contribution_type.asc())),
                Direction::Desc => query.order((contribution_id, contribution_type.desc())),
            },
            ContributionField::MainContribution => match order.direction {
                Direction::Asc => query.order((contribution_id, main_contribution.asc())),
                Direction::Desc => query.order((contribution_id, main_contribution.desc())),
            },
            ContributionField::Biography => match order.direction {
                Direction::Asc => {
                    query.order((contribution_id, crate::schema::biography::content.asc()))
                }
                Direction::Desc => {
                    query.order((contribution_id, crate::schema::biography::content.desc()))
                }
            },
            ContributionField::CreatedAt => match order.direction {
                Direction::Asc => query.order((contribution_id, created_at.asc())),
                Direction::Desc => query.order((contribution_id, created_at.desc())),
            },
            ContributionField::UpdatedAt => match order.direction {
                Direction::Asc => query.order((contribution_id, updated_at.asc())),
                Direction::Desc => query.order((contribution_id, updated_at.desc())),
            },
            ContributionField::FirstName => match order.direction {
                Direction::Asc => query.order((contribution_id, first_name.asc())),
                Direction::Desc => query.order((contribution_id, first_name.desc())),
            },
            ContributionField::LastName => match order.direction {
                Direction::Asc => query.order((contribution_id, last_name.asc())),
                Direction::Desc => query.order((contribution_id, last_name.desc())),
            },
            ContributionField::FullName => match order.direction {
                Direction::Asc => query.order((contribution_id, full_name.asc())),
                Direction::Desc => query.order((contribution_id, full_name.desc())),
            },
            ContributionField::ContributionOrdinal => match order.direction {
                Direction::Asc => query.order((contribution_id, contribution_ordinal.asc())),
                Direction::Desc => query.order((contribution_id, contribution_ordinal.desc())),
            },
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
