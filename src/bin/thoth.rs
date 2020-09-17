extern crate clap;
use clap::{crate_authors, crate_version, App, AppSettings, Arg};

use thoth::server::api::start_server as api_server;
use thoth::server::gui::start_server as gui_server;
use thoth_api::db::run_migrations;
use thoth_api::errors::Result;
use thoth_api::errors::ThothError;

fn main() -> Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(crate_version!())
        .author(crate_authors!())
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(App::new("migrate").about("Run the database migrations"))
        .subcommand(
            App::new("start")
                .about("Start an instance of Thoth API or GUI")
                .subcommand(
                    App::new("api")
                        .about("Start the thoth API server")
                        .arg(
                            Arg::with_name("port")
                                .short("p")
                                .long("port")
                                .value_name("PORT")
                                .default_value("8000")
                                .help("Port to bind")
                                .takes_value(true),
                        ),

                )
                .subcommand(
                    App::new("gui")
                        .about("Start the thoth client GUI")
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
        )
        .subcommand(
            App::new("init")
                .about("Run the database migrations and start the thoth API server")
                .arg(
                    Arg::with_name("port")
                        .short("p")
                        .long("port")
                        .value_name("PORT")
                        .default_value("8000")
                        .help("Port to bind")
                        .takes_value(true),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("start", Some(start_matches)) => {
            match start_matches.subcommand() {
                ("api", Some(api_matches)) => {
                    let port = api_matches.value_of("port").unwrap();
                    match api_server(port.to_owned()) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(ThothError::from(e).into()),
                    }
                }
                ("gui", Some(client_matches)) => {
                    let port = client_matches.value_of("port").unwrap();
                    match gui_server(port.to_owned()) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(ThothError::from(e).into()),
                    }
                }
                _ => unreachable!(),
            }
        }
        ("migrate", Some(_)) => run_migrations(),
        ("init", Some(init_matches)) => {
            let port = init_matches.value_of("port").unwrap();
            run_migrations()?;
            match api_server(port.to_owned()) {
                Ok(_) => Ok(()),
                Err(e) => Err(ThothError::from(e).into()),
            }
        }
        _ => unreachable!(),
    }
}
