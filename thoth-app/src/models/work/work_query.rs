use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::imprint::ImprintWithPublisher;
use thoth_api::model::work::WorkWithRelations;
use uuid::Uuid;

use super::WorkStatusDefinition;
use super::WorkTypeDefinition;

pub const WORK_QUERY: &str = "
    query WorkQuery($workId: Uuid!, $publishers: [Uuid!]) {
        work(workId: $workId) {
            workId
            workType
            workStatus
            reference
            edition
            doi
            publicationDate
            withdrawnDate
            place
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
            bibliographyNote
            toc
            coverUrl
            coverCaption
            updatedAt
            firstPage
            lastPage
            pageInterval
            relations(order: {field: RELATION_ORDINAL, direction: ASC}) {
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
                    imprintId
                    createdAt
                    updatedAt
                    updatedAtWithRelations
                }
            }
            contributions(order: {field: CONTRIBUTION_ORDINAL, direction: ASC}) {
                contributionId
                workId
                contributorId
                contributionType
                mainContribution
                biography
                createdAt
                updatedAt
                firstName
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
                weightG: weight(units: G)
                weightOz: weight(units: OZ)
                widthMm: width(units: MM)
                widthIn: width(units: IN)
                heightMm: height(units: MM)
                heightIn: height(units: IN)
                depthMm: depth(units: MM)
                depthIn: depth(units: IN)
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
            references(order: {field: REFERENCE_ORDINAL, direction: ASC}) {
                referenceId
                workId
                referenceOrdinal
                doi
                unstructuredCitation
                issn
                isbn
                journalTitle
                articleTitle
                seriesTitle
                volumeTitle
                edition
                author
                volume
                issue
                firstPage
                componentNumber
                standardDesignator
                standardsBodyName
                standardsBodyAcronym
                url
                publicationDate
                retrievalDate
                createdAt
                updatedAt
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
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
