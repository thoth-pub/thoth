use serde::Deserialize;
use serde::Serialize;

const UPDATE_FUNDING_MUTATION: &str = "
    mutation UpdateFunding(
        $fundingId: Uuid!,
        $workId: Uuid!,
        $funderId: Uuid!,
        $program: String,
        $projectName: String,
        $projectShortname: String,
        $grantNumber: String,
        $jurisdiction: String
    ) {
        updateFunding(data: {
            fundingId: $fundingId
            workId: $workId
            funderId: $funderId
            program: $program
            projectName: $projectName
            projectShortname: $projectShortname
            grantNumber: $grantNumber
            jurisdicion: $jurisdiction
        }){
            fundingId
        }
    }
";

graphql_query_builder! {
    UpdateFundingRequest,
    UpdateFundingRequestBody,
    Variables,
    UPDATE_FUNDING_MUTATION,
    UpdateFundingResponseBody,
    UpdateFundingResponseData,
    PushUpdateFunding,
    PushActionUpdateFunding
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub funding_id: String,
    pub work_id: String,
    pub funder_id: String,
    pub program: Option<String>,
    pub project_name: Option<String>,
    pub project_shortname: Option<String>,
    pub grant_number: Option<String>,
    pub jurisdiction: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlimFunding {
    pub funding_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFundingResponseData {
    pub update_funding: Option<SlimFunding>,
}
