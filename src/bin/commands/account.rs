use super::get_pg_pool;
use crate::arguments;
use clap::Command;
use dialoguer::{console::Term, theme::ColorfulTheme, Input, MultiSelect, Password, Select};
use lazy_static::lazy_static;
use std::collections::HashSet;
use thoth::{
    api::{
        account::{
            model::{Account, LinkedPublisher},
            service::{
                all_emails, all_publishers, get_account, register as register_account,
                update_password,
            },
        },
        db::PgPool,
    },
    errors::{ThothError, ThothResult},
};

lazy_static! {
    pub(crate) static ref COMMAND: Command = Command::new("account")
        .about("Manage user accounts")
        .arg(arguments::database())
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("register").about("Create a new user account"))
        .subcommand(
            Command::new("publishers").about("Select which publisher(s) this account can manage"),
        )
        .subcommand(Command::new("password").about("Reset a password"));
}

pub fn register(arguments: &clap::ArgMatches) -> ThothResult<()> {
    let pool = get_pg_pool(arguments);

    let name = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter given name")
        .interact_on(&Term::stdout())?;
    let surname = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter family name")
        .interact_on(&Term::stdout())?;
    let email = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter email address")
        .interact_on(&Term::stdout())?;
    let password = password_input()?;
    let is_superuser: bool = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Is this a superuser account")
        .default(false)
        .interact_on(&Term::stdout())?;
    let is_bot: bool = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Is this a bot account")
        .default(false)
        .interact_on(&Term::stdout())?;

    let account = register_account(&pool, name, surname, email, password, is_superuser, is_bot)?;
    select_and_link_publishers(&pool, &account)
}

pub fn publishers(arguments: &clap::ArgMatches) -> ThothResult<()> {
    let pool = get_pg_pool(arguments);
    let account = email_selection(&pool).and_then(|email| get_account(&email, &pool))?;
    select_and_link_publishers(&pool, &account)
}

pub fn password(arguments: &clap::ArgMatches) -> ThothResult<()> {
    let pool = get_pg_pool(arguments);
    let email = email_selection(&pool)?;
    let password = password_input()?;

    update_password(&email, &password, &pool).map(|_| ())
}

fn email_selection(pool: &PgPool) -> ThothResult<String> {
    let all_emails = all_emails(pool).expect("No user accounts present in database.");
    let email_labels: Vec<String> = all_emails
        .iter()
        .map(|(email, is_superuser, is_bot, is_active)| {
            let mut label = email.clone();
            if *is_superuser {
                label.push_str(" 👑");
            }
            if *is_bot {
                label.push_str(" 🤖");
            }
            if !is_active {
                label.push_str(" ❌");
            }
            label
        })
        .collect();
    let email_selection = Select::with_theme(&ColorfulTheme::default())
        .items(&email_labels)
        .default(0)
        .with_prompt("Select a user account")
        .interact_on(&Term::stdout())?;
    all_emails
        .get(email_selection)
        .map(|(email, _, _, _)| email.clone())
        .ok_or_else(|| ThothError::InternalError("Invalid user selection".into()))
}

fn password_input() -> ThothResult<String> {
    Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter password")
        .with_confirmation("Confirm password", "Passwords do not match")
        .interact_on(&Term::stdout())
        .map_err(Into::into)
}

fn is_admin_input(publisher_name: &str) -> ThothResult<bool> {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Make user an admin of '{}'?", publisher_name))
        .default(false)
        .interact_on(&Term::stdout())
        .map_err(Into::into)
}

fn select_and_link_publishers(pool: &PgPool, account: &Account) -> ThothResult<()> {
    let publishers = all_publishers(pool)?;
    let publisher_accounts = account.get_publisher_accounts(pool)?;
    let current_ids: HashSet<(_, _)> = publisher_accounts
        .iter()
        .map(|pa| (pa.publisher_id, pa.is_admin))
        .collect();

    let items_checked: Vec<(_, _)> = publishers
        .iter()
        .map(|p| {
            let is_admin = current_ids
                .iter()
                .find(|(id, _)| *id == p.publisher_id)
                .is_some_and(|(_, admin)| *admin);
            let is_linked = current_ids.iter().any(|(id, _)| *id == p.publisher_id);
            let admin_label = if is_admin { "Admin" } else { "" };
            let mut publisher = p.clone();
            publisher.publisher_name = format!("{:<65}| {}", publisher.publisher_name, admin_label);
            (publisher, is_linked)
        })
        .collect();

    let chosen: Vec<usize> = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select publishers to link this account to")
        .items_checked(&items_checked)
        .interact_on(&Term::stdout())?;
    let chosen_ids: HashSet<_> = chosen
        .iter()
        .map(|&index| items_checked[index].0.publisher_id)
        .collect();
    let to_add: Vec<_> = publishers
        .iter()
        .filter(|p| {
            chosen_ids.contains(&p.publisher_id)
                && !current_ids.iter().any(|(id, _)| id == &p.publisher_id)
        })
        .collect();
    let to_remove: Vec<_> = publisher_accounts
        .iter()
        .filter(|pa| !chosen_ids.contains(&pa.publisher_id))
        .collect();

    for publisher in to_add {
        let is_admin: bool = is_admin_input(&publisher.publisher_name)?;
        let linked_publisher = LinkedPublisher {
            publisher_id: publisher.publisher_id,
            is_admin,
        };
        account.add_publisher_account(pool, linked_publisher)?;
    }
    for publisher_account in to_remove {
        publisher_account.delete(pool)?;
    }
    Ok(())
}
