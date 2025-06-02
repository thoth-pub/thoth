use serde::Deserialize;
use serde::Serialize;

use super::LocaleCodeDefinition;

const LOCALE_CODES_QUERY: &str = "
    {
        locale_codes: __type(name: \"LocaleCode\") {
            enumValues {
                name
            }
        }
    }
";

graphql_query_builder! {
    LocaleCodesRequest,
    LocaleCodesRequestBody,
    Variables,
    LOCALE_CODES_QUERY,
    LocaleCodesResponseBody,
    LocaleCodesResponseData,
    FetchLocaleCodes,
    FetchActionLocaleCodes
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Variables {}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocaleCodesResponseData {
    pub locale_codes: LocaleCodeDefinition,
}
