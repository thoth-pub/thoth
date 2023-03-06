use super::{Issue, IssueField, IssueHistory, NewIssue, NewIssueHistory, PatchIssue};
use crate::graphql::model::IssueOrderBy;
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{issue, issue_history};
use crate::{crud_methods, db_insert};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

impl Crud for Issue {
    type NewEntity = NewIssue;
    type PatchEntity = PatchIssue;
    type OrderByEntity = IssueOrderBy;
    type FilterParameter1 = ();
    type FilterParameter2 = ();

    fn pk(&self) -> Uuid {
        self.issue_id
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
    ) -> ThothResult<Vec<Issue>> {
        use crate::schema::issue::dsl::*;
        let mut connection = db.get().unwrap();
        let mut query = issue
            .inner_join(crate::schema::series::table.inner_join(crate::schema::imprint::table))
            .select(crate::schema::issue::all_columns)
            .into_boxed();

        query = match order.field {
            IssueField::IssueId => match order.direction {
                Direction::Asc => query.order(issue_id.asc()),
                Direction::Desc => query.order(issue_id.desc()),
            },
            IssueField::SeriesId => match order.direction {
                Direction::Asc => query.order(series_id.asc()),
                Direction::Desc => query.order(series_id.desc()),
            },
            IssueField::WorkId => match order.direction {
                Direction::Asc => query.order(work_id.asc()),
                Direction::Desc => query.order(work_id.desc()),
            },
            IssueField::IssueOrdinal => match order.direction {
                Direction::Asc => query.order(issue_ordinal.asc()),
                Direction::Desc => query.order(issue_ordinal.desc()),
            },
            IssueField::CreatedAt => match order.direction {
                Direction::Asc => query.order(created_at.asc()),
                Direction::Desc => query.order(created_at.desc()),
            },
            IssueField::UpdatedAt => match order.direction {
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
            query = query.filter(series_id.eq(pid));
        }
        match query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Issue>(&mut connection)
        {
            Ok(t) => Ok(t),
            Err(e) => Err(ThothError::from(e)),
        }
    }

    fn count(
        db: &crate::db::PgPool,
        _: Option<String>,
        _: Vec<Uuid>,
        _: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
    ) -> ThothResult<i32> {
        use crate::schema::issue::dsl::*;
        let mut connection = db.get().unwrap();

        // `SELECT COUNT(*)` in postgres returns a BIGINT, which diesel parses as i64. Juniper does
        // not implement i64 yet, only i32. The only sensible way, albeit shameful, to solve this
        // is converting i64 to string and then parsing it as i32. This should work until we reach
        // 2147483647 records - if you are fixing this bug, congratulations on book number 2147483647!
        match issue.count().get_result::<i64>(&mut connection) {
            Ok(t) => Ok(t.to_string().parse::<i32>().unwrap()),
            Err(e) => Err(ThothError::from(e)),
        }
    }

    fn publisher_id(&self, db: &crate::db::PgPool) -> ThothResult<Uuid> {
        crate::model::work::Work::from_id(db, &self.work_id)?.publisher_id(db)
    }

    crud_methods!(issue::table, issue::dsl::issue);
}

impl HistoryEntry for Issue {
    type NewHistoryEntity = NewIssueHistory;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            issue_id: self.issue_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewIssueHistory {
    type MainEntity = IssueHistory;

    db_insert!(issue_history::table);
}

impl NewIssue {
    pub fn imprints_match(&self, db: &crate::db::PgPool) -> ThothResult<()> {
        issue_imprints_match(self.work_id, self.series_id, db)
    }
}

impl PatchIssue {
    pub fn imprints_match(&self, db: &crate::db::PgPool) -> ThothResult<()> {
        issue_imprints_match(self.work_id, self.series_id, db)
    }
}

fn issue_imprints_match(work_id: Uuid, series_id: Uuid, db: &crate::db::PgPool) -> ThothResult<()> {
    use diesel::prelude::*;

    let mut connection = db.get().unwrap();
    let series_imprint = crate::schema::series::table
        .select(crate::schema::series::imprint_id)
        .filter(crate::schema::series::series_id.eq(series_id))
        .first::<Uuid>(&mut connection)
        .expect("Error loading series for issue");
    let work_imprint = crate::schema::work::table
        .select(crate::schema::work::imprint_id)
        .filter(crate::schema::work::work_id.eq(work_id))
        .first::<Uuid>(&mut connection)
        .expect("Error loading work for issue");
    if work_imprint == series_imprint {
        Ok(())
    } else {
        Err(ThothError::IssueImprintsError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_pk() {
        let issue: Issue = Default::default();
        assert_eq!(issue.pk(), issue.issue_id);
    }

    #[test]
    fn test_new_issue_history_from_issue() {
        let issue: Issue = Default::default();
        let account_id: Uuid = Default::default();
        let new_issue_history = issue.new_history_entry(&account_id);
        assert_eq!(new_issue_history.issue_id, issue.issue_id);
        assert_eq!(new_issue_history.account_id, account_id);
        assert_eq!(
            new_issue_history.data,
            serde_json::Value::String(serde_json::to_string(&issue).unwrap())
        );
    }
}
