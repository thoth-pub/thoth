use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::publication::Publication;
use thoth_api::model::publication::PublicationType;
use thoth_api::model::publication::PublicationWithRelations;
use yew::html;
use yew::prelude::Html;
use yew::Callback;
use yew::MouseEvent;

use super::{CreateRoute, EditRoute, MetadataTable};
use crate::route::AdminRoute;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PublicationTypeDefinition {
    pub enum_values: Vec<PublicationTypeValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PublicationTypeValues {
    pub name: PublicationType,
}

impl EditRoute for Publication {
    fn edit_route(&self) -> AdminRoute {
        AdminRoute::Publication {
            id: self.publication_id,
        }
    }
}

impl EditRoute for PublicationWithRelations {
    fn edit_route(&self) -> AdminRoute {
        AdminRoute::Publication {
            id: self.publication_id,
        }
    }
}

impl MetadataTable for PublicationWithRelations {
    fn as_table_row(&self, callback: Callback<MouseEvent>) -> Html {
        let isbn = self
            .isbn
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_default();
        let doi = self
            .work
            .doi
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_default();
        html! {
            <tr
                class="row"
                onclick={ callback }
            >
                <td>{&self.publication_id}</td>
                <td>{&self.work.full_title}</td>
                <td>{doi}</td>
                <td>{&self.work.publisher()}</td>
                <td>{&self.publication_type}</td>
                <td>{isbn}</td>
                <td>{&self.updated_at}</td>
            </tr>
        }
    }
}

impl CreateRoute for PublicationWithRelations {
    fn create_route() -> AdminRoute {
        AdminRoute::NewPublication
    }
}

pub mod create_publication_mutation;
pub mod delete_publication_mutation;
pub mod publication_query;
pub mod publication_types_query;
pub mod publications_query;
pub mod update_publication_mutation;
