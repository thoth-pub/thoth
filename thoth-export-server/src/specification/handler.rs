use actix_web::Error;
use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
};
use thoth_api::errors::ThothError;

use super::model::Specification;
use crate::record::MetadataRecord;
use crate::ApiConfig;
use thoth_client::work::{get_work, work_query::WorkQueryWork};
use uuid::Uuid;

const ALL_SPECIFICATIONS: [Specification<'static>; 2] = [
    Specification {
        id: "onix_3.0::project_muse",
        name: "Project MUSE ONIX 3.0",
    },
    Specification {
        id: "csv::thoth",
        name: "Thoth CSV",
    },
];

#[api_v2_operation(
    summary = "List supported specifications",
    description = "Full list of metadata specifications that can be output by Thoth",
    tags(Specifications)
)]
pub(crate) async fn get_all() -> Json<[Specification<'static>; 2]> {
    Json(ALL_SPECIFICATIONS)
}

#[api_v2_operation(
    summary = "Describe a metadata specification",
    description = "Find the details of a metadata specification that can be output by Thoth",
    tags(Specifications)
)]
pub(crate) async fn get_one(
    web::Path(specification_id): web::Path<String>,
) -> Result<Json<Specification<'static>>, Error> {
    ALL_SPECIFICATIONS
        .iter()
        .find(|s| s.id == specification_id)
        .map(|s| Json(s.clone()))
        .ok_or(ThothError::InvalidMetadataSpecification(specification_id))
        .map_err(|e| e.into())
}

#[api_v2_operation(
    summary = "Get a work's metadata record",
    description = "Obtain a metadata record that adheres to a particular specification for a given work",
    produces = "text/xml",
    tags(Specifications)
)]
pub(crate) async fn by_work(
    web::Path((specification_id, work_id)): web::Path<(String, Uuid)>,
    config: web::Data<ApiConfig>,
) -> Result<MetadataRecord<WorkQueryWork>, Error> {
    match specification_id.parse() {
        Ok(specification) => get_work(work_id, &config.graphql_endpoint)
            .await
            .map(|data| MetadataRecord::new(specification, data))
            .map_err(|e| e.into()),
        Err(e) => Err(e.into()),
    }
}
