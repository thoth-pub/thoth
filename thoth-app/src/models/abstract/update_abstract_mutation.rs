use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::locale::LocaleCode;
use thoth_api::model::r#abstract::Abstract;
use thoth_api::model::r#abstract::AbstractType;
use thoth_api::model::MarkupFormat;
use uuid::Uuid;

const UPDATE_ABSTRACT_MUTATION: &str = "
    mutation UpdateAbstract(
        $abstractId: Uuid!,
        $workId: Uuid!,
        $content: String!,
        $localeCode: LocaleCode!,
        $abstractType: AbstractType!,
        $canonical: Boolean!,
        $markupFormat: MarkupFormat!
    ) {
        updateAbstract(
            markupFormat: $markupFormat,
            data: {
                abstractId: $abstractId,
                workId: $workId,
                content: $content,
                localeCode: $localeCode,
                abstractType: $abstractType,
                canonical: $canonical
            }
        ){
            abstractId
            workId
            content
            localeCode
            abstractType
            canonical
        }
    }
";

graphql_query_builder! {
    UpdateAbstractRequest,
    UpdateAbstractRequestBody,
    Variables,
    UPDATE_ABSTRACT_MUTATION,
    UpdateAbstractResponseBody,
    UpdateAbstractResponseData,
    PushUpdateAbstract,
    PushActionUpdateAbstract
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub abstract_id: Uuid,
    pub work_id: Uuid,
    pub content: String,
    pub locale_code: LocaleCode,
    pub abstract_type: AbstractType,
    pub canonical: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAbstractResponseData {
    pub update_abstract: Option<Abstract>,
} 