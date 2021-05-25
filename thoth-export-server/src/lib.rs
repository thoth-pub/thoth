use std::io;

use actix_cors::Cors;
use actix_web::{middleware::Logger, App, Error, HttpServer};
use paperclip::actix::{
    api_v2_operation,
    web::{self, HttpResponse, Json},
    Apiv2Schema, OpenApiExt,
};
use paperclip::v2::models::{Contact, DefaultApiRaw, Info, License, Tag};
use serde::{Deserialize, Serialize};
use thoth_api::errors::ThothError;
use thoth_client::work::get_work;
use uuid::Uuid;

mod onix;
mod rapidoc;
mod record;
mod xml;

use crate::rapidoc::rapidoc_source;
use crate::xml::Xml;
use crate::record::MetadataRecord;

struct ApiConfig {
    graphql_endpoint: String,
}

#[derive(Clone, Serialize, Deserialize, Apiv2Schema)]
struct Format<'a> {
    id: &'a str,
    name: &'a str,
    version: Option<&'a str>,
}

#[derive(Clone, Serialize, Deserialize, Apiv2Schema)]
struct Platform<'a> {
    id: &'a str,
    name: &'a str,
}

#[derive(Clone, Serialize, Deserialize, Apiv2Schema)]
pub(crate) struct Specification<'a> {
    id: SpecificationId,
    name: &'a str,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Apiv2Schema)]
pub enum SpecificationId {
    #[serde(rename = "onix_3.0::project_muse")]
    Onix3ProjectMuse,
    #[serde(rename = "csv::thoth")]
    CsvThoth,
}

const ALL_FORMATS: [Format<'static>; 2] = [
    Format {
        id: "onix_3.0",
        name: "ONIX",
        version: Some("3.0"),
    },
    Format {
        id: "csv",
        name: "CSV",
        version: None,
    }
];

const ALL_PLATFORMS: [Platform<'static>; 2] = [
    Platform {
        id: "thoth",
        name: "Thoth",
    },
    Platform {
        id: "project_muse",
        name: "Project MUSE",
    },
];

const ALL_SPECIFICATIONS: [Specification<'static>; 2] = [
    Specification {
        id: SpecificationId::Onix3ProjectMuse,
        name: "Project MUSE ONIX 3.0",
    },
    Specification {
        id: SpecificationId::CsvThoth,
        name: "Thoth CSV",
    },
];

async fn index() -> HttpResponse {
    let html = rapidoc_source("/swagger.json");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[api_v2_operation(
    summary = "List supported formats",
    description = "Full list of metadata formats that can be output by Thoth",
    tags(Formats)
)]
async fn formats() -> Json<[Format<'static>; 2]> {
    Json(ALL_FORMATS)
}

#[api_v2_operation(
    summary = "Describe a metadata format",
    description = "Find the details of a format that can be output by Thoth",
    tags(Formats)
)]
async fn format(web::Path(format_id): web::Path<String>) -> Result<Json<Format<'static>>, Error> {
    ALL_FORMATS
        .iter()
        .find(|f| f.id == format_id)
        .map(|f| Json(f.clone()))
        .ok_or(ThothError::EntityNotFound)
        .map_err(|e| e.into())
}

#[api_v2_operation(
    summary = "List supported platforms",
    description = "Full list of platforms supported by Thoth's outputs",
    tags(Platforms)
)]
async fn platforms() -> Json<[Platform<'static>; 2]> {
    Json(ALL_PLATFORMS)
}

#[api_v2_operation(
    summary = "Describe a platform",
    description = "Find the details of a platform supported by Thoth's outputs",
    tags(Platforms)
)]
async fn platform(
    web::Path(platform_id): web::Path<String>,
) -> Result<Json<Platform<'static>>, Error> {
    ALL_PLATFORMS
        .iter()
        .find(|p| p.id == platform_id)
        .map(|p| Json(p.clone()))
        .ok_or(ThothError::EntityNotFound)
        .map_err(|e| e.into())
}

