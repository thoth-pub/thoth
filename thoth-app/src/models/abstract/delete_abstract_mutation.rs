use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::r#abstract::Abstract;
use uuid::Uuid;

const DELETE_ABSTRACT_MUTATION: &str = "
    mutation DeleteAbstract(
        $abstractId: Uuid!
    ) {
        deleteAbstracts(
            abstractId: $abstractId
        ){
            abstractId
            workId
            content
            localeCode
            abstractType
            canonical
        }
    }
";

graphql_query_builder! {
    DeleteAbstractRequest,
    DeleteAbstractRequestBody,
    Variables,
    DELETE_ABSTRACT_MUTATION,
    DeleteAbstractResponseBody,
    DeleteAbstractResponseData,
    PushDeleteAbstract,
    PushActionDeleteAbstract
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub abstract_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteAbstractResponseData {
    pub delete_abstract: Option<Abstract>,
} 