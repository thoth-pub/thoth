use std::io;

use actix_cors::Cors;
use actix_web::{middleware::Logger, App, Error, HttpServer};
use paperclip::actix::{
    api_v2_operation,
    web::{self, HttpResponse, Json},
    Apiv2Schema, OpenApiExt,
};
use paperclip::v2::models::{DefaultApiRaw, Info, Tag, Contact, License};
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

struct ApiConfig {
    graphql_endpoint: String,
}

#[derive(Serialize, Deserialize, Apiv2Schema)]
struct Format {
    id: String,
    name: String,
    version: String,
}

#[derive(Serialize, Deserialize, Apiv2Schema)]
struct Platform {
    id: String,
    name: String,
}

#[derive(Serialize, Deserialize, Apiv2Schema)]
struct Output {
    id: String,
    name: String,
    format_url: String,
    platform_url: String,
}

fn all_formats() -> Vec<Format> {
    vec![Format {
        id: "onix_3.0".to_string(),
        name: "ONIX".to_string(),
        version: "3.0".to_string(),
    }]
}

fn all_platforms() -> Vec<Platform> {
    vec![Platform {
        id: "project_muse".to_string(),
        name: "Project MUSE".to_string(),
    }]
}

fn all_outputs() -> Vec<Output> {
    vec![Output {
        id: "onix_3.0::project_muse".to_string(),
        name: "Project MUSE ONIX 3.0".to_string(),
        format_url: "https://localhost:8181/formats/onix_3.0".to_string(),
        platform_url: "https://localhost:8181/platforms/project_muse".to_string(),
    }]
}

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
async fn formats() -> Result<Json<Vec<Format>>, ()> {
    Ok(Json(all_formats()))
}

#[api_v2_operation(
summary = "Get a single format",
description = "Find the details of a format that can be output by Thoth",
tags(Formats)
)]
async fn format(format_id: web::Path<String>) -> Result<Json<Format>, ()> {
    // TODO: NotFound error
    let all_formats = all_formats();
    let id = format_id.into_inner();
    all_formats.into_iter().find(|f| f.id == id).map(|f| Json(f)).ok_or(())
}

#[api_v2_operation(
    summary = "List supported platforms",
    description = "Full list of platforms supported by Thoth's outputs",
    tags(Platforms)
)]
async fn platforms() -> Result<Json<Vec<Platform>>, ()> {
    Ok(Json(all_platforms()))
}

#[api_v2_operation(
summary = "Get a single platform",
description = "Find the details of a platform supported by Thoth's outpus",
tags(Platforms)
)]
async fn platform(platform_id: web::Path<String>) -> Result<Json<Platform>, ()> {
    // TODO: NotFound error
    let all_platforms = all_platforms();
    let id = platform_id.into_inner();
    all_platforms.into_iter().find(|p| p.id == id).map(|p| Json(p)).ok_or(())
}

#[api_v2_operation(
    summary = "List supported outputs",
    description = "Full list of metadata standards that can be output by Thoth",
    tags(Outputs)
)]
async fn outputs() -> Result<Json<Vec<Output>>, ()> {
    Ok(Json(all_outputs()))
}

#[api_v2_operation(
    summary = "Get a single output format",
    description = "Find the details of a metadata output given its ID",
    tags(Outputs)
)]
async fn output(output_id: web::Path<String>) -> Result<Json<Output>, ()> {
    // TODO: NotFound error
    let all_outputs = all_outputs();
    let id = output_id.into_inner();
    all_outputs.into_iter().find(|o| o.id == id).map(|o| Json(o)).ok_or(())
}

#[api_v2_operation(
    summary = "Get a metadata output record",
    description = "Obtain a metadata output record, given its ID, for a given work",
    produces = "text/xml",
    tags(Outputs)
)]
async fn output_by_work(
    _output_id: web::Path<String>,
    work_id: web::Path<Uuid>,
    config: web::Data<ApiConfig>,
) -> Result<Xml<String>, Error> {
    get_work(work_id.into_inner(), &config.graphql_endpoint)
        .await
        .and_then(generate_onix_3)
        .and_then(|onix| {
            String::from_utf8(onix)
                .map_err(|_| ThothError::InternalError("Could not generate ONIX".to_string()))
        })
        .map(Xml)
        .map_err(|e| e.into())
}

#[api_v2_operation(
    summary = "Get ONIX file",
    description = "Obtain an ONIX 3.0 file for a given work_id",
    produces = "text/xml",
    tags(Outputs)
)]
async fn onix_endpoint(
    work_id: web::Path<Uuid>,
    config: web::Data<ApiConfig>,
) -> Result<Xml<String>, Error> {
    get_work(work_id.into_inner(), &config.graphql_endpoint)
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
                    name: "Outputs".to_string(),
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
                    url: None
                }),
                ..Default::default()
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
            .service(web::resource("/outputs").route(web::get().to(outputs)))
            .service(web::resource("/outputs/{output_id}").route(web::get().to(output)))
            .service(web::resource("/outputs/{output_id}/work/{work_id}").route(web::get().to(output_by_work)))
            .service(web::resource("/onix/{work_id}").route(web::get().to(onix_endpoint)))
            .with_json_spec_at("/swagger.json")
            .build()
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
