use serde::Deserialize;
use serde::Serialize;
use thoth_api::publication::model::Publication;
use thoth_api::publication::model::DetailedPublication;
use thoth_api::publication::model::PublicationExtended;
use thoth_api::publication::model::PublicationType;
use yew::html;
use yew::prelude::Html;
use yew::Callback;
use yew::MouseEvent;

use super::{CreateRoute, EditRoute, MetadataTable};
use crate::route::AdminRoute;
use crate::route::AppRoute;

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

impl EditRoute for Publication {
    fn edit_route(&self) -> AppRoute {
        AppRoute::Admin(AdminRoute::Publication(self.publication_id))
    }
}

impl EditRoute for PublicationExtended {
    fn edit_route(&self) -> AppRoute {
        AppRoute::Admin(AdminRoute::Publication(self.publication_id))
    }
}

impl MetadataTable for DetailedPublication {
    fn as_table_row(&self, callback: Callback<MouseEvent>) -> Html {
        let isbn = &self.isbn.clone().unwrap_or_else(|| "".to_string());
        let doi = self
            .work
            .doi
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "".to_string());
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
                <td>{&self.updated_at}</td>
            </tr>
        }
    }
}

impl CreateRoute for DetailedPublication {
    fn create_route() -> AppRoute {
        AppRoute::Admin(AdminRoute::NewPublication)
    }
}

impl EditRoute for DetailedPublication {
    fn edit_route(&self) -> AppRoute {
        AppRoute::Admin(AdminRoute::Publication(self.publication_id))
    }
}

pub mod create_publication_mutation;
pub mod delete_publication_mutation;
pub mod publication_query;
pub mod publication_types_query;
pub mod publications_query;
