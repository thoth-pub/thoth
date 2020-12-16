use serde::Deserialize;
use serde::Serialize;
use yew::html;
use yew::prelude::Html;
use yew::Callback;
use yew::MouseEvent;

use crate::route::AdminRoute;
use crate::route::AppRoute;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Publisher {
    pub publisher_id: String,
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
}

impl Publisher {
    pub fn create_route() -> AppRoute {
        AppRoute::Admin(AdminRoute::NewPublisher)
    }

    pub fn edit_route(&self) -> AppRoute {
        AppRoute::Admin(AdminRoute::Publisher(self.publisher_id.clone()))
    }

    pub fn as_table_row(&self, callback: Callback<MouseEvent>) -> Html {
        let publisher_shortname = self
            .publisher_shortname
            .clone()
            .unwrap_or_else(|| "".to_string());
        let publisher_url = self.publisher_url.clone().unwrap_or_else(|| "".to_string());
        html! {
            <tr
                class="row"
                onclick=callback
            >
                <td>{&self.publisher_id}</td>
                <td>{&self.publisher_name}</td>
                <td>{publisher_shortname}</td>
                <td>{publisher_url}</td>
            </tr>
        }
    }
}

impl Default for Publisher {
    fn default() -> Publisher {
        Publisher {
            publisher_id: "".to_string(),
            publisher_name: "".to_string(),
            publisher_shortname: None,
            publisher_url: None,
        }
    }
}

pub mod create_publisher_mutation;
pub mod delete_publisher_mutation;
pub mod publisher_query;
pub mod publishers_query;
pub mod update_publisher_mutation;
