use std::sync::Arc;

use actix_web::{web, Error, HttpResponse};
use dotenv::dotenv;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;

use crate::db::establish_connection;
use crate::graphql_handlers::{create_schema, Context, Schema};

pub async fn graphiql() -> HttpResponse {
    let html = graphiql_source("/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

pub async fn graphql(
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

pub fn config(cfg: &mut web::ServiceConfig) {
    dotenv().ok();
    let pool = establish_connection();
    let schema_context = Context { db: pool.clone() };
    let schema = std::sync::Arc::new(create_schema());

    cfg.data(
        schema.clone()
    );
    cfg.data(
        schema_context.clone()
    );
    cfg.service(
        web::resource("/graphql").route(web::post().to(graphql))
    );
    cfg.service(
        web::resource("/graphiql").route(web::get().to(graphiql))
    );
}
