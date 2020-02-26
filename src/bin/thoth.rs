use std::io;

use actix_web::{App, HttpServer};

use thoth::server::config;

#[actix_rt::main]
async fn main() -> io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .configure(config)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
