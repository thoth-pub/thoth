use serde::Deserialize;
use serde::Serialize;
use thoth_api::series::model::SeriesExtended as Series;
use thoth_api::series::model::SeriesType;
use yew::html;
use yew::prelude::Html;
use yew::Callback;
use yew::MouseEvent;

use super::{CreateRoute, Dropdown, EditRoute, MetadataTable};
use crate::route::AdminRoute;
use crate::route::AppRoute;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SeriesTypeDefinition {
    pub enum_values: Vec<SeriesTypeValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SeriesTypeValues {
    pub name: SeriesType,
}

impl Dropdown for Series {}

impl CreateRoute for Series {
    fn create_route() -> AppRoute {
        AppRoute::Admin(AdminRoute::NewSeries)
    }
}

impl EditRoute for Series {
    fn edit_route(&self) -> AppRoute {
        AppRoute::Admin(AdminRoute::Series(self.series_id))
    }
}

impl MetadataTable for Series {
    fn as_table_row(&self, callback: Callback<MouseEvent>) -> Html {
        html! {
            <tr
                class="row"
                onclick=callback
            >
                <td>{&self.series_id}</td>
                <td>{&self.series_name}</td>
                <td>{&self.series_type}</td>
                <td>{&self.issn_print}</td>
                <td>{&self.issn_digital}</td>
                <td>{&self.updated_at.format("%F %T")}</td>
            </tr>
        }
    }
}

pub mod create_series_mutation;
pub mod delete_series_mutation;
pub mod series_query;
pub mod series_types_query;
pub mod serieses_query;
pub mod update_series_mutation;
