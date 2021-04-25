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

#[cfg(test)]
mod tests {
    use super::*;

    impl Default for Imprint {
        fn default() -> Self {
            Imprint {
                imprint_id: Default::default(),
                publisher_id: Default::default(),
                imprint_name: Default::default(),
                imprint_url: Default::default(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }
        }
    }

    #[test]
    fn test_imprint_pk() {
        let imprint: Imprint = Default::default();
        assert_eq!(imprint.pk(), imprint.imprint_id);
    }

    #[test]
    fn test_new_imprint_history_from_imprint() {
        let imprint: Imprint = Default::default();
        let account_id: uuid::Uuid = Default::default();
        let new_imprint_history = ImprintHistory::new(&imprint, &account_id);
        assert_eq!(new_imprint_history.imprint_id, imprint.imprint_id);
        assert_eq!(new_imprint_history.account_id, account_id);
        assert_eq!(new_imprint_history.data, serde_json::Value::String(serde_json::to_string(&imprint).unwrap()));
    }
}
