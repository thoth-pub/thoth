use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::biography::Biography;
use thoth_api::model::locale::LocaleCode;
use uuid::Uuid;

const UPDATE_BIOGRAPHY_MUTATION: &str = "
    mutation UpdateBiography(
        $biographyId: Uuid!,
        $contributionId: Uuid!,
        $workId: Uuid!,
        $content: String!,
        $canonical: Boolean!,
        $localeCode: LocaleCode!,
        $markupFormat: MarkupFormat!
    ) {
        updateBiography(
            markupFormat: $markupFormat,
            data: {
                biographyId: $biographyId,
                contributionId: $contributionId,
                workId: $workId,
                content: $content,
                canonical: $canonical,
                localeCode: $localeCode
            }
        ){
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
    UpdateBiographyRequest,
    UpdateBiographyRequestBody,
    Variables,
    UPDATE_BIOGRAPHY_MUTATION,
    UpdateBiographyResponseBody,
    UpdateBiographyResponseData,
    PushUpdateBiography,
    PushActionUpdateBiography
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub biography_id: Uuid,
    pub contribution_id: Uuid,
    pub work_id: Uuid,
    pub content: String,
    pub canonical: bool,
    pub locale_code: LocaleCode,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBiographyResponseData {
    pub update_biography: Option<Biography>,
}
