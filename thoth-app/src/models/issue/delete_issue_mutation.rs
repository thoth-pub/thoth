use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

const DELETE_ISSUE_MUTATION: &str = "
    mutation DeleteIssue(
        $issueId: Uuid!,
    ) {
        deleteIssue(
            issueId: $issueId
        ){
            issueId
            workId
            seriesId
            issueOrdinal
        }
    }
";

graphql_query_builder! {
    DeleteIssueRequest,
    DeleteIssueRequestBody,
    Variables,
    DELETE_ISSUE_MUTATION,
    DeleteIssueResponseBody,
    DeleteIssueResponseData,
    PushDeleteIssue,
    PushActionDeleteIssue
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub issue_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlimIssue {
    pub issue_id: Uuid,
    pub work_id: Uuid,
    pub series_id: Uuid,
    pub issue_ordinal: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteIssueResponseData {
    pub delete_issue: Option<SlimIssue>,
}
