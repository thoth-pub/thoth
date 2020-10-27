use serde::Deserialize;
use serde::Serialize;
use yew::html;
use yew::prelude::Html;
use yew::Callback;
use yew::MouseEvent;

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
    pub fn as_dropdown_item(&self, callback: Callback<MouseEvent>) -> Html {
        // since contributions dropdown has an onblur event, we need to use onmousedown i  nstead of
        // onclick. This is not ideal, but it seems to be the only event that'd do the ca  lback
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
}

pub mod contributor_query;
pub mod contributors_query;
pub mod create_contributor_mutation;
pub mod update_contributor_mutation;
