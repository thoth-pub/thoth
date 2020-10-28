use serde::Deserialize;
use serde::Serialize;

const UPDATE_ISSUE_MUTATION: &str = "
    mutation UpdateIssue(
        $workId: Uuid!,
        $seriesId: Uuid!,
        $issueOrinal: Int!,
    ) {
        updateIssue(data: {
            workId: $workId
            seriesId: $seriesId
            issueOrdinal: $issueOrdinal
        }){
            workId
            seriesId
            issueOrdinal
        }
    }
";

graphql_query_builder! {
    UpdateIssueRequest,
    UpdateIssueRequestBody,
    Variables,
    UPDATE_ISSUE_MUTATION,
    UpdateIssueResponseBody,
    UpdateIssueResponseData,
    PushUpdateIssue,
    PushActionUpdateIssue
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub work_id: String,
    pub series_id: String,
    pub series_ordinal: i32,
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
pub struct UpdateIssueResponseData {
    pub update_issue: Option<SlimIssue>,
}
