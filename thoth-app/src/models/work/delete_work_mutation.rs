use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

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
    pub work_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlimWork {
    pub work_id: Uuid,
    pub title: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteWorkResponseData {
    pub delete_work: Option<SlimWork>,
}
