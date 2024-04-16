use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::series::Series;
use thoth_api::model::series::SeriesType;
use thoth_api::model::series::SeriesWithImprint;
use yew::html;
use yew::prelude::Html;
use yew::Callback;
use yew::MouseEvent;

use super::{CreateRoute, Dropdown, EditRoute, MetadataTable};
use crate::route::AdminRoute;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SeriesTypeDefinition {
    pub enum_values: Vec<SeriesTypeValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SeriesTypeValues {
    pub name: SeriesType,
}

impl EditRoute for Series {
    fn edit_route(&self) -> AdminRoute {
        AdminRoute::Series { id: self.series_id }
    }
}

impl Dropdown for SeriesWithImprint {}

impl CreateRoute for SeriesWithImprint {
    fn create_route() -> AdminRoute {
        AdminRoute::NewSeries
    }
}

impl EditRoute for SeriesWithImprint {
    fn edit_route(&self) -> AdminRoute {
        AdminRoute::Series { id: self.series_id }
    }
}

impl MetadataTable for SeriesWithImprint {
    fn as_table_row(&self, callback: Callback<MouseEvent>) -> Html {
        html! {
            <tr
                class="row"
                onclick={ callback }
            >
                <td>{&self.series_id}</td>
                <td>{&self.series_name}</td>
                <td>{&self.series_type}</td>
                <td>{&self.issn_print.as_ref().unwrap_or(&String::default())}</td>
                <td>{&self.issn_digital.as_ref().unwrap_or(&String::default())}</td>
                <td>{&self.updated_at}</td>
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
