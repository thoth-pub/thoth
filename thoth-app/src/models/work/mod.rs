use std::str::FromStr;
use std::string::ParseError;
use serde::Deserialize;
use serde::Serialize;
use thoth_api::work::model::WorkStatus;
use thoth_api::work::model::WorkType;
use yew::html;
use yew::prelude::Html;
use yew::Callback;
use yew::MouseEvent;

use super::contribution::Contribution;
use super::funding::Funding;
use super::imprint::Imprint;
use super::issue::Issue;
use super::language::Language;
use super::publication::Publication;
use super::subject::Subject;
use crate::route::AdminRoute;
use crate::route::AppRoute;

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
    pub license: Option<String>,
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
    pub publications: Option<Vec<Publication>>,
    pub languages: Option<Vec<Language>>,
    pub fundings: Option<Vec<Funding>>,
    pub subjects: Option<Vec<Subject>>,
    pub issues: Option<Vec<Issue>>,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkTypeDefinition {
    pub enum_values: Vec<WorkTypeValues>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkStatusDefinition {
    pub enum_values: Vec<WorkStatusValues>,
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

impl Work {
    pub fn compile_fulltitle(&self) -> String {
        if let Some(subtitle) = &self.subtitle.clone() {
            format!("{}: {}", self.title, subtitle)
        } else {
            self.title.to_string()
        }
    }

    pub fn publisher(&self) -> String {
        if let Some(short_name) = &self.imprint.publisher.publisher_shortname.clone() {
            short_name.to_string()
        } else {
            self.imprint.publisher.publisher_name.to_string()
        }
    }

    pub fn edit_route(&self) -> AppRoute {
        AppRoute::Admin(AdminRoute::Work(self.work_id.clone()))
    }

    pub fn as_table_row(&self, callback: Callback<MouseEvent>) -> Html {
        let doi = self.doi.clone().unwrap_or_else(|| "".to_string());
        html! {
            <tr
                class="row"
                onclick=callback
            >
                <td>{&self.work_id}</td>
                <td>{&self.title}</td>
                <td>{&self.work_type}</td>
                <td>
                    {
                        if let Some(contributions) = &self.contributions {
                            contributions.iter().map(|c| c.main_contribution_item()).collect::<Html>()
                        } else {
                            html! {}
                        }
                    }
                </td>
                <td>{doi}</td>
                <td>{&self.publisher()}</td>
            </tr>
        }
    }
}

impl FromStr for License {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<License, ParseError> {
        let license = match input {
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
            publications: None,
            languages: None,
            fundings: None,
            subjects: None,
            issues: None,
            imprint: Default::default(),
        }
    }
}

pub mod create_work_mutation;
pub mod delete_work_mutation;
pub mod update_work_mutation;
pub mod work_statuses_query;
pub mod work_types_query;
pub mod work_query;
pub mod works_query;
