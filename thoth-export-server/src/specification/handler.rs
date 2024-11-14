use super::model::Specification;
use crate::data::{find_specification, ALL_SPECIFICATIONS};
use crate::record::{MetadataRecord, MetadataSpecification};
use crate::specification_query::SpecificationQuery;
use actix_web::Error;
use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
};
use thoth_api::model::Timestamp;
use thoth_api::redis::{get, set, RedisPool};
use thoth_client::{ThothClient, Work};
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
) -> Result<MetadataRecord<Vec<Work>>, Error> {
    let thoth = thoth_client.into_inner();
    let (specification_id, work_id) = path.into_inner();
    let specification: MetadataSpecification = specification_id.parse()?;

    let cache_key = format!("{}:{}", specification_id, work_id);
    let cache_timestamp_key = format!("{}:timestamp", cache_key);
    let last_updated = thoth.get_work_last_updated(work_id).await?;

    let cached_timestamp_raw = get(&redis_pool, &cache_timestamp_key).await;
    if let Ok(cached_timestamp_value) = cached_timestamp_raw {
        let cached_timestamp = Timestamp::parse_from_rfc3339(&cached_timestamp_value)?;
        if cached_timestamp >= last_updated {
            let cached_record_raw = get(&redis_pool, &cache_key).await;
            if let Ok(cached_record) = cached_record_raw {
                return Ok(MetadataRecord::cached(
                    work_id.to_string(),
                    specification,
                    vec![],
                    cached_record,
                ));
            }
        }
    }

    let data = SpecificationQuery::new(thoth, specification)
        .by_work(work_id)
        .await?;
    let mut metadata_record = MetadataRecord::new(work_id.to_string(), specification, vec![data]);
    metadata_record.generate();
    if metadata_record.is_ok() {
        set(&redis_pool, &cache_key, metadata_record.record()).await?;
        set(
            &redis_pool,
            &cache_timestamp_key,
            &last_updated.to_rfc3339(),
        )
        .await?;
    }
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
