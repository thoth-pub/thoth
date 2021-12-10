use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::imprint::ImprintWithPublisher;
use thoth_api::model::work::WorkWithRelations;
use thoth_api::model::LengthUnit;
use uuid::Uuid;

use super::LengthUnitDefinition;
use super::WorkStatusDefinition;
use super::WorkTypeDefinition;

pub const WORK_QUERY: &str = "
    query WorkQuery($workId: Uuid!, $publishers: [Uuid!], $units: LengthUnit) {
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
            width(units: $units)
            height(units: $units)
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
            firstPage
            lastPage
            pageInterval
            relations {
                workRelationId
                relatorWorkId
                relatedWorkId
                relationType
                relationOrdinal
                createdAt
                updatedAt
                relatedWork {
                    workId
                    workType
                    workStatus
                    fullTitle
                    title
                    edition
                    imprintId
                    copyrightHolder
                    createdAt
                    updatedAt
                }
            }
            contributions {
                contributionId
                workId
                contributorId
                contributionType
                mainContribution
                biography
                createdAt
                updatedAt
                lastName
                fullName
                contributionOrdinal
            }
            publications {
                publicationId
                publicationType
                workId
                isbn
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
        length_units: __type(name: \"LengthUnit\") {
            enumValues {
                name
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
    pub units: LengthUnit,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct WorkResponseData {
    pub work: Option<WorkWithRelations>,
    pub imprints: Vec<ImprintWithPublisher>,
    pub length_units: LengthUnitDefinition,
    pub work_types: WorkTypeDefinition,
    pub work_statuses: WorkStatusDefinition,
}
