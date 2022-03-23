use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::publication::Publication;
use thoth_api::model::publication::PublicationType;
use thoth_api::model::Isbn;
use uuid::Uuid;

const UPDATE_PUBLICATION_MUTATION: &str = "
    mutation UpdatePublication(
        $publicationId: Uuid!,
        $publicationType: PublicationType!,
        $workId: Uuid!,
        $isbn: Isbn,
        $weightG: Float,
        $weightOz: Float,
        $widthMm: Float,
        $widthIn: Float,
        $heightMm: Float,
        $heightIn: Float,
        $depthMm: Float,
        $depthIn: Float,
    ) {
        updatePublication(
            data: {
            publicationId: $publicationId
            publicationType: $publicationType
            workId: $workId
            isbn: $isbn
            weightG: $weightG
            weightOz: $weightOz
            widthMm: $widthMm
            widthIn: $widthIn
            heightMm: $heightMm
            heightIn: $heightIn
            depthMm: $depthMm
            depthIn: $depthIn
        }){
            publicationId
            publicationType
            workId
            isbn
            createdAt
            updatedAt
            weightG: weight(units: G)
            weightOz: weight(units: OZ)
            widthMm: width(units: MM)
            widthIn: width(units: IN)
            heightMm: height(units: MM)
            heightIn: height(units: IN)
            depthMm: depth(units: MM)
            depthIn: depth(units: IN)
        }
    }
";

graphql_query_builder! {
    UpdatePublicationRequest,
    UpdatePublicationRequestBody,
    Variables,
    UPDATE_PUBLICATION_MUTATION,
    UpdatePublicationResponseBody,
    UpdatePublicationResponseData,
    PushUpdatePublication,
    PushActionUpdatePublication
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub publication_id: Uuid,
    pub publication_type: PublicationType,
    pub work_id: Uuid,
    pub isbn: Option<Isbn>,
    pub width_mm: Option<f64>,
    pub width_in: Option<f64>,
    pub height_mm: Option<f64>,
    pub height_in: Option<f64>,
    pub depth_mm: Option<f64>,
    pub depth_in: Option<f64>,
    pub weight_g: Option<f64>,
    pub weight_oz: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePublicationResponseData {
    pub update_publication: Option<Publication>,
}