#[api_v2_operation(
    summary = "List supported specifications",
    description = "Full list of metadata specifications that can be output by Thoth",
    tags(Specifications)
)]
async fn specifications() -> Json<[Specification<'static>; 2]> {
    Json(ALL_SPECIFICATIONS)
}

#[api_v2_operation(
    summary = "Describe a metadata specification",
    description = "Find the details of a metadata specification that can be output by Thoth",
    tags(Specifications)
)]
async fn specification(
    web::Path(specification_id): web::Path<SpecificationId>,
) -> Result<Json<Specification<'static>>, Error> {
    ALL_SPECIFICATIONS
        .iter()
        .find(|s| s.id == specification_id)
        .map(|s| Json(s.clone()))
        .ok_or(ThothError::EntityNotFound)
        .map_err(|e| e.into())
}

#[api_v2_operation(
    summary = "Get a work's metadata record",
    description = "Obtain a metadata record that adheres to a particular specification for a given work",
    produces = "text/xml",
    tags(Specifications)
)]
async fn specification_by_work(
    web::Path((specification_id, work_id)): web::Path<(SpecificationId, Uuid)>,
    config: web::Data<ApiConfig>,
) -> Result<Xml<String>, Error> {
    let specification = ALL_SPECIFICATIONS
        .iter()
        .find(|s| s.id == specification_id)
        .ok_or(ThothError::EntityNotFound)
        .unwrap();
    let data = get_work(work_id, &config.graphql_endpoint).await?;
    MetadataRecord::new(specification, data)
        .generate()
        .and_then(|onix| {
            String::from_utf8(onix)
                .map_err(|_| ThothError::InternalError("Could not generate ONIX".to_string()))
        })
        .map(Xml)
        .map_err(|e| e.into())
}

#[actix_web::main]
pub async fn start_server(host: String, port: String, gql_endpoint: String) -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("Setting Thoth GraphQL endpoint to {}", gql_endpoint);

    HttpServer::new(move || {
        let spec = DefaultApiRaw {
            // TODO get host and path from input
            host: Some("api.thoth.pub".to_string()),
            base_path: Some("/export".to_string()),
            tags: vec![
                Tag {
                    name: "Formats".to_string(),
                    description: None,
                    external_docs: None,
                },
                Tag {
                    name: "Specifications".to_string(),
                    description: None,
                    external_docs: None,
                },
                Tag {
                    name: "Platforms".to_string(),
                    description: None,
                    external_docs: None,
                },
            ],
            info: Info {
                version: env!("CARGO_PKG_VERSION").parse().unwrap(),
                title: "Thoth Metadata Export API".to_string(),
                description: Some(
                    "Obtain Thoth metadata records in various formats and platform specifications"
                        .to_string(),
                ),
                contact: Some(Contact {
                    name: Some("Thoth Support".to_string()),
                    url: Some("https://thoth.pub".to_string()),
                    email: Some("support@thoth.pub".to_string()),
                }),
                license: Some(License {
                    name: Some(env!("CARGO_PKG_LICENSE").parse().unwrap()),
                    url: None,
                }),
            },
            ..Default::default()
        };

        App::new()
            .wrap(Logger::default())
            .wrap(Cors::default().allowed_methods(vec!["GET", "OPTIONS"]))
            .data(ApiConfig {
                graphql_endpoint: gql_endpoint.clone(),
            })
            .service(actix_web::web::resource("/").route(actix_web::web::get().to(index)))
            .wrap_api_with_spec(spec)
            .service(web::resource("/formats").route(web::get().to(formats)))
            .service(web::resource("/formats/{format_id}").route(web::get().to(format)))
            .service(web::resource("/platforms").route(web::get().to(platforms)))
            .service(web::resource("/platforms/{platform_id}").route(web::get().to(platform)))
            .service(web::resource("/specifications").route(web::get().to(specifications)))
            .service(
                web::resource("/specifications/{specification_id}")
                    .route(web::get().to(specification)),
            )
            .service(
                web::resource("/specifications/{specification_id}/work/{work_id}")
                    .route(web::get().to(specification_by_work)),
            )
            .with_json_spec_at("/swagger.json")
            .build()
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
