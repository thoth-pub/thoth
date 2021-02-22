use diesel::prelude::*;
use dotenv::dotenv;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use regex::Regex;
use std::env;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use uuid::Uuid;

use crate::account::model::Account;
use crate::account::model::AccountAccess;
use crate::account::model::AccountData;
use crate::account::model::DecodedToken;
use crate::account::model::LinkedPublisher;
use crate::account::model::NewAccount;
use crate::account::model::NewPassword;
use crate::account::model::PublisherAccount;
use crate::account::model::Token;
use crate::account::util::make_hash;
use crate::account::util::make_salt;
use crate::db::PgPool;
use crate::errors::ThothError;

impl Account {
    pub fn get_permissions(&self, pool: &PgPool) -> Result<Vec<LinkedPublisher>, ThothError> {
        use crate::schema::publisher_account::dsl::*;
        let conn = pool.get().unwrap();

        let linked_publishers = publisher_account
            .filter(account_id.eq(self.account_id))
            .load::<PublisherAccount>(&conn)
            .expect("Error loading publisher accounts");
        let permissions: Vec<LinkedPublisher> =
            linked_publishers.into_iter().map(|p| p.into()).collect();
        Ok(permissions)
    }

    pub fn get_account_access(&self, linked_publishers: Vec<LinkedPublisher>) -> AccountAccess {
        AccountAccess {
            is_superuser: self.is_superuser,
            is_bot: self.is_bot,
            linked_publishers,
        }
    }

    pub fn issue_token(&self, pool: &PgPool) -> Result<String, ThothError> {
        const DEFAULT_TOKEN_VALIDITY: i64 = 24 * 60 * 60;
        let connection = pool.get().unwrap();
        dotenv().ok();
        let linked_publishers: Vec<LinkedPublisher> =
            self.get_permissions(&pool).unwrap_or_default();
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
            .set(dsl::token.eq(token.unwrap()))
            .get_result::<Account>(&connection)
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
    pub fn verify(token: &str) -> Result<Token, ThothError> {
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
}

lazy_static::lazy_static! {
    static ref BEARER_REGEXP : Regex = Regex::new(r"^Bearer\s(.*)$").expect("Bearer regexp failed!");
}

impl actix_web::FromRequest for DecodedToken {
    type Error = actix_web::Error;
    type Future = futures::future::Ready<Result<Self, Self::Error>>;
    type Config = ();

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
        Self {
            email: email.into(),
            hash,
            salt,
        }
    }
}
