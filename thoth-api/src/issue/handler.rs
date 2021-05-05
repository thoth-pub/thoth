use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::errors::ThothError;
use crate::issue::model::Issue;
use crate::issue::model::IssueHistory;
use crate::issue::model::NewIssueHistory;
use crate::schema::issue_history;

impl NewIssueHistory {
    pub fn new(issue: Issue, account_id: Uuid) -> Self {
        Self {
            issue_id: issue.issue_id,
            account_id,
            data: serde_json::Value::String(serde_json::to_string(&issue).unwrap()),
        }
    }

    pub fn insert(&self, connection: &PgConnection) -> Result<IssueHistory, ThothError> {
        match diesel::insert_into(issue_history::table)
            .values(self)
            .get_result(connection)
        {
            Ok(history) => Ok(history),
            Err(e) => Err(ThothError::from(e)),
        }
    }
}
