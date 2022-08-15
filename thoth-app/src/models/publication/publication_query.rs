use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::publication::PublicationWithRelations;
use uuid::Uuid;

pub const PUBLICATION_QUERY: &str = "
    query PublicationQuery($publicationId: Uuid!) {
        publication(publicationId: $publicationId) {
            publicationId
            publicationType
            workId
            isbn
            updatedAt
            weightG: weight(units: G)
            weightOz: weight(units: OZ)
            widthMm: width(units: MM)
            widthIn: width(units: IN)
            heightMm: height(units: MM)
            heightIn: height(units: IN)
            depthMm: depth(units: MM)
            depthIn: depth(units: IN)
            prices {
                priceId
                publicationId
                currencyCode
                unitPrice
                createdAt
                updatedAt
            }
            locations {
                locationId
                publicationId
                landingPage
                fullTextUrl
                locationPlatform
                canonical
                createdAt
                updatedAt
            }
            work {
                workId
                workType
                workStatus
                fullTitle
                title
                edition
                updatedAt
                imprint {
                    imprintId
                    imprintName
                    updatedAt
                    publisher {
                        publisherId
                        publisherName
                        createdAt
                        updatedAt
                    }
                }
            }
        }
    }
";

graphql_query_builder! {
    PublicationRequest,
    PublicationRequestBody,
    Variables,
    PUBLICATION_QUERY,
    PublicationResponseBody,
    PublicationResponseData,
    FetchPublication,
    FetchActionPublication
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub publication_id: Option<Uuid>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PublicationResponseData {
    pub publication: Option<PublicationWithRelations>,
}
