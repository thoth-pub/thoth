use clap::{crate_authors, crate_version, value_t, App, AppSettings, Arg};
use dialoguer::{console::Term, theme::ColorfulTheme, Input, MultiSelect, Password, Select};
use dotenv::dotenv;
use std::env;
use thoth::api::account::model::{AccountData, LinkedPublisher};
use thoth::api::account::service::{all_emails, all_publishers, register, update_password};
use thoth::api::db::{establish_connection, run_migrations};
use thoth::api::errors::ThothResult;
use thoth::api_server;
use thoth::app_server;

fn host_argument(env_value: &'static str) -> Arg<'static, 'static> {
    Arg::with_name("host")
        .short("h")
        .long("host")
        .value_name("HOST")
        .env(env_value)
        .default_value("0.0.0.0")
        .help("host to bind")
        .takes_value(true)
}

fn port_argument(default_value: &'static str, env_value: &'static str) -> Arg<'static, 'static> {
    Arg::with_name("port")
        .short("p")
        .long("port")
        .value_name("PORT")
        .env(env_value)
        .default_value(default_value)
        .help("Port to bind")
        .takes_value(true)
}

fn domain_argument() -> Arg<'static, 'static> {
    Arg::with_name("domain")
        .short("d")
        .long("domain")
        .value_name("THOTH_DOMAIN")
        .env("THOTH_DOMAIN")
        .default_value("localhost")
        .help("Authentication cookie domain")
        .takes_value(true)
}

fn key_argument() -> Arg<'static, 'static> {
    Arg::with_name("key")
        .short("k")
        .long("secret-key")
        .value_name("SECRET")
        .env("SECRET_KEY")
        .help("Authentication cookie secret key")
        .takes_value(true)
}

fn session_argument() -> Arg<'static, 'static> {
    Arg::with_name("duration")
        .short("s")
        .long("session-length")
        .value_name("DURATION")
        .env("SESSION_DURATION_SECONDS")
        .default_value("3600")
        .help("Authentication cookie session duration (seconds)")
        .takes_value(true)
}

fn thoth_commands() -> App<'static, 'static> {
    App::new(env!("CARGO_PKG_NAME"))
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(App::new("migrate").about("Run the database migrations"))
        .subcommand(
            App::new("start")
                .about("Start an instance of Thoth API or GUI")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    App::new("api")
                        .about("Start the thoth API server")
                        .arg(host_argument("API_HOST"))
                        .arg(port_argument("8000", "API_PORT"))
                        .arg(domain_argument())
                        .arg(key_argument())
                        .arg(session_argument()),
                )
                .subcommand(
                    App::new("app")
                        .about("Start the thoth client GUI")
                        .arg(host_argument("APP_HOST"))
                        .arg(port_argument("8080", "APP_PORT")),
                ),
        )
        .subcommand(
            App::new("init")
                .about("Run the database migrations and start the thoth API server")
                .arg(host_argument("API_HOST"))
                .arg(port_argument("8000", "API_PORT"))
                .arg(domain_argument())
                .arg(key_argument())
                .arg(session_argument()),
        )
        .subcommand(
            App::new("account")
                .about("Manage user accounts")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(App::new("register").about("Create a new user account"))
                .subcommand(App::new("password").about("Reset a password")),
        )
}

fn main() -> ThothResult<()> {
    // load environment variables from `.env`
    dotenv().ok();

    match thoth_commands().get_matches().subcommand() {
        ("start", Some(start_matches)) => match start_matches.subcommand() {
            ("api", Some(api_matches)) => {
                let host = api_matches.value_of("host").unwrap().to_owned();
                let port = api_matches.value_of("port").unwrap().to_owned();
                let domain = api_matches.value_of("domain").unwrap().to_owned();
                let secret_str = api_matches.value_of("key").unwrap().to_owned();
                let session_duration = value_t!(api_matches.value_of("duration"), i64).unwrap();
                api_server(host, port, domain, secret_str, session_duration).map_err(|e| e.into())
            }
            ("app", Some(client_matches)) => {
                let host = client_matches.value_of("host").unwrap().to_owned();
                let port = client_matches.value_of("port").unwrap().to_owned();
                app_server(host, port).map_err(|e| e.into())
            }
            _ => unreachable!(),
        },
        ("migrate", Some(_)) => run_migrations(),
        ("init", Some(init_matches)) => {
            let host = init_matches.value_of("host").unwrap().to_owned();
            let port = init_matches.value_of("port").unwrap().to_owned();
            let domain = init_matches.value_of("domain").unwrap().to_owned();
            let secret_str = init_matches.value_of("key").unwrap().to_owned();
            let session_duration = value_t!(init_matches.value_of("duration"), i64).unwrap();
            run_migrations()?;
            api_server(host, port, domain, secret_str, session_duration).map_err(|e| e.into())
        }
        ("account", Some(account_matches)) => match account_matches.subcommand() {
            ("register", Some(_)) => {
                let pool = establish_connection();

                let name: String = Input::new()
                    .with_prompt("Enter given name")
                    .interact_on(&Term::stdout())?;
                let surname: String = Input::new()
                    .with_prompt("Enter family name")
                    .interact_on(&Term::stdout())?;
                let email: String = Input::new()
                    .with_prompt("Enter email address")
                    .interact_on(&Term::stdout())?;
                let password = Password::new()
                    .with_prompt("Enter password")
                    .with_confirmation("Confirm password", "Passwords do not match")
                    .interact_on(&Term::stdout())?;
                let is_superuser: bool = Input::new()
                    .with_prompt("Is this a superuser account")
                    .default(false)
                    .interact_on(&Term::stdout())?;
                let is_bot: bool = Input::new()
                    .with_prompt("Is this a bot account")
                    .default(false)
                    .interact_on(&Term::stdout())?;

                let mut linked_publishers = vec![];
                if let Ok(publishers) = all_publishers(&pool) {
                    let chosen: Vec<usize> = MultiSelect::new()
                        .items(&publishers)
                        .with_prompt("Select publishers to link this account to")
                        .interact_on(&Term::stdout())?;
                    for index in chosen {
                        let publisher = publishers.get(index).unwrap();
                        let is_admin: bool = Input::new()
                            .with_prompt(format!(
                                "Make user an admin of '{}'?",
                                publisher.publisher_name
                            ))
                            .default(false)
                            .interact_on(&Term::stdout())?;
                        let linked_publisher = LinkedPublisher {
                            publisher_id: publisher.publisher_id,
                            is_admin,
                        };
                        linked_publishers.push(linked_publisher);
                    }
                }
                let account_data = AccountData {
                    name,
                    surname,
                    email,
                    password,
                    is_superuser,
                    is_bot,
                };
                register(account_data, linked_publishers, &pool).map(|_| ())
            }
            ("password", Some(_)) => {
                let pool = establish_connection();
                let all_emails = all_emails(&pool).expect("No user accounts present in database.");
                let email_selection = Select::with_theme(&ColorfulTheme::default())
                    .items(&all_emails)
                    .default(0)
                    .with_prompt("Select a user account")
                    .interact_on(&Term::stdout())?;
                let password = Password::new()
                    .with_prompt("Enter new password")
                    .with_confirmation("Confirm password", "Passwords do not match")
                    .interact_on(&Term::stdout())?;
                let email = all_emails.get(email_selection).unwrap();

                update_password(&email, &password, &pool).map(|_| ())
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}
