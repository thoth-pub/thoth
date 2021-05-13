use std::{io, sync::Arc};

use actix_cors::Cors;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_web::{
    error, get, middleware::Logger, post, web, App, Error, HttpResponse, HttpServer,
    Result,
};
use juniper::{http::graphiql::graphiql_source, http::GraphQLRequest};
use serde::Serialize;
use thoth_api::{
    account::model::AccountDetails,
    account::model::DecodedToken,
    account::model::LoginCredentials,
    account::service::get_account,
    account::service::get_account_details,
    account::service::login,
    db::establish_connection,
    db::PgPool,
    errors::ThothError,
    graphql::model::Context,
    graphql::model::{create_schema, Schema},
};

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
            public_url: format!("{}/graphql", public_url),
            schema_explorer_url: format!("{}/graphiql", public_url),
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

#[get("/graphiql")]
async fn graphiql(config: web::Data<ApiConfig>) -> HttpResponse {
    let html = graphiql_source(&config.public_url);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}
#[get("/graphql")]
async fn graphql_index(config: web::Data<ApiConfig>) -> HttpResponse {
    HttpResponse::Ok().json(ApiConfig::new(config.public_url.to_string()))
}

#[post("/graphql")]
async fn graphql(
    st: web::Data<Arc<Schema>>,
    pool: web::Data<PgPool>,
    token: DecodedToken,
    data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, Error> {
    let ctx = Context::new(pool.into_inner(), token);
    let result = web::block(move || {
        let res = data.execute(&st, &ctx);
        serde_json::to_string(&res)
    })
    .await?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(result))
}

#[post("/account/login")]
async fn login_credentials(
    payload: web::Json<LoginCredentials>,
    id: Identity,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let r = payload.into_inner();

    login(&r.email, &r.password, &pool)
        .and_then(|account| {
            account.issue_token(&pool)?;
            let details = get_account_details(&account.email, &pool).unwrap();
            let user_string = serde_json::to_string(&details)
                .map_err(|_| ThothError::InternalError("Serder error".into()))?;
            id.remember(user_string);
            Ok(HttpResponse::Ok().json(details))
        })
        .map_err(error::ErrorUnauthorized)
}

#[post("/account/token/renew")]
async fn login_session(
    token: DecodedToken,
    id: Identity,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let email = match id.identity() {
        Some(id) => {
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
            id.remember(user_string);
            Ok(HttpResponse::Ok().json(details))
        })
        .map_err(error::ErrorUnauthorized)
}

#[get("/account")]
async fn account_details(
    token: DecodedToken,
    id: Identity,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let email = match id.identity() {
        Some(id) => {
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

fn config(cfg: &mut web::ServiceConfig) {
    let pool = establish_connection();
    let schema = std::sync::Arc::new(create_schema());

    cfg.data(schema.clone());
    cfg.data(pool);
    cfg.service(graphql_index);
    cfg.service(graphql);
    cfg.service(graphiql);
    cfg.service(login_credentials);
    cfg.service(login_session);
    cfg.service(account_details);
}

#[actix_rt::main]
pub async fn start_server(
    host: String,
    port: String,
    public_url: String,
    domain: String,
    secret_str: String,
    session_duration: i64,
) -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(secret_str.as_bytes())
                    .name("auth")
                    .path("/")
                    .domain(&domain)
                    .max_age(session_duration),
            ))
            .wrap(
                Cors::new()
                    .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                    .finish(),
            )
            .data(ApiConfig {
                public_url: public_url.clone(),
                ..Default::default()
            })
            .configure(config)
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
