use actix_web::Error;
use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
};
use thoth_api::errors::ThothError;
use thoth_client::{ThothClient, Work};
use uuid::Uuid;

use super::model::Specification;
use crate::data::ALL_SPECIFICATIONS;
use crate::record::MetadataRecord;
use crate::ApiConfig;

#[api_v2_operation(
    summary = "List supported specifications",
    description = "Full list of metadata specifications that can be output by Thoth",
    tags(Specifications)
)]
pub(crate) async fn get_all() -> Json<Vec<Specification<'static>>> {
    Json(ALL_SPECIFICATIONS.clone())
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
    produces = "text/xml, text/csv",
    tags(Specifications)
)]
pub(crate) async fn by_work(
    web::Path((specification_id, work_id)): web::Path<(String, Uuid)>,
    config: web::Data<ApiConfig>,
) -> Result<MetadataRecord<Vec<Work>>, Error> {
    ThothClient::new(&config.graphql_endpoint)
        .get_work(work_id)
        .await
        .and_then(|data| {
            specification_id.parse().map(|specification| {
                MetadataRecord::new(work_id.to_string(), specification, vec![data])
            })
        })
        .map_err(|e| e.into())
}

#[api_v2_operation(
    summary = "Get a work's metadata record",
    description = "Obtain a metadata record that adheres to a particular specification for a given work",
    produces = "text/xml, text/csv",
    tags(Specifications)
)]
pub(crate) async fn by_publisher(
    web::Path((specification_id, publisher_id)): web::Path<(String, Uuid)>,
    config: web::Data<ApiConfig>,
) -> Result<MetadataRecord<Vec<Work>>, Error> {
    ThothClient::new(&config.graphql_endpoint)
        .get_works(Some(vec![publisher_id]))
        .await
        .and_then(|data| {
            specification_id.parse().map(|specification| {
                MetadataRecord::new(publisher_id.to_string(), specification, data)
            })
        })
        .map_err(|e| e.into())
}
