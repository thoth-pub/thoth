use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::errors::ThothError;
use crate::publisher::model::NewPublisherHistory;
use crate::publisher::model::Publisher;
use crate::publisher::model::PublisherHistory;
use crate::schema::publisher_history;

impl NewPublisherHistory {
    pub fn new(publisher: Publisher, account_id: Uuid) -> Self {
        Self {
            publisher_id: publisher.publisher_id,
            account_id,
            data: serde_json::Value::String(serde_json::to_string(&publisher).unwrap()),
        }
    }

    pub fn insert(&self, connection: &PgConnection) -> Result<PublisherHistory, ThothError> {
        match diesel::insert_into(publisher_history::table)
            .values(self)
            .get_result(connection)
        {
            Ok(history) => Ok(history),
            Err(e) => Err(ThothError::from(e)),
        }
    }
}
