use chrono::naive::NaiveDateTime;
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
pub struct Funder {
    pub funder_id: String,
    pub funder_name: String,
    pub funder_doi: Option<String>,
    pub updated_at: serde_json::Value,
}

impl Funder {
    pub fn create_route() -> AppRoute {
        AppRoute::Admin(AdminRoute::NewFunder)
    }

    pub fn as_dropdown_item(&self, callback: Callback<MouseEvent>) -> Html {
        // since funders dropdown has an onblur event, we need to use onmousedown instead of
        // onclick. This is not ideal, but it seems to be the only event that'd do the calback
        // without disabling onblur so that onclick can take effect
        html! {
            <div onmousedown=callback class="dropdown-item">
            {
                if let Some(doi) = &self.funder_doi {
                    format!("{} - {}", &self.funder_name, doi)
                } else {
                    format!("{}", &self.funder_name )
                }
            }
            </div>
        }
    }

    pub fn edit_route(&self) -> AppRoute {
        AppRoute::Admin(AdminRoute::Funder(self.funder_id.clone()))
    }

    pub fn as_table_row(&self, callback: Callback<MouseEvent>) -> Html {
        let funder_doi = self.funder_doi.clone().unwrap_or_else(|| "".to_string());
        let updated =
            NaiveDateTime::from_timestamp(self.updated_at.as_f64().unwrap_or(0.0) as i64, 0);
        html! {
            <tr
                class="row"
                onclick=callback
            >
                <td>{&self.funder_id}</td>
                <td>{&self.funder_name}</td>
                <td>{funder_doi}</td>
                <td>{updated}</td>
            </tr>
        }
    }
}

pub mod create_funder_mutation;
pub mod delete_funder_mutation;
pub mod funder_activity_query;
pub mod funder_query;
pub mod funders_query;
pub mod update_funder_mutation;
