use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use thoth_api::publication::model::PublicationOrderBy;
use thoth_api::publication::model::PublicationType;

use super::super::work::Work;

pub const PUBLICATIONS_QUERY: &str = "
    query PublicationsQuery($limit: Int, $offset: Int, $filter: String, $publishers: [Uuid!], $order: PublicationOrderBy) {
        publications(limit: $limit, offset: $offset, filter: $filter, publishers: $publishers, order: $order) {
            publicationId
            publicationType
            workId
            isbn
            publicationUrl
            updatedAt
            work {
                workId
                workType
                workStatus
                fullTitle
                doi
                title
                edition
                copyrightHolder
                updatedAt
                imprint {
                    imprintId
                    imprintName
                    updatedAt
                    publisher {
                        publisherId
                        publisherName
                        publisherShortname
                        publisherUrl
                        updatedAt
                    }
                }
            }
        }
        publicationCount(filter: $filter, publishers: $publishers)
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
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub filter: Option<String>,
    pub order: Option<PublicationOrderBy>,
    pub publishers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DetailedPublication {
    pub publication_id: String,
    pub publication_type: PublicationType,
    pub work_id: String,
    pub isbn: Option<String>,
    pub publication_url: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub work: Work,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PublicationsResponseData {
    pub publications: Vec<DetailedPublication>,
    pub publication_count: i32,
}

impl Default for DetailedPublication {
    fn default() -> DetailedPublication {
        DetailedPublication {
            publication_id: "".to_string(),
            publication_type: Default::default(),
            work_id: "".to_string(),
            isbn: None,
            publication_url: None,
            updated_at: chrono::TimeZone::timestamp(&Utc, 0, 0),
            work: Default::default(),
        }
    }
}
