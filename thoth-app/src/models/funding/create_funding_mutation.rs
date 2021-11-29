use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::funding::FundingWithInstitution;
use uuid::Uuid;

const CREATE_FUNDING_MUTATION: &str = "
    mutation CreateFunding(
        $workId: Uuid!,
        $institutionId: Uuid!,
        $program: String,
        $projectName: String,
        $projectShortname: String,
        $grantNumber: String,
        $jurisdiction: String
    ) {
        createFunding(data: {
            workId: $workId
            institutionId: $institutionId
            program: $program
            projectName: $projectName
            projectShortname: $projectShortname
            grantNumber: $grantNumber
            jurisdiction: $jurisdiction
        }){
            fundingId
            workId
            institutionId
            program
            projectName
            projectShortname
            grantNumber
            jurisdiction
            institution {
                institutionId
                institutionName
                createdAt
                updatedAt
            }
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
    pub work_id: Uuid,
    pub institution_id: Uuid,
    pub program: Option<String>,
    pub project_name: Option<String>,
    pub project_shortname: Option<String>,
    pub grant_number: Option<String>,
    pub jurisdiction: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateFundingResponseData {
    pub create_funding: Option<FundingWithInstitution>,
}
