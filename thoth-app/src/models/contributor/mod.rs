use serde::Deserialize;
use serde::Serialize;
use yew::html;
use yew::Callback;
use yew::MouseEvent;
use yew::prelude::Html;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contributor {
    pub contributor_id: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub full_name: String,
    pub orcid: Option<String>,
    pub website: Option<String>,
}

impl Default for Contributor {
    fn default() -> Contributor {
        Contributor {
            contributor_id: "".to_string(),
            first_name: None,
            last_name: None,
            full_name: "".to_string(),
            orcid: None,
            website: None,
        }
    }
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
