extern crate clap;
use std::io;

use clap::{Arg, App, AppSettings, crate_version, crate_authors};

use thoth::server::start_server;
use thoth::db::run_migrations;
use thoth::onix::generate_onix_3;

fn main() -> io::Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(crate_version!())
        .author(crate_authors!())
        .about(env!("CARGO_PKG_DESCRIPTION"))
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
                    .default_value("8080")
                    .help("Port to bind")
                    .takes_value(true)))
        .subcommand(
            App::new("init")
                .about("Run the database migrations and start the thoth server")
                .arg(Arg::with_name("port")
                    .short("p")
                    .long("port")
                    .value_name("PORT")
                    .default_value("8080")
                    .help("Port to bind")
                    .takes_value(true)))
        .subcommand(
            App::new("onix")
                .about("Produce an ONIX 3.0 file for a work")
                .arg(Arg::with_name("work-id")
                    .short("w")
                    .long("work-id")
                    .value_name("UUID")
                    .help("ID of the work to generate ONIX for")
                    .takes_value(true)
                    .required(true))
                .arg(Arg::with_name("thoth-url")
                    .short("u")
                    .long("thoth-url")
                    .value_name("URL")
                    .default_value("http://localhost:8080/graphql")
                    .help("URL to thoth's GraphQL endpoint")
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
            ("init", Some(init_matches)) => {
                let port = init_matches.value_of("port").unwrap();
                run_migrations()?;
                start_server(port.to_owned())
            }
            ("onix", Some(onix_matches)) => {
                let work_id = onix_matches.value_of("work-id").unwrap();
                let thoth_url = onix_matches.value_of("thoth-url").unwrap();
                match generate_onix_3(
                    thoth_url.to_owned(), work_id.to_owned()
                ) {
                    Ok(_) => Ok(()),
                    Err(_) => Ok(())
                }
            }
            _ => unreachable!(),
    }
}
