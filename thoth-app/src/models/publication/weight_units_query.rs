use serde::Deserialize;
use serde::Serialize;

use super::WeightUnitDefinition;

const WEIGHT_UNITS_QUERY: &str = "
    {
        weight_units: __type(name: \"WeightUnit\") {
            enumValues {
                name
            }
        }
    }
";

graphql_query_builder! {
    WeightUnitsRequest,
    WeightUnitsRequestBody,
    Variables,
    WEIGHT_UNITS_QUERY,
    WeightUnitsResponseBody,
    WeightUnitsResponseData,
    FetchWeightUnits,
    FetchActionWeightUnits
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Variables {}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct WeightUnitsResponseData {
    pub weight_units: WeightUnitDefinition,
}
