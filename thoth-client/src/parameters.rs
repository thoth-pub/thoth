use crate::queries::{work_query, works_query};
use uuid::Uuid;

/// A set of booleans to toggle directives in the GraphQL queries
#[cfg_attr(test, derive(Debug, PartialEq))]
#[derive(Default)]
pub struct QueryParameters {
    with_issues: bool,
    with_languages: bool,
    with_publications: bool,
    with_subjects: bool,
    with_fundings: bool,
    with_relations: bool,
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
            parameters,
        }
    }
}

impl WorksQueryVariables {
    pub(crate) fn new(publishers: Option<Vec<Uuid>>, parameters: QueryParameters) -> Self {
        WorksQueryVariables {
            publishers,
            parameters,
        }
    }
}

/// Implement builder pattern for `QueryParameters`
///
/// # Example
///
/// ```
/// # use thoth_client::{QueryParameters};
///
/// # async fn run() -> QueryParameters {
/// let parameters = QueryParameters::new().with_series().with_languages();
/// # parameters
/// # }
/// ```
impl QueryParameters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_all(self) -> Self {
        self.with_issues()
            .with_languages()
            .with_publications()
            .with_subjects()
            .with_fundings()
            .with_relations()
    }

    pub fn with_issues(mut self) -> Self {
        self.with_issues = true;
        self
    }

    pub fn with_languages(mut self) -> Self {
        self.with_languages = true;
        self
    }

    pub fn with_publications(mut self) -> Self {
        self.with_publications = true;
        self
    }

    pub fn with_subjects(mut self) -> Self {
        self.with_subjects = true;
        self
    }

    pub fn with_fundings(mut self) -> Self {
        self.with_fundings = true;
        self
    }

    pub fn with_relations(mut self) -> Self {
        self.with_relations = true;
        self
    }

    pub fn without_issues(mut self) -> Self {
        self.with_issues = false;
        self
    }

    pub fn without_languages(mut self) -> Self {
        self.with_languages = false;
        self
    }

    pub fn without_publications(mut self) -> Self {
        self.with_publications = false;
        self
    }

    pub fn without_subjects(mut self) -> Self {
        self.with_subjects = false;
        self
    }

    pub fn without_fundings(mut self) -> Self {
        self.with_fundings = false;
        self
    }

    pub fn without_relations(mut self) -> Self {
        self.with_relations = false;
        self
    }
}

impl From<WorkQueryVariables> for work_query::Variables {
    fn from(v: WorkQueryVariables) -> Self {
        work_query::Variables {
            work_id: v.work_id,
            issues_limit: if v.parameters.with_issues { 99999 } else { 0 },
            languages_limit: if v.parameters.with_languages {
                99999
            } else {
                0
            },
            publications_limit: if v.parameters.with_publications {
                99999
            } else {
                0
            },
            subjects_limit: if v.parameters.with_subjects { 99999 } else { 0 },
            fundings_limit: if v.parameters.with_fundings { 99999 } else { 0 },
            relations_limit: if v.parameters.with_relations {
                99999
            } else {
                0
            },
        }
    }
}

