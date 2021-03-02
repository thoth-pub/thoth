use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::errors::ThothError;
use crate::subject::model::NewSubjectHistory;
use crate::subject::model::Subject;
use crate::subject::model::SubjectHistory;
use crate::schema::subject_history;

impl NewSubjectHistory {
    pub fn new(subject: Subject, account_id: Uuid) -> Self {
        Self {
            subject_id: subject.subject_id,
            account_id,
            data: serde_json::Value::String(serde_json::to_string(&subject).unwrap()),
        }
    }

    pub fn insert(&self, connection: &PgConnection) -> Result<SubjectHistory, ThothError> {
        match diesel::insert_into(subject_history::table)
            .values(self)
            .get_result(connection)
        {
            Ok(history) => Ok(history),
            Err(e) => Err(ThothError::from(e)),
        }
    }
}
