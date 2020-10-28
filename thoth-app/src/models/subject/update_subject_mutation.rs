use serde::Deserialize;
use serde::Serialize;
use thoth_api::subject::model::SubjectType;

use super::Subject;

const UPDATE_SUBJECT_MUTATION: &str = "
    mutation UpdateSubject(
        $subjectId: Uuid!,
        $workId: Uuid!,
        $subjectType: SubjectType!,
        $subjectCode: String!,
        $subjectOrdinal: Int!,
    ) {
        updateSubject(data: {
            subjectId: $subjectId
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
    UpdateSubjectRequest,
    UpdateSubjectRequestBody,
    Variables,
    UPDATE_SUBJECT_MUTATION,
    UpdateSubjectResponseBody,
    UpdateSubjectResponseData,
    PushUpdateSubject,
    PushActionUpdateSubject
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub subject_id: String,
    pub work_id: String,
    pub subject_type: SubjectType,
    pub subject_code: String,
    pub subject_ordinal: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSubjectResponseData {
    pub update_subject: Option<Subject>,
}
