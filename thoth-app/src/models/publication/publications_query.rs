use serde::Deserialize;
use serde::Serialize;
use thoth_api::publication::model::PublicationOrderBy;
use thoth_api::publication::model::PublicationWithRelations;

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
                        createdAt
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
pub struct PublicationsResponseData {
    pub publications: Vec<PublicationWithRelations>,
    pub publication_count: i32,
}
