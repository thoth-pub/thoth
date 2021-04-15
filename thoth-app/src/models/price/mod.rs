use serde::Deserialize;
use serde::Serialize;
use thoth_api::price::model::CurrencyCode;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Price {
    pub price_id: Uuid,
    pub publication_id: Uuid,
    pub currency_code: CurrencyCode,
    pub unit_price: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyCodeDefinition {
    pub enum_values: Vec<CurrencyCodeValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyCodeValues {
    pub name: CurrencyCode,
}

impl Default for Price {
    fn default() -> Price {
        Price {
            price_id: Default::default(),
            publication_id: Default::default(),
            currency_code: CurrencyCode::Gbp,
            unit_price: 0.00,
        }
    }
}

pub mod create_price_mutation;
pub mod currency_codes_query;
pub mod delete_price_mutation;
