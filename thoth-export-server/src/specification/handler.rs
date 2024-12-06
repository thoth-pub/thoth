use super::model::Specification;
use crate::data::{find_specification, ALL_SPECIFICATIONS};
use crate::record::{MetadataRecord, MetadataSpecification};
use crate::specification_query::SpecificationQuery;
use actix_web::Error;
use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
};
use thoth_api::redis::RedisPool;
use thoth_client::ThothClient;
use uuid::Uuid;

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
    redis_pool: web::Data<RedisPool>,
    thoth_client: web::Data<ThothClient>,
) -> Result<MetadataRecord, Error> {
    let thoth = thoth_client.into_inner();
    let (specification_id, work_id) = path.into_inner();
    let specification: MetadataSpecification = specification_id.parse()?;

    let last_updated = thoth.get_work_last_updated(work_id).await?;
    let specification_query = SpecificationQuery::by_work(thoth, work_id, specification);

    let mut metadata_record = MetadataRecord::new(work_id.to_string(), specification, last_updated);
    metadata_record
        .load_or_generate(specification_query, redis_pool.into_inner())
        .await?;
    Ok(metadata_record)
}

#[api_v2_operation(
    summary = "Get a publisher's metadata record",
    description = "Obtain a metadata record that adheres to a particular specification for all of a given publisher's works",
    produces = "text/xml, text/csv, text/plain, application/x-bibtex",
    tags(Specifications)
)]
pub(crate) async fn by_publisher(
    path: web::Path<(String, Uuid)>,
    redis_pool: web::Data<RedisPool>,
    thoth_client: web::Data<ThothClient>,
) -> Result<MetadataRecord, Error> {
    let thoth = thoth_client.into_inner();
    let (specification_id, publisher_id) = path.into_inner();
    let specification: MetadataSpecification = specification_id.parse()?;

    let last_updated = thoth
        .get_works_last_updated(Some(vec![publisher_id]))
        .await?;
    let specification_query = SpecificationQuery::by_publisher(thoth, publisher_id, specification);

    let mut metadata_record =
        MetadataRecord::new(publisher_id.to_string(), specification, last_updated);
    metadata_record
        .load_or_generate(specification_query, redis_pool.into_inner())
        .await?;
    Ok(metadata_record)
}
