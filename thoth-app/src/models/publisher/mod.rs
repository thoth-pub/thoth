use thoth_api::model::publisher::Publisher;
use yew::html;
use yew::prelude::Html;
use yew::Callback;
use yew::MouseEvent;

use crate::route::AdminRoute;

use super::{CreateRoute, EditRoute, MetadataTable};

impl CreateRoute for Publisher {
    fn create_route() -> AdminRoute {
        AdminRoute::NewPublisher
    }
}

impl EditRoute for Publisher {
    fn edit_route(&self) -> AdminRoute {
        AdminRoute::Publisher {
            id: self.publisher_id,
        }
    }
}

impl MetadataTable for Publisher {
    fn as_table_row(&self, callback: Callback<MouseEvent>) -> Html {
        let publisher_shortname = self.publisher_shortname.clone().unwrap_or_default();
        let publisher_url = self.publisher_url.clone().unwrap_or_default();
        html! {
            <tr
                class="row"
                onclick={ callback }
            >
                <td>{&self.publisher_id}</td>
                <td>{&self.publisher_name}</td>
                <td>{publisher_shortname}</td>
                <td>{publisher_url}</td>
                <td>{&self.updated_at}</td>
            </tr>
        }
    }
}

pub mod create_publisher_mutation;
pub mod delete_publisher_mutation;
pub mod publisher_query;
pub mod publishers_query;
pub mod update_publisher_mutation;
