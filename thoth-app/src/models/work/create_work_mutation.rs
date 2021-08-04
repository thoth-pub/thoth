use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::Doi;
use thoth_api::model::LengthUnit;
use thoth_api::work::model::Work;
use thoth_api::work::model::WorkStatus;
use thoth_api::work::model::WorkType;
use uuid::Uuid;

const CREATE_WORK_MUTATION: &str = "
    mutation CreateWork(
        $units: LengthUnit!
        $workType: WorkType!,
        $workStatus: WorkStatus!,
        $fullTitle: String!,
        $title: String!,
        $subtitle: String,
        $reference: String,
        $edition: Int!,
        $imprintId: Uuid!,
        $doi: Doi,
        $publicationDate: NaiveDate,
        $place: String,
        $width: Int,
        $height: Int,
        $pageCount: Int,
        $pageBreakdown: String,
        $imageCount: Int,
        $tableCount: Int,
        $audioCount: Int,
        $videoCount: Int,
        $license: String,
        $copyrightHolder: String!,
        $landingPage: String,
        $lccn: String,
        $oclc: String,
        $shortAbstract: String,
        $longAbstract: String,
        $generalNote: String,
        $toc: String,
        $coverUrl: String,
        $coverCaption: String
    ) {
        createWork(units: $units,
            data: {
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
            width: $width
            height: $height
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
            toc: $toc
            coverUrl: $coverUrl
            coverCaption: $coverCaption
        }){
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
";

graphql_query_builder! {
    CreateWorkRequest,
    CreateWorkRequestBody,
    Variables,
    CREATE_WORK_MUTATION,
    CreateWorkResponseBody,
    CreateWorkResponseData,
    PushCreateWork,
    PushActionCreateWork
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub work_type: WorkType,
    pub work_status: WorkStatus,
    pub full_title: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub reference: Option<String>,
    pub edition: i32,
    pub doi: Option<Doi>,
    pub publication_date: Option<String>,
    pub place: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub page_count: Option<i32>,
    pub page_breakdown: Option<String>,
    pub image_count: Option<i32>,
    pub table_count: Option<i32>,
    pub audio_count: Option<i32>,
    pub video_count: Option<i32>,
    pub license: Option<String>,
    pub copyright_holder: String,
    pub landing_page: Option<String>,
    pub lccn: Option<String>,
    pub oclc: Option<String>,
    pub short_abstract: Option<String>,
    pub long_abstract: Option<String>,
    pub general_note: Option<String>,
    pub toc: Option<String>,
    pub cover_url: Option<String>,
    pub cover_caption: Option<String>,
    pub imprint_id: Uuid,
    pub units: LengthUnit,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkResponseData {
    pub create_work: Option<Work>,
}
