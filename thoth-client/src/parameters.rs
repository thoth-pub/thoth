use uuid::Uuid;
use crate::queries::{work_query, works_query};

/// A set of booleans to toggle directives in the GraphQL queries
pub struct QueryParameters {
    pub with_issues: bool,
    pub with_languages: bool,
    pub with_publications: bool,
    pub with_subjects: bool,
    pub with_fundings: bool,
    pub with_relations: bool,
}

/// An intermediate struct to parse QueryParameters into work_query::Variables
pub(crate) struct WorkQueryVariables {
    pub work_id: Uuid,
    pub parameters: QueryParameters,
}

/// An intermediate struct to parse QueryParameters into works_query::Variables
pub(crate) struct WorksQueryVariables {
    pub publishers: Option<Vec<Uuid>>,
    pub parameters: QueryParameters,
}

impl WorkQueryVariables {
    pub(crate) fn new(work_id: Uuid, parameters: QueryParameters) -> Self {
        WorkQueryVariables {
            work_id,
            parameters
        }
    }
}

impl WorksQueryVariables {
    pub(crate) fn new(publishers: Option<Vec<Uuid>>, parameters: QueryParameters) -> Self {
        WorksQueryVariables {
            publishers,
            parameters
        }
    }
}

impl QueryParameters {
    /// Get a `QueryParameters` with all its attributes set to `true`
    ///
    /// # Example
    ///
    /// ```
    /// # use thoth_client::{QueryParameters};
    ///
    /// # async fn run() -> QueryParameters {
    /// let parameters = QueryParameters::all_on();
    /// # parameters
    /// # }
    /// ```
    pub fn all_on() -> Self {
        QueryParameters {
            with_issues: true,
            with_languages: true,
            with_publications: true,
            with_subjects: true,
            with_fundings: true,
            with_relations: true,
        }
    }

    /// Get a `QueryParameters` with all its attributes set to `false`
    ///
    /// # Example
    ///
    /// ```
    /// # use thoth_client::{QueryParameters};
    ///
    /// # async fn run() -> QueryParameters {
    /// let parameters = QueryParameters::all_off();
    /// # parameters
    /// # }
    /// ```
    pub fn all_off() -> Self {
        QueryParameters {
            with_issues: false,
            with_languages: false,
            with_publications: false,
            with_subjects: false,
            with_fundings: false,
            with_relations: false,
        }
    }
}

impl From<WorkQueryVariables> for work_query::Variables {
    fn from(v: WorkQueryVariables) -> Self {
        work_query::Variables {
            work_id: v.work_id,
            issues_limit: if v.parameters.with_issues { 99999 } else { 0 },
            languages_limit: if v.parameters.with_languages { 99999 } else { 0 },
            publications_limit: if v.parameters.with_publications { 99999 } else { 0 },
            subjects_limit: if v.parameters.with_subjects { 99999 } else { 0 },
            fundings_limit: if v.parameters.with_fundings { 99999 } else { 0 },
            relations_limit: if v.parameters.with_relations { 99999 } else { 0 },
        }
    }
}

impl From<WorksQueryVariables> for works_query::Variables {
    fn from(v: WorksQueryVariables) -> Self {
        works_query::Variables {
            publishers: v.publishers,
            issues_limit: if v.parameters.with_issues { 99999 } else { 0 },
            languages_limit: if v.parameters.with_languages { 99999 } else { 0 },
            publications_limit: if v.parameters.with_publications { 99999 } else { 0 },
            subjects_limit: if v.parameters.with_subjects { 99999 } else { 0 },
            fundings_limit: if v.parameters.with_fundings { 99999 } else { 0 },
            relations_limit: if v.parameters.with_relations { 99999 } else { 0 },
        }
    }
}