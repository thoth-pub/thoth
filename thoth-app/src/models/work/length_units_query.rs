use serde::Deserialize;
use serde::Serialize;

use super::LengthUnitDefinition;

const LENGTH_UNITS_QUERY: &str = "
    {
        length_units: __type(name: \"LengthUnit\") {
            enumValues {
                name
            }
        }
    }
";

graphql_query_builder! {
    LengthUnitsRequest,
    LengthUnitsRequestBody,
    Variables,
    LENGTH_UNITS_QUERY,
    LengthUnitsResponseBody,
    LengthUnitsResponseData,
    FetchLengthUnits,
    FetchActionLengthUnits
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Variables {}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct LengthUnitsResponseData {
    pub length_units: LengthUnitDefinition,
}
