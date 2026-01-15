mod graphiql;
mod logger;

use std::{io, sync::Arc, time::Duration};

use actix_cors::Cors;
use actix_web::{
    get,
    http::header,
    middleware::Compress,
    post,
    web::{Data, Json},
    App, Error, HttpResponse, HttpServer, Result,
};
use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;
use thoth_api::{
    db::{init_pool, PgPool},
    graphql::{
        model::{create_schema, Context, Schema},
        GraphQLRequest,
    },
};
use zitadel::{
    actix::introspection::{IntrospectedUser, IntrospectionConfigBuilder},
    credentials::Application,
};

use crate::graphiql::graphiql_source;
use crate::logger::{BodyLogger, Logger};

#[derive(Serialize)]
struct ApiConfig {
    api_name: String,
    api_version: String,
    api_schema: String,
    public_url: String,
    schema_explorer_url: String,
}

impl ApiConfig {
    pub fn new(public_url: String) -> Self {
        Self {
            public_url: format!("{public_url}/graphql"),
            schema_explorer_url: format!("{public_url}/graphiql"),
            ..Default::default()
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            api_name: "Thoth Metadata GraphQL API".to_string(),
            api_version: env!("CARGO_PKG_VERSION").parse().unwrap(),
            api_schema: "".to_string(),
            public_url: "".to_string(),
            schema_explorer_url: "".to_string(),
        }
    }
}

#[get("/")]
async fn index(config: Data<ApiConfig>) -> HttpResponse {
    HttpResponse::Ok().json(config.into_inner())
}

#[get("/graphiql")]
async fn graphiql_interface(config: Data<ApiConfig>) -> HttpResponse {
    let html = graphiql_source(&config.public_url);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[get("/graphql")]
async fn graphql_index(config: Data<ApiConfig>) -> HttpResponse {
    HttpResponse::MethodNotAllowed().json(format!(
        "GraphQL API must be queried making a POST request to {}",
        config.public_url
    ))
}

#[get("/schema.graphql")]
async fn graphql_schema(st: Data<Arc<Schema>>) -> HttpResponse {
    HttpResponse::Ok().body(st.as_sdl())
}

#[post("/graphql")]
async fn graphql(
    st: Data<Arc<Schema>>,
    pool: Data<PgPool>,
    user: Option<IntrospectedUser>,
    data: Json<GraphQLRequest>,
) -> Result<HttpResponse, Error> {
    let ctx = Context::new(pool.into_inner(), user);
    let result = data.execute(&st, &ctx).await;
    match result.is_ok() {
        true => Ok(HttpResponse::Ok().json(result)),
        false => Ok(HttpResponse::BadRequest().json(result)),
    }
}

#[allow(clippy::too_many_arguments)]
#[actix_web::main]
pub async fn start_server(
    database_url: String,
    host: String,
    port: String,
    threads: usize,
    keep_alive: u64,
    public_url: String,
    private_key: String,
) -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let decoded_private_key = general_purpose::STANDARD
        .decode(&private_key)
        .expect("Failed to base64-decode private key");
    let decoded_str =
        std::str::from_utf8(&decoded_private_key).expect("Decoded key is not valid UTF-8");
    let auth = IntrospectionConfigBuilder::new("http://localhost:8282")
        .with_jwt_profile(Application::load_from_json(decoded_str).unwrap())
        .build()
        .await
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(Compress::default())
            .wrap(Logger::default())
            .wrap(BodyLogger)
            .wrap(
                Cors::default()
                    .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                    .allow_any_origin()
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials(),
            )
            .app_data(auth.clone())
            .app_data(Data::new(ApiConfig::new(public_url.clone())))
            .app_data(Data::new(init_pool(&database_url)))
            .app_data(Data::new(Arc::new(create_schema())))
            .service(index)
            .service(graphql_index)
            .service(graphql)
            .service(graphiql_interface)
            .service(graphql_schema)
    })
    .workers(threads)
    .keep_alive(Duration::from_secs(keep_alive))
    .bind(format!("{host}:{port}"))?
    .run()
    .await
}
