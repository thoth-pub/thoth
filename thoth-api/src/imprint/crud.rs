use super::model::{Imprint, NewImprint, NewImprintHistory, PatchImprint};
use crate::crud_methods;
pub use crate::model::Crud;
use crate::schema::imprint;

impl Crud for Imprint {
    type NewEntity = NewImprint;
    type PatchEntity = PatchImprint;

    fn pk(&self) -> uuid::Uuid {
        self.imprint_id
    }

    crud_methods!(imprint::table, imprint::dsl::imprint, NewImprintHistory);
}
