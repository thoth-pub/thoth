use serde::Deserialize;
use serde::Serialize;
use yew::html;
use yew::prelude::Html;
use yew::Callback;
use yew::MouseEvent;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Funder {
    pub funder_id: String,
    pub funder_name: String,
    pub funder_doi: Option<String>,
}

impl Funder {
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
}

pub mod create_funder_mutation;
pub mod delete_funder_mutation;
pub mod funder_query;
pub mod funders_query;
pub mod update_funder_mutation;
