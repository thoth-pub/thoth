use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::errors::{ThothError, ThothResult};
use crate::issue::model::Issue;
use crate::issue::model::IssueHistory;
use crate::issue::model::NewIssueHistory;
use crate::schema::issue_history;

impl NewIssueHistory {
    pub fn new(issue: Issue, account_id: Uuid) -> Self {
        Self {
            series_id: issue.series_id,
            work_id: issue.work_id,
            account_id,
            data: serde_json::Value::String(serde_json::to_string(&issue).unwrap()),
        }
    }

    pub fn insert(&self, connection: &PgConnection) -> ThothResult<IssueHistory> {
        match diesel::insert_into(issue_history::table)
            .values(self)
            .get_result(connection)
        {
            Ok(history) => Ok(history),
            Err(e) => Err(ThothError::from(e)),
        }
    }
}
