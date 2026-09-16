use super::model::Specification;
use crate::data::{find_specification, ALL_SPECIFICATIONS};
use crate::record::{MetadataRecord, MetadataSpecification};
use crate::specification_query::SpecificationQuery;
use actix_web::{Error, HttpResponse};
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

/// Why a prepared-deposit request was refused before any query.
#[derive(Debug, PartialEq, Eq)]
enum PreparedRefusal {
    /// The released "not a valid metadata specification" refusal.
    Specification(String),
    /// The path's timestamp is not a 17-digit encoding of an instant.
    Timestamp,
}

/// The stable body of the prepared route's HTTP 400 (R52B section 15.3).
const TIMESTAMP_NOT_DECODABLE: &str = "CROSSREF_TIMESTAMP_NOT_DECODABLE";
/// The released XML content type of every XML metadata record.
const PREPARED_XML_MIME_TYPE: &str = "text/xml; charset=utf-8";

/// Validate a prepared-deposit request, before any query: only
/// `doideposit::crossref`, and only an exact 17-digit timestamp.
fn validate_prepared_request(
    specification_id: &str,
    crossref_timestamp: &str,
) -> Result<(), PreparedRefusal> {
    match specification_id.parse::<MetadataSpecification>() {
        Ok(MetadataSpecification::DoiDepositCrossref(_)) => {}
        _ => return Err(PreparedRefusal::Specification(specification_id.to_string())),
    }
    if !crate::xml::DoiDepositCrossref::is_valid_timestamp(crossref_timestamp) {
        return Err(PreparedRefusal::Timestamp);
    }
    Ok(())
}

#[api_v2_operation(
    summary = "Get a work's prepared Crossref deposit",
    description = "Generate, never from cache, the Crossref deposit for a given work with the supplied 17-digit deposit timestamp allocated by a Crossref write reservation",
    produces = "text/xml",
    tags(Specifications)
)]
pub(crate) async fn by_work_prepared(
    path: web::Path<(String, Uuid, String)>,
    thoth_client: web::Data<ThothClient>,
) -> Result<HttpResponse, Error> {
    let (specification_id, work_id, crossref_timestamp) = path.into_inner();
    match validate_prepared_request(&specification_id, &crossref_timestamp) {
        Ok(()) => {}
        Err(PreparedRefusal::Specification(specification_id)) => {
            return Err(
                thoth_errors::ThothError::InvalidMetadataSpecification(specification_id).into(),
            )
        }
        Err(PreparedRefusal::Timestamp) => {
            return Ok(HttpResponse::BadRequest()
                .content_type("text/plain; charset=utf-8")
                .body(TIMESTAMP_NOT_DECODABLE))
        }
    }
    let specification =
        MetadataSpecification::DoiDepositCrossref(crate::xml::DoiDepositCrossref {});
    let works = SpecificationQuery::by_work(thoth_client.into_inner(), work_id, specification)
        .run()
        .await?;
    let deposit =
        crate::xml::DoiDepositCrossref {}.generate_prepared(&works, &crossref_timestamp)?;
    Ok(HttpResponse::Ok()
        .content_type(PREPARED_XML_MIME_TYPE)
        .body(deposit))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t111_a_prepared_request_is_validated_before_any_query() {
        assert_eq!(
            validate_prepared_request("doideposit::crossref", "20260904120000123"),
            Ok(())
        );
        for invalid in [
            "2026090412000012",
            "202609041200001234",
            "20260904120060000",
            "20260904240000000",
            "20250229120000000",
            "02026090412000000",
            "abc",
        ] {
            assert_eq!(
                validate_prepared_request("doideposit::crossref", invalid),
                Err(PreparedRefusal::Timestamp),
                "{invalid}"
            );
        }
        for specification in ["onix_3.0::thoth", "csv::thoth", "not-a-specification"] {
            assert_eq!(
                validate_prepared_request(specification, "20260904120000123"),
                Err(PreparedRefusal::Specification(specification.to_string())),
                "{specification}"
            );
        }
    }

    #[test]
    fn t109_the_prepared_route_never_touches_the_export_cache() {
        let source = include_str!("handler.rs");
        let body = source
            .split_once("pub(crate) async fn by_work_prepared(")
            .expect("handler")
            .1
            .split_once("\n}\n")
            .expect("end")
            .0;
        for forbidden in [
            "redis",
            "Redis",
            "MetadataRecord",
            "get_work_last_updated",
            "cache_key",
            "load_or_generate",
        ] {
            assert!(!body.contains(forbidden), "{forbidden}");
        }
        assert!(body.contains("generate_prepared("));
        assert!(
            body.find("validate_prepared_request(").expect("validation")
                < body.find(".run()").expect("query")
        );
    }
}
