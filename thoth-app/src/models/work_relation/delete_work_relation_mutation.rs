use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::work_relation::WorkRelation;
use uuid::Uuid;

const DELETE_WORK_RELATION_MUTATION: &str = "
    mutation DeleteWorkRelation(
        $workRelationId: Uuid!
    ) {
        deleteWorkRelation(
            workRelationId: $workRelationId
        ){
            workRelationId
            relatorWorkId
            relatedWorkId
            relationType
            relationOrdinal
            createdAt
            updatedAt
        }
    }
";

graphql_query_builder! {
    DeleteWorkRelationRequest,
    DeleteWorkRelationRequestBody,
    Variables,
    DELETE_WORK_RELATION_MUTATION,
    DeleteWorkRelationResponseBody,
    DeleteWorkRelationResponseData,
    PushDeleteWorkRelation,
    PushActionDeleteWorkRelation
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub work_relation_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteWorkRelationResponseData {
    pub delete_work_relation: Option<WorkRelation>,
}
