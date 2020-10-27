use serde::Deserialize;
use serde::Serialize;

use super::super::imprint::Imprint;
use super::Work;
use super::WorkStatusDefinition;
use super::WorkTypeDefinition;

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
                workId
                contributorId
                contributionType
                mainContribution
                biography
                institution
                contributor {
                    contributorId
                    lastName
                    fullName
                }
            }
            publications {
                publicationId
                publicationType
                workId
                isbn
                publicationUrl
            }
            languages {
                languageId
                workId
                languageCode
                languageRelation
                mainLanguage
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
            }
            subjects {
                subjectId
                workId
                subjectType
                subjectCode
                subjectOrdinal
            }
            issues {
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
    pub work_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct WorkResponseData {
    pub work: Option<Work>,
    pub imprints: Vec<Imprint>,
    pub work_types: WorkTypeDefinition,
    pub work_statuses: WorkStatusDefinition,
}
