use serde::Deserialize;
use serde::Serialize;
use yew::html;
use yew::prelude::Html;
use yew::Callback;
use yew::MouseEvent;

use crate::route::AdminRoute;
use crate::route::AppRoute;

use super::Direction;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contributor {
    pub contributor_id: String,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub orcid: Option<String>,
    pub website: Option<String>,
}

impl Contributor {
    pub fn create_route() -> AppRoute {
        AppRoute::Admin(AdminRoute::NewContributor)
    }

    pub fn as_dropdown_item(&self, callback: Callback<MouseEvent>) -> Html {
        // since contributions dropdown has an onblur event, we need to use onmousedown instead of
        // onclick. This is not ideal, but it seems to be the only event that'd do the callback
        // without disabling onblur so that onclick can take effect
        html! {
            <div onmousedown=callback class="dropdown-item">
            {
              if let Some(orcid) = &self.orcid {
                  format!("{} - {}", &self.full_name, orcid)
                } else {
                  format!("{}", &self.full_name )
                }
            }
            </div>
        }
    }

    pub fn edit_route(&self) -> AppRoute {
        AppRoute::Admin(AdminRoute::Contributor(self.contributor_id.clone()))
    }

    pub fn as_table_row(&self, callback: Callback<MouseEvent>) -> Html {
        let orcid = self.orcid.clone().unwrap_or_else(|| "".to_string());
        html! {
            <tr
                class="row"
                onclick=callback
            >
                <td>{&self.contributor_id}</td>
                <td>{&self.full_name}</td>
                <td>{orcid}</td>
            </tr>
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContributorField {
    #[serde(rename = "CONTRIBUTOR_ID")]
    ContributorID,
    FirstName,
    LastName,
    FullName,
    #[serde(rename = "ORCID")]
    ORCID,
    Website,
    CreatedAt,
    UpdatedAt,
}

impl From<String> for ContributorField {
    fn from(input: String) -> Self {
        match input.as_ref() {
            // Only match the headers which are currently defined/sortable in the UI
            "ID" => ContributorField::ContributorID,
            "FullName" => ContributorField::FullName,
            "ORCID" => ContributorField::ORCID,
            // Default to full name (although ideally we'd default to Null)
            _ => ContributorField::FullName,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContributorOrderBy {
    pub field: ContributorField,
    pub direction: Direction,
}

impl From<String> for ContributorOrderBy {
    fn from(input: String) -> Self {
        ContributorOrderBy {
            field: input.into(),
            direction: Direction::ASC,
        }
    }
}

pub mod contributor_activity_query;
pub mod contributor_query;
pub mod contributors_query;
pub mod create_contributor_mutation;
pub mod delete_contributor_mutation;
pub mod update_contributor_mutation;
