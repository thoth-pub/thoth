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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DetailedPublication {
    pub publication_id: String,
    pub publication_type: PublicationType,
    pub work_id: String,
    pub isbn: Option<String>,
    pub publication_url: Option<String>,
    pub updated_at: serde_json::Value,
    pub work: Work,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PublicationsResponseData {
    pub publications: Vec<DetailedPublication>,
    pub publication_count: i32,
}
