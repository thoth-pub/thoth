use serde::Deserialize;
use serde::Serialize;

use crate::api::models::Work;

const WORKS_QUERY: &str = "
    {
        works(limit: 9999) {
            workId
            fullTitle
            title
            doi
            coverUrl
            license
            publicationDate
            place
            contributions {
                mainContribution
                contributor {
                    fullName
                }
            }
            imprint {
                publisher {
                    publisherId
                    publisherName
                    publisherShortname
                    publisherUrl
                }
            }
        }
    }
";

query_builder!{
    WorksRequest,
    WorksRequestBody,
    WORKS_QUERY,
    WorksResponseBody,
    WorksResponseData,
    FetchWorks,
    FetchActionWorks
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorksResponseData {
    pub works: Vec<Work>,
}

impl Default for WorksResponseBody {
    fn default() -> WorksResponseBody {
        WorksResponseBody { data: WorksResponseData { works: vec![] } }
    }
}
