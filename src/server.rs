use std::sync::Arc;
use std::io;

use actix_web::{App, HttpServer, web, Error, HttpResponse};
use dotenv::dotenv;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;

use crate::db::establish_connection;
use crate::graphql_handlers::{create_schema, Context, Schema};

async fn graphiql() -> HttpResponse {
    let html = graphiql_source("/graphql");
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

fn config(cfg: &mut web::ServiceConfig) {
    dotenv().ok();
    let pool = establish_connection();
    let schema_context = Context { db: pool };
    let schema = std::sync::Arc::new(create_schema());

    cfg.data(
        schema.clone()
    );
    cfg.data(
        schema_context
    );
    cfg.service(
        web::resource("/graphql").route(web::post().to(graphql))
    );
    cfg.service(
        web::resource("/graphiql").route(web::get().to(graphiql))
    );
}

#[actix_rt::main]
pub async fn start_server(port: String) -> io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .configure(config)
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
