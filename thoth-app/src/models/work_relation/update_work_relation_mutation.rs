use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::work_relation::RelationType;
use thoth_api::model::work_relation::WorkRelationWithRelatedWork;
use uuid::Uuid;

const UPDATE_WORK_RELATION_MUTATION: &str = "
    mutation UpdateWorkRelation(
        $workRelationId: Uuid!,
        $relatorWorkId: Uuid!,
        $relatedWorkId: Uuid!,
        $relationType: RelationType!,
        $relationOrdinal: Int!
    ) {
        updateWorkRelation(data: {
            workRelationId: $workRelationId
            relatorWorkId: $relatorWorkId
            relatedWorkId: $relatedWorkId
            relationType: $relationType
            relationOrdinal: $relationOrdinal
        }){
            workRelationId
            relatorWorkId
            relatedWorkId
            relationType
            relationOrdinal
            createdAt
            updatedAt
            relatedWork {
                workId
                workType
                workStatus
                fullTitle
                title
                imprintId
                copyrightHolder
                createdAt
                updatedAt
            }
        }
    }
";

graphql_query_builder! {
    UpdateWorkRelationRequest,
    UpdateWorkRelationRequestBody,
    Variables,
    UPDATE_WORK_RELATION_MUTATION,
    UpdateWorkRelationResponseBody,
    UpdateWorkRelationResponseData,
    PushUpdateWorkRelation,
    PushActionUpdateWorkRelation
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub work_relation_id: Uuid,
    pub relator_work_id: Uuid,
    pub related_work_id: Uuid,
    pub relation_type: RelationType,
    pub relation_ordinal: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkRelationResponseData {
    pub update_work_relation: Option<WorkRelationWithRelatedWork>,
}
