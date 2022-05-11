use actix_web::Error;
use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
};
use thoth_client::{ThothClient, Work};
use thoth_errors::ThothError;
use uuid::Uuid;

use super::model::Specification;
use crate::data::{find_specification, ALL_SPECIFICATIONS};
use crate::record::MetadataRecord;

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
    find_specification(specification_id)
        .map(Json)
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
    thoth_client: web::Data<ThothClient>,
) -> Result<MetadataRecord<Vec<Work>>, Error> {
    thoth_client
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
    summary = "Get a publisher's metadata record",
    description = "Obtain a metadata record that adheres to a particular specification for all of a given publisher's works",
    produces = "text/xml, text/csv",
    tags(Specifications)
)]
pub(crate) async fn by_publisher(
    web::Path((specification_id, publisher_id)): web::Path<(String, Uuid)>,
    thoth_client: web::Data<ThothClient>,
) -> Result<MetadataRecord<Vec<Work>>, Error> {
    if specification_id.eq("doideposit::crossref") {
        // Full publisher record is not supported for this specification
        return Err(ThothError::IncompleteMetadataRecord(
            "doideposit::crossref".to_string(),
            "Output can only be generated for one work at a time".to_string(),
        )
        .into());
    }
    thoth_client
        .get_works(Some(vec![publisher_id]))
        .await
        .and_then(|data| {
            specification_id.parse().map(|specification| {
                MetadataRecord::new(publisher_id.to_string(), specification, data)
            })
        })
        .map_err(|e| e.into())
}
