use serde::Deserialize;
use serde::Serialize;
use yew::html;
use yew::Callback;
use yew::MouseEvent;
use yew::prelude::Html;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Funder {
    pub funder_id: String,
    pub funder_name: String,
    pub funder_doi: Option<String>,
}

impl Default for Funder {
    fn default() -> Funder {
        Funder {
            funder_id: "".to_string(),
            funder_name: "".to_string(),
            funder_doi: None,
        }
    }
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

pub mod funders_query;
