use super::model::{Imprint, ImprintHistory, NewImprint, NewImprintHistory, PatchImprint};
pub use crate::model::Crud;
use crate::model::{DbInsert, HistoryEntry};
use crate::schema::{imprint, imprint_history};
use crate::{crud_methods, db_insert};

impl Crud for Imprint {
    type NewEntity = NewImprint;
    type PatchEntity = PatchImprint;

    fn pk(&self) -> uuid::Uuid {
        self.imprint_id
    }

    crud_methods!(imprint::table, imprint::dsl::imprint, ImprintHistory);
}

impl HistoryEntry for ImprintHistory {
    type MainEntity = Imprint;
    type NewEntity = NewImprintHistory;

    fn new(entity: &Self::MainEntity, account_id: &uuid::Uuid) -> Self::NewEntity {
        Self::NewEntity {
            imprint_id: entity.imprint_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&entity).unwrap()),
        }
    }
}

impl DbInsert for NewImprintHistory {
    type MainEntity = ImprintHistory;

    db_insert!(imprint_history::table);
}
