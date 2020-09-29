use serde::de::Deserializer;
use serde::Deserialize;
use serde::Serialize;

use thoth_api::models::contributor::ContributionType;
use thoth_api::models::series::SeriesType;
use thoth_api::models::work::WorkStatus;
use thoth_api::models::work::WorkType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Work {
    pub work_id: String,
    pub work_type: WorkType,
    pub work_status: WorkStatus,
    pub full_title: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub reference: Option<String>,
    pub edition: i32,
    pub doi: Option<String>,
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
    pub license: Option<License>,
    pub copyright_holder: String,
    pub landing_page: Option<String>,
    pub lccn: Option<i32>,
    pub oclc: Option<i32>,
    pub short_abstract: Option<String>,
    pub long_abstract: Option<String>,
    pub general_note: Option<String>,
    pub toc: Option<String>,
    pub cover_url: Option<String>,
    pub cover_caption: Option<String>,
    pub contributions: Option<Vec<Contribution>>,
    pub imprint: Imprint,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub enum License {
    By,
    BySa,
    ByNd,
    ByNc,
    ByNcSa,
    ByNcNd,
    Zero,
    Undefined,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Imprint {
    pub imprint_id: String,
    pub imprint_name: String,
    pub imprint_url: Option<String>,
    pub publisher: Publisher,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Publisher {
    pub publisher_id: String,
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contribution {
    pub work_id: String,
    pub contributor_id: String,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    pub biography: Option<String>,
    pub institution: Option<String>,
    pub contributor: Contributor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contributor {
    pub contributor_id: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub full_name: String,
    pub orcid: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub series_id: String,
    pub series_type: SeriesType,
    pub series_name: String,
    pub issn_print: String,
    pub issn_digital: String,
    pub series_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkTypeDefinition {
    pub enum_values: Vec<WorkTypeValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkStatusDefinition {
    pub enum_values: Vec<WorkStatusValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContributionTypeDefinition {
    pub enum_values: Vec<ContributionTypeValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkTypeValues {
    pub name: WorkType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkStatusValues {
    pub name: WorkStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContributionTypeValues {
    pub name: ContributionType,
}

impl Work {
    pub fn compile_fulltitle(&self) -> String {
        if let Some(subtitle) = &self.subtitle.clone() {
            format!("{}: {}", self.title, subtitle)
        } else {
            self.title.to_string()
        }
    }
}

impl<'de> Deserialize<'de> for License {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let l = String::deserialize(deserializer)?.to_lowercase();
        let license = match l.as_str() {
            "http://creativecommons.org/licenses/by/1.0/"
            | "http://creativecommons.org/licenses/by/2.0/"
            | "http://creativecommons.org/licenses/by/2.5/"
            | "http://creativecommons.org/licenses/by/3.0/"
            | "http://creativecommons.org/licenses/by/4.0/" => License::By,
            "http://creativecommons.org/licenses/by-sa/1.0/"
            | "http://creativecommons.org/licenses/by-sa/2.0/"
            | "http://creativecommons.org/licenses/by-sa/2.5/"
            | "http://creativecommons.org/licenses/by-sa/3.0/"
            | "http://creativecommons.org/licenses/by-sa/4.0/" => License::BySa,
            "http://creativecommons.org/licenses/by-nd/1.0/"
            | "http://creativecommons.org/licenses/by-nd/2.0/"
            | "http://creativecommons.org/licenses/by-nd/2.5/"
            | "http://creativecommons.org/licenses/by-nd/3.0/"
            | "http://creativecommons.org/licenses/by-nd/4.0/" => License::ByNd,
            "http://creativecommons.org/licenses/by-nc/1.0/"
            | "http://creativecommons.org/licenses/by-nc/2.0/"
            | "http://creativecommons.org/licenses/by-nc/2.5/"
            | "http://creativecommons.org/licenses/by-nc/3.0/"
            | "http://creativecommons.org/licenses/by-nc/4.0/" => License::ByNc,
            "http://creativecommons.org/licenses/by-nc-sa/1.0/"
            | "http://creativecommons.org/licenses/by-nc-sa/2.0/"
            | "http://creativecommons.org/licenses/by-nc-sa/2.5/"
            | "http://creativecommons.org/licenses/by-nc-sa/3.0/"
            | "http://creativecommons.org/licenses/by-nc-sa/4.0/" => License::ByNcSa,
            "http://creativecommons.org/licenses/by-nc-nd/1.0/"
            | "http://creativecommons.org/licenses/by-nc-nd/2.0/"
            | "http://creativecommons.org/licenses/by-nc-nd/2.5/"
            | "http://creativecommons.org/licenses/by-nc-nd/3.0/"
            | "http://creativecommons.org/licenses/by-nc-nd/4.0/" => License::ByNcNd,
            "https://creativecommons.org/publicdomain/zero/1.0/" => License::Zero,
            _other => License::Undefined,
        };
        Ok(license)
    }
}


impl Default for Work {
    fn default() -> Work {
        Work {
            work_id: "".to_string(),
            work_type: WorkType::Monograph,
            work_status: WorkStatus::Inactive,
            full_title: "".to_string(),
            title: "".to_string(),
            subtitle: None,
            reference: None,
            edition: 1,
            doi: None,
            publication_date: None,
            place: None,
            width: None,
            height: None,
            page_count: None,
            page_breakdown: None,
            image_count: None,
            table_count: None,
            audio_count: None,
            video_count: None,
            license: None,
            copyright_holder: "".to_string(),
            landing_page: None,
            lccn: None,
            oclc: None,
            short_abstract: None,
            long_abstract: None,
            general_note: None,
            toc: None,
            cover_url: None,
            cover_caption: None,
            contributions: None,
            imprint: Default::default(),
        }
    }
}

impl Default for Imprint {
    fn default() -> Imprint {
        Imprint {
            imprint_id: "".to_string(),
            imprint_name: "".to_string(),
            imprint_url: None,
            publisher: Default::default(),
        }
    }
}

impl Default for Publisher {
    fn default() -> Publisher {
        Publisher {
            publisher_id: "".to_string(),
            publisher_name: "".to_string(),
            publisher_shortname: None,
            publisher_url: None,
        }
    }
}
