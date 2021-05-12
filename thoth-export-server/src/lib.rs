use std::io;

use actix_cors::Cors;
use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer};
use thoth_client::work::get_work;
use uuid::Uuid;

mod onix;
use crate::onix::generate_onix_3;

struct ApiConfig {
    graphql_endpoint: String,
}

#[get("/onix/{uuid}")]
async fn onix_endpoint(path: web::Path<(Uuid,)>, config: web::Data<ApiConfig>) -> HttpResponse {
    let work_id = (path.0).0;

    if let Ok(work) = get_work(work_id, &config.graphql_endpoint).await {
        if let Ok(body) = generate_onix_3(work) {
            HttpResponse::Ok()
                .header(
                    "Content-Disposition",
                    format!("attachment; filename=\"{}.xml\"", work_id),
                )
                .content_type("text/xml; charset=utf-8")
                .body(String::from_utf8(body).unwrap())
        } else {
            HttpResponse::InternalServerError()
                .body(format!("Could not generate ONIX for: {}", work_id))
        }
    } else {
        HttpResponse::NotFound().body(format!("Not found: {}", work_id))
    }
}

#[actix_rt::main]
pub async fn start_server(host: String, port: String, gql_endpoint: String) -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("Setting Thoth GraphQL endpoint to {}", gql_endpoint);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(
                Cors::new()
                    .allowed_methods(vec!["GET", "OPTIONS"])
                    .finish(),
            )
            .data(ApiConfig {
                graphql_endpoint: gql_endpoint.clone(),
            })
            .service(onix_endpoint)
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
