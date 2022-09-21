use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::contribution::Contribution;
use thoth_api::model::contribution::ContributionType;
use yew::prelude::html;
use yew::Html;

use super::ListString;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContributionTypeDefinition {
    pub enum_values: Vec<ContributionTypeValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContributionTypeValues {
    pub name: ContributionType,
}

impl ListString for Contribution {
    fn separated_list_item(&self, is_small: bool, separator: &str) -> Html {
        // Only include contributions marked as "Main" in summary list
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
                        <span>{ separator }</span>
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
pub mod update_contribution_mutation;
