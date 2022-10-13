use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::reference::Reference;
use uuid::Uuid;

const DELETE_REFERENCE_MUTATION: &str = "
    mutation DeleteReference(
        $referenceId: Uuid!
    ) {
        deleteReference(
            referenceId: $referenceId
        ){
            referenceId
            workId
            referenceOrdinal
            doi
            unstructuredCitation
            issn
            isbn
            journalTitle
            articleTitle
            seriesTitle
            volumeTitle
            edition
            author
            volume
            issue
            firstPage
            componentNumber
            standardDesignator
            standardsBodyName
            standardsBodyAcronym
            url
            publicationDate
            retrievalDate
            createdAt
            updatedAt
        }
    }
";

graphql_query_builder! {
    DeleteReferenceRequest,
    DeleteReferenceRequestBody,
    Variables,
    DELETE_REFERENCE_MUTATION,
    DeleteReferenceResponseBody,
    DeleteReferenceResponseData,
    PushDeleteReference,
    PushActionDeleteReference
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub reference_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteReferenceResponseData {
    pub delete_reference: Option<Reference>,
}
