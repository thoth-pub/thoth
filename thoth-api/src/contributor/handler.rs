use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::errors::ThothError;
use crate::contributor::model::NewContributorHistory;
use crate::contributor::model::Contributor;
use crate::contributor::model::ContributorHistory;
use crate::schema::contributor_history;

impl NewContributorHistory {
    pub fn new(contributor: Contributor, account_id: Uuid) -> Self {
        Self {
            contributor_id: contributor.contributor_id,
            account_id,
            data: serde_json::Value::String(serde_json::to_string(&contributor).unwrap()),
        }
    }

    pub fn insert(&self, connection: &PgConnection) -> Result<ContributorHistory, ThothError> {
        match diesel::insert_into(contributor_history::table)
            .values(self)
            .get_result(connection)
        {
            Ok(history) => Ok(history),
            Err(e) => Err(ThothError::from(e)),
        }
    }
}
