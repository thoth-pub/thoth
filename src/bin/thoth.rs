use std::io;

use actix_web::{web, App, HttpServer};
use dotenv::dotenv;

use thoth::server::{graphql, graphiql};
use thoth::db::establish_connection;
use thoth::graphql_handlers::{create_schema, Context};

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
