use diesel::prelude::*;

use crate::account::model::Account;
use crate::account::model::AccountData;
use crate::account::model::AccountDetails;
use crate::account::model::LinkedPublisher;
use crate::account::model::NewAccount;
use crate::account::model::NewPassword;
use crate::account::model::NewPublisherAccount;
use crate::account::model::PublisherAccount;
use crate::account::util::verify;
use crate::db::PgPool;
use crate::model::publisher::Publisher;
use crate::schema::account::is_superuser;
use thoth_errors::{ThothError, ThothResult};

pub fn login(user_email: &str, user_password: &str, pool: &PgPool) -> ThothResult<Account> {
    use crate::schema::account::dsl;

    let mut conn = pool.get()?;
    let account = dsl::account
        .filter(dsl::email.eq(user_email))
        .first::<Account>(&mut conn)
        .map_err(|_| ThothError::Unauthorised)?;

    if verify(&account, user_password) {
        Ok(account)
    } else {
        Err(ThothError::Unauthorised)
    }
}

pub fn get_account(email: &str, pool: &PgPool) -> ThothResult<Account> {
    use crate::schema::account::dsl;

    let mut conn = pool.get()?;
    let account = dsl::account
        .filter(dsl::email.eq(email))
        .first::<Account>(&mut conn)
        .map_err(|_| ThothError::Unauthorised)?;
    Ok(account)
}

pub fn get_account_details(email: &str, pool: &PgPool) -> ThothResult<AccountDetails> {
    use crate::schema::account::dsl;

    let mut conn = pool.get()?;
    let account = dsl::account
        .filter(dsl::email.eq(email))
        .first::<Account>(&mut conn)
        .map_err(|_| ThothError::Unauthorised)?;
    let linked_publishers: Vec<LinkedPublisher> = account.get_permissions(pool).unwrap_or_default();
    let resource_access = account.get_account_access(linked_publishers);
    let account_details = AccountDetails {
        account_id: account.account_id,
        name: account.name,
        surname: account.surname,
        email: account.email,
        token: account.token,
        created_at: account.created_at,
        updated_at: account.updated_at,
        resource_access,
    };
    Ok(account_details)
}

pub fn register(
    pool: &PgPool,
    name: String,
    surname: String,
    email: String,
    password: String,
    is_superuser: bool,
    is_bot: bool,
) -> ThothResult<Account> {
    use crate::schema::account::dsl;

    let mut connection = pool.get()?;
    let account: NewAccount = AccountData {
        name,
        surname,
        email,
        password,
        is_superuser,
        is_bot,
    }
    .into();
    let created_account: Account = diesel::insert_into(dsl::account)
        .values(&account)
        .get_result::<Account>(&mut connection)?;
    Ok(created_account)
}

pub fn all_emails(pool: &PgPool) -> ThothResult<Vec<String>> {
    let mut connection = pool.get()?;

    use crate::schema::account::dsl;
    let emails = dsl::account
        .select(dsl::email)
        .order(dsl::email.asc())
        .load::<String>(&mut connection)
        .map_err(|_| ThothError::InternalError("Unable to load records".into()))?;
    Ok(emails)
}

pub fn all_publishers(pool: &PgPool) -> ThothResult<Vec<Publisher>> {
    let mut connection = pool.get()?;

    use crate::schema::publisher::dsl;
    let publishers = dsl::publisher
        .order(dsl::publisher_name.asc())
        .load::<Publisher>(&mut connection)
        .map_err(|_| ThothError::InternalError("Unable to load records".into()))?;
    Ok(publishers)
}

pub fn update_password(email: &str, password: &str, pool: &PgPool) -> ThothResult<Account> {
    let mut connection = pool.get()?;

    let new_password = NewPassword::new(email.to_string(), password.to_string());
    use crate::schema::account::dsl;

    let account_obj = dsl::account
        .filter(dsl::email.eq(email))
        .first::<Account>(&mut connection)
        .map_err(Into::<ThothError>::into)?;

    diesel::update(dsl::account.find(&account_obj.account_id))
        .set(&new_password)
        .get_result(&mut connection)
        .map_err(Into::into)
}
