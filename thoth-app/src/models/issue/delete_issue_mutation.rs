use serde::Deserialize;
use serde::Serialize;
use thoth_api::issue::model::Issue;
use uuid::Uuid;

const DELETE_ISSUE_MUTATION: &str = "
    mutation DeleteIssue(
        $issueId: Uuid!,
    ) {
        deleteIssue(
            issueId: $issueId
        ){
            issueId
            seriesId
            workId
            issueOrdinal
            createdAt
            updatedAt
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
pub struct DeleteIssueResponseData {
    pub delete_issue: Option<Issue>,
}
