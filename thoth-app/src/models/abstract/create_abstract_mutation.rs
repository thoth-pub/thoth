use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::locale::LocaleCode;
use thoth_api::model::r#abstract::Abstract;
use thoth_api::model::r#abstract::AbstractType;
use thoth_api::model::MarkupFormat;
use uuid::Uuid;

const CREATE_ABSTRACT_MUTATION: &str = "
    mutation CreateAbstract(
        $workId: Uuid!,
        $content: String!,
        $localeCode: LocaleCode!,
        $abstractType: AbstractType!,
        $canonical: Boolean!,
        $markupFormat: MarkupFormat!
    ) {
        createAbstract(
            markupFormat: $markupFormat,
            data: {
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
    CreateAbstractRequest,
    CreateAbstractRequestBody,
    Variables,
    CREATE_ABSTRACT_MUTATION,
    CreateAbstractResponseBody,
    CreateAbstractResponseData,
    PushCreateAbstract,
    PushActionCreateAbstract
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub work_id: Uuid,
    pub content: String,
    pub locale_code: LocaleCode,
    pub abstract_type: AbstractType,
    pub canonical: bool,
    pub markup_format: MarkupFormat,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateAbstractResponseData {
    pub create_abstract: Option<Abstract>,
}
