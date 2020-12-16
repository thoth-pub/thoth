use serde::Deserialize;
use serde::Serialize;

const DELETE_ISSUE_MUTATION: &str = "
    mutation DeleteIssue(
        $workId: Uuid!,
        $seriesId: Uuid!
    ) {
        deleteIssue(
            workId: $workId
            seriesId: $seriesId
        ){
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
    pub work_id: String,
    pub series_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlimIssue {
    pub work_id: String,
    pub series_id: String,
    pub issue_ordinal: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteIssueResponseData {
    pub delete_issue: Option<SlimIssue>,
}
