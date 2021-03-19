use chrono::naive::NaiveDateTime;
use serde::Deserialize;
use serde::Serialize;
use thoth_api::series::model::SeriesType;
use yew::html;
use yew::prelude::Html;
use yew::Callback;
use yew::MouseEvent;

use super::imprint::Imprint;
use crate::route::AdminRoute;
use crate::route::AppRoute;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub series_id: String,
    pub series_type: SeriesType,
    pub series_name: String,
    pub issn_print: String,
    pub issn_digital: String,
    pub series_url: Option<String>,
    pub updated_at: serde_json::Value,
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
            series_id: "".to_string(),
            series_type: SeriesType::BookSeries,
            series_name: "".to_string(),
            issn_print: "".to_string(),
            issn_digital: "".to_string(),
            series_url: None,
            updated_at: Default::default(),
            imprint: Default::default(),
        }
    }
}

impl Series {
    pub fn create_route() -> AppRoute {
        AppRoute::Admin(AdminRoute::NewSeries)
    }

    pub fn as_dropdown_item(&self, callback: Callback<MouseEvent>) -> Html {
        // since serieses dropdown has an onblur event, we need to use onmousedown instead of
        // onclick. This is not ideal, but it seems to be the only event that'd do the calback
        // without disabling onblur so that onclick can take effect
        html! {
            <div onmousedown=callback class="dropdown-item">
                { format!("{} ({}, {})", self.series_name, self.issn_print, self.issn_digital) }
            </div>
        }
    }

    pub fn edit_route(&self) -> AppRoute {
        AppRoute::Admin(AdminRoute::Series(self.series_id.clone()))
    }

    pub fn as_table_row(&self, callback: Callback<MouseEvent>) -> Html {
        let updated =
            NaiveDateTime::from_timestamp(self.updated_at.as_f64().unwrap_or(0.0) as i64, 0);
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
                <td>{updated}</td>
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
