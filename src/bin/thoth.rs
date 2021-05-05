extern crate clap;
use clap::{crate_authors, crate_version, App, AppSettings, Arg};
use dialoguer::{console::Term, theme::ColorfulTheme, Input, MultiSelect, Password, Select};
use dotenv::dotenv;

use thoth::server::api::start_server as api_server;
use thoth::server::app::start_server as app_server;
use thoth_api::account::model::{AccountData, LinkedPublisher};
use thoth_api::account::service::{all_emails, all_publishers, register, update_password};
use thoth_api::db::{establish_connection, run_migrations};
use thoth_api::errors::ThothResult;

fn main() -> ThothResult<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
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
                .subcommand(App::new("register").about("Create a new user account"))
                .subcommand(App::new("password").about("Reset a password")),
        )
        .get_matches();

    match matches.subcommand() {
        ("start", Some(start_matches)) => match start_matches.subcommand() {
            ("api", Some(api_matches)) => {
                let port = api_matches.value_of("port").unwrap();
                match api_server(port.to_owned()) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.into()),
                }
            }
            ("app", Some(client_matches)) => {
                let port = client_matches.value_of("port").unwrap();
                match app_server(port.to_owned()) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.into()),
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
                Err(e) => Err(e.into()),
            }
        }
        ("account", Some(account_matches)) => match account_matches.subcommand() {
            ("register", Some(_)) => {
                dotenv().ok();
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
                match register(account_data, linked_publishers, &pool) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            }
            ("password", Some(_)) => {
                dotenv().ok();
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

                dotenv().ok();
                let pool = establish_connection();
                match update_password(&email, &password, &pool) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}
