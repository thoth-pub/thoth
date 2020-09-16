use serde::Deserialize;
use serde::Serialize;

use crate::api::models::Work;
use crate::api::models::License;
use crate::api::models::Imprint;
use crate::api::models::Publisher;

// $query here is filled in when instantiated. TODO make use of variables and predefine the query
query_builder!{
    WorkRequest,
    WorkRequestBody,
    "",
    WorkResponseBody,
    WorkResponseData,
    FetchWork,
    FetchActionWork
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkResponseData {
    pub work: Work,
}

impl Default for WorkResponseBody {
    fn default() -> WorkResponseBody {
        WorkResponseBody {
            data: WorkResponseData {
                work: Work {
                    work_id: "".to_string(),
                    full_title: "".to_string(),
                    title: "".to_string(),
                    subtitle: None,
                    doi: "".to_string(),
                    cover_url: "".to_string(),
                    license: License::By,
                    place: "".to_string(),
                    publication_date: None,
                    contributions: None,
                    imprint: Imprint {
                        publisher: Publisher {
                            publisher_id: "".to_string(),
                            publisher_name: "".to_string(),
                            publisher_shortname: None,
                            publisher_url: None,
                        }
                    },
                },
            }
        }
    }
}
