use serde::Deserialize;
use serde::Serialize;
use yew::html;
use yew::prelude::Html;
use yew::Callback;
use yew::MouseEvent;

use crate::route::AdminRoute;
use crate::route::AppRoute;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contributor {
    pub contributor_id: String,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub orcid: Option<String>,
    pub website: Option<String>,
    pub contributions: Option<Vec<SlimContribution>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlimContribution {
    pub work: SlimWork,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlimWork {
    pub title: String,
    pub imprint: SlimImprint,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlimImprint {
    pub publisher: SlimPublisher,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlimPublisher {
    pub publisher_name: String,
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

pub mod contributor_query;
pub mod contributors_query;
pub mod create_contributor_mutation;
pub mod delete_contributor_mutation;
pub mod update_contributor_mutation;
