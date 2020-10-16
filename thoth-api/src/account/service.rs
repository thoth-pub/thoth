use std::result::Result;
use diesel::prelude::*;

use crate::db::Context;
use crate::account::model::Account;
use crate::account::model::AccountData;
use crate::account::model::NewAccount;
use crate::account::util::verify;
use crate::errors::ThothError;

pub fn login(
    user_email: &str,
    user_password: &str,
    context: &Context,
) -> Result<Account, ThothError> {
    use crate::schema::account::dsl;

    let conn = context.db.get().unwrap();
    let account = dsl::account
        .filter(dsl::email.eq(user_email))
        .first::<Account>(&conn)
        .map_err(|_| ThothError::Unauthorised)?;

    if verify(&account, &user_password) {
        Ok(account.into())
    } else {
        Err(ThothError::Unauthorised)
    }
}

pub fn register(
    name: &str,
    surname: &str,
    email: &str,
    password: &str,
    is_admin: &bool,
    is_bot: &bool,
    context: &Context,
) ->  Result<Account, ThothError> {
    println!("in");
    let connection = context.db.get().unwrap();
    let account_data = AccountData {
        name: name.to_owned(),
        surname: surname.to_owned(),
        email: email.to_owned(),
        password: password.to_owned(),
        is_admin: is_admin.to_owned(),
        is_bot: is_bot.to_owned(),
    };

    use crate::schema::account::dsl;
    let account: NewAccount = account_data.into();
    let created_account: Account = diesel::insert_into(dsl::account)
        .values(&account)
        .get_result::<Account>(&connection)?;
    Ok(created_account)
}
