use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use thoth_api::contribution::model::ContributionType;
use uuid::Uuid;
use yew::prelude::html;
use yew::Html;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contribution {
    pub work_id: Uuid,
    pub contributor_id: Uuid,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    pub biography: Option<String>,
    pub institution: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContributionTypeDefinition {
    pub enum_values: Vec<ContributionTypeValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContributionTypeValues {
    pub name: ContributionType,
}

const BULLET_SEPARATOR: &str = " • ";
const COMMA_SEPARATOR: &str = ", ";

impl Contribution {
    pub fn main_contribution_item_bullet_small(&self) -> Html {
        self.main_contribution_item(true, BULLET_SEPARATOR)
    }

    pub fn main_contribution_item_comma(&self) -> Html {
        self.main_contribution_item(false, COMMA_SEPARATOR)
    }

    fn main_contribution_item(&self, is_small: bool, separator: &str) -> Html {
        if self.main_contribution {
            if is_small {
                html! {
                    <small class="contributor">
                        {&self.full_name}
                        <span>{ separator }</span>
                    </small>
                }
            } else {
                html! {
                    <span class="contributor">
                        {&self.full_name}
                        <span>{ ", " }</span>
                    </span>
                }
            }
        } else {
            html! {}
        }
    }
}

impl Default for Contribution {
    fn default() -> Contribution {
        Contribution {
            work_id: Default::default(),
            contributor_id: Default::default(),
            contribution_type: Default::default(),
            main_contribution: Default::default(),
            biography: None,
            institution: None,
            created_at: chrono::TimeZone::timestamp(&Utc, 0, 0),
            updated_at: chrono::TimeZone::timestamp(&Utc, 0, 0),
            first_name: None,
            last_name: Default::default(),
            full_name: Default::default(),
        }
    }
}

pub mod contribution_types_query;
pub mod create_contribution_mutation;
pub mod delete_contribution_mutation;
