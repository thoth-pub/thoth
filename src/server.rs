use std::io;
use std::sync::Arc;

use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Result};
use dotenv::dotenv;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use thoth_api::db::establish_connection;
use thoth_api::graphql_handlers::{create_schema, Context, Schema};
use uuid::Uuid;

use crate::client::get_work;
use crate::onix::generate_onix_3;

const INDEX_FILE: &[u8] = include_bytes!("../assets/index.html");
const ICON_FILE: &[u8] = include_bytes!("../assets/favicon.ico");
const LOGO_FILE: &[u8] = include_bytes!("../assets/thoth-logo.png");

#[get("/favicon.ico")]
async fn favicon() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("image/x-icon")
        .body(ICON_FILE)
}

#[get("/thoth-logo.png")]
async fn logo() -> HttpResponse {
    HttpResponse::Ok().content_type("image/png").body(LOGO_FILE)
}

#[get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(INDEX_FILE)
}

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
    let work_id = path.0;
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

fn config(cfg: &mut web::ServiceConfig) {
    dotenv().ok();
    let pool = establish_connection();
    let schema_context = Context { db: pool };
    let schema = std::sync::Arc::new(create_schema());

    cfg.data(schema.clone());
    cfg.data(schema_context);
    cfg.service(favicon);
    cfg.service(logo);
    cfg.service(index);
    cfg.service(graphql);
    cfg.service(graphiql);
    cfg.service(onix);
}

#[actix_rt::main]
pub async fn start_server(port: String) -> io::Result<()> {
    HttpServer::new(move || App::new().configure(config))
        .bind(format!("0.0.0.0:{}", port))?
        .run()
        .await
}
