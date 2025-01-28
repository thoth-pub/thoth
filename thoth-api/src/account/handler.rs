use diesel::prelude::*;
use dotenv::dotenv;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use regex::Regex;
use std::env;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use uuid::Uuid;

use crate::account::{
    model::{
        Account, AccountAccess, AccountData, DecodedToken, LinkedPublisher, NewAccount,
        NewPassword, NewPublisherAccount, PublisherAccount, Token,
    },
    service::get_account,
    util::{make_hash, make_salt},
};
use crate::db::PgPool;
use thoth_errors::{ThothError, ThothResult};

impl Account {
    pub fn get_permissions(&self, pool: &PgPool) -> ThothResult<Vec<LinkedPublisher>> {
        let publisher_accounts = self.get_publisher_accounts(pool)?;
        let permissions: Vec<LinkedPublisher> =
            publisher_accounts.into_iter().map(|p| p.into()).collect();
        Ok(permissions)
    }

    pub fn get_publisher_accounts(&self, pool: &PgPool) -> ThothResult<Vec<PublisherAccount>> {
        use crate::schema::publisher_account::dsl::*;
        let mut conn = pool.get()?;

        let publisher_accounts = publisher_account
            .filter(account_id.eq(self.account_id))
            .load::<PublisherAccount>(&mut conn)
            .expect("Error loading publisher accounts");
        Ok(publisher_accounts)
    }

    pub fn add_publisher_account(
        &self,
        pool: &PgPool,
        linked_publisher: LinkedPublisher,
    ) -> ThothResult<PublisherAccount> {
        use crate::schema::publisher_account::dsl::*;
        let mut conn = pool.get()?;
        let new_publisher_account = NewPublisherAccount {
            account_id: self.account_id,
            publisher_id: linked_publisher.publisher_id,
            is_admin: linked_publisher.is_admin,
        };
        diesel::insert_into(publisher_account)
            .values(&new_publisher_account)
            .get_result::<PublisherAccount>(&mut conn)
            .map_err(Into::into)
    }

    pub fn get_account_access(&self, linked_publishers: Vec<LinkedPublisher>) -> AccountAccess {
        AccountAccess {
            is_superuser: self.is_superuser,
            is_bot: self.is_bot,
            linked_publishers,
        }
    }

    pub fn issue_token(&self, pool: &PgPool) -> ThothResult<String> {
        const DEFAULT_TOKEN_VALIDITY: i64 = 24 * 60 * 60;
        let mut connection = pool.get()?;
        dotenv().ok();
        let linked_publishers: Vec<LinkedPublisher> =
            self.get_permissions(pool).unwrap_or_default();
        let namespace = self.get_account_access(linked_publishers);
        let secret_str = env::var("SECRET_KEY").expect("SECRET_KEY must be set");
        let secret: &[u8] = secret_str.as_bytes();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ThothError::InternalError("Unable to set token iat".into()))?;
        let claim = Token {
            sub: self.email.clone(),
            exp: now.as_secs() as i64 + DEFAULT_TOKEN_VALIDITY,
            iat: now.as_secs() as i64,
            jti: Uuid::new_v4().to_string(),
            namespace,
        };
        let token = encode(
            &Header::default(),
            &claim,
            &EncodingKey::from_secret(secret),
        )
        .map_err(|_| ThothError::InternalError("Unable to create token".into()));

        use crate::schema::account::dsl;
        let updated_account = diesel::update(dsl::account.find(self.account_id))
            .set(dsl::token.eq(token?))
            .get_result::<Account>(&mut connection)
            .expect("Unable to set token");
        Ok(updated_account.token.unwrap())
    }
}

impl From<AccountData> for NewAccount {
    fn from(account_data: AccountData) -> Self {
        let AccountData {
            name,
            surname,
            email,
            password,
            is_superuser,
            is_bot,
            ..
        } = account_data;

        let salt = make_salt();
        let hash = make_hash(&password, &salt).to_vec();
        Self {
            name,
            surname,
            email,
            hash,
            salt,
            is_superuser,
            is_bot,
        }
    }
}

impl From<PublisherAccount> for LinkedPublisher {
    fn from(publisher_account: PublisherAccount) -> Self {
        let PublisherAccount {
            publisher_id,
            is_admin,
            ..
        } = publisher_account;
        Self {
            publisher_id,
            is_admin,
        }
    }
}

impl Token {
    pub fn verify(token: &str) -> ThothResult<Token> {
        dotenv().ok();
        let secret_str = env::var("SECRET_KEY").expect("SECRET_KEY must be set");
        let secret: &[u8] = secret_str.as_bytes();

        let data = decode::<Token>(
            token,
            &DecodingKey::from_secret(secret),
            &Validation::default(),
        )
        .map_err(|_| ThothError::InvalidToken)?;
        Ok(data.claims)
    }

    pub fn account_id(&self, pool: &PgPool) -> Uuid {
        get_account(&self.sub, pool).unwrap().account_id
    }
}

lazy_static::lazy_static! {
    static ref BEARER_REGEXP : Regex = Regex::new(r"^Bearer\s(.*)$").expect("Bearer regexp failed!");
}

impl actix_web::FromRequest for DecodedToken {
    type Error = actix_web::Error;
    type Future = futures::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let token = req
            .headers()
            .get(actix_web::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|authorization| {
                BEARER_REGEXP
                    .captures(authorization)
                    .and_then(|captures| captures.get(1))
            })
            .map(|v| v.as_str());

        futures::future::ready(Ok(match token {
            None => DecodedToken { jwt: None },
            Some(token) => match Token::verify(token) {
                Ok(decoded) => DecodedToken { jwt: Some(decoded) },
                Err(_) => DecodedToken { jwt: None },
            },
        }))
    }
}

impl NewPassword {
    pub fn new(email: String, password: String) -> Self {
        let salt = make_salt();
        let hash = make_hash(&password, &salt).to_vec();
        Self { email, hash, salt }
    }
}

impl PublisherAccount {
    pub fn delete(&self, pool: &PgPool) -> ThothResult<()> {
        use crate::schema::publisher_account::dsl::*;

        pool.get()?.transaction(|connection| {
            diesel::delete(
                publisher_account.filter(
                    account_id
                        .eq(self.account_id)
                        .and(publisher_id.eq(self.publisher_id)),
                ),
            )
            .execute(connection)
            .map(|_| ())
            .map_err(Into::into)
        })
    }
}
