use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::errors::ThothError;
use crate::funding::model::Funding;
use crate::funding::model::FundingHistory;
use crate::funding::model::NewFundingHistory;
use crate::schema::funding_history;

impl NewFundingHistory {
    pub fn new(funding: Funding, account_id: Uuid) -> Self {
        Self {
            funding_id: funding.funding_id,
            account_id,
            data: serde_json::Value::String(serde_json::to_string(&funding).unwrap()),
        }
    }

    pub fn insert(&self, connection: &PgConnection) -> Result<FundingHistory, ThothError> {
        match diesel::insert_into(funding_history::table)
            .values(self)
            .get_result(connection)
        {
            Ok(history) => Ok(history),
            Err(e) => Err(ThothError::from(e)),
        }
    }
}
