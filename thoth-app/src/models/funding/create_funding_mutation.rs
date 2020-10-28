use serde::Deserialize;
use serde::Serialize;

const CREATE_FUNDING_MUTATION: &str = "
    mutation CreateFunding(
        $workId: Uuid!,
        $funderId: Uuid!,
        $program: String,
        $projectName: String,
        $projectShortname: String,
        $grantNumber: String,
        $jurisdiction: String
    ) {
        createFunding(data: {
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
    CreateFundingRequest,
    CreateFundingRequestBody,
    Variables,
    CREATE_FUNDING_MUTATION,
    CreateFundingResponseBody,
    CreateFundingResponseData,
    PushCreateFunding,
    PushActionCreateFunding
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
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
pub struct CreateFundingResponseData {
    pub create_funding: Option<SlimFunding>,
}
