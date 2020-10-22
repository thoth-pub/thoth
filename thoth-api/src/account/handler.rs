use diesel::prelude::*;
use dotenv::dotenv;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use std::env;
use uuid::Uuid;
use regex::Regex;

use crate::errors::ThothError;
use crate::db::PgPool;
use crate::account::model::Account;
use crate::account::model::Token;
use crate::account::model::DecodedToken;
use crate::account::model::AccountData;
use crate::account::model::NewAccount;
use crate::account::util::make_hash;
use crate::account::util::make_salt;

impl Account {
    pub fn issue_token(&self, pool: &PgPool) -> Result<String, ThothError> {
        const DEFAULT_TOKEN_VALIDITY: i64 = 3600;
        let connection = pool.get().unwrap();
        dotenv().ok();
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
            is_admin,
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
            is_admin,
            is_bot,
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
