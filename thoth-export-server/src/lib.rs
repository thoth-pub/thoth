use std::io;

use actix_cors::Cors;
use actix_web::{middleware::Logger, App, HttpServer};
use paperclip::actix::{web, web::HttpResponse, OpenApiExt};
use paperclip::v2::models::{Contact, DefaultApiRaw, Info, License, Tag};
use thoth_client::ThothClient;

mod csv;
mod data;
mod format;
mod platform;
mod rapidoc;
mod record;
mod specification;
mod xml;

use crate::rapidoc::rapidoc_source;

struct ApiConfig {
    api_schema: String,
}

impl ApiConfig {
    pub fn new(public_url: String) -> Self {
        Self {
            api_schema: format!("{}/swagger.json", public_url),
        }
    }
}

async fn index(config: web::Data<ApiConfig>) -> HttpResponse {
    let html = rapidoc_source(&config.api_schema);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[actix_web::main]
pub async fn start_server(
    host: String,
    port: String,
    public_url: String,
    gql_endpoint: String,
) -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("Setting Thoth GraphQL endpoint to {}", gql_endpoint);

    HttpServer::new(move || {
        let spec = DefaultApiRaw {
            host: Some(public_url.clone()),
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
            .data(ThothClient::new(gql_endpoint.clone()))
            .data(ApiConfig::new(public_url.clone()))
            .service(actix_web::web::resource("/").route(actix_web::web::get().to(index)))
            .wrap_api_with_spec(spec)
            .configure(format::route)
            .configure(platform::route)
            .configure(specification::route)
            .with_json_spec_at("/swagger.json")
            .build()
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
