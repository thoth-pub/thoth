use thoth_api::model::contributor::Contributor;
use yew::html;
use yew::prelude::Html;
use yew::Callback;
use yew::MouseEvent;

use super::{CreateRoute, Dropdown, EditRoute, MetadataTable};
use crate::route::AdminRoute;
use crate::route::AppRoute;

impl Dropdown for Contributor {}

impl CreateRoute for Contributor {
    fn create_route() -> AppRoute {
        AppRoute::Admin(AdminRoute::NewContributor)
    }
}

impl EditRoute for Contributor {
    fn edit_route(&self) -> AppRoute {
        AppRoute::Admin(AdminRoute::Contributor(self.contributor_id))
    }
}

impl MetadataTable for Contributor {
    fn as_table_row(&self, callback: Callback<MouseEvent>) -> Html {
        let orcid = self
            .orcid
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "".to_string());
        html! {
            <tr
                class="row"
                onclick=callback
            >
                <td>{&self.contributor_id}</td>
                <td>{&self.full_name}</td>
                <td>{orcid}</td>
                <td>{&self.updated_at}</td>
            </tr>
        }
    }
}

pub mod contributor_activity_query;
pub mod contributor_query;
pub mod contributors_query;
pub mod create_contributor_mutation;
pub mod delete_contributor_mutation;
pub mod update_contributor_mutation;
