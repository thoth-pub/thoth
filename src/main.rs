#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate juniper;
#[macro_use]
extern crate diesel_derive_enum;

use std::io;
use std::sync::Arc;

use actix_web::{web, App, Error, HttpResponse, HttpServer};
use dotenv::dotenv;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;

mod db;
mod graphql_handlers;
mod schema;
pub mod models;

use crate::db::establish_connection;
use crate::graphql_handlers::{create_schema, Context, Schema};

async fn graphiql() -> HttpResponse {
    let html = graphiql_source("http://localhost:8080/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

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

#[actix_rt::main]
async fn main() -> io::Result<()> {
     dotenv().ok();
    let pool = establish_connection();
    let schema_context = Context { db: pool.clone() };
    let schema = std::sync::Arc::new(create_schema());
    HttpServer::new(move || {
        App::new()
            .data(schema.clone())
            .data(schema_context.clone())
            .service(web::resource("/graphql").route(web::post().to(graphql)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
    })
    .bind("localhost:8080")?
    .run()
    .await
}
