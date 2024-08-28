use chrono::NaiveDate;
use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::reference::Reference;
use thoth_api::model::{Doi, Isbn};
use uuid::Uuid;

const CREATE_REFERENCE_MUTATION: &str = "
    mutation CreateReference(
        $workId: Uuid!,
        $referenceOrdinal: Int!,
        $doi: Doi,
        $unstructuredCitation: String,
        $issn: String,
        $isbn: Isbn,
        $journalTitle: String,
        $articleTitle: String,
        $seriesTitle: String,
        $volumeTitle: String,
        $edition: Int,
        $author: String,
        $volume: String
        $issue: String,
        $firstPage: String,
        $componentNumber: String,
        $standardDesignator: String,
        $standardsBodyName: String,
        $standardsBodyAcronym: String,
        $url: String,
        $publicationDate: Date,
        $retrievalDate: Date,
    ) {
        createReference(
            data: {
                workId: $workId,
                referenceOrdinal: $referenceOrdinal,
                doi: $doi,
                unstructuredCitation: $unstructuredCitation,
                issn: $issn,
                isbn: $isbn,
                journalTitle: $journalTitle,
                articleTitle: $articleTitle,
                seriesTitle: $seriesTitle,
                volumeTitle: $volumeTitle,
                edition: $edition,
                author: $author,
                volume: $volume,
                issue: $issue,
                firstPage: $firstPage,
                componentNumber: $componentNumber,
                standardDesignator: $standardDesignator,
                standardsBodyName: $standardsBodyName,
                standardsBodyAcronym: $standardsBodyAcronym,
                url: $url,
                publicationDate: $publicationDate,
                retrievalDate: $retrievalDate,
            }
        ) {
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
";

graphql_query_builder! {
    CreateReferenceRequest,
    CreateReferenceRequestBody,
    Variables,
    CREATE_REFERENCE_MUTATION,
    CreateReferenceResponseBody,
    CreateReferenceResponseData,
    PushCreateReference,
    PushActionCreateReference
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub work_id: Uuid,
    pub reference_ordinal: i32,
    pub doi: Option<Doi>,
    pub unstructured_citation: Option<String>,
    pub issn: Option<String>,
    pub isbn: Option<Isbn>,
    pub journal_title: Option<String>,
    pub article_title: Option<String>,
    pub series_title: Option<String>,
    pub volume_title: Option<String>,
    pub edition: Option<i32>,
    pub author: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub first_page: Option<String>,
    pub component_number: Option<String>,
    pub standard_designator: Option<String>,
    pub standards_body_name: Option<String>,
    pub standards_body_acronym: Option<String>,
    pub url: Option<String>,
    pub publication_date: Option<NaiveDate>,
    pub retrieval_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateReferenceResponseData {
    pub create_reference: Option<Reference>,
}
