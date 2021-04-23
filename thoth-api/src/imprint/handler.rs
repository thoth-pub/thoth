use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::errors::{ThothError, ThothResult};
use crate::imprint::model::Imprint;
use crate::imprint::model::ImprintHistory;
use crate::imprint::model::NewImprintHistory;
use crate::schema::imprint_history;

impl NewImprintHistory {
    pub fn new(imprint: &Imprint, account_id: Uuid) -> Self {
        Self {
            imprint_id: imprint.imprint_id,
            account_id,
            data: serde_json::Value::String(serde_json::to_string(&imprint).unwrap()),
        }
    }

    pub fn insert(&self, connection: &PgConnection) -> ThothResult<ImprintHistory> {
        match diesel::insert_into(imprint_history::table)
            .values(self)
            .get_result(connection)
        {
            Ok(history) => Ok(history),
            Err(e) => Err(ThothError::from(e)),
        }
    }
}
