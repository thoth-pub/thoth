use serde::Deserialize;
use serde::Serialize;
use thoth_api::work::model::WorkType;
use thoth_api::work::model::WorkStatus;

const DELETE_WORK_MUTATION: &str = "
    mutation DeleteWork(
        $workId: Uuid!
    ) {
        deleteWork(
            workId: $workId
        ){
            workId
            title
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
    pub work_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlimWork {
    pub work_id: String,
    pub title: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteWorkResponseData {
    pub delete_work: Option<SlimWork>,
}
