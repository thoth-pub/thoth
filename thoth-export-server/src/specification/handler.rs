use actix_web::Error;
use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
};
use thoth_client::{ThothClient, Work};
use uuid::Uuid;

use super::model::Specification;
use crate::data::{find_specification, ALL_SPECIFICATIONS};
use crate::record::{MetadataRecord, MetadataSpecification};
use crate::specification_query::SpecificationQuery;

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
    specification_id: web::Path<String>,
) -> Result<Json<Specification<'static>>, Error> {
    find_specification(specification_id.into_inner())
        .map(Json)
        .map_err(|e| e.into())
}

#[api_v2_operation(
    summary = "Get a work's metadata record",
    description = "Obtain a metadata record that adheres to a particular specification for a given work",
    produces = "text/xml, text/csv, text/plain, application/x-bibtex, application/json",
    tags(Specifications)
)]
pub(crate) async fn by_work(
    path: web::Path<(String, Uuid)>,
    thoth_client: web::Data<ThothClient>,
) -> Result<MetadataRecord<Vec<Work>>, Error> {
    let (specification_id, work_id) = path.into_inner();
    let specification: MetadataSpecification = specification_id.parse()?;

    SpecificationQuery::new(thoth_client.into_inner(), specification)
        .by_work(work_id)
        .await
        .map(|data| MetadataRecord::new(work_id.to_string(), specification, vec![data]))
        .map_err(|e| e.into())
}

#[api_v2_operation(
    summary = "Get a publisher's metadata record",
    description = "Obtain a metadata record that adheres to a particular specification for all of a given publisher's works",
    produces = "text/xml, text/csv, text/plain, application/x-bibtex",
    tags(Specifications)
)]
pub(crate) async fn by_publisher(
    path: web::Path<(String, Uuid)>,
    thoth_client: web::Data<ThothClient>,
) -> Result<MetadataRecord<Vec<Work>>, Error> {
    let (specification_id, publisher_id) = path.into_inner();
    let specification: MetadataSpecification = specification_id.parse()?;

    SpecificationQuery::new(thoth_client.into_inner(), specification)
        .by_publisher(publisher_id)
        .await
        .map(|data| MetadataRecord::new(publisher_id.to_string(), specification, data))
        .map_err(|e| e.into())
}
