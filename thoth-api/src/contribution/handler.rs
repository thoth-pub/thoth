use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::errors::ThothError;
use crate::contribution::model::NewContributionHistory;
use crate::contribution::model::Contribution;
use crate::contribution::model::ContributionHistory;
use crate::schema::contribution_history;

impl NewContributionHistory {
    pub fn new(contribution: Contribution, account_id: Uuid) -> Self {
        Self {
            work_id: contribution.work_id,
            contributor_id: contribution.contributor_id,
            contribution_type: contribution.contribution_type,
            account_id,
            data: serde_json::Value::String(serde_json::to_string(&contribution).unwrap()),
        }
    }

    pub fn insert(&self, connection: &PgConnection) -> Result<ContributionHistory, ThothError> {
        match diesel::insert_into(contribution_history::table)
            .values(self)
            .get_result(connection)
        {
            Ok(history) => Ok(history),
            Err(e) => Err(ThothError::from(e)),
        }
    }
}
