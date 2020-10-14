use crate::account::model::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct LoginSession(pub Session);

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Logout(pub Session);
