use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use thoth_api::series::model::SeriesType;
use uuid::Uuid;
use yew::html;
use yew::prelude::Html;
use yew::Callback;
use yew::MouseEvent;

use super::imprint::Imprint;
use super::Dropdown;
use super::MetadataObject;
use crate::route::AdminRoute;
use crate::route::AppRoute;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub series_id: Uuid,
    pub series_type: SeriesType,
    pub series_name: String,
    pub issn_print: String,
    pub issn_digital: String,
    pub series_url: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub imprint: Imprint,
}

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

impl Default for Series {
    fn default() -> Series {
        Series {
            series_id: Default::default(),
            series_type: Default::default(),
            series_name: "".to_string(),
            issn_print: "".to_string(),
            issn_digital: "".to_string(),
            series_url: None,
            updated_at: chrono::TimeZone::timestamp(&Utc, 0, 0),
            imprint: Default::default(),
        }
    }
}

impl fmt::Display for Series {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} ({}, {})",
            self.series_name, self.issn_print, self.issn_digital
        )
    }
}

impl Dropdown for Series {}

impl MetadataObject for Series {
    fn create_route() -> AppRoute {
        AppRoute::Admin(AdminRoute::NewSeries)
    }

    fn edit_route(&self) -> AppRoute {
        AppRoute::Admin(AdminRoute::Series(self.series_id))
    }

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
