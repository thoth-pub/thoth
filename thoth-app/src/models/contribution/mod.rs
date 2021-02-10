use serde::Deserialize;
use serde::Serialize;
use thoth_api::contribution::model::ContributionType;
use yew::prelude::html;
use yew::Html;

use super::contributor::Contributor;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contribution {
    pub work_id: String,
    pub contributor_id: String,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    pub biography: Option<String>,
    pub institution: Option<String>,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub contributor: Contributor,
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

pub mod contribution_types_query;
pub mod create_contribution_mutation;
pub mod delete_contribution_mutation;
