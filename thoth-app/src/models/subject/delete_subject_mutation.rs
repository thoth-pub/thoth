use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::subject::Subject;
use uuid::Uuid;

const DELETE_SUBJECT_MUTATION: &str = "
    mutation DeleteSubject(
        $subjectId: Uuid!
    ) {
        deleteSubject(
            subjectId: $subjectId
        ){
            subjectId
            workId
            subjectType
            subjectCode
            subjectOrdinal
            createdAt
            updatedAt
        }
    }
";

graphql_query_builder! {
    DeleteSubjectRequest,
    DeleteSubjectRequestBody,
    Variables,
    DELETE_SUBJECT_MUTATION,
    DeleteSubjectResponseBody,
    DeleteSubjectResponseData,
    PushDeleteSubject,
    PushActionDeleteSubject
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub subject_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSubjectResponseData {
    pub delete_subject: Option<Subject>,
}
