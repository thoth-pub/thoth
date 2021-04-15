use serde::Deserialize;
use serde::Serialize;
use thoth_api::price::model::CurrencyCode;
use uuid::Uuid;

use super::Price;

const CREATE_PRICE_MUTATION: &str = "
    mutation CreatePrice(
        $publicationId: Uuid!,
        $currencyCode: CurrencyCode!,
        $unitPrice: Float!
    ) {
        createPrice(data: {
            publicationId: $publicationId
            currencyCode: $currencyCode
            unitPrice: $unitPrice
        }){
            priceId
            publicationId
            currencyCode
            unitPrice
        }
    }
";

graphql_query_builder! {
    CreatePriceRequest,
    CreatePriceRequestBody,
    Variables,
    CREATE_PRICE_MUTATION,
    CreatePriceResponseBody,
    CreatePriceResponseData,
    PushCreatePrice,
    PushActionCreatePrice
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub publication_id: Uuid,
    pub currency_code: CurrencyCode,
    pub unit_price: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreatePriceResponseData {
    pub create_price: Option<Price>,
}
