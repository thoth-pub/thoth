use serde::Deserialize;
use serde::Serialize;
use thoth_api::publication::model::PublicationType;
use yew::html;
use yew::prelude::Html;
use yew::Callback;
use yew::MouseEvent;

use super::price::Price;
use crate::route::AdminRoute;
use crate::route::AppRoute;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Publication {
    pub publication_id: String,
    pub publication_type: PublicationType,
    pub work_id: String,
    pub isbn: Option<String>,
    pub publication_url: Option<String>,
    pub prices: Option<Vec<Price>>,
    pub work: SlimWork,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlimWork {
    pub imprint: SlimImprint,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlimImprint {
    pub publisher: SlimPublisher,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlimPublisher {
    pub publisher_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PublicationTypeDefinition {
    pub enum_values: Vec<PublicationTypeValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PublicationTypeValues {
    pub name: PublicationType,
}

impl crate::models::publication::publications_query::DetailedPublication {
    pub fn create_route() -> AppRoute {
        AppRoute::Admin(AdminRoute::NewPublication)
    }

    pub fn edit_route(&self) -> AppRoute {
        AppRoute::Admin(AdminRoute::Publication(self.publication_id.clone()))
    }

    pub fn as_table_row(&self, callback: Callback<MouseEvent>) -> Html {
        let isbn = &self.isbn.clone().unwrap_or_else(|| "".to_string());
        let doi = &self.work.doi.clone().unwrap_or_else(|| "".to_string());
        let publication_url = &self
            .publication_url
            .clone()
            .unwrap_or_else(|| "".to_string());
        html! {
            <tr
                class="row"
                onclick=callback
            >
                <td>{&self.publication_id}</td>
                <td>{&self.work.title}</td>
                <td>{doi}</td>
                <td>{&self.work.publisher()}</td>
                <td>{&self.publication_type}</td>
                <td>{isbn}</td>
                <td>{publication_url}</td>
                <td>{&self.updated_at.format("%F %T")}</td>
            </tr>
        }
    }
}

impl Default for Publication {
    fn default() -> Publication {
        Publication {
            publication_id: "".to_string(),
            publication_type: PublicationType::Paperback,
            work_id: "".to_string(),
            isbn: None,
            publication_url: None,
            prices: Default::default(),
            work: Default::default(),
        }
    }
}

pub mod create_publication_mutation;
pub mod delete_publication_mutation;
pub mod publication_query;
pub mod publication_types_query;
pub mod publications_query;
