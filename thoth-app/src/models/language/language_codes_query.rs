use serde::Deserialize;
use serde::Serialize;

use super::LanguageCodeDefinition;

const LANGUAGE_CODES_QUERY: &str = "
    {
        language_codes: __type(name: \"LanguageCode\") {
            enumValues {
                name
            }
        }
    }
";

graphql_query_builder! {
    LanguageCodesRequest,
    LanguageCodesRequestBody,
    Variables,
    LANGUAGE_CODES_QUERY,
    LanguageCodesResponseBody,
    LanguageCodesResponseData,
    FetchLanguageCodes,
    FetchActionLanguageCodes
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Variables {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LanguageCodesResponseData {
    pub language_codes: LanguageCodeDefinition,
}

impl Default for LanguageCodesResponseData {
    fn default() -> LanguageCodesResponseData {
        LanguageCodesResponseData {
            language_codes: LanguageCodeDefinition {
                enum_values: vec![],
            },
        }
    }
}
