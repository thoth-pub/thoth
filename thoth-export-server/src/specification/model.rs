use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
pub struct Specification<'a> {
    pub(crate) id: &'a str,
    pub(crate) name: &'a str,
    pub(crate) format: &'a str,
    pub(crate) accepted_by: Vec<&'a str>,
}

impl std::fmt::Display for Specification<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}
