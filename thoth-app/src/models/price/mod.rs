use serde::Deserialize;
use serde::Serialize;
use thoth_api::price::model::CurrencyCode;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Price {
    pub price_id: String,
    pub publication_id: String,
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
            price_id: "".to_string(),
            publication_id: "".to_string(),
            currency_code: CurrencyCode::Gbp,
            unit_price: 0.00,
        }
    }
}

pub mod create_price_mutation;
pub mod delete_price_mutation;
pub mod currency_codes_query;
