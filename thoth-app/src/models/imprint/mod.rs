use serde::Deserialize;
use serde::Serialize;
use yew::html;
use yew::prelude::Html;
use yew::Callback;
use yew::MouseEvent;

use crate::route::AdminRoute;
use crate::route::AppRoute;
use super::publisher::Publisher;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Imprint {
    pub imprint_id: String,
    pub imprint_name: String,
    pub imprint_url: Option<String>,
    pub publisher: Publisher,
}

impl Imprint {
    pub fn create_route() -> AppRoute {
        AppRoute::Admin(AdminRoute::NewImprint)
    }

    pub fn edit_route(&self) -> AppRoute {
        AppRoute::Admin(AdminRoute::Imprint(self.imprint_id.clone()))
    }

    pub fn as_table_row(&self, callback: Callback<MouseEvent>) -> Html {
        let imprint_url = self.imprint_url.clone().unwrap_or_else(|| "".to_string());
        html! {
            <tr
                class="row"
                onclick=callback
            >
                <td>{&self.imprint_id}</td>
                <td>{&self.imprint_name}</td>
                <td>{&self.publisher.publisher_name}</td>
                <td>{imprint_url}</td>
            </tr>
        }
    }
}

impl Default for Imprint {
    fn default() -> Imprint {
        Imprint {
            imprint_id: "".to_string(),
            imprint_name: "".to_string(),
            imprint_url: None,
            publisher: Default::default(),
        }
    }
}

pub mod create_imprint_mutation;
pub mod delete_imprint_mutation;
pub mod imprint_query;
pub mod imprints_query;
pub mod update_imprint_mutation;
