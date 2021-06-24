use serde::Deserialize;
use serde::Serialize;
use thoth_api::work::model::Work;
use uuid::Uuid;

const DELETE_WORK_MUTATION: &str = "
    mutation DeleteWork(
        $workId: Uuid!
    ) {
        deleteWork(
            workId: $workId
        ){
            workId
            workType
            workStatus
            fullTitle
            title
            edition
            imprintId
            copyrightHolder
            createdAt
            updatedAt
        }
    }
";

graphql_query_builder! {
    DeleteWorkRequest,
    DeleteWorkRequestBody,
    Variables,
    DELETE_WORK_MUTATION,
    DeleteWorkResponseBody,
    DeleteWorkResponseData,
    PushDeleteWork,
    PushActionDeleteWork
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub work_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteWorkResponseData {
    pub delete_work: Option<Work>,
}
