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
mod xml;

use crate::onix::generate_onix_3;
use crate::rapidoc::rapidoc_source;
use crate::xml::Xml;
use actix_web::error::ErrorNotFound;

struct ApiConfig {
    graphql_endpoint: String,
}

#[derive(Clone, Serialize, Deserialize, Apiv2Schema)]
struct Format<'a> {
    id: &'a str,
    name: &'a str,
    version: &'a str,
}

#[derive(Clone, Serialize, Deserialize, Apiv2Schema)]
struct Platform<'a> {
    id: &'a str,
    name: &'a str,
}

#[derive(Clone, Serialize, Deserialize, Apiv2Schema)]
struct Specification<'a> {
    id: &'a str,
    name: &'a str,
}

const FORMATS: [Format<'static>; 1] = [Format {
    id: "onix_3.0",
    name: "ONIX",
    version: "3.0",
}];

const PLATFORMS: [Platform<'static>; 1] = [Platform {
    id: "project_muse",
    name: "Project MUSE",
}];

const SPECIFICATIONS: [Specification<'static>; 1] = [Specification {
    id: "onix_3.0::project_muse",
    name: "Project MUSE ONIX 3.0",
}];

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
async fn formats() -> Result<Json<[Format<'static>; 1]>, ()> {
    Ok(Json(FORMATS))
}

#[api_v2_operation(
    summary = "Describe a metadata format",
    description = "Find the details of a format that can be output by Thoth",
    tags(Formats)
)]
async fn format(web::Path(format_id): web::Path<String>) -> Result<Json<Format<'static>>, Error> {
    FORMATS
        .iter()
        .find(|f| f.id == format_id)
        .map(|f| Json(f.clone()))
        .ok_or_else(|| ErrorNotFound("Format not found"))
}

#[api_v2_operation(
    summary = "List supported platforms",
    description = "Full list of platforms supported by Thoth's outputs",
    tags(Platforms)
)]
async fn platforms() -> Result<Json<[Platform<'static>; 1]>, ()> {
    Ok(Json(PLATFORMS))
}

#[api_v2_operation(
    summary = "Describe a platform",
    description = "Find the details of a platform supported by Thoth's outputs",
    tags(Platforms)
)]
async fn platform(
    web::Path(platform_id): web::Path<String>,
) -> Result<Json<Platform<'static>>, Error> {
    PLATFORMS
        .iter()
        .find(|p| p.id == platform_id)
        .map(|p| Json(p.clone()))
        .ok_or_else(|| ErrorNotFound("Platform not found"))
}

#[api_v2_operation(
    summary = "List supported specifications",
    description = "Full list of metadata specifications that can be output by Thoth",
    tags(Specifications)
)]
async fn specifications() -> Result<Json<[Specification<'static>; 1]>, ()> {
    Ok(Json(SPECIFICATIONS))
}

#[api_v2_operation(
    summary = "Describe a metadata specification",
    description = "Find the details of a metadata specification that can be output by Thoth",
    tags(Specifications)
)]
async fn specification(
    web::Path(specification_id): web::Path<String>,
) -> Result<Json<Specification<'static>>, Error> {
    SPECIFICATIONS
        .iter()
        .find(|s| s.id == specification_id)
        .map(|s| Json(s.clone()))
        .ok_or_else(|| ErrorNotFound("Specification not found"))
}

#[api_v2_operation(
    summary = "Get a work's metadata record",
    description = "Obtain a metadata record that adheres to a particular specification for a given work",
    produces = "text/xml",
    tags(Specifications)
)]
async fn specification_by_work(
    web::Path((_specification_id, work_id)): web::Path<(String, Uuid)>,
    config: web::Data<ApiConfig>,
) -> Result<Xml<String>, Error> {
    get_work(work_id, &config.graphql_endpoint)
        .await
        .and_then(generate_onix_3)
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
