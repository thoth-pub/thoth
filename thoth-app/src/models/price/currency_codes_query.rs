use serde::Deserialize;
use serde::Serialize;

use super::CurrencyCodeDefinition;

const CURRENCY_CODES_QUERY: &str = "
    {
        currency_codes: __type(name: \"CurrencyCode\") {
            enumValues {
                name
            }
        }
    }
";

graphql_query_builder! {
    CurrencyCodesRequest,
    CurrencyCodesRequestBody,
    Variables,
    CURRENCY_CODES_QUERY,
    CurrencyCodesResponseBody,
    CurrencyCodesResponseData,
    FetchCurrencyCodes,
    FetchActionCurrencyCodes
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Variables {}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CurrencyCodesResponseData {
    pub currency_codes: CurrencyCodeDefinition,
}
