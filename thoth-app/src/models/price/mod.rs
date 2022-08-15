use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::price::CurrencyCode;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyCodeDefinition {
    pub enum_values: Vec<CurrencyCodeValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyCodeValues {
    pub name: CurrencyCode,
}

pub mod create_price_mutation;
pub mod currency_codes_query;
pub mod delete_price_mutation;
