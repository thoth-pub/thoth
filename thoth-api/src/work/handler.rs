use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::errors::ThothError;
use crate::work::model::NewWorkHistory;
use crate::work::model::Work;
use crate::work::model::WorkHistory;
use crate::schema::work_history;

impl NewWorkHistory {
    pub fn new(work: Work, account_id: Uuid) -> Self {
        Self {
            work_id: work.work_id,
            account_id,
            data: serde_json::Value::String(serde_json::to_string(&work).unwrap()),
        }
    }

    pub fn insert(&self, connection: &PgConnection) -> Result<WorkHistory, ThothError> {
        match diesel::insert_into(work_history::table)
            .values(self)
            .get_result(connection)
        {
            Ok(history) => Ok(history),
            Err(e) => Err(ThothError::from(e)),
        }
    }
}
