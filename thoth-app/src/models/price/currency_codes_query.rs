use serde::Deserialize;
use serde::Serialize;

use super::CurrencyCodeDefinition;

const LANGUAGE_CODES_QUERY: &str = "
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
    LANGUAGE_CODES_QUERY,
    CurrencyCodesResponseBody,
    CurrencyCodesResponseData,
    FetchCurrencyCodes,
    FetchActionCurrencyCodes
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Variables {}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CurrencyCodesResponseData {
    pub currency_codes: CurrencyCodeDefinition,
}
