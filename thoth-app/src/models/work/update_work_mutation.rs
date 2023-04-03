use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::work::Work;
use thoth_api::model::work::WorkStatus;
use thoth_api::model::work::WorkType;
use thoth_api::model::Doi;
use uuid::Uuid;

const UPDATE_WORK_MUTATION: &str = "
    mutation UpdateWork(
        $workId: Uuid!,
        $workType: WorkType!,
        $workStatus: WorkStatus!,
        $fullTitle: String!,
        $title: String!,
        $subtitle: String,
        $reference: String,
        $edition: Int,
        $imprintId: Uuid!,
        $doi: Doi,
        $publicationDate: NaiveDate,
        $place: String,
        $pageCount: Int,
        $pageBreakdown: String,
        $imageCount: Int,
        $tableCount: Int,
        $audioCount: Int,
        $videoCount: Int,
        $license: String,
        $copyrightHolder: String,
        $landingPage: String,
        $lccn: String,
        $oclc: String,
        $shortAbstract: String,
        $longAbstract: String,
        $generalNote: String,
        $bibliographyNote: String,
        $toc: String,
        $coverUrl: String,
        $coverCaption: String,
        $firstPage: String,
        $lastPage: String,
        $pageInterval: String
    ) {
        updateWork(
            data: {
            workId: $workId
            workType: $workType
            workStatus: $workStatus
            fullTitle: $fullTitle
            title: $title
            subtitle: $subtitle
            reference: $reference
            edition: $edition
            imprintId: $imprintId
            doi: $doi
            publicationDate: $publicationDate
            place: $place
            pageCount: $pageCount
            pageBreakdown: $pageBreakdown
            imageCount: $imageCount
            tableCount: $tableCount
            audioCount: $audioCount
            videoCount: $videoCount
            license: $license
            copyrightHolder: $copyrightHolder
            landingPage: $landingPage
            lccn: $lccn
            oclc: $oclc
            shortAbstract: $shortAbstract
            longAbstract: $longAbstract
            generalNote: $generalNote
            bibliographyNote: $bibliographyNote
            toc: $toc
            coverUrl: $coverUrl
            coverCaption: $coverCaption
            firstPage: $firstPage
            lastPage: $lastPage
            pageInterval: $pageInterval
        }){
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
";

graphql_query_builder! {
    UpdateWorkRequest,
    UpdateWorkRequestBody,
    Variables,
    UPDATE_WORK_MUTATION,
    UpdateWorkResponseBody,
    UpdateWorkResponseData,
    PushUpdateWork,
    PushActionUpdateWork
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub work_id: Uuid,
    pub work_type: WorkType,
    pub work_status: WorkStatus,
    pub full_title: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub reference: Option<String>,
    pub edition: Option<i32>,
    pub doi: Option<Doi>,
    pub publication_date: Option<String>,
    pub place: Option<String>,
    pub page_count: Option<i32>,
    pub page_breakdown: Option<String>,
    pub image_count: Option<i32>,
    pub table_count: Option<i32>,
    pub audio_count: Option<i32>,
    pub video_count: Option<i32>,
    pub license: Option<String>,
    pub copyright_holder: Option<String>,
    pub landing_page: Option<String>,
    pub lccn: Option<String>,
    pub oclc: Option<String>,
    pub short_abstract: Option<String>,
    pub long_abstract: Option<String>,
    pub general_note: Option<String>,
    pub bibliography_note: Option<String>,
    pub toc: Option<String>,
    pub cover_url: Option<String>,
    pub cover_caption: Option<String>,
    pub imprint_id: Uuid,
    pub first_page: Option<String>,
    pub last_page: Option<String>,
    pub page_interval: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkResponseData {
    pub update_work: Option<Work>,
}
