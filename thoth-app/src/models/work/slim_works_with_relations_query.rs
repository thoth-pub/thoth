use crate::models::Dropdown;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use thoth_api::model::imprint::ImprintWithPublisher;
use thoth_api::model::publisher::Publisher;
use thoth_api::model::title::Title;
use thoth_api::model::work::Work;
use thoth_api::model::work::WorkWithRelations;

pub use crate::models::work::works_query::Variables;
use crate::models::work::works_query::{WORKS_QUERY_FOOTER, WORKS_QUERY_HEADER};

pub const SLIM_WORKS_WITH_RELATIONS_QUERY_BODY: &str = "
            workId
            workType
            workStatus
            imprintId
            doi
            copyrightHolder
            createdAt
            updatedAt
            updatedAtWithRelations
            titles {
                titleId
                localeCode
                fullTitle
                title
                subtitle
                canonical
            }
        }";

graphql_query_builder! {
    SlimWorksWithRelationsRequest,
    SlimWorksWithRelationsRequestBody,
    Variables,
    format!("{WORKS_QUERY_HEADER}{SLIM_WORKS_WITH_RELATIONS_QUERY_BODY}{WORKS_QUERY_FOOTER}"),
    SlimWorksWithRelationsResponseBody,
    SlimWorksWithRelationsResponseData,
    FetchSlimWorksWithRelations,
    FetchActionSlimWorksWithRelations
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SlimWorkWithRelations {
    pub work_id: uuid::Uuid,
    pub work_type: thoth_api::model::work::WorkType,
    pub work_status: thoth_api::model::work::WorkStatus,
    pub imprint_id: uuid::Uuid,
    pub doi: Option<thoth_api::model::Doi>,
    pub copyright_holder: Option<String>,
    pub created_at: thoth_api::model::Timestamp,
    pub updated_at: thoth_api::model::Timestamp,
    pub updated_at_with_relations: thoth_api::model::Timestamp,
    pub titles: Option<Vec<Title>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SlimWorksWithRelationsResponseData {
    pub works: Vec<SlimWorkWithRelations>,
    pub work_count: i32,
}

impl fmt::Display for SlimWorkWithRelations {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Get the title from the first title entry, or fall back to work_id
        let title = if let Some(titles) = &self.titles {
            if let Some(first_title) = titles.first() {
                first_title.title.clone()
            } else {
                self.work_id.to_string()
            }
        } else {
            self.work_id.to_string()
        };

        match &self.doi {
            Some(doi) => write!(f, "{} - {}", title, doi),
            None => write!(f, "{}", title),
        }
    }
}

impl Dropdown for SlimWorkWithRelations {}

impl From<SlimWorkWithRelations> for Work {
    fn from(work: SlimWorkWithRelations) -> Self {
        Self {
            work_id: work.work_id,
            work_type: work.work_type,
            work_status: work.work_status,
            reference: None,
            edition: None,
            imprint_id: work.imprint_id,
            doi: work.doi,
            publication_date: None,
            withdrawn_date: None,
            place: None,
            page_count: None,
            page_breakdown: None,
            image_count: None,
            table_count: None,
            audio_count: None,
            video_count: None,
            license: None,
            copyright_holder: work.copyright_holder,
            landing_page: None,
            lccn: None,
            oclc: None,
            general_note: None,
            bibliography_note: None,
            toc: None,
            cover_url: None,
            cover_caption: None,
            created_at: work.created_at,
            updated_at: work.updated_at,
            first_page: None,
            last_page: None,
            page_interval: None,
            updated_at_with_relations: work.updated_at_with_relations,
        }
    }
}

impl From<SlimWorkWithRelations> for WorkWithRelations {
    fn from(work: SlimWorkWithRelations) -> Self {
        // Extract title information from the titles field
        let (full_title, title, subtitle) = if let Some(titles) = &work.titles {
            if let Some(first_title) = titles.first() {
                (
                    first_title.full_title.clone(),
                    first_title.title.clone(),
                    first_title.subtitle.clone(),
                )
            } else {
                (work.work_id.to_string(), work.work_id.to_string(), None)
            }
        } else {
            (work.work_id.to_string(), work.work_id.to_string(), None)
        };

        Self {
            work_id: work.work_id,
            work_type: work.work_type,
            work_status: work.work_status,
            full_title,
            title,
            subtitle,
            reference: None,
            edition: None,
            doi: work.doi,
            publication_date: None,
            withdrawn_date: None,
            place: None,
            page_count: None,
            page_breakdown: None,
            image_count: None,
            table_count: None,
            audio_count: None,
            video_count: None,
            license: None,
            copyright_holder: work.copyright_holder,
            landing_page: None,
            lccn: None,
            oclc: None,
            short_abstract: None,
            long_abstract: None,
            general_note: None,
            bibliography_note: None,
            toc: None,
            cover_url: None,
            cover_caption: None,
            updated_at: work.updated_at,
            first_page: None,
            last_page: None,
            page_interval: None,
            contributions: None,
            publications: None,
            languages: None,
            fundings: None,
            subjects: None,
            issues: None,
            imprint: ImprintWithPublisher {
                imprint_id: work.imprint_id,
                imprint_name: "".to_string(),
                imprint_url: None,
                crossmark_doi: None,
                updated_at: work.updated_at,
                publisher: Publisher {
                    publisher_id: uuid::Uuid::new_v4(),
                    publisher_name: "".to_string(),
                    publisher_shortname: None,
                    publisher_url: None,
                    created_at: work.created_at,
                    updated_at: work.updated_at,
                },
            },
            relations: None,
            references: None,
            titles: work.titles,
            abstracts: None,
        }
    }
}
