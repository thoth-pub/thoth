use super::model::{
    Contributor, ContributorField, ContributorHistory, ContributorOrderBy, NewContributor,
    NewContributorHistory, PatchContributor,
};
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{contributor, contributor_history};
use crate::{crud_methods, db_insert};
use diesel::{ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

impl Crud for Contributor {
    type NewEntity = NewContributor;
    type PatchEntity = PatchContributor;
    type OrderByEntity = ContributorOrderBy;
    type FilterParameter1 = ();
    type FilterParameter2 = ();

    fn pk(&self) -> Uuid {
        self.contributor_id
    }

    fn all(
        db: &crate::db::PgPool,
        limit: i32,
        offset: i32,
        filter: Option<String>,
        order: Self::OrderByEntity,
        _: Vec<Uuid>,
        _: Option<Uuid>,
        _: Option<Uuid>,
        _: Option<Self::FilterParameter1>,
        _: Option<Self::FilterParameter2>,
    ) -> ThothResult<Vec<Contributor>> {
        use crate::schema::contributor::dsl::*;
        let connection = db.get().unwrap();
        let mut query = contributor.into_boxed();

        match order.field {
            ContributorField::ContributorId => match order.direction {
                Direction::Asc => query = query.order(contributor_id.asc()),
                Direction::Desc => query = query.order(contributor_id.desc()),
            },
            ContributorField::FirstName => match order.direction {
                Direction::Asc => query = query.order(first_name.asc()),
                Direction::Desc => query = query.order(first_name.desc()),
            },
            ContributorField::LastName => match order.direction {
                Direction::Asc => query = query.order(last_name.asc()),
                Direction::Desc => query = query.order(last_name.desc()),
            },
            ContributorField::FullName => match order.direction {
                Direction::Asc => query = query.order(full_name.asc()),
                Direction::Desc => query = query.order(full_name.desc()),
            },
            ContributorField::Orcid => match order.direction {
                Direction::Asc => query = query.order(orcid.asc()),
                Direction::Desc => query = query.order(orcid.desc()),
            },
            ContributorField::Website => match order.direction {
                Direction::Asc => query = query.order(website.asc()),
                Direction::Desc => query = query.order(website.desc()),
            },
            ContributorField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(created_at.asc()),
                Direction::Desc => query = query.order(created_at.desc()),
            },
            ContributorField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(updated_at.asc()),
                Direction::Desc => query = query.order(updated_at.desc()),
            },
        }
        if let Some(filter) = filter {
            for substring in filter.split_whitespace() {
                query = query
                    .or_filter(full_name.ilike(format!("%{}%", substring)))
                    .or_filter(last_name.ilike(format!("%{}%", substring)))
                    .or_filter(orcid.ilike(format!("%{}%", substring)));
            }
        }
        match query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Contributor>(&connection)
        {
            Ok(t) => Ok(t),
            Err(e) => Err(ThothError::from(e)),
        }
    }

    fn count(
        db: &crate::db::PgPool,
        filter: Option<String>,
        _: Vec<Uuid>,
        _: Option<Self::FilterParameter1>,
        _: Option<Self::FilterParameter2>,
    ) -> ThothResult<i32> {
        use crate::schema::contributor::dsl::*;
        let connection = db.get().unwrap();
        let mut query = contributor.into_boxed();
        if let Some(filter) = filter {
            for substring in filter.split_whitespace() {
                query = query
                    .or_filter(full_name.ilike(format!("%{}%", substring)))
                    .or_filter(last_name.ilike(format!("%{}%", substring)))
                    .or_filter(orcid.ilike(format!("%{}%", substring)));
            }
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

    fn publisher_id(&self, _db: &crate::db::PgPool) -> ThothResult<Uuid> {
        Err(ThothError::InternalError(
            "Method publisher_id() is not supported for Contributor objects".to_string(),
        ))
    }

    crud_methods!(contributor::table, contributor::dsl::contributor);
}

impl HistoryEntry for Contributor {
    type NewHistoryEntity = NewContributorHistory;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            contributor_id: self.contributor_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewContributorHistory {
    type MainEntity = ContributorHistory;

    db_insert!(contributor_history::table);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contributor_pk() {
        let contributor: Contributor = Default::default();
        assert_eq!(contributor.pk(), contributor.contributor_id);
    }

    #[test]
    fn test_new_contributor_history_from_contributor() {
        let contributor: Contributor = Default::default();
        let account_id: Uuid = Default::default();
        let new_contributor_history = contributor.new_history_entry(&account_id);
        assert_eq!(
            new_contributor_history.contributor_id,
            contributor.contributor_id
        );
        assert_eq!(new_contributor_history.account_id, account_id);
        assert_eq!(
            new_contributor_history.data,
            serde_json::Value::String(serde_json::to_string(&contributor).unwrap())
        );
    }
}
