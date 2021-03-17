use serde::Deserialize;
use serde::Serialize;
use thoth_api::contributor::model::ContributorOrderBy;
use thoth_api::publication::model::PublicationType;

use super::super::work::Work;

pub const PUBLICATIONS_QUERY: &str = "
    query PublicationsQuery($limit: Int, $offset: Int, $filter: String, $publishers: [Uuid!]) {
        publications(limit: $limit, offset: $offset, filter: $filter, publishers: $publishers) {
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
    pub order: Option<ContributorOrderBy>,
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
    pub work: Work,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PublicationsResponseData {
    pub publications: Vec<DetailedPublication>,
    pub publication_count: i32,
}
