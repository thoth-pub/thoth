use serde::Deserialize;
use serde::Serialize;
use thoth_api::imprint::model::ImprintWithPublisher;
use thoth_api::work::model::WorkWithRelations;
use uuid::Uuid;

use super::WorkStatusDefinition;
use super::WorkTypeDefinition;

pub const WORK_QUERY: &str = "
    query WorkQuery($workId: Uuid!, $publishers: [Uuid!]) {
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
            audioCount
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
            updatedAt
            contributions {
                contributionId
                workId
                contributorId
                contributionType
                mainContribution
                biography
                institution
                createdAt
                updatedAt
                lastName
                fullName
            }
            publications {
                publicationId
                publicationType
                workId
                isbn
                publicationUrl
                createdAt
                updatedAt
            }
            languages {
                languageId
                workId
                languageCode
                languageRelation
                mainLanguage
                createdAt
                updatedAt
            }
            fundings {
                fundingId
                workId
                funderId
                program
                projectName
                projectShortname
                grantNumber
                jurisdiction
                funder {
                    funderId
                    funderName
                    createdAt
                    updatedAt
                }
            }
            subjects {
                subjectId
                workId
                subjectType
                subjectCode
                subjectOrdinal
                createdAt
                updatedAt
            }
            issues {
                issueId
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
                            createdAt
                            updatedAt
                        }
                    }
                }
            }
            imprint {
                imprintId
                imprintName
                updatedAt
                publisher {
                    publisherId
                    publisherName
                    publisherShortname
                    publisherUrl
                    createdAt
                    updatedAt
                }
            }
        }
        imprints(limit: 9999, publishers: $publishers) {
            imprintId
            imprintName
            updatedAt
            publisher {
                publisherId
                publisherName
                publisherShortname
                publisherUrl
                createdAt
                updatedAt
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

graphql_query_builder! {
    WorkRequest,
    WorkRequestBody,
    Variables,
    WORK_QUERY,
    WorkResponseBody,
    WorkResponseData,
    FetchWork,
    FetchActionWork
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub work_id: Option<Uuid>,
    pub publishers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct WorkResponseData {
    pub work: Option<WorkWithRelations>,
    pub imprints: Vec<ImprintWithPublisher>,
    pub work_types: WorkTypeDefinition,
    pub work_statuses: WorkStatusDefinition,
}
