use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::affiliation::AffiliationWithContribution;
use thoth_api::model::funding::FundingWithWork;
use uuid::Uuid;

use crate::graphql_query_builder;

pub const INSTITUTION_ACTIVITY_QUERY: &str = "
    query InstitutionActivityQuery($institutionId: Uuid!) {
        institution(institutionId: $institutionId) {
            fundings {
                work {
                    workId
                    workType
                    workStatus
                    fullTitle
                    title
                    edition
                    copyrightHolder
                    updatedAt
                    imprint {
                        imprintId
                        imprintName
                        updatedAt
                        publisher {
                            publisherId
                            publisherName
                            createdAt
                            updatedAt
                        }
                    }
                }
            }
            affiliations {
                contribution {
                    work {
                        workId
                        workType
                        workStatus
                        fullTitle
                        title
                        edition
                        updatedAt
                        imprint {
                            imprintId
                            imprintName
                            updatedAt
                            publisher {
                                publisherId
                                publisherName
                                createdAt
                                updatedAt
                            }
                        }
                    }
                }
            }
        }
    }
";

graphql_query_builder! {
    InstitutionActivityRequest,
    InstitutionActivityRequestBody,
    Variables,
    INSTITUTION_ACTIVITY_QUERY,
    InstitutionActivityResponseBody,
    InstitutionActivityResponseData,
    FetchInstitutionActivity,
    FetchActionInstitutionActivity
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub institution_id: Option<Uuid>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct InstitutionActivityResponseData {
    pub institution: Option<InstitutionActivity>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionActivity {
    pub fundings: Option<Vec<FundingWithWork>>,
    pub affiliations: Option<Vec<AffiliationWithContribution>>,
}
