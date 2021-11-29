use serde::Deserialize;
use serde::Serialize;

use super::CountryCodeDefinition;

const COUNTRY_CODES_QUERY: &str = "
    {
        country_codes: __type(name: \"CountryCode\") {
            enumValues {
                name
            }
        }
    }
";

graphql_query_builder! {
    CountryCodesRequest,
    CountryCodesRequestBody,
    Variables,
    COUNTRY_CODES_QUERY,
    CountryCodesResponseBody,
    CountryCodesResponseData,
    FetchCountryCodes,
    FetchActionCountryCodes
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Variables {}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CountryCodesResponseData {
    pub country_codes: CountryCodeDefinition,
}
