use serde::{Deserialize, Serialize};
use thoth_api::model::institution::CountryCode;
use thoth_api::model::institution::Institution;
use yew::html;
use yew::prelude::Html;
use yew::Callback;
use yew::MouseEvent;

use super::{CreateRoute, Dropdown, EditRoute, MetadataTable};
use crate::route::AdminRoute;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CountryCodeDefinition {
    pub enum_values: Vec<CountryCodeValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CountryCodeValues {
    pub name: CountryCode,
}

impl Dropdown for Institution {}

impl CreateRoute for Institution {
    fn create_route() -> AdminRoute {
        AdminRoute::NewInstitution
    }
}

impl EditRoute for Institution {
    fn edit_route(&self) -> AdminRoute {
        AdminRoute::Institution {
            id: self.institution_id,
        }
    }
}

impl MetadataTable for Institution {
    fn as_table_row(&self, callback: Callback<MouseEvent>) -> Html {
        let institution_doi = self
            .institution_doi
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_default();
        let ror = self.ror.as_ref().map(|s| s.to_string()).unwrap_or_default();
        let country_code = self
            .country_code
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_default();
        html! {
            <tr
                class="row"
                onclick={ callback }
            >
                <td>{&self.institution_id}</td>
                <td>{&self.institution_name}</td>
                <td>{institution_doi}</td>
                <td>{ror}</td>
                <td>{country_code}</td>
                <td>{&self.updated_at}</td>
            </tr>
        }
    }
}

pub mod country_codes_query;
pub mod create_institution_mutation;
pub mod delete_institution_mutation;
pub mod institution_activity_query;
pub mod institution_query;
pub mod institutions_query;
pub mod update_institution_mutation;
