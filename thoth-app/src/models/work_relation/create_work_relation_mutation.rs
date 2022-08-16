use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::work_relation::RelationType;
use thoth_api::model::work_relation::WorkRelationWithRelatedWork;
use uuid::Uuid;

const CREATE_WORK_RELATION_MUTATION: &str = "
    mutation CreateWorkRelation(
        $relatorWorkId: Uuid!,
        $relatedWorkId: Uuid!,
        $relationType: RelationType!,
        $relationOrdinal: Int!
    ) {
        createWorkRelation(data: {
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
                createdAt
                updatedAt
            }
        }
    }
";

graphql_query_builder! {
    CreateWorkRelationRequest,
    CreateWorkRelationRequestBody,
    Variables,
    CREATE_WORK_RELATION_MUTATION,
    CreateWorkRelationResponseBody,
    CreateWorkRelationResponseData,
    PushCreateWorkRelation,
    PushActionCreateWorkRelation
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub relator_work_id: Uuid,
    pub related_work_id: Uuid,
    pub relation_type: RelationType,
    pub relation_ordinal: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkRelationResponseData {
    pub create_work_relation: Option<WorkRelationWithRelatedWork>,
}
