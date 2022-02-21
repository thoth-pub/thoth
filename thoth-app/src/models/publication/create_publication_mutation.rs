use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::publication::Publication;
use thoth_api::model::publication::PublicationType;
use thoth_api::model::Isbn;
use uuid::Uuid;

const CREATE_PUBLICATION_MUTATION: &str = "
    mutation CreatePublication(
        $publicationType: PublicationType!,
        $workId: Uuid!,
        $isbn: Isbn,
        $weightG: Float,
        $weightOz: Float,
    ) {
        createPublication(
            data: {
            publicationType: $publicationType
            workId: $workId
            isbn: $isbn
            weightG: $weightG
            weightOz: $weightOz
        }){
            publicationId
            publicationType
            workId
            isbn
            createdAt
            updatedAt
            weightG: weight(units: G)
            weightOz: weight(units: OZ)
        }
    }
";

graphql_query_builder! {
    CreatePublicationRequest,
    CreatePublicationRequestBody,
    Variables,
    CREATE_PUBLICATION_MUTATION,
    CreatePublicationResponseBody,
    CreatePublicationResponseData,
    PushCreatePublication,
    PushActionCreatePublication
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub publication_type: PublicationType,
    pub work_id: Uuid,
    pub isbn: Option<Isbn>,
    pub weight_g: Option<f64>,
    pub weight_oz: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreatePublicationResponseData {
    pub create_publication: Option<Publication>,
}
