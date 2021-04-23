use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::errors::{ThothError, ThothResult};
use crate::price::model::NewPriceHistory;
use crate::price::model::Price;
use crate::price::model::PriceHistory;
use crate::schema::price_history;

impl NewPriceHistory {
    pub fn new(price: Price, account_id: Uuid) -> Self {
        Self {
            price_id: price.price_id,
            account_id,
            data: serde_json::Value::String(serde_json::to_string(&price).unwrap()),
        }
    }

    pub fn insert(&self, connection: &PgConnection) -> ThothResult<PriceHistory> {
        match diesel::insert_into(price_history::table)
            .values(self)
            .get_result(connection)
        {
            Ok(history) => Ok(history),
            Err(e) => Err(ThothError::from(e)),
        }
    }
}
