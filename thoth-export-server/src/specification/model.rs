use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Apiv2Schema)]
pub(crate) struct Specification<'a> {
    pub(crate) id: SpecificationId,
    pub(crate) name: &'a str,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Apiv2Schema)]
pub enum SpecificationId {
    #[serde(rename = "onix_3.0::project_muse")]
    Onix3ProjectMuse,
    #[serde(rename = "csv::thoth")]
    CsvThoth,
}