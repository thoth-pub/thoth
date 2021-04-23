use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::errors::{ThothError, ThothResult};
use crate::funder::model::Funder;
use crate::funder::model::FunderHistory;
use crate::funder::model::NewFunderHistory;
use crate::schema::funder_history;

impl NewFunderHistory {
    pub fn new(funder: Funder, account_id: Uuid) -> Self {
        Self {
            funder_id: funder.funder_id,
            account_id,
            data: serde_json::Value::String(serde_json::to_string(&funder).unwrap()),
        }
    }

    pub fn insert(&self, connection: &PgConnection) -> ThothResult<FunderHistory> {
        match diesel::insert_into(funder_history::table)
            .values(self)
            .get_result(connection)
        {
            Ok(history) => Ok(history),
            Err(e) => Err(ThothError::from(e)),
        }
    }
}