impl From<WorksQueryVariables> for works_query::Variables {
    fn from(v: WorksQueryVariables) -> Self {
        works_query::Variables {
            publishers: v.publishers,
            issues_limit: if v.parameters.with_issues { 99999 } else { 0 },
            languages_limit: if v.parameters.with_languages {
                99999
            } else {
                0
            },
            publications_limit: if v.parameters.with_publications {
                99999
            } else {
                0
            },
            subjects_limit: if v.parameters.with_subjects { 99999 } else { 0 },
            fundings_limit: if v.parameters.with_fundings { 99999 } else { 0 },
            relations_limit: if v.parameters.with_relations {
                99999
            } else {
                0
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queries::{work_query, works_query};

    #[test]
    fn test_default_query_parameters() {
        let to_test = QueryParameters {
            with_issues: false,
            with_languages: false,
            with_publications: false,
            with_subjects: false,
            with_fundings: false,
            with_relations: false,
        };
        assert_eq!(to_test, QueryParameters::default());
        assert_eq!(to_test, QueryParameters::new())
    }

    #[test]
    fn test_query_parameters_builder() {
        assert_eq!(
            QueryParameters::new().with_all(),
            QueryParameters {
                with_issues: true,
                with_languages: true,
                with_publications: true,
                with_subjects: true,
                with_fundings: true,
                with_relations: true
            },
        );
        assert_eq!(
            QueryParameters::new()
                .with_all()
                .without_issues()
                .without_languages()
                .without_publications()
                .without_subjects()
                .without_fundings()
                .without_relations(),
            QueryParameters {
                with_issues: false,
                with_languages: false,
                with_publications: false,
                with_subjects: false,
                with_fundings: false,
                with_relations: false
            },
        );
        assert_eq!(
            QueryParameters::new()
                .with_issues()
                .with_languages()
                .with_publications()
                .with_subjects()
                .with_fundings()
                .with_relations(),
            QueryParameters {
                with_issues: true,
                with_languages: true,
                with_publications: true,
                with_subjects: true,
                with_fundings: true,
                with_relations: true
            },
        );
    }

    #[test]
    fn test_convert_parameters_to_work_query_variables() {
        let work_id: Uuid = Uuid::parse_str("00000000-0000-0000-AAAA-000000000001").unwrap();
        let mut parameters = QueryParameters::new().with_all();
        let mut variables: work_query::Variables =
            WorkQueryVariables::new(work_id, parameters).into();
        assert_eq!(
            variables,
            work_query::Variables {
                work_id,
                issues_limit: 99999,
                languages_limit: 99999,
                publications_limit: 99999,
                subjects_limit: 99999,
                fundings_limit: 99999,
                relations_limit: 99999,
            }
        );
        parameters = QueryParameters::new();
        variables = WorkQueryVariables::new(work_id, parameters).into();
        assert_eq!(
            variables,
            work_query::Variables {
                work_id,
                issues_limit: 0,
                languages_limit: 0,
                publications_limit: 0,
                subjects_limit: 0,
                fundings_limit: 0,
                relations_limit: 0,
            }
        );
        parameters = QueryParameters::new().with_all().without_relations();
        variables = WorkQueryVariables::new(work_id, parameters).into();
        assert_eq!(
            variables,
            work_query::Variables {
                work_id,
                issues_limit: 99999,
                languages_limit: 99999,
                publications_limit: 99999,
                subjects_limit: 99999,
                fundings_limit: 99999,
                relations_limit: 0,
            }
        );
    }

    #[test]
    fn test_convert_parameters_to_works_query_variables() {
        let publisher_id: Uuid = Uuid::parse_str("00000000-0000-0000-AAAA-000000000001").unwrap();
        let publishers = Some(vec![publisher_id]);
        let mut parameters = QueryParameters::new().with_all();
        let mut variables: works_query::Variables =
            WorksQueryVariables::new(publishers.clone(), parameters).into();
        assert_eq!(
            variables,
            works_query::Variables {
                publishers: publishers.clone(),
                issues_limit: 99999,
                languages_limit: 99999,
                publications_limit: 99999,
                subjects_limit: 99999,
                fundings_limit: 99999,
                relations_limit: 99999,
            }
        );
        parameters = QueryParameters::new();
        variables = WorksQueryVariables::new(publishers.clone(), parameters).into();
        assert_eq!(
            variables,
            works_query::Variables {
                publishers: publishers.clone(),
                issues_limit: 0,
                languages_limit: 0,
                publications_limit: 0,
                subjects_limit: 0,
                fundings_limit: 0,
                relations_limit: 0,
            }
        );
        parameters = QueryParameters::new().with_all().without_relations();
        variables = WorksQueryVariables::new(publishers.clone(), parameters).into();
        assert_eq!(
            variables,
            works_query::Variables {
                publishers: publishers.clone(),
                issues_limit: 99999,
                languages_limit: 99999,
                publications_limit: 99999,
                subjects_limit: 99999,
                fundings_limit: 99999,
                relations_limit: 0,
            }
        );
    }
}
