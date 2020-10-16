extern crate clap;
use clap::{crate_authors, crate_version, App, AppSettings, Arg};
use dotenv::dotenv;

use thoth::server::api::start_server as api_server;
use thoth::server::app::start_server as app_server;
use thoth_api::account::service::register;
use thoth_api::db::run_migrations;
use thoth_api::db::Context;
use thoth_api::db::establish_connection;
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
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    App::new("api").about("Start the thoth API server").arg(
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
                    App::new("app").about("Start the thoth client GUI").arg(
                        Arg::with_name("port")
                            .short("p")
                            .long("port")
                            .value_name("PORT")
                            .default_value("8080")
                            .help("Port to bind")
                            .takes_value(true),
                    ),
                ),
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
        .subcommand(
            App::new("account")
                .about("Manage user accounts")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    App::new("register").about("Create a new user account")
                        .arg(
                            Arg::with_name("name")
                                .short("n")
                                .long("name")
                                .value_name("NAME")
                                .help("First name")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("surname")
                                .short("s")
                                .long("surname")
                                .value_name("NAME")
                                .help("Last name")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("email")
                                .short("e")
                                .long("email")
                                .value_name("EMAIL")
                                .help("First name")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("password")
                                .short("p")
                                .long("password")
                                .value_name("password")
                                .help("User password")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("is-admin")
                                .long("is-admin")
                                .multiple(false)
                                .help("Is the user an admin"),
                        )
                        .arg(
                            Arg::with_name("is-bot")
                                .long("is-bot")
                                .multiple(false)
                                .help("Is the user a bot"),
                        )
                    ),
        )
        .get_matches();

    match matches.subcommand() {
        ("start", Some(start_matches)) => match start_matches.subcommand() {
            ("api", Some(api_matches)) => {
                let port = api_matches.value_of("port").unwrap();
                match api_server(port.to_owned()) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(ThothError::from(e).into()),
                }
            }
            ("app", Some(client_matches)) => {
                let port = client_matches.value_of("port").unwrap();
                match app_server(port.to_owned()) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(ThothError::from(e).into()),
                }
            }
            _ => unreachable!(),
        },
        ("migrate", Some(_)) => run_migrations(),
        ("init", Some(init_matches)) => {
            let port = init_matches.value_of("port").unwrap();
            run_migrations()?;
            match api_server(port.to_owned()) {
                Ok(_) => Ok(()),
                Err(e) => Err(ThothError::from(e).into()),
            }
        },
        ("account", Some(account_matches)) => match account_matches.subcommand() {
            ("register", Some(register_matches)) => {
                let name = register_matches.value_of("name").unwrap();
                let surname = register_matches.value_of("surname").unwrap();
                let email = register_matches.value_of("email").unwrap();
                let password = register_matches.value_of("password").unwrap();
                let is_admin = register_matches.is_present("is-admin");
                let is_bot = register_matches.is_present("is-bot");

                dotenv().ok();
                let context = Context { db: establish_connection() };
                match register(&name, &surname, &email, &password, &is_admin, &is_bot, &context) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(ThothError::from(e).into()),
                }
            },
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}
