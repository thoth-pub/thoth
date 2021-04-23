use super::model::{NewImprint, PatchImprint, Imprint, NewImprintHistory};
use crate::schema::imprint;
use crate::crud_methods;
pub use crate::model::Crud;

impl Crud for Imprint {
    type NewEntity = NewImprint;
    type PatchEntity = PatchImprint;

    fn pk(&self) -> uuid::Uuid {
        self.imprint_id
    }

    crud_methods!(imprint::table, imprint::dsl::imprint, NewImprintHistory);
}