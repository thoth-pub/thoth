use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::errors::{ThothError, ThothResult};
use crate::publication::model::NewPublicationHistory;
use crate::publication::model::Publication;
use crate::publication::model::PublicationHistory;
use crate::schema::publication_history;

impl NewPublicationHistory {
    pub fn new(publication: Publication, account_id: Uuid) -> Self {
        Self {
            publication_id: publication.publication_id,
            account_id,
            data: serde_json::Value::String(serde_json::to_string(&publication).unwrap()),
        }
    }

    pub fn insert(&self, connection: &PgConnection) -> ThothResult<PublicationHistory> {
        match diesel::insert_into(publication_history::table)
            .values(self)
            .get_result(connection)
        {
            Ok(history) => Ok(history),
            Err(e) => Err(ThothError::from(e)),
        }
    }
}
