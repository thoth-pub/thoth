use std::io;
use std::time::Duration;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web::Data, App, HttpServer};
pub use data::ALL_SPECIFICATIONS;
use paperclip::actix::{web, web::HttpResponse, OpenApiExt};
use paperclip::v2::models::{Contact, DefaultApiRaw, Info, License, OperationProtocol, Tag};
use thoth_api::redis::init_pool;
use thoth_client::ThothClient;

mod bibtex;
mod csv;
mod data;
mod format;
mod json;
mod marc21;
mod platform;
mod rapidoc;
mod record;
mod specification;
mod specification_query;
mod xml;

use crate::rapidoc::rapidoc_source;

const LOG_FORMAT: &str = r#"%{r}a %a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#;

struct ApiConfig {
    api_schema: String,
}

impl ApiConfig {
    pub fn new(public_url: String) -> Self {
        Self {
            api_schema: format!("{public_url}/openapi.json"),
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
    redis_url: String,
    host: String,
    port: String,
    threads: usize,
    keep_alive: u64,
    public_url: String,
    gql_endpoint: String,
) -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("Setting Thoth GraphQL endpoint to {}", gql_endpoint);

    HttpServer::new(move || {
        // extract hostname and protocol from public URL
        let (protocol, host) = public_url
            .strip_prefix("https://")
            .map(|stripped| (OperationProtocol::Https, stripped))
            .or_else(|| {
                public_url
                    .strip_prefix("http://")
                    .map(|stripped| (OperationProtocol::Http, stripped))
            })
            .unwrap_or((OperationProtocol::Http, public_url.as_str()));

        let spec = DefaultApiRaw {
            host: Some(host.to_string()),
            schemes: [protocol].into_iter().collect(),
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
                terms_of_service: Some("https://thoth.pub/policies/terms-thoth-free".to_string()),
                contact: Some(Contact {
                    name: Some("Thoth Support".to_string()),
                    url: Some("https://thoth.pub".to_string()),
                    email: Some("support@thoth.pub".to_string()),
                }),
                license: Some(License {
                    name: Some(env!("CARGO_PKG_LICENSE").parse().unwrap()),
                    url: None,
                }),
                extensions: Default::default(),
            },
            ..Default::default()
        };

        App::new()
            .wrap(Logger::new(LOG_FORMAT))
            .wrap(Cors::default().allowed_methods(vec!["GET", "OPTIONS"]))
            .app_data(Data::new(ThothClient::new(gql_endpoint.clone())))
            .app_data(Data::new(ApiConfig::new(public_url.clone())))
            .app_data(Data::new(init_pool(&redis_url)))
            .service(actix_web::web::resource("/").route(actix_web::web::get().to(index)))
            .wrap_api_with_spec(spec)
            .configure(format::route)
            .configure(platform::route)
            .configure(specification::route)
            .with_json_spec_at("/swagger.json")
            .with_json_spec_v3_at("/openapi.json")
            .build()
    })
    .workers(threads)
    .keep_alive(Duration::from_secs(keep_alive))
    .bind(format!("{host}:{port}"))?
    .run()
    .await
}
