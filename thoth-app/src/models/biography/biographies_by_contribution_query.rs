use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::biography::Biography;
use thoth_api::model::LocaleCode;
use thoth_api::model::MarkupFormat;
use uuid::Uuid;

pub const BIOGRAPHIES_BY_CONTRIBUTION_QUERY: &str = "
    query BiographiesByContributionQuery(
        $contributionId: Uuid!,
        $limit: Int,
        $offset: Int,
        $filter: String,
        $localeCodes: [LocaleCode!] = [EN],
        $markupFormat: MarkupFormat = JATS_XML
    ) {
        biographies(
            contributionId: $contributionId,
            limit: $limit,
            offset: $offset,
            filter: $filter,
            localeCodes: $localeCodes,
            markupFormat: $markupFormat
        ) {
            biographyId
            contributionId
            workId
            content
            canonical
            localeCode
        }
    }
";

graphql_query_builder! {
    BiographiesByContributionRequest,
    BiographiesByContributionRequestBody,
    Variables,
    BIOGRAPHIES_BY_CONTRIBUTION_QUERY,
    BiographiesByContributionResponseBody,
    BiographiesByContributionResponseData,
    FetchBiographiesByContribution,
    FetchActionBiographiesByContribution
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub contribution_id: Uuid,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub filter: Option<String>,
    #[serde(default = "default_locale_codes")]
    pub locale_codes: Vec<LocaleCode>,
    #[serde(default = "default_markup_format")]
    pub markup_format: MarkupFormat,
}

fn default_locale_codes() -> Vec<LocaleCode> {
    vec![LocaleCode::En]
}

fn default_markup_format() -> MarkupFormat {
    MarkupFormat::JatsXml
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BiographiesByContributionResponseData {
    pub biographies: Option<Vec<Biography>>,
}
