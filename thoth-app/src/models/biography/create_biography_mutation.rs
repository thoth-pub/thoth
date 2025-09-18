use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::biography::Biography;
use thoth_api::model::locale::LocaleCode;
use thoth_api::model::MarkupFormat;
use uuid::Uuid;

const CREATE_BIOGRAPHY_MUTATION: &str = "
    mutation CreateBiography(
        $contributionId: Uuid!,
        $content: String!,
        $canonical: Boolean!,
        $localeCode: LocaleCode!,
        $markupFormat: MarkupFormat!
    ) {
        createBiography(
            markupFormat: $markupFormat,
            data: {
                contributionId: $contributionId,
                localeCode: $localeCode,
                canonical: $canonical,
                content: $content,
            }
        ){
            biographyId
            contributionId
            content
            canonical
            localeCode
        }
    }
";

graphql_query_builder! {
    CreateBiographyRequest,
    CreateBiographyRequestBody,
    Variables,
    CREATE_BIOGRAPHY_MUTATION,
    CreateBiographyResponseBody,
    CreateBiographyResponseData,
    PushCreateBiography,
    PushActionCreateBiography
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub contribution_id: Uuid,
    pub content: String,
    pub canonical: bool,
    pub locale_code: LocaleCode,
    pub markup_format: MarkupFormat,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateBiographyResponseData {
    pub create_biography: Option<Biography>,
}
