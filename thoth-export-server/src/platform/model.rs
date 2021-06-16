use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Platform<'a> {
    pub(crate) id: &'a str,
    pub(crate) name: &'a str,
    pub(crate) accepts: Vec<&'a str>,
}
