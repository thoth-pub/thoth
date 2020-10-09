use serde::Deserialize;
use serde::Serialize;

use super::Imprint;

const IMPRINTS_QUERY: &str = "
    {
        imprints(limit: 9999) {
            imprintId
            imprintName
            imprintUrl
            publisher {
                publisherId
                publisherName
                publisherShortname
                publisherUrl
            }
        }
    }
";

query_builder! {
    ImprintsRequest,
    ImprintsRequestBody,
    IMPRINTS_QUERY,
    ImprintsResponseBody,
    ImprintsResponseData,
    FetchImprints,
    FetchActionImprints
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImprintsResponseData {
    pub imprints: Vec<Imprint>,
}

impl Default for ImprintsResponseData {
    fn default() -> ImprintsResponseData {
        ImprintsResponseData { imprints: vec![] }
    }
}
