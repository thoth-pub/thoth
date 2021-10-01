use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::price::CurrencyCode;
use thoth_api::model::price::Price;
use uuid::Uuid;

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
            createdAt
            updatedAt
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
