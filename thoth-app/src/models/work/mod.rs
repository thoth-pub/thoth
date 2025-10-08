use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::work::Work;
use thoth_api::model::work::WorkStatus;
use thoth_api::model::work::WorkType;
use thoth_api::model::work::WorkWithRelations;
use thoth_api::model::LengthUnit;
use yew::html;
use yew::prelude::Html;
use yew::Callback;
use yew::MouseEvent;

use super::{CreateRoute, Dropdown, EditRoute, ListString, MetadataTable};
use crate::route::AdminRoute;

#[allow(dead_code)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LengthUnitDefinition {
    pub enum_values: Vec<LengthUnitValues>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkTypeDefinition {
    pub enum_values: Vec<WorkTypeValues>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkStatusDefinition {
    pub enum_values: Vec<WorkStatusValues>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LengthUnitValues {
    pub name: LengthUnit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkTypeValues {
    pub name: WorkType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkStatusValues {
    pub name: WorkStatus,
}

impl Dropdown for Work {}

impl EditRoute for Work {
    fn edit_route(&self) -> AdminRoute {
        AdminRoute::Work { id: self.work_id }
    }
}

impl CreateRoute for WorkWithRelations {
    fn create_route() -> AdminRoute {
        AdminRoute::NewWork
    }
}

impl EditRoute for WorkWithRelations {
    fn edit_route(&self) -> AdminRoute {
        AdminRoute::Work { id: self.work_id }
    }
}

impl MetadataTable for WorkWithRelations {
    fn as_table_row(&self, callback: Callback<MouseEvent>) -> Html {
        let doi = self.doi.as_ref().map(|s| s.to_string()).unwrap_or_default();
        html! {
            <tr
                class="row"
                onclick={ callback }
            >
                <td>{&self.work_id}</td>
                <td>{&self.title}</td>
                <td>{&self.work_type}</td>
                <td>
                    {
                        if let Some(contributions) = &self.contributions {
                            contributions.iter().map(|c| c.separated_list_item_comma()).collect::<Html>()
                        } else {
                            html! {}
                        }
                    }
                </td>
                <td>{doi}</td>
                <td>{&self.publisher()}</td>
                <td>{&self.updated_at}</td>
            </tr>
        }
    }
}

pub mod create_work_mutation;
pub mod delete_work_mutation;
pub mod slim_works_query;
pub mod slim_works_with_relations_query;
pub mod update_work_mutation;
pub mod work_query;
pub mod work_statuses_query;
pub mod work_types_query;
pub mod works_query;
