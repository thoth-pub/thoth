mod graphiql;
mod logger;

use std::{io, sync::Arc, time::Duration};

use actix_cors::Cors;
use actix_identity::{Identity, IdentityMiddleware};
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::{time::Duration as CookieDuration, Key},
    error, get,
    http::header,
    middleware::Compress,
    post,
    web::{Data, Json},
    App, Error, HttpMessage, HttpRequest, HttpResponse, HttpServer, Result,
};
use serde::Serialize;
use thoth_api::{
    account::model::{AccountDetails, DecodedToken, LoginCredentials},
    account::service::{get_account, get_account_details, login},
    db::{init_pool as init_pg_pool, PgPool},
    graphql::{
        model::{create_schema, Context, Schema},
        GraphQLRequest,
    },
    redis::{init_pool as init_redis_pool, RedisPool},
};
use thoth_errors::ThothError;

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
    redis_pool: Data<RedisPool>,
    token: DecodedToken,
    data: Json<GraphQLRequest>,
) -> Result<HttpResponse, Error> {
    let ctx = Context::new(pool.into_inner(), redis_pool.into_inner(), token);
    let result = data.execute(&st, &ctx).await;
    match result.is_ok() {
        true => Ok(HttpResponse::Ok().json(result)),
        false => Ok(HttpResponse::BadRequest().json(result)),
    }
}

#[post("/account/login")]
async fn login_credentials(
    request: HttpRequest,
    payload: Json<LoginCredentials>,
    pool: Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let r = payload.into_inner();

    login(&r.email, &r.password, &pool)
        .and_then(|account| {
            account.issue_token(&pool)?;
            let details = get_account_details(&account.email, &pool).unwrap();
            let user_string = serde_json::to_string(&details)
                .map_err(|_| ThothError::InternalError("Serder error".into()))?;
            Identity::login(&request.extensions(), user_string)
                .map_err(|_| ThothError::InternalError("Failed to store session cookie".into()))?;
            Ok(HttpResponse::Ok().json(details))
        })
        .map_err(error::ErrorUnauthorized)
}

#[post("/account/token/renew")]
async fn login_session(
    request: HttpRequest,
    token: DecodedToken,
    identity: Option<Identity>,
    pool: Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let email = match identity {
        Some(session) => {
            let id = session.id().map_err(|_| ThothError::Unauthorised)?;
            let details: AccountDetails =
                serde_json::from_str(&id).map_err(|_| ThothError::Unauthorised)?;
            details.email
        }
        None => {
            token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
            let t = token.jwt.unwrap();
            t.sub
        }
    };

    get_account(&email, &pool)
        .and_then(|account| {
            account.issue_token(&pool)?;
            let details = get_account_details(&account.email, &pool).unwrap();
            let user_string = serde_json::to_string(&details)
                .map_err(|_| ThothError::InternalError("Serder error".into()))?;
            Identity::login(&request.extensions(), user_string)
                .map_err(|_| ThothError::InternalError("Failed to store session cookie".into()))?;
            Ok(HttpResponse::Ok().json(details))
        })
        .map_err(error::ErrorUnauthorized)
}

#[get("/account")]
async fn account_details(
    token: DecodedToken,
    identity: Option<Identity>,
    pool: Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let email = match identity {
        Some(session) => {
            let id = session.id().map_err(|_| ThothError::Unauthorised)?;
            let details: AccountDetails =
                serde_json::from_str(&id).map_err(|_| ThothError::Unauthorised)?;
            details.email
        }
        None => {
            token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
            let t = token.jwt.unwrap();
            t.sub
        }
    };

    get_account_details(&email, &pool)
        .map(|account_details| HttpResponse::Ok().json(account_details))
        .map_err(error::ErrorUnauthorized)
}

#[allow(clippy::too_many_arguments)]
#[actix_web::main]
pub async fn start_server(
    database_url: String,
    redis_url: String,
    host: String,
    port: String,
    threads: usize,
    keep_alive: u64,
    public_url: String,
    domain: String,
    secret_str: String,
    session_duration: i64,
) -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            .wrap(Compress::default())
            .wrap(Logger::default())
            .wrap(BodyLogger)
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    Key::from(secret_str.as_bytes()),
                )
                .cookie_name("auth".to_string())
                .cookie_path("/".to_string())
                .cookie_domain(Some(domain.clone()))
                .cookie_secure(domain.clone().ne("localhost")) // Authentication requires https unless running on localhost
                .session_lifecycle(
                    PersistentSession::default()
                        .session_ttl(CookieDuration::seconds(session_duration)),
                )
                .build(),
            )
            .wrap(
                Cors::default()
                    .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                    .allow_any_origin()
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials(),
            )
            .app_data(Data::new(ApiConfig::new(public_url.clone())))
            .app_data(Data::new(init_pg_pool(&database_url)))
            .app_data(Data::new(init_redis_pool(&redis_url)))
            .app_data(Data::new(Arc::new(create_schema())))
            .service(index)
            .service(graphql_index)
            .service(graphql)
            .service(graphiql_interface)
            .service(login_credentials)
            .service(login_session)
            .service(account_details)
            .service(graphql_schema)
    })
    .workers(threads)
    .keep_alive(Duration::from_secs(keep_alive))
    .bind(format!("{host}:{port}"))?
    .run()
    .await
}
