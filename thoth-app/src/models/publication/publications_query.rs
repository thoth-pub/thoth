use serde::Deserialize;
use serde::Serialize;
use thoth_api::publication::model::PublicationType;

use super::super::work::Work;

pub const PUBLICATIONS_QUERY: &str = "
    query PublicationsQuery($filter: String) {
        publications(limit: 9999, filter: $filter) {
            publicationId
            publicationType
            workId
            isbn
            publicationUrl
            work {
                workId
                workType
                workStatus
                fullTitle
                doi
                title
                edition
                copyrightHolder
                imprint {
                    imprintId
                    imprintName
                    publisher {
                        publisherId
                        publisherName
                        publisherShortname
                        publisherUrl
                    }
                }
            }
        }
    }
";

graphql_query_builder! {
    PublicationsRequest,
    PublicationsRequestBody,
    Variables,
    PUBLICATIONS_QUERY,
    PublicationsResponseBody,
    PublicationsResponseData,
    FetchPublications,
    FetchActionPublications
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub filter: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DetailedPublication {
    pub publication_id: String,
    pub publication_type: PublicationType,
    pub work_id: String,
    pub isbn: Option<String>,
    pub publication_url: Option<String>,
    pub work: Work,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PublicationsResponseData {
    pub publications: Vec<DetailedPublication>,
}
