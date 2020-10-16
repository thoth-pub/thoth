use std::env;
use std::io;
use std::sync::Arc;

use actix_cors::Cors;
use actix_identity::CookieIdentityPolicy;
use actix_identity::IdentityService;
use actix_web::middleware::Logger;
use actix_web::{web, error, App, Error, HttpRequest, HttpResponse, HttpServer, Result};
use dotenv::dotenv;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use thoth_api::db::Context;
use thoth_api::db::establish_connection;
use thoth_api::graphql::model::{create_schema, Schema};
use thoth_api::account::model::LoginCredentials;
use thoth_api::account::service::login;
use thoth_client::work::get_work;
use uuid::Uuid;

use crate::onix::generate_onix_3;

#[get("/graphiql")]
async fn graphiql() -> HttpResponse {
    let html = graphiql_source("/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[post("/graphql")]
async fn graphql(
    st: web::Data<Arc<Schema>>,
    ctx: web::Data<Context>,
    data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, Error> {
    let result = web::block(move || {
        let res = data.execute(&st, &ctx);
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .await?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(result))
}

#[get("/onix/{uuid}")]
async fn onix(req: HttpRequest, path: web::Path<(Uuid,)>) -> HttpResponse {
    let work_id = path.0.0;
    let scheme = if req.app_config().secure() {
        "https".to_string()
    } else {
        "http".to_string()
    };
    let thoth_url = format!("{}://{}/graphql", scheme, req.app_config().local_addr());
    if let Ok(work) = get_work(work_id, thoth_url).await {
        if let Ok(body) = generate_onix_3(work) {
            HttpResponse::Ok()
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

#[post("/account/login")]
async fn login_credentials(
    payload: web::Json<LoginCredentials>,
    ctx: web::Data<Context>,
) -> Result<HttpResponse, Error> {
    let r = payload.into_inner();

    login(&r.email, &r.password, &ctx).and_then(|account| {
        let token = account.issue_token(&ctx).unwrap();
        Ok(HttpResponse::Ok().json(token))
    }).map_err(error::ErrorUnauthorized)
}

fn config(cfg: &mut web::ServiceConfig) {
    dotenv().ok();
    let pool = establish_connection();
    let schema_context = Context { db: pool };
    let schema = std::sync::Arc::new(create_schema());

    cfg.data(schema.clone());
    cfg.data(schema_context);
    cfg.service(graphql);
    cfg.service(graphiql);
    cfg.service(onix);
    cfg.service(login_credentials);
}

#[actix_rt::main]
pub async fn start_server(port: String) -> io::Result<()> {
    env_logger::init();

    dotenv().ok();
    let secret_str = env::var("SECRET_KEY").expect("SECRET_KEY must be set");
    let domain = env::var("THOTH_DOMAIN").expect("THOTH_DOMAIN must be set");
    let session_duration = env::var("SESSION_DURATION_SECONDS").expect("SESSION_DURATION_SECONDS must be set");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(secret_str.as_bytes())
                    .name("auth")
                    .path("/")
                    .domain(&domain)
                    .max_age(session_duration.parse::<i64>().unwrap()),
            ))
            .wrap(
                Cors::new()
                    .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                    .finish(),
            )
            .configure(config)
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
