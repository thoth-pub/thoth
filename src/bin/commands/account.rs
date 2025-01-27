use super::get_pg_pool;
use dialoguer::{console::Term, theme::ColorfulTheme, Input, MultiSelect, Password, Select};
use std::collections::HashSet;
use thoth::api::account::{
    model::{AccountData, LinkedPublisher},
    service::{
        all_emails, all_publishers, get_account, register as register_account, update_password,
    },
};
use thoth_api::db::PgPool;
use thoth_errors::{ThothError, ThothResult};

fn email_selection(pool: &PgPool) -> ThothResult<String> {
    let all_emails = all_emails(&pool).expect("No user accounts present in database.");
    let email_selection = Select::with_theme(&ColorfulTheme::default())
        .items(&all_emails)
        .default(0)
        .with_prompt("Select a user account")
        .interact_on(&Term::stdout())?;
    all_emails
        .get(email_selection)
        .cloned()
        .ok_or_else(|| ThothError::InternalError("Invalid user selection".into()))
}

fn password_input() -> ThothResult<String> {
    Password::new()
        .with_prompt("Enter password")
        .with_confirmation("Confirm password", "Passwords do not match")
        .interact_on(&Term::stdout())
        .map_err(Into::into)
}

fn is_admin_input(publisher_name: &str) -> ThothResult<bool> {
    Input::new()
        .with_prompt(format!("Make user an admin of '{}'?", publisher_name))
        .default(false)
        .interact_on(&Term::stdout())
        .map_err(Into::into)
}

pub fn register(arguments: &clap::ArgMatches) -> ThothResult<()> {
    let pool = get_pg_pool(arguments);

    let name = Input::new()
        .with_prompt("Enter given name")
        .interact_on(&Term::stdout())?;
    let surname = Input::new()
        .with_prompt("Enter family name")
        .interact_on(&Term::stdout())?;
    let email = Input::new()
        .with_prompt("Enter email address")
        .interact_on(&Term::stdout())?;
    let password = password_input()?;
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
            let is_admin: bool = is_admin_input(&publisher.publisher_name)?;
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
    let account = register_account(&pool, account_data)?;
    for linked_publisher in linked_publishers {
        account.add_publisher_account(&pool, linked_publisher)?;
    }
    Ok(())
}

pub fn publishers(arguments: &clap::ArgMatches) -> ThothResult<()> {
    let pool = get_pg_pool(arguments);

    let account = email_selection(&pool).and_then(|email| get_account(&email, &pool))?;

    let publishers = all_publishers(&pool)?;
    let publisher_accounts = account.get_publisher_accounts(&pool)?;
    let items_checked: Vec<(_, bool)> = publishers
        .iter()
        .map(|publisher| {
            let is_linked = publisher_accounts
                .iter()
                .any(|pa| pa.publisher_id == publisher.publisher_id);
            (publisher, is_linked)
        })
        .collect();

    let chosen: Vec<usize> = MultiSelect::new()
        .with_prompt("Select publishers to link this account to")
        .items_checked(&items_checked)
        .interact_on(&Term::stdout())?;
    let chosen_ids: HashSet<_> = chosen
        .iter()
        .map(|&index| items_checked[index].0.publisher_id)
        .collect();
    let current_ids: HashSet<_> = publisher_accounts
        .iter()
        .map(|pa| pa.publisher_id)
        .collect();
    let to_add: Vec<_> = publishers
        .iter()
        .filter(|p| chosen_ids.contains(&p.publisher_id) && !current_ids.contains(&p.publisher_id))
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
        account.add_publisher_account(&pool, linked_publisher)?;
    }
    for publisher_account in to_remove {
        publisher_account.delete(&pool)?;
    }

    Ok(())
}

pub fn password(arguments: &clap::ArgMatches) -> ThothResult<()> {
    let pool = get_pg_pool(arguments);
    let email = email_selection(&pool)?;
    let password = password_input()?;

    update_password(&email, &password, &pool).map(|_| ())
}
