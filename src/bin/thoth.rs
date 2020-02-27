#[macro_use]
extern crate diesel_migrations;
extern crate clap;
use std::io;
use std::io::stdout;

use clap::{Arg, App, AppSettings};
use actix_web::{App as WebApp, HttpServer};
use diesel_migrations::embed_migrations;

use thoth::server::config;
use thoth::db::establish_connection;

#[actix_rt::main]
async fn start_server(port: String) -> io::Result<()> {
    HttpServer::new(move || {
        WebApp::new()
            .configure(config)
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}

fn run_migrations() -> io::Result<()> {
    embed_migrations!("migrations");
    let connection = establish_connection().get().unwrap();
    embedded_migrations::run_with_output(&connection, &mut stdout())
        .expect("Can't run migrations");
    Ok(())
}

fn main() -> io::Result<()> {
    let matches = App::new("Thoth")
        .version("0.1.0")
        .author("Javier Arias <javi@openbookpublishers.com>")
        .about("GraphQL API for bibliographic data")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            App::new("migrate")
                .about("Run the database migrations"))
        .subcommand(
            App::new("start")
                .about("Start the thoth server")
                .arg(Arg::with_name("port")
                    .short("p")
                    .long("port")
                    .value_name("PORT")
                    .help("Port to bind")
                    .takes_value(true)))
        .get_matches();

    match matches.subcommand() {
            ("start", Some(start_matches)) => {
                let port = start_matches.value_of("port").unwrap();
                start_server(port.to_owned())
            }
            ("migrate", Some(_)) => {
                run_migrations()
            }
            _ => unreachable!(),
    }
}
