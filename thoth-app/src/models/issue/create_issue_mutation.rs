use serde::Deserialize;
use serde::Serialize;

use super::Issue;

const CREATE_ISSUE_MUTATION: &str = "
    mutation CreateIssue(
        $workId: Uuid!,
        $seriesId: Uuid!,
        $issueOrdinal: Int!,
    ) {
        createIssue(data: {
            workId: $workId
            seriesId: $seriesId
            issueOrdinal: $issueOrdinal
        }){
            workId
            seriesId
            issueOrdinal
            series {
                seriesId
                seriesType
                seriesName
                issnPrint
                issnDigital
                seriesUrl
                updatedAt
                imprint {
                    imprintId
                    imprintName
                    updatedAt
                    publisher {
                        publisherId
                        publisherName
                        publisherShortname
                        publisherUrl
                        updatedAt
                    }
                }
            }
        }
    }
";

graphql_query_builder! {
    CreateIssueRequest,
    CreateIssueRequestBody,
    Variables,
    CREATE_ISSUE_MUTATION,
    CreateIssueResponseBody,
    CreateIssueResponseData,
    PushCreateIssue,
    PushActionCreateIssue
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub work_id: String,
    pub series_id: String,
    pub issue_ordinal: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateIssueResponseData {
    pub create_issue: Option<Issue>,
}
