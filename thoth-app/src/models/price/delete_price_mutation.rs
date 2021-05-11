use serde::Deserialize;
use serde::Serialize;
use thoth_api::price::model::Price;
use uuid::Uuid;

const DELETE_PRICE_MUTATION: &str = "
    mutation DeletePrice(
        $priceId: Uuid!
    ) {
        deletePrice(
            priceId: $priceId
        ){
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
    DeletePriceRequest,
    DeletePriceRequestBody,
    Variables,
    DELETE_PRICE_MUTATION,
    DeletePriceResponseBody,
    DeletePriceResponseData,
    PushDeletePrice,
    PushActionDeletePrice
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub price_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeletePriceResponseData {
    pub delete_price: Option<Price>,
}
