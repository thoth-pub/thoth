use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::errors::ThothError;
use crate::language::model::NewLanguageHistory;
use crate::language::model::Language;
use crate::language::model::LanguageHistory;
use crate::schema::language_history;

impl NewLanguageHistory {
    pub fn new(language: Language, account_id: Uuid) -> Self {
        Self {
            language_id: language.language_id,
            account_id,
            data: serde_json::Value::String(serde_json::to_string(&language).unwrap()),
        }
    }

    pub fn insert(&self, connection: &PgConnection) -> Result<LanguageHistory, ThothError> {
        match diesel::insert_into(language_history::table)
            .values(self)
            .get_result(connection)
        {
            Ok(history) => Ok(history),
            Err(e) => Err(ThothError::from(e)),
        }
    }
}
