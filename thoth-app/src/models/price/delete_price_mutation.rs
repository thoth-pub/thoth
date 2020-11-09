use serde::Deserialize;
use serde::Serialize;

use super::Price;

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
    pub price_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeletePriceResponseData {
    pub delete_price: Option<Price>,
}
