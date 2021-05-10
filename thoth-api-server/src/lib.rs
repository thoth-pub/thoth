use std::{io, sync::Arc};

use actix_cors::Cors;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_web::{
    error, get, middleware::Logger, post, web, App, Error, HttpRequest, HttpResponse, HttpServer,
    Result,
};
use juniper::{http::graphiql::graphiql_source, http::GraphQLRequest};
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
use thoth_client::work::get_work;
use uuid::Uuid;

mod onix;

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

#[get("/onix/{uuid}")]
async fn onix_endpoint(req: HttpRequest, path: web::Path<(Uuid,)>) -> HttpResponse {
    let work_id = (path.0).0;
    let scheme = if req.app_config().secure() {
        "https".to_string()
    } else {
        "http".to_string()
    };
    let thoth_url = format!("{}://{}/graphql", scheme, req.app_config().local_addr());
    if let Ok(work) = get_work(work_id, thoth_url).await {
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
    cfg.service(graphql);
    cfg.service(graphiql);
    cfg.service(onix_endpoint);
    cfg.service(login_credentials);
    cfg.service(login_session);
    cfg.service(account_details);
}

#[actix_rt::main]
pub async fn start_server(
    host: String,
    port: String,
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
            .configure(config)
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
