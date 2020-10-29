use serde::Deserialize;
use serde::Serialize;
use thoth_api::subject::model::SubjectType;

use super::Subject;

const CREATE_SUBJECT_MUTATION: &str = "
    mutation CreateSubject(
        $workId: Uuid!,
        $subjectType: SubjectType!,
        $subjectCode: String!,
        $subjectOrdinal: Int!,
    ) {
        createSubject(data: {
            workId: $workId
            subjectType: $subjectType
            subjectCode: $subjectCode
            subjectOrdinal: $subjectOrdinal
        }){
            subjectId
            workId
            subjectType
            subjectCode
            subjectOrdinal
        }
    }
";

graphql_query_builder! {
    CreateSubjectRequest,
    CreateSubjectRequestBody,
    Variables,
    CREATE_SUBJECT_MUTATION,
    CreateSubjectResponseBody,
    CreateSubjectResponseData,
    PushCreateSubject,
    PushActionCreateSubject
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub work_id: String,
    pub subject_type: SubjectType,
    pub subject_code: String,
    pub subject_ordinal: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubjectResponseData {
    pub create_subject: Option<Subject>,
}
