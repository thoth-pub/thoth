use serde::Deserialize;
use serde::Serialize;

use crate::api::models::Imprint;
use crate::api::models::Work;
use crate::api::models::WorkStatusDefinition;
use crate::api::models::WorkTypeDefinition;

pub const WORK_QUERY: &str = "
    query WorkQuery($workId: Uuid!) {
        work(workId: $workId) {
            workId
            workType
            workStatus
            fullTitle
            title
            subtitle
            reference
            edition
            doi
            publicationDate
            place
            width
            height
            pageCount
            pageBreakdown
            imageCount
            tableCount
            videoCount
            license
            copyrightHolder
            landingPage
            lccn
            oclc
            shortAbstract
            longAbstract
            generalNote
            toc
            coverUrl
            coverCaption
            contributions {
                contributionType
                mainContribution
                biography
                institution
                contributor {
                    contributorId
                    fullName
                }
            }
            imprint {
                imprintId
                imprintName
                publisher {
                    publisherId
                    publisherName
                    publisherShortname
                    publisherUrl
                }
            }
        }
        imprints(limit: 9999) {
            imprintId
            imprintName
            publisher {
                publisherId
                publisherName
                publisherShortname
                publisherUrl
            }
        }
        work_types: __type(name: \"WorkType\") {
            enumValues {
                name
            }
        }
        work_statuses: __type(name: \"WorkStatus\") {
            enumValues {
                name
            }
        }
    }
";

query_builder! {
    WorkRequest,
    WorkRequestBody,
    WORK_QUERY,
    WorkResponseBody,
    WorkResponseData,
    FetchWork,
    FetchActionWork
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkResponseData {
    pub work: Option<Work>,
    pub imprints: Vec<Imprint>,
    pub work_types: WorkTypeDefinition,
    pub work_statuses: WorkStatusDefinition,
}

impl Default for WorkResponseData {
    fn default() -> WorkResponseData {
        WorkResponseData {
            work: None,
            imprints: vec![],
            work_types: WorkTypeDefinition {
                enum_values: vec![],
            },
            work_statuses: WorkStatusDefinition {
                enum_values: vec![],
            },
        }
    }
}
