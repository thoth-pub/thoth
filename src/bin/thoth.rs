extern crate clap;
use clap::{crate_authors, crate_version, App, AppSettings, Arg};

use thoth::db::run_migrations;
use thoth::errors::Result;
use thoth::errors::ThothError;
use thoth::server::start_server;

fn main() -> Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(crate_version!())
        .author(crate_authors!())
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(App::new("migrate").about("Run the database migrations"))
        .subcommand(
            App::new("start").about("Start the thoth server").arg(
                Arg::with_name("port")
                    .short("p")
                    .long("port")
                    .value_name("PORT")
                    .default_value("8080")
                    .help("Port to bind")
                    .takes_value(true),
            ),
        )
        .subcommand(
            App::new("init")
                .about("Run the database migrations and start the thoth server")
                .arg(
                    Arg::with_name("port")
                        .short("p")
                        .long("port")
                        .value_name("PORT")
                        .default_value("8080")
                        .help("Port to bind")
                        .takes_value(true),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("start", Some(start_matches)) => {
            let port = start_matches.value_of("port").unwrap();
            match start_server(port.to_owned()) {
                Ok(_) => Ok(()),
                Err(e) => Err(ThothError::from(e).into()),
            }
        }
        ("migrate", Some(_)) => run_migrations(),
        ("init", Some(init_matches)) => {
            let port = init_matches.value_of("port").unwrap();
            run_migrations()?;
            match start_server(port.to_owned()) {
                Ok(_) => Ok(()),
                Err(e) => Err(ThothError::from(e).into()),
            }
        }
        _ => unreachable!(),
    }
}
