use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Apiv2Schema)]
pub(crate) struct Specification<'a> {
    pub(crate) id: &'a str,
    pub(crate) name: &'a str,
}
