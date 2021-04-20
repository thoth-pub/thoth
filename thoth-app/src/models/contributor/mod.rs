use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;
use yew::html;
use yew::prelude::Html;
use yew::Callback;
use yew::MouseEvent;

use crate::route::AdminRoute;
use crate::route::AppRoute;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contributor {
    pub contributor_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub orcid: Option<String>,
    pub website: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
                { self.as_formatted_string() }
            </div>
        }
    }

    pub fn edit_route(&self) -> AppRoute {
        AppRoute::Admin(AdminRoute::Contributor(self.contributor_id))
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
                <td>{&self.updated_at.format("%F %T")}</td>
            </tr>
        }
    }

    pub fn as_formatted_string(&self) -> String {
        if let Some(orcid) = &self.orcid {
            format!("{} - {}", &self.full_name, orcid)
        } else {
            self.full_name.clone()
        }
    }
}

impl Default for Contributor {
    fn default() -> Contributor {
        Contributor {
            contributor_id: Default::default(),
            first_name: None,
            last_name: "".to_string(),
            full_name: "".to_string(),
            orcid: None,
            website: None,
            created_at: chrono::TimeZone::timestamp(&Utc, 0, 0),
            updated_at: chrono::TimeZone::timestamp(&Utc, 0, 0),
        }
    }
}

pub mod contributor_activity_query;
pub mod contributor_query;
pub mod contributors_query;
pub mod create_contributor_mutation;
pub mod delete_contributor_mutation;
pub mod update_contributor_mutation;
