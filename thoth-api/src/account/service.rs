use diesel::prelude::*;
use std::result::Result;

use crate::account::model::Account;
use crate::account::model::AccountData;
use crate::account::model::AccountDetails;
use crate::account::model::LinkedPublisher;
use crate::account::model::NewAccount;
use crate::account::util::verify;
use crate::db::PgPool;
use crate::errors::ThothError;

pub fn login(user_email: &str, user_password: &str, pool: &PgPool) -> Result<Account, ThothError> {
    use crate::schema::account::dsl;

    let conn = pool.get().unwrap();
    let account = dsl::account
        .filter(dsl::email.eq(user_email))
        .first::<Account>(&conn)
        .map_err(|_| ThothError::Unauthorised)?;

    if verify(&account, &user_password) {
        Ok(account)
    } else {
        Err(ThothError::Unauthorised)
    }
}

pub fn login_with_token(token: &str, pool: &PgPool) -> Result<Account, ThothError> {
    use crate::schema::account::dsl;

    let conn = pool.get().unwrap();
    let account = dsl::account
        .filter(dsl::token.eq(token))
        .first::<Account>(&conn)
        .map_err(|_| ThothError::Unauthorised)?;
    Ok(account)
}

pub fn get_account_details(email: &str, pool: &PgPool) -> Result<AccountDetails, ThothError> {
    use crate::schema::account::dsl;

    let conn = pool.get().unwrap();
    let account = dsl::account
        .filter(dsl::email.eq(email))
        .first::<Account>(&conn)
        .map_err(|_| ThothError::Unauthorised)?;
    let linked_publishers: Vec<LinkedPublisher> =
        account.get_permissions(&pool).unwrap_or_default();
    let resource_access = account.get_account_access(linked_publishers);
    let account_details = AccountDetails {
        account_id: account.account_id,
        name: account.name,
        surname: account.surname,
        email: account.email,
        created_at: account.created_at,
        updated_at: account.updated_at,
        resource_access,
    };
    Ok(account_details)
}

pub fn register(
    name: &str,
    surname: &str,
    email: &str,
    password: &str,
    is_superuser: &bool,
    is_bot: &bool,
    pool: &PgPool,
) -> Result<Account, ThothError> {
    let connection = pool.get().unwrap();
    let account_data = AccountData {
        name: name.to_owned(),
        surname: surname.to_owned(),
        email: email.to_owned(),
        password: password.to_owned(),
        is_superuser: is_superuser.to_owned(),
        is_bot: is_bot.to_owned(),
    };

    use crate::schema::account::dsl;
    let account: NewAccount = account_data.into();
    let created_account: Account = diesel::insert_into(dsl::account)
        .values(&account)
        .get_result::<Account>(&connection)?;
    Ok(created_account)
}
