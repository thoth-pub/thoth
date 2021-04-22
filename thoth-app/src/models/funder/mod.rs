use thoth_api::funder::model::Funder;
use yew::html;
use yew::prelude::Html;
use yew::Callback;
use yew::MouseEvent;

use super::Dropdown;
use super::MetadataObject;
use crate::route::AdminRoute;
use crate::route::AppRoute;

impl Dropdown for Funder {}

impl MetadataObject for Funder {
    fn create_route() -> AppRoute {
        AppRoute::Admin(AdminRoute::NewFunder)
    }

    fn edit_route(&self) -> AppRoute {
        AppRoute::Admin(AdminRoute::Funder(self.funder_id))
    }

    fn as_table_row(&self, callback: Callback<MouseEvent>) -> Html {
        let funder_doi = self.funder_doi.clone().unwrap_or_else(|| "".to_string());
        html! {
            <tr
                class="row"
                onclick=callback
            >
                <td>{&self.funder_id}</td>
                <td>{&self.funder_name}</td>
                <td>{funder_doi}</td>
                <td>{&self.updated_at.format("%F %T")}</td>
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
