use crate::queries::{work_query, works_query};
use uuid::Uuid;

/// A set of booleans to toggle directives in the GraphQL queries
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
#[derive(Default, Clone, Copy)]
pub struct QueryParameters {
    with_issues: bool,
    with_languages: bool,
    with_publications: bool,
    with_subjects: bool,
    with_fundings: bool,
    with_relations: bool,
    with_references: bool,
    with_abstracts: bool,
    with_titles: bool,
}

/// An intermediate struct to parse QueryParameters into work_query::Variables
pub(crate) struct WorkQueryVariables {
    pub work_id: Uuid,
    pub parameters: QueryParameters,
}

/// An intermediate struct to parse QueryParameters into works_query::Variables
pub(crate) struct WorksQueryVariables {
    pub publishers: Option<Vec<Uuid>>,
    pub limit: i64,
    pub offset: i64,
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
    pub(crate) fn new(
        publishers: Option<Vec<Uuid>>,
        limit: i64,
        offset: i64,
        parameters: QueryParameters,
    ) -> Self {
        WorksQueryVariables {
            publishers,
            limit,
            offset,
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
/// let parameters = QueryParameters::new().with_issues().with_languages();
/// # parameters
/// # }
/// ```
impl QueryParameters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_all(self) -> Self {
        self.with_all_abstracts()
            .with_all_titles()
            .with_issues()
            .with_languages()
            .with_publications()
            .with_subjects()
            .with_fundings()
            .with_relations()
            .with_references()
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

    pub fn with_references(mut self) -> Self {
        self.with_references = true;
        self
    }

    pub fn with_all_abstracts(mut self) -> Self {
        self.with_abstracts = true;
        self
    }

    pub fn with_all_titles(mut self) -> Self {
        self.with_titles = true;
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

    pub fn without_references(mut self) -> Self {
        self.with_references = false;
        self
    }

    pub fn with_canonical_abstracts_only(mut self) -> Self {
        self.with_abstracts = false;
        self
    }

    pub fn with_canonical_title_only(mut self) -> Self {
        self.with_titles = false;
        self
    }
}

pub const FILTER_INCLUDE_ALL: i64 = 99999;
pub const FILTER_INCLUDE_NONE: i64 = 0;
/// For abstracts: fetch only canonical ones (typically 1 LONG and 1 SHORT)
pub const FILTER_INCLUDE_CANONICAL: i64 = 2;

impl From<WorkQueryVariables> for work_query::Variables {
    fn from(v: WorkQueryVariables) -> Self {
        work_query::Variables {
            work_id: v.work_id,
            abstracts_limit: if v.parameters.with_abstracts {
                FILTER_INCLUDE_ALL
            } else {
                FILTER_INCLUDE_CANONICAL
            },
            issues_limit: if v.parameters.with_issues {
                FILTER_INCLUDE_ALL
            } else {
                FILTER_INCLUDE_NONE
            },
            languages_limit: if v.parameters.with_languages {
                FILTER_INCLUDE_ALL
            } else {
                FILTER_INCLUDE_NONE
            },
            publications_limit: if v.parameters.with_publications {
                FILTER_INCLUDE_ALL
            } else {
                FILTER_INCLUDE_NONE
            },
            subjects_limit: if v.parameters.with_subjects {
                FILTER_INCLUDE_ALL
            } else {
                FILTER_INCLUDE_NONE
            },
            fundings_limit: if v.parameters.with_fundings {
                FILTER_INCLUDE_ALL
            } else {
                FILTER_INCLUDE_NONE
            },
            relations_limit: if v.parameters.with_relations {
                FILTER_INCLUDE_ALL
            } else {
                FILTER_INCLUDE_NONE
            },
            references_limit: if v.parameters.with_references {
                FILTER_INCLUDE_ALL
            } else {
                FILTER_INCLUDE_NONE
            },
            titles_limit: if v.parameters.with_titles {
                FILTER_INCLUDE_ALL
            } else {
                FILTER_INCLUDE_CANONICAL
            },
        }
    }
}

impl From<WorksQueryVariables> for works_query::Variables {
    fn from(v: WorksQueryVariables) -> Self {
        works_query::Variables {
            publishers: v.publishers,
            limit: v.limit,
            offset: v.offset,
            abstracts_limit: if v.parameters.with_abstracts {
                FILTER_INCLUDE_ALL
            } else {
                FILTER_INCLUDE_CANONICAL
            },
            issues_limit: if v.parameters.with_issues {
                FILTER_INCLUDE_ALL
            } else {
                FILTER_INCLUDE_NONE
            },
            languages_limit: if v.parameters.with_languages {
                FILTER_INCLUDE_ALL
            } else {
                FILTER_INCLUDE_NONE
            },
            publications_limit: if v.parameters.with_publications {
                FILTER_INCLUDE_ALL
            } else {
                FILTER_INCLUDE_NONE
            },
            subjects_limit: if v.parameters.with_subjects {
                FILTER_INCLUDE_ALL
            } else {
                FILTER_INCLUDE_NONE
            },
            fundings_limit: if v.parameters.with_fundings {
                FILTER_INCLUDE_ALL
            } else {
                FILTER_INCLUDE_NONE
            },
            relations_limit: if v.parameters.with_relations {
                FILTER_INCLUDE_ALL
            } else {
                FILTER_INCLUDE_NONE
            },
            references_limit: if v.parameters.with_references {
                FILTER_INCLUDE_ALL
            } else {
                FILTER_INCLUDE_NONE
            },
            titles_limit: if v.parameters.with_titles {
                FILTER_INCLUDE_ALL
            } else {
                FILTER_INCLUDE_CANONICAL
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
            with_references: false,
            with_abstracts: false,
            with_titles: false,
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
                with_relations: true,
                with_references: true,
                with_abstracts: true,
                with_titles: true,
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
                .without_relations()
                .without_references()
                .with_canonical_abstracts_only()
                .with_canonical_title_only(),
            QueryParameters {
                with_issues: false,
                with_languages: false,
                with_publications: false,
                with_subjects: false,
                with_fundings: false,
                with_relations: false,
                with_references: false,
                with_abstracts: false,
                with_titles: false,
            },
        );
        assert_eq!(
            QueryParameters::new()
                .with_issues()
                .with_languages()
                .with_publications()
                .with_subjects()
                .with_fundings()
                .with_relations()
                .with_references(),
            QueryParameters {
                with_issues: true,
                with_languages: true,
                with_publications: true,
                with_subjects: true,
                with_fundings: true,
                with_relations: true,
                with_references: true,
                with_abstracts: false,
                with_titles: false,
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
                abstracts_limit: FILTER_INCLUDE_ALL,
                issues_limit: FILTER_INCLUDE_ALL,
                languages_limit: FILTER_INCLUDE_ALL,
                publications_limit: FILTER_INCLUDE_ALL,
                subjects_limit: FILTER_INCLUDE_ALL,
                fundings_limit: FILTER_INCLUDE_ALL,
                relations_limit: FILTER_INCLUDE_ALL,
                references_limit: FILTER_INCLUDE_ALL,
                titles_limit: FILTER_INCLUDE_ALL,
            }
        );
        parameters = QueryParameters::new();
        variables = WorkQueryVariables::new(work_id, parameters).into();
        assert_eq!(
            variables,
            work_query::Variables {
                work_id,
                abstracts_limit: FILTER_INCLUDE_CANONICAL,
                issues_limit: FILTER_INCLUDE_NONE,
                languages_limit: FILTER_INCLUDE_NONE,
                publications_limit: FILTER_INCLUDE_NONE,
                subjects_limit: FILTER_INCLUDE_NONE,
                fundings_limit: FILTER_INCLUDE_NONE,
                relations_limit: FILTER_INCLUDE_NONE,
                references_limit: FILTER_INCLUDE_NONE,
                titles_limit: FILTER_INCLUDE_CANONICAL,
            }
        );
        parameters = QueryParameters::new().with_all().without_relations();
        variables = WorkQueryVariables::new(work_id, parameters).into();
        assert_eq!(
            variables,
            work_query::Variables {
                work_id,
                abstracts_limit: FILTER_INCLUDE_ALL,
                issues_limit: FILTER_INCLUDE_ALL,
                languages_limit: FILTER_INCLUDE_ALL,
                publications_limit: FILTER_INCLUDE_ALL,
                subjects_limit: FILTER_INCLUDE_ALL,
                fundings_limit: FILTER_INCLUDE_ALL,
                relations_limit: FILTER_INCLUDE_NONE,
                references_limit: FILTER_INCLUDE_ALL,
                titles_limit: FILTER_INCLUDE_ALL,
            }
        );
    }

    #[test]
    fn test_convert_parameters_to_works_query_variables() {
        let publisher_id: Uuid = Uuid::parse_str("00000000-0000-0000-AAAA-000000000001").unwrap();
        let publishers = Some(vec![publisher_id]);
        let mut parameters = QueryParameters::new().with_all();
        let mut variables: works_query::Variables =
            WorksQueryVariables::new(publishers.clone(), 100, 0, parameters).into();
        assert_eq!(
            variables,
            works_query::Variables {
                publishers: publishers.clone(),
                limit: 100,
                offset: 0,
                abstracts_limit: FILTER_INCLUDE_ALL,
                issues_limit: FILTER_INCLUDE_ALL,
                languages_limit: FILTER_INCLUDE_ALL,
                publications_limit: FILTER_INCLUDE_ALL,
                subjects_limit: FILTER_INCLUDE_ALL,
                fundings_limit: FILTER_INCLUDE_ALL,
                relations_limit: FILTER_INCLUDE_ALL,
                references_limit: FILTER_INCLUDE_ALL,
                titles_limit: FILTER_INCLUDE_ALL,
            }
        );
        parameters = QueryParameters::new();
        variables = WorksQueryVariables::new(publishers.clone(), 100, 0, parameters).into();
        assert_eq!(
            variables,
            works_query::Variables {
                publishers: publishers.clone(),
                limit: 100,
                offset: 0,
                abstracts_limit: FILTER_INCLUDE_CANONICAL,
                issues_limit: FILTER_INCLUDE_NONE,
                languages_limit: FILTER_INCLUDE_NONE,
                publications_limit: FILTER_INCLUDE_NONE,
                subjects_limit: FILTER_INCLUDE_NONE,
                fundings_limit: FILTER_INCLUDE_NONE,
                relations_limit: FILTER_INCLUDE_NONE,
                references_limit: FILTER_INCLUDE_NONE,
                titles_limit: FILTER_INCLUDE_CANONICAL,
            }
        );
        parameters = QueryParameters::new()
            .with_all()
            .without_relations()
            .without_references();
        variables = WorksQueryVariables::new(publishers.clone(), 100, 0, parameters).into();
        assert_eq!(
            variables,
            works_query::Variables {
                publishers,
                limit: 100,
                offset: 0,
                abstracts_limit: FILTER_INCLUDE_ALL,
                issues_limit: FILTER_INCLUDE_ALL,
                languages_limit: FILTER_INCLUDE_ALL,
                publications_limit: FILTER_INCLUDE_ALL,
                subjects_limit: FILTER_INCLUDE_ALL,
                fundings_limit: FILTER_INCLUDE_ALL,
                relations_limit: FILTER_INCLUDE_NONE,
                references_limit: FILTER_INCLUDE_NONE,
                titles_limit: FILTER_INCLUDE_ALL,
            }
        );
    }
}
