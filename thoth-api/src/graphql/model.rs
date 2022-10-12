use chrono::naive::NaiveDate;
use juniper::FieldResult;
use juniper::RootNode;
use std::cell::Ref;
use std::sync::Arc;
use uuid::Uuid;

use crate::account::model::AccountAccess;
use crate::account::model::DecodedToken;
use crate::db::PgPool;
use crate::model::affiliation::*;
use crate::model::contribution::*;
use crate::model::contributor::*;
use crate::model::funding::*;
use crate::model::imprint::*;
use crate::model::institution::*;
use crate::model::issue::*;
use crate::model::language::*;
use crate::model::location::*;
use crate::model::price::*;
use crate::model::publication::crud::PublicationValidation;
use crate::model::publication::*;
use crate::model::publisher::*;
use crate::model::reference::*;
use crate::model::series::*;
use crate::model::subject::*;
use crate::model::work::*;
use crate::model::work_relation::*;
use crate::model::Convert;
use crate::model::Crud;
use crate::model::Doi;
use crate::model::Isbn;
use crate::model::LengthUnit;
use crate::model::Orcid;
use crate::model::Ror;
use crate::model::Timestamp;
use crate::model::WeightUnit;
use thoth_errors::{ThothError, ThothResult};

use super::utils::Direction;

impl juniper::Context for Context {}

#[derive(Clone)]
pub struct Context {
    pub db: Arc<PgPool>,
    pub account_access: AccountAccess,
    pub token: DecodedToken,
}

impl Context {
    pub fn new(pool: Arc<PgPool>, token: DecodedToken) -> Self {
        Self {
            db: pool,
            account_access: token.get_user_permissions(),
            token,
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting contributions list")]
pub struct ContributionOrderBy {
    pub field: ContributionField,
    pub direction: Direction,
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting issues list")]
pub struct IssueOrderBy {
    pub field: IssueField,
    pub direction: Direction,
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting languages list")]
pub struct LanguageOrderBy {
    pub field: LanguageField,
    pub direction: Direction,
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting prices list")]
pub struct PriceOrderBy {
    pub field: PriceField,
    pub direction: Direction,
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting subjects list")]
pub struct SubjectOrderBy {
    pub field: SubjectField,
    pub direction: Direction,
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting fundings list")]
pub struct FundingOrderBy {
    pub field: FundingField,
    pub direction: Direction,
}

pub struct QueryRoot;

#[juniper::object(Context = Context)]
impl QueryRoot {
    #[graphql(
    description="Query the full list of works",
    arguments(
        limit(
            default = 100,
            description = "The number of items to return"
        ),
        offset(
            default = 0,
            description = "The number of items to skip"
        ),
        filter(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_title, doi, reference, short_abstract, long_abstract, and landing_page"
        ),
        order(
            default = WorkOrderBy::default(),
            description = "The order in which to sort the results",
        ),
        publishers(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs",
        ),
        work_types(
            default = vec![],
            description = "Specific types to filter by",
        ),
        work_status(description = "A specific status to filter by"),
    )
  )]
    fn works(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: WorkOrderBy,
        publishers: Vec<Uuid>,
        work_types: Vec<WorkType>,
        work_status: Option<WorkStatus>,
    ) -> FieldResult<Vec<Work>> {
        Work::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            publishers,
            None,
            None,
            work_types,
            work_status,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single work using its id")]
    fn work(context: &Context, work_id: Uuid) -> FieldResult<Work> {
        Work::from_id(&context.db, &work_id).map_err(|e| e.into())
    }

    #[graphql(description = "Query a single work using its DOI")]
    fn work_by_doi(context: &Context, doi: Doi) -> FieldResult<Work> {
        Work::from_doi(&context.db, doi, vec![]).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get the total number of works",
        arguments(
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_title, doi, reference, short_abstract, long_abstract, and landing_page",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            work_types(
                default = vec![],
                description = "Specific types to filter by",
            ),
            work_status(description = "A specific status to filter by"),
        )
    )]
    fn work_count(
        context: &Context,
        filter: String,
        publishers: Vec<Uuid>,
        work_types: Vec<WorkType>,
        work_status: Option<WorkStatus>,
    ) -> FieldResult<i32> {
        Work::count(
            &context.db,
            Some(filter),
            publishers,
            work_types,
            work_status,
        )
        .map_err(|e| e.into())
    }

    #[graphql(
        description="Query the full list of books (a subset of the full list of works)",
        arguments(
            limit(
                default = 100,
                description = "The number of items to return"
            ),
            offset(
                default = 0,
                description = "The number of items to skip"
            ),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_title, doi, reference, short_abstract, long_abstract, and landing_page"
            ),
            order(
                default = WorkOrderBy::default(),
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            work_status(description = "A specific status to filter by"),
        )
    )]
    fn books(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: WorkOrderBy,
        publishers: Vec<Uuid>,
        work_status: Option<WorkStatus>,
    ) -> FieldResult<Vec<Work>> {
        Work::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            publishers,
            None,
            None,
            vec![
                WorkType::Monograph,
                WorkType::EditedBook,
                WorkType::Textbook,
                WorkType::JournalIssue,
            ],
            work_status,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single book using its DOI")]
    fn book_by_doi(context: &Context, doi: Doi) -> FieldResult<Work> {
        Work::from_doi(
            &context.db,
            doi,
            vec![
                WorkType::Monograph,
                WorkType::EditedBook,
                WorkType::Textbook,
                WorkType::JournalIssue,
            ],
        )
        .map_err(|e| e.into())
    }

    #[graphql(
        description = "Get the total number of books (a subset of the total number of works)",
        arguments(
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_title, doi, reference, short_abstract, long_abstract, and landing_page",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            work_status(description = "A specific status to filter by"),
        )
    )]
    fn book_count(
        context: &Context,
        filter: String,
        publishers: Vec<Uuid>,
        work_status: Option<WorkStatus>,
    ) -> FieldResult<i32> {
        Work::count(
            &context.db,
            Some(filter),
            publishers,
            vec![
                WorkType::Monograph,
                WorkType::EditedBook,
                WorkType::Textbook,
                WorkType::JournalIssue,
            ],
            work_status,
        )
        .map_err(|e| e.into())
    }

    #[graphql(
        description="Query the full list of chapters (a subset of the full list of works)",
        arguments(
            limit(
                default = 100,
                description = "The number of items to return"
            ),
            offset(
                default = 0,
                description = "The number of items to skip"
            ),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_title, doi, reference, short_abstract, long_abstract, and landing_page"
            ),
            order(
                default = WorkOrderBy::default(),
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            work_status(description = "A specific status to filter by"),
        )
    )]
    fn chapters(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: WorkOrderBy,
        publishers: Vec<Uuid>,
        work_status: Option<WorkStatus>,
    ) -> FieldResult<Vec<Work>> {
        Work::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            publishers,
            None,
            None,
            vec![WorkType::BookChapter],
            work_status,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single chapter using its DOI")]
    fn chapter_by_doi(context: &Context, doi: Doi) -> FieldResult<Work> {
        Work::from_doi(&context.db, doi, vec![WorkType::BookChapter]).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get the total number of chapters (a subset of the total number of works)",
        arguments(
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_title, doi, reference, short_abstract, long_abstract, and landing_page",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            work_status(description = "A specific status to filter by"),
        )
    )]
    fn chapter_count(
        context: &Context,
        filter: String,
        publishers: Vec<Uuid>,
        work_status: Option<WorkStatus>,
    ) -> FieldResult<i32> {
        Work::count(
            &context.db,
            Some(filter),
            publishers,
            vec![WorkType::BookChapter],
            work_status,
        )
        .map_err(|e| e.into())
    }

    #[graphql(
        description = "Query the full list of publications",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on isbn"
            ),
            order(
                default = PublicationOrderBy::default(),
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            publication_types(
                default = vec![],
                description = "Specific types to filter by",
            ),
        )
    )]
    fn publications(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: PublicationOrderBy,
        publishers: Vec<Uuid>,
        publication_types: Vec<PublicationType>,
    ) -> FieldResult<Vec<Publication>> {
        Publication::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            publishers,
            None,
            None,
            publication_types,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single publication using its id")]
    fn publication(context: &Context, publication_id: Uuid) -> FieldResult<Publication> {
        Publication::from_id(&context.db, &publication_id).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get the total number of publications",
        arguments(
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on isbn",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            publication_types(
                default = vec![],
                description = "Specific types to filter by",
            ),
        )
    )]
    fn publication_count(
        context: &Context,
        filter: String,
        publishers: Vec<Uuid>,
        publication_types: Vec<PublicationType>,
    ) -> FieldResult<i32> {
        Publication::count(
            &context.db,
            Some(filter),
            publishers,
            publication_types,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(
    description="Query the full list of publishers",
    arguments(
        limit(
            default = 100,
            description = "The number of items to return"
        ),
        offset(
            default = 0,
            description = "The number of items to skip"
        ),
        filter(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on publisher_name and publisher_shortname"

        ),
        order(
            default = PublisherOrderBy::default(),
            description = "The order in which to sort the results",
        ),
        publishers(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs",
        ),
    )
  )]
    fn publishers(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: PublisherOrderBy,
        publishers: Vec<Uuid>,
    ) -> FieldResult<Vec<Publisher>> {
        Publisher::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            publishers,
            None,
            None,
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single publisher using its id")]
    fn publisher(context: &Context, publisher_id: Uuid) -> FieldResult<Publisher> {
        Publisher::from_id(&context.db, &publisher_id).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get the total number of publishers",
        arguments(
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on publisher_name and publisher_shortname",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
        )
    )]
    fn publisher_count(
        context: &Context,
        filter: String,
        publishers: Vec<Uuid>,
    ) -> FieldResult<i32> {
        Publisher::count(&context.db, Some(filter), publishers, vec![], None).map_err(|e| e.into())
    }

    #[graphql(
        description = "Query the full list of imprints",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on imprint_name and imprint_url"
            ),
            order(
                default = ImprintOrderBy::default(),
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
        )
    )]
    fn imprints(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: ImprintOrderBy,
        publishers: Vec<Uuid>,
    ) -> FieldResult<Vec<Imprint>> {
        Imprint::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            publishers,
            None,
            None,
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single imprint using its id")]
    fn imprint(context: &Context, imprint_id: Uuid) -> FieldResult<Imprint> {
        Imprint::from_id(&context.db, &imprint_id).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get the total number of imprints",
        arguments(
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on imprint_name and imprint_url",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
        )
    )]
    fn imprint_count(context: &Context, filter: String, publishers: Vec<Uuid>) -> FieldResult<i32> {
        Imprint::count(&context.db, Some(filter), publishers, vec![], None).map_err(|e| e.into())
    }

    #[graphql(
        description = "Query the full list of contributors",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_name, last_name and orcid"
            ),
            order(
                default = ContributorOrderBy::default(),
                description = "The order in which to sort the results",
            ),
        )
    )]
    fn contributors(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: ContributorOrderBy,
    ) -> FieldResult<Vec<Contributor>> {
        Contributor::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            vec![],
            None,
            None,
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single contributor using its id")]
    fn contributor(context: &Context, contributor_id: Uuid) -> FieldResult<Contributor> {
        Contributor::from_id(&context.db, &contributor_id).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get the total number of contributors",
        arguments(
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_name, last_name and orcid",
            ),
        )
    )]
    fn contributor_count(context: &Context, filter: String) -> FieldResult<i32> {
        Contributor::count(&context.db, Some(filter), vec![], vec![], None).map_err(|e| e.into())
    }

    #[graphql(
        description = "Query the full list of contributions",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    ContributionOrderBy {
                        field: ContributionField::ContributionType,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            contribution_types(
                default = vec![],
                description = "Specific types to filter by",
            ),
        )
    )]
    fn contributions(
        context: &Context,
        limit: i32,
        offset: i32,
        order: ContributionOrderBy,
        publishers: Vec<Uuid>,
        contribution_types: Vec<ContributionType>,
    ) -> FieldResult<Vec<Contribution>> {
        Contribution::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            publishers,
            None,
            None,
            contribution_types,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single contribution using its id")]
    fn contribution(context: &Context, contribution_id: Uuid) -> FieldResult<Contribution> {
        Contribution::from_id(&context.db, &contribution_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of contributions",
        arguments(
            contribution_types(
                default = vec![],
                description = "Specific types to filter by",
            ),
        )
    )]
    fn contribution_count(
        context: &Context,
        contribution_types: Vec<ContributionType>,
    ) -> FieldResult<i32> {
        Contribution::count(&context.db, None, vec![], contribution_types, None)
            .map_err(|e| e.into())
    }

    #[graphql(
        description = "Query the full list of series",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on series_name, issn_print, issn_digital, series_url and series_description"
            ),
            order(
                default = SeriesOrderBy::default(),
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            series_types(
                default = vec![],
                description = "Specific types to filter by",
            ),
        ),
    )]
    fn serieses(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: SeriesOrderBy,
        publishers: Vec<Uuid>,
        series_types: Vec<SeriesType>,
    ) -> FieldResult<Vec<Series>> {
        Series::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            publishers,
            None,
            None,
            series_types,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single series using its id")]
    fn series(context: &Context, series_id: Uuid) -> FieldResult<Series> {
        Series::from_id(&context.db, &series_id).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get the total number of series",
        arguments(
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on series_name, issn_print, issn_digital, series_url and series_description",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            series_types(
                default = vec![],
                description = "Specific types to filter by",
            ),
        )
    )]
    fn series_count(
        context: &Context,
        filter: String,
        publishers: Vec<Uuid>,
        series_types: Vec<SeriesType>,
    ) -> FieldResult<i32> {
        Series::count(&context.db, Some(filter), publishers, series_types, None)
            .map_err(|e| e.into())
    }

    #[graphql(
        description = "Query the full list of issues",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    IssueOrderBy {
                        field: IssueField::IssueOrdinal,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
        )
    )]
    fn issues(
        context: &Context,
        limit: i32,
        offset: i32,
        order: IssueOrderBy,
        publishers: Vec<Uuid>,
    ) -> FieldResult<Vec<Issue>> {
        Issue::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            publishers,
            None,
            None,
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single issue using its id")]
    fn issue(context: &Context, issue_id: Uuid) -> FieldResult<Issue> {
        Issue::from_id(&context.db, &issue_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of issues")]
    fn issue_count(context: &Context) -> FieldResult<i32> {
        Issue::count(&context.db, None, vec![], vec![], None).map_err(|e| e.into())
    }

    #[graphql(
        description = "Query the full list of languages",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    LanguageOrderBy {
                        field: LanguageField::LanguageCode,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            language_codes(
                default = vec![],
                description = "Specific languages to filter by",
            ),
            language_relation(description = "A specific relation to filter by"),
        )
    )]
    fn languages(
        context: &Context,
        limit: i32,
        offset: i32,
        order: LanguageOrderBy,
        publishers: Vec<Uuid>,
        language_codes: Vec<LanguageCode>,
        language_relation: Option<LanguageRelation>,
    ) -> FieldResult<Vec<Language>> {
        Language::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            publishers,
            None,
            None,
            language_codes,
            language_relation,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single language using its id")]
    fn language(context: &Context, language_id: Uuid) -> FieldResult<Language> {
        Language::from_id(&context.db, &language_id).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get the total number of languages associated to works",
        arguments(
            language_codes(
                default = vec![],
                description = "Specific languages to filter by",
            ),
            language_relation(description = "A specific relation to filter by"),
        )
    )]
    fn language_count(
        context: &Context,
        language_codes: Vec<LanguageCode>,
        language_relation: Option<LanguageRelation>,
    ) -> FieldResult<i32> {
        Language::count(&context.db, None, vec![], language_codes, language_relation)
            .map_err(|e| e.into())
    }

    #[graphql(
        description = "Query the full list of locations",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    LocationOrderBy {
                        field: LocationField::LocationPlatform,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            location_platform(
                default = vec![],
                description = "Specific platforms to filter by"
            ),
        )
    )]
    fn locations(
        context: &Context,
        limit: i32,
        offset: i32,
        order: LocationOrderBy,
        publishers: Vec<Uuid>,
        location_platforms: Vec<LocationPlatform>,
    ) -> FieldResult<Vec<Location>> {
        Location::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            publishers,
            None,
            None,
            location_platforms,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single location using its id")]
    fn location(context: &Context, location_id: Uuid) -> FieldResult<Location> {
        Location::from_id(&context.db, &location_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of locations associated to works")]
    fn location_count(
        context: &Context,
        location_platforms: Vec<LocationPlatform>,
    ) -> FieldResult<i32> {
        Location::count(&context.db, None, vec![], location_platforms, None).map_err(|e| e.into())
    }

    #[graphql(
        description = "Query the full list of prices",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    PriceOrderBy {
                        field: PriceField::CurrencyCode,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            currency_codes(
                default = vec![],
                description = "Specific currencies to filter by",
            ),
        )
    )]
    fn prices(
        context: &Context,
        limit: i32,
        offset: i32,
        order: PriceOrderBy,
        publishers: Vec<Uuid>,
        currency_codes: Vec<CurrencyCode>,
    ) -> FieldResult<Vec<Price>> {
        Price::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            publishers,
            None,
            None,
            currency_codes,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single price using its id")]
    fn price(context: &Context, price_id: Uuid) -> FieldResult<Price> {
        Price::from_id(&context.db, &price_id).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get the total number of prices associated to works",
        arguments(
            currency_codes(
                default = vec![],
                description = "Specific currencies to filter by",
            ),
        )
    )]
    fn price_count(context: &Context, currency_codes: Vec<CurrencyCode>) -> FieldResult<i32> {
        Price::count(&context.db, None, vec![], currency_codes, None).map_err(|e| e.into())
    }

    #[graphql(
        description = "Query the full list of subjects",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on subject_code",
            ),
            order(
                default = {
                    SubjectOrderBy {
                        field: SubjectField::SubjectType,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            subject_types(
                default = vec![],
                description = "Specific types to filter by",
            ),
        )
    )]
    fn subjects(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: SubjectOrderBy,
        publishers: Vec<Uuid>,
        subject_types: Vec<SubjectType>,
    ) -> FieldResult<Vec<Subject>> {
        Subject::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            publishers,
            None,
            None,
            subject_types,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single subject using its id")]
    fn subject(context: &Context, subject_id: Uuid) -> FieldResult<Subject> {
        Subject::from_id(&context.db, &subject_id).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get the total number of subjects associated to works",
        arguments(
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on subject_code",
            ),
            subject_types(
                default = vec![],
                description = "Specific types to filter by",
            ),
        )
    )]
    fn subject_count(
        context: &Context,
        filter: String,
        subject_types: Vec<SubjectType>,
    ) -> FieldResult<i32> {
        Subject::count(&context.db, Some(filter), vec![], subject_types, None).map_err(|e| e.into())
    }

    #[graphql(
        description = "Query the full list of institutions",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on institution_name, ror and institution_doi",
            ),
            order(
                default = InstitutionOrderBy::default(),
                description = "The order in which to sort the results",
            ),
        )
    )]
    fn institutions(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: InstitutionOrderBy,
    ) -> FieldResult<Vec<Institution>> {
        Institution::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            vec![],
            None,
            None,
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single institution using its id")]
    fn institution(context: &Context, institution_id: Uuid) -> FieldResult<Institution> {
        Institution::from_id(&context.db, &institution_id).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get the total number of institutions",
        arguments(
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on institution_name, ror and institution_doi",
            ),
        )
    )]
    fn institution_count(context: &Context, filter: String) -> FieldResult<i32> {
        Institution::count(&context.db, Some(filter), vec![], vec![], None).map_err(|e| e.into())
    }

    #[graphql(
        description = "Query the full list of fundings",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    FundingOrderBy {
                        field: FundingField::Program,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
        )
    )]
    fn fundings(
        context: &Context,
        limit: i32,
        offset: i32,
        order: FundingOrderBy,
        publishers: Vec<Uuid>,
    ) -> FieldResult<Vec<Funding>> {
        Funding::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            publishers,
            None,
            None,
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single funding using its id")]
    fn funding(context: &Context, funding_id: Uuid) -> FieldResult<Funding> {
        Funding::from_id(&context.db, &funding_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of funding instances associated to works")]
    fn funding_count(context: &Context) -> FieldResult<i32> {
        Funding::count(&context.db, None, vec![], vec![], None).map_err(|e| e.into())
    }

    #[graphql(
        description = "Query the full list of affiliations",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    AffiliationOrderBy {
                        field: AffiliationField::AffiliationOrdinal,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
        )
    )]
    fn affiliations(
        context: &Context,
        limit: i32,
        offset: i32,
        order: AffiliationOrderBy,
        publishers: Vec<Uuid>,
    ) -> FieldResult<Vec<Affiliation>> {
        Affiliation::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            publishers,
            None,
            None,
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single affiliation using its id")]
    fn affiliation(context: &Context, affiliation_id: Uuid) -> FieldResult<Affiliation> {
        Affiliation::from_id(&context.db, &affiliation_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of affiliations")]
    fn affiliation_count(context: &Context) -> FieldResult<i32> {
        Affiliation::count(&context.db, None, vec![], vec![], None).map_err(|e| e.into())
    }
}

pub struct MutationRoot;

#[juniper::object(Context = Context)]
impl MutationRoot {
    fn create_work(context: &Context, data: NewWork) -> FieldResult<Work> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_imprint_id(&context.db, data.imprint_id)?)?;

        Work::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_publisher(context: &Context, data: NewPublisher) -> FieldResult<Publisher> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        // Only superusers can create new publishers - NewPublisher has no ID field
        if !context.account_access.is_superuser {
            return Err(ThothError::Unauthorised.into());
        }

        Publisher::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_imprint(context: &Context, data: NewImprint) -> FieldResult<Imprint> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context.account_access.can_edit(data.publisher_id)?;

        Imprint::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_contributor(context: &Context, data: NewContributor) -> FieldResult<Contributor> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        Contributor::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_contribution(context: &Context, data: NewContribution) -> FieldResult<Contribution> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;

        Contribution::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_publication(context: &Context, data: NewPublication) -> FieldResult<Publication> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;

        data.validate(&context.db)?;

        Publication::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_series(context: &Context, data: NewSeries) -> FieldResult<Series> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_imprint_id(&context.db, data.imprint_id)?)?;

        Series::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_issue(context: &Context, data: NewIssue) -> FieldResult<Issue> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;

        data.imprints_match(&context.db)?;

        Issue::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_language(context: &Context, data: NewLanguage) -> FieldResult<Language> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;

        Language::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_institution(context: &Context, data: NewInstitution) -> FieldResult<Institution> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        Institution::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_funding(context: &Context, data: NewFunding) -> FieldResult<Funding> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;

        Funding::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_location(context: &Context, data: NewLocation) -> FieldResult<Location> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_publication_id(
                &context.db,
                data.publication_id,
            )?)?;

        if data.canonical {
            data.canonical_record_complete(&context.db)?;
        } else {
            data.can_be_non_canonical(&context.db)?;
        }

        Location::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_price(context: &Context, data: NewPrice) -> FieldResult<Price> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_publication_id(
                &context.db,
                data.publication_id,
            )?)?;

        if data.unit_price <= 0.0 {
            // Prices must be non-zero (and non-negative).
            return Err(ThothError::PriceZeroError.into());
        }

        Price::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_subject(context: &Context, data: NewSubject) -> FieldResult<Subject> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;

        check_subject(&data.subject_type, &data.subject_code)?;

        Subject::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_affiliation(context: &Context, data: NewAffiliation) -> FieldResult<Affiliation> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_contribution_id(
                &context.db,
                data.contribution_id,
            )?)?;

        Affiliation::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_work_relation(context: &Context, data: NewWorkRelation) -> FieldResult<WorkRelation> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        // Work relations may link works from different publishers.
        // User must have permissions for all relevant publishers.
        context.account_access.can_edit(publisher_id_from_work_id(
            &context.db,
            data.relator_work_id,
        )?)?;
        context.account_access.can_edit(publisher_id_from_work_id(
            &context.db,
            data.related_work_id,
        )?)?;

        WorkRelation::create(&context.db, &data).map_err(|e| e.into())
    }

    fn create_reference(context: &Context, data: NewReference) -> FieldResult<Reference> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;

        Reference::create(&context.db, &data).map_err(|e| e.into())
    }

    fn update_work(context: &Context, data: PatchWork) -> FieldResult<Work> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let work = Work::from_id(&context.db, &data.work_id).unwrap();
        context
            .account_access
            .can_edit(work.publisher_id(&context.db)?)?;

        if data.imprint_id != work.imprint_id {
            context
                .account_access
                .can_edit(publisher_id_from_imprint_id(&context.db, data.imprint_id)?)?;
            work.can_update_imprint(&context.db)?;
        }

        if data.work_type == WorkType::BookChapter {
            work.can_be_chapter(&context.db)?;
        }

        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        work.update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_publisher(context: &Context, data: PatchPublisher) -> FieldResult<Publisher> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let publisher = Publisher::from_id(&context.db, &data.publisher_id).unwrap();
        context.account_access.can_edit(publisher.publisher_id)?;

        if data.publisher_id != publisher.publisher_id {
            context.account_access.can_edit(data.publisher_id)?;
        }
        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        publisher
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_imprint(context: &Context, data: PatchImprint) -> FieldResult<Imprint> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let imprint = Imprint::from_id(&context.db, &data.imprint_id).unwrap();
        context
            .account_access
            .can_edit(imprint.publisher_id(&context.db)?)?;

        if data.publisher_id != imprint.publisher_id {
            context.account_access.can_edit(data.publisher_id)?;
        }
        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        imprint
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_contributor(context: &Context, data: PatchContributor) -> FieldResult<Contributor> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        Contributor::from_id(&context.db, &data.contributor_id)
            .unwrap()
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_contribution(
        context: &Context,
        data: PatchContribution,
    ) -> FieldResult<Contribution> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let contribution = Contribution::from_id(&context.db, &data.contribution_id).unwrap();
        context
            .account_access
            .can_edit(contribution.publisher_id(&context.db)?)?;

        if data.work_id != contribution.work_id {
            context
                .account_access
                .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;
        }
        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        contribution
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_publication(context: &Context, data: PatchPublication) -> FieldResult<Publication> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let publication = Publication::from_id(&context.db, &data.publication_id).unwrap();
        context
            .account_access
            .can_edit(publication.publisher_id(&context.db)?)?;

        if data.work_id != publication.work_id {
            context
                .account_access
                .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;
        }

        data.validate(&context.db)?;

        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        publication
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_series(context: &Context, data: PatchSeries) -> FieldResult<Series> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let series = Series::from_id(&context.db, &data.series_id).unwrap();
        context
            .account_access
            .can_edit(series.publisher_id(&context.db)?)?;

        if data.imprint_id != series.imprint_id {
            context
                .account_access
                .can_edit(publisher_id_from_imprint_id(&context.db, data.imprint_id)?)?;
        }
        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        series
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_issue(context: &Context, data: PatchIssue) -> FieldResult<Issue> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let issue = Issue::from_id(&context.db, &data.issue_id).unwrap();
        context
            .account_access
            .can_edit(issue.publisher_id(&context.db)?)?;

        data.imprints_match(&context.db)?;

        if data.work_id != issue.work_id {
            context
                .account_access
                .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;
        }
        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        issue
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_language(context: &Context, data: PatchLanguage) -> FieldResult<Language> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let language = Language::from_id(&context.db, &data.language_id).unwrap();
        context
            .account_access
            .can_edit(language.publisher_id(&context.db)?)?;

        if data.work_id != language.work_id {
            context
                .account_access
                .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;
        }

        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        language
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_institution(context: &Context, data: PatchInstitution) -> FieldResult<Institution> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        Institution::from_id(&context.db, &data.institution_id)
            .unwrap()
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_funding(context: &Context, data: PatchFunding) -> FieldResult<Funding> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let funding = Funding::from_id(&context.db, &data.funding_id).unwrap();
        context
            .account_access
            .can_edit(funding.publisher_id(&context.db)?)?;

        if data.work_id != funding.work_id {
            context
                .account_access
                .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;
        }

        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        funding
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_location(context: &Context, data: PatchLocation) -> FieldResult<Location> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let location = Location::from_id(&context.db, &data.location_id).unwrap();
        context
            .account_access
            .can_edit(location.publisher_id(&context.db)?)?;

        if data.publication_id != location.publication_id {
            context
                .account_access
                .can_edit(publisher_id_from_publication_id(
                    &context.db,
                    data.publication_id,
                )?)?;
        }

        if data.canonical != location.canonical {
            // Each publication must have exactly one canonical location.
            // Updating an existing location would always violate this,
            // as it should always result in either zero or two canonical locations.
            return Err(ThothError::CanonicalLocationError.into());
        }

        if data.canonical {
            data.canonical_record_complete(&context.db)?;
        }

        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        location
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_price(context: &Context, data: PatchPrice) -> FieldResult<Price> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let price = Price::from_id(&context.db, &data.price_id).unwrap();
        context
            .account_access
            .can_edit(price.publisher_id(&context.db)?)?;

        if data.publication_id != price.publication_id {
            context
                .account_access
                .can_edit(publisher_id_from_publication_id(
                    &context.db,
                    data.publication_id,
                )?)?;
        }

        if data.unit_price <= 0.0 {
            // Prices must be non-zero (and non-negative).
            return Err(ThothError::PriceZeroError.into());
        }

        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        price
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_subject(context: &Context, data: PatchSubject) -> FieldResult<Subject> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let subject = Subject::from_id(&context.db, &data.subject_id).unwrap();
        context
            .account_access
            .can_edit(subject.publisher_id(&context.db)?)?;

        if data.work_id != subject.work_id {
            context
                .account_access
                .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;
        }

        check_subject(&data.subject_type, &data.subject_code)?;

        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        subject
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_affiliation(context: &Context, data: PatchAffiliation) -> FieldResult<Affiliation> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let affiliation = Affiliation::from_id(&context.db, &data.affiliation_id).unwrap();
        context
            .account_access
            .can_edit(affiliation.publisher_id(&context.db)?)?;

        if data.contribution_id != affiliation.contribution_id {
            context
                .account_access
                .can_edit(publisher_id_from_contribution_id(
                    &context.db,
                    data.contribution_id,
                )?)?;
        }

        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        affiliation
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_work_relation(
        context: &Context,
        data: PatchWorkRelation,
    ) -> FieldResult<WorkRelation> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let work_relation = WorkRelation::from_id(&context.db, &data.work_relation_id).unwrap();
        // Work relations may link works from different publishers.
        // User must have permissions for all relevant publishers.
        context.account_access.can_edit(publisher_id_from_work_id(
            &context.db,
            work_relation.relator_work_id,
        )?)?;
        context.account_access.can_edit(publisher_id_from_work_id(
            &context.db,
            work_relation.related_work_id,
        )?)?;

        if data.relator_work_id != work_relation.relator_work_id {
            context.account_access.can_edit(publisher_id_from_work_id(
                &context.db,
                data.relator_work_id,
            )?)?;
        }
        if data.related_work_id != work_relation.related_work_id {
            context.account_access.can_edit(publisher_id_from_work_id(
                &context.db,
                data.related_work_id,
            )?)?;
        }

        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        work_relation
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn update_reference(context: &Context, data: PatchReference) -> FieldResult<Reference> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let reference = Reference::from_id(&context.db, &data.reference_id).unwrap();
        context
            .account_access
            .can_edit(reference.publisher_id(&context.db)?)?;

        if data.work_id != reference.work_id {
            context
                .account_access
                .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;
        }

        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        reference
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    fn delete_work(context: &Context, work_id: Uuid) -> FieldResult<Work> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let work = Work::from_id(&context.db, &work_id).unwrap();
        context
            .account_access
            .can_edit(work.publisher_id(&context.db)?)?;

        work.delete(&context.db).map_err(|e| e.into())
    }

    fn delete_publisher(context: &Context, publisher_id: Uuid) -> FieldResult<Publisher> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let publisher = Publisher::from_id(&context.db, &publisher_id).unwrap();
        context.account_access.can_edit(publisher_id)?;

        publisher.delete(&context.db).map_err(|e| e.into())
    }

    fn delete_imprint(context: &Context, imprint_id: Uuid) -> FieldResult<Imprint> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let imprint = Imprint::from_id(&context.db, &imprint_id).unwrap();
        context
            .account_access
            .can_edit(imprint.publisher_id(&context.db)?)?;

        imprint.delete(&context.db).map_err(|e| e.into())
    }

    fn delete_contributor(context: &Context, contributor_id: Uuid) -> FieldResult<Contributor> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        Contributor::from_id(&context.db, &contributor_id)
            .unwrap()
            .delete(&context.db)
            .map_err(|e| e.into())
    }

    fn delete_contribution(context: &Context, contribution_id: Uuid) -> FieldResult<Contribution> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let contribution = Contribution::from_id(&context.db, &contribution_id).unwrap();
        context
            .account_access
            .can_edit(contribution.publisher_id(&context.db)?)?;

        contribution.delete(&context.db).map_err(|e| e.into())
    }

    fn delete_publication(context: &Context, publication_id: Uuid) -> FieldResult<Publication> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let publication = Publication::from_id(&context.db, &publication_id).unwrap();
        context
            .account_access
            .can_edit(publication.publisher_id(&context.db)?)?;

        publication.delete(&context.db).map_err(|e| e.into())
    }

    fn delete_series(context: &Context, series_id: Uuid) -> FieldResult<Series> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let series = Series::from_id(&context.db, &series_id).unwrap();
        context
            .account_access
            .can_edit(series.publisher_id(&context.db)?)?;

        series.delete(&context.db).map_err(|e| e.into())
    }

    fn delete_issue(context: &Context, issue_id: Uuid) -> FieldResult<Issue> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let issue = Issue::from_id(&context.db, &issue_id).unwrap();
        context
            .account_access
            .can_edit(issue.publisher_id(&context.db)?)?;

        issue.delete(&context.db).map_err(|e| e.into())
    }

    fn delete_language(context: &Context, language_id: Uuid) -> FieldResult<Language> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let language = Language::from_id(&context.db, &language_id).unwrap();
        context
            .account_access
            .can_edit(language.publisher_id(&context.db)?)?;

        language.delete(&context.db).map_err(|e| e.into())
    }

    fn delete_institution(context: &Context, institution_id: Uuid) -> FieldResult<Institution> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        Institution::from_id(&context.db, &institution_id)
            .unwrap()
            .delete(&context.db)
            .map_err(|e| e.into())
    }

    fn delete_funding(context: &Context, funding_id: Uuid) -> FieldResult<Funding> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let funding = Funding::from_id(&context.db, &funding_id).unwrap();
        context
            .account_access
            .can_edit(funding.publisher_id(&context.db)?)?;

        funding.delete(&context.db).map_err(|e| e.into())
    }

    fn delete_location(context: &Context, location_id: Uuid) -> FieldResult<Location> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let location = Location::from_id(&context.db, &location_id).unwrap();
        context
            .account_access
            .can_edit(location.publisher_id(&context.db)?)?;

        location.delete(&context.db).map_err(|e| e.into())
    }

    fn delete_price(context: &Context, price_id: Uuid) -> FieldResult<Price> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let price = Price::from_id(&context.db, &price_id).unwrap();
        context
            .account_access
            .can_edit(price.publisher_id(&context.db)?)?;

        price.delete(&context.db).map_err(|e| e.into())
    }

    fn delete_subject(context: &Context, subject_id: Uuid) -> FieldResult<Subject> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let subject = Subject::from_id(&context.db, &subject_id).unwrap();
        context
            .account_access
            .can_edit(subject.publisher_id(&context.db)?)?;

        subject.delete(&context.db).map_err(|e| e.into())
    }

    fn delete_affiliation(context: &Context, affiliation_id: Uuid) -> FieldResult<Affiliation> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let affiliation = Affiliation::from_id(&context.db, &affiliation_id).unwrap();
        context
            .account_access
            .can_edit(affiliation.publisher_id(&context.db)?)?;

        affiliation.delete(&context.db).map_err(|e| e.into())
    }

    fn delete_work_relation(
        context: &Context,
        work_relation_id: Uuid,
    ) -> FieldResult<WorkRelation> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let work_relation = WorkRelation::from_id(&context.db, &work_relation_id).unwrap();
        // Work relations may link works from different publishers.
        // User must have permissions for all relevant publishers.
        context.account_access.can_edit(publisher_id_from_work_id(
            &context.db,
            work_relation.relator_work_id,
        )?)?;
        context.account_access.can_edit(publisher_id_from_work_id(
            &context.db,
            work_relation.related_work_id,
        )?)?;

        work_relation.delete(&context.db).map_err(|e| e.into())
    }

    fn delete_reference(context: &Context, reference_id: Uuid) -> FieldResult<Reference> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let reference = Reference::from_id(&context.db, &reference_id).unwrap();
        context
            .account_access
            .can_edit(reference.publisher_id(&context.db)?)?;

        reference.delete(&context.db).map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "A written text that can be published")]
impl Work {
    pub fn work_id(&self) -> &Uuid {
        &self.work_id
    }

    pub fn work_type(&self) -> &WorkType {
        &self.work_type
    }

    pub fn work_status(&self) -> &WorkStatus {
        &self.work_status
    }

    #[graphql(description = "Concatenation of title and subtitle with punctuation mark")]
    pub fn full_title(&self) -> &str {
        self.full_title.as_str()
    }

    #[graphql(description = "Main title of the work (excluding subtitle)")]
    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    #[graphql(description = "Secondary title of the work (excluding main title)")]
    pub fn subtitle(&self) -> Option<&String> {
        self.subtitle.as_ref()
    }

    #[graphql(description = "Internal reference code")]
    pub fn reference(&self) -> Option<&String> {
        self.reference.as_ref()
    }

    #[graphql(description = "Edition number of the work (not applicable to chapters)")]
    pub fn edition(&self) -> Option<&i32> {
        self.edition.as_ref()
    }

    pub fn imprint_id(&self) -> Uuid {
        self.imprint_id
    }

    #[graphql(
        description = "Digital Object Identifier of the work as full URL. It must use the HTTPS scheme and the doi.org domain (e.g. https://doi.org/10.11647/obp.0001)"
    )]
    pub fn doi(&self) -> Option<&Doi> {
        self.doi.as_ref()
    }

    pub fn publication_date(&self) -> Option<NaiveDate> {
        self.publication_date
    }

    pub fn place(&self) -> Option<&String> {
        self.place.as_ref()
    }

    pub fn page_count(&self) -> Option<&i32> {
        self.page_count.as_ref()
    }

    pub fn page_breakdown(&self) -> Option<&String> {
        self.page_breakdown.as_ref()
    }

    pub fn image_count(&self) -> Option<&i32> {
        self.image_count.as_ref()
    }

    pub fn table_count(&self) -> Option<&i32> {
        self.table_count.as_ref()
    }

    pub fn audio_count(&self) -> Option<&i32> {
        self.audio_count.as_ref()
    }

    pub fn video_count(&self) -> Option<&i32> {
        self.video_count.as_ref()
    }

    pub fn license(&self) -> Option<&String> {
        self.license.as_ref()
    }

    pub fn copyright_holder(&self) -> Option<&String> {
        self.copyright_holder.as_ref()
    }

    pub fn landing_page(&self) -> Option<&String> {
        self.landing_page.as_ref()
    }

    #[graphql(
        description = "Library of Congress Control Number of the work (not applicable to chapters)"
    )]
    pub fn lccn(&self) -> Option<&String> {
        self.lccn.as_ref()
    }

    #[graphql(
        description = "OCLC (WorldCat) Control Number of the work (not applicable to chapters)"
    )]
    pub fn oclc(&self) -> Option<&String> {
        self.oclc.as_ref()
    }

    pub fn short_abstract(&self) -> Option<&String> {
        self.short_abstract.as_ref()
    }

    pub fn long_abstract(&self) -> Option<&String> {
        self.long_abstract.as_ref()
    }

    pub fn general_note(&self) -> Option<&String> {
        self.general_note.as_ref()
    }

    #[graphql(description = "Table of contents of the work (not applicable to chapters)")]
    pub fn toc(&self) -> Option<&String> {
        self.toc.as_ref()
    }

    pub fn cover_url(&self) -> Option<&String> {
        self.cover_url.as_ref()
    }

    pub fn cover_caption(&self) -> Option<&String> {
        self.cover_caption.as_ref()
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at.clone()
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at.clone()
    }

    #[graphql(description = "Page number on which the work begins (only applicable to chapters)")]
    pub fn first_page(&self) -> Option<&String> {
        self.first_page.as_ref()
    }

    #[graphql(description = "Page number on which the work ends (only applicable to chapters)")]
    pub fn last_page(&self) -> Option<&String> {
        self.last_page.as_ref()
    }

    #[graphql(
        description = "Concatenation of first page and last page with dash (only applicable to chapters)"
    )]
    pub fn page_interval(&self) -> Option<&String> {
        self.page_interval.as_ref()
    }

    pub fn imprint(&self, context: &Context) -> FieldResult<Imprint> {
        Imprint::from_id(&context.db, &self.imprint_id).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get contributions linked to this work",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    ContributionOrderBy {
                        field: ContributionField::ContributionType,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            contribution_types(
                default = vec![],
                description = "Specific types to filter by",
            ),
        )
    )]
    pub fn contributions(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        order: ContributionOrderBy,
        contribution_types: Vec<ContributionType>,
    ) -> FieldResult<Vec<Contribution>> {
        Contribution::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            vec![],
            Some(self.work_id),
            None,
            contribution_types,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(
        description = "Get languages linked to this work",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    LanguageOrderBy {
                        field: LanguageField::LanguageCode,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            language_codes(
                default = vec![],
                description = "Specific languages to filter by",
            ),
            language_relation(description = "A specific relation to filter by"),
        )
    )]
    pub fn languages(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        order: LanguageOrderBy,
        language_codes: Vec<LanguageCode>,
        language_relation: Option<LanguageRelation>,
    ) -> FieldResult<Vec<Language>> {
        Language::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            vec![],
            Some(self.work_id),
            None,
            language_codes,
            language_relation,
        )
        .map_err(|e| e.into())
    }

    #[graphql(
        description = "Get publications linked to this work",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on isbn"
            ),
            order(
                default = {
                    PublicationOrderBy {
                        field: PublicationField::PublicationType,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            publication_types(
                default = vec![],
                description = "Specific types to filter by",
            ),
        )
    )]
    pub fn publications(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: PublicationOrderBy,
        publication_types: Vec<PublicationType>,
    ) -> FieldResult<Vec<Publication>> {
        Publication::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            vec![],
            Some(self.work_id),
            None,
            publication_types,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(
        description = "Get subjects linked to this work",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on subject_code",
            ),
            order(
                default = {
                    SubjectOrderBy {
                        field: SubjectField::SubjectType,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            subject_types(
                default = vec![],
                description = "Specific types to filter by",
            ),
        )
    )]
    pub fn subjects(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: SubjectOrderBy,
        subject_types: Vec<SubjectType>,
    ) -> FieldResult<Vec<Subject>> {
        Subject::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            vec![],
            Some(self.work_id),
            None,
            subject_types,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(
        description = "Get fundings linked to this work",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    FundingOrderBy {
                        field: FundingField::Program,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
        )
    )]
    pub fn fundings(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        order: FundingOrderBy,
    ) -> FieldResult<Vec<Funding>> {
        Funding::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            vec![],
            Some(self.work_id),
            None,
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(
        description = "Get issues linked to this work",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    IssueOrderBy {
                        field: IssueField::IssueOrdinal,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
        )
    )]
    pub fn issues(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        order: IssueOrderBy,
    ) -> FieldResult<Vec<Issue>> {
        Issue::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            vec![],
            Some(self.work_id),
            None,
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }
    #[graphql(
        description = "Get other works related to this work",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = WorkRelationOrderBy::default(),
                description = "The order in which to sort the results",
            ),
            relation_types(
                default = vec![],
                description = "Specific types to filter by",
            ),
        )
    )]
    pub fn relations(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        order: WorkRelationOrderBy,
        relation_types: Vec<RelationType>,
    ) -> FieldResult<Vec<WorkRelation>> {
        WorkRelation::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            vec![],
            Some(self.work_id),
            None,
            relation_types,
            None,
        )
        .map_err(|e| e.into())
    }
    #[graphql(
        description = "Get references cited by this work",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on doi, unstructured_citation, issn, isbn, journal_title, article_title, series_title, volume_title, author, standard_designator, standards_body_name, and standards_body_acronym",
            ),
            order(
                default = ReferenceOrderBy::default(),
                description = "The order in which to sort the results",
            ),
        )
    )]
    pub fn references(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        order: ReferenceOrderBy,
    ) -> FieldResult<Vec<Reference>> {
        Reference::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            publishers,
            None,
            None,
            None,
            None,
        )
        .map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "A manifestation of a written text")]
impl Publication {
    pub fn publication_id(&self) -> Uuid {
        self.publication_id
    }

    pub fn publication_type(&self) -> &PublicationType {
        &self.publication_type
    }

    pub fn work_id(&self) -> Uuid {
        self.work_id
    }

    pub fn isbn(&self) -> Option<&Isbn> {
        self.isbn.as_ref()
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at.clone()
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at.clone()
    }

    #[graphql(
        description = "Width of the physical Publication (in mm, cm or in) (only applicable to non-Chapter Paperbacks and Hardbacks)",
        arguments(
            units(
                default = LengthUnit::default(),
                description = "Unit of measurement in which to represent the width (mm, cm or in)",
            ),
        )
    )]
    pub fn width(&self, units: LengthUnit) -> Option<f64> {
        match units {
            LengthUnit::Mm => self.width_mm,
            LengthUnit::Cm => self
                .width_mm
                .map(|w| w.convert_length_from_to(&LengthUnit::Mm, &LengthUnit::Cm)),
            LengthUnit::In => self.width_in,
        }
    }

    #[graphql(
        description = "Height of the physical Publication (in mm, cm or in) (only applicable to non-Chapter Paperbacks and Hardbacks)",
        arguments(
            units(
                default = LengthUnit::default(),
                description = "Unit of measurement in which to represent the height (mm, cm or in)",
            ),
        )
    )]
    pub fn height(&self, units: LengthUnit) -> Option<f64> {
        match units {
            LengthUnit::Mm => self.height_mm,
            LengthUnit::Cm => self
                .height_mm
                .map(|w| w.convert_length_from_to(&LengthUnit::Mm, &LengthUnit::Cm)),
            LengthUnit::In => self.height_in,
        }
    }

    #[graphql(
        description = "Depth of the physical Publication (in mm, cm or in) (only applicable to non-Chapter Paperbacks and Hardbacks)",
        arguments(
            units(
                default = LengthUnit::default(),
                description = "Unit of measurement in which to represent the depth (mm, cm or in)",
            ),
        )
    )]
    pub fn depth(&self, units: LengthUnit) -> Option<f64> {
        match units {
            LengthUnit::Mm => self.depth_mm,
            LengthUnit::Cm => self
                .depth_mm
                .map(|w| w.convert_length_from_to(&LengthUnit::Mm, &LengthUnit::Cm)),
            LengthUnit::In => self.depth_in,
        }
    }

    #[graphql(
        description = "Weight of the physical Publication (in g or oz) (only applicable to non-Chapter Paperbacks and Hardbacks)",
        arguments(
            units(
                default = WeightUnit::default(),
                description = "Unit of measurement in which to represent the weight (grams or ounces)",
            ),
        )
    )]
    pub fn weight(&self, units: WeightUnit) -> Option<f64> {
        match units {
            WeightUnit::G => self.weight_g,
            WeightUnit::Oz => self.weight_oz,
        }
    }

    #[graphql(
        description = "Get prices linked to this publication",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    PriceOrderBy {
                        field: PriceField::CurrencyCode,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            currency_codes(
                default = vec![],
                description = "Specific currencies to filter by",
            ),
        )
    )]
    pub fn prices(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        order: PriceOrderBy,
        currency_codes: Vec<CurrencyCode>,
    ) -> FieldResult<Vec<Price>> {
        Price::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            vec![],
            Some(self.publication_id),
            None,
            currency_codes,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(
        description = "Get locations linked to this publication",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    LocationOrderBy {
                        field: LocationField::LocationPlatform,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            location_platforms(
                default = vec![],
                description = "Specific platforms to filter by",
            ),
        )
    )]
    pub fn locations(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        order: LocationOrderBy,
        location_platforms: Vec<LocationPlatform>,
    ) -> FieldResult<Vec<Location>> {
        Location::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            vec![],
            Some(self.publication_id),
            None,
            location_platforms,
            None,
        )
        .map_err(|e| e.into())
    }

    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "An organisation that produces and distributes written texts.")]
impl Publisher {
    pub fn publisher_id(&self) -> Uuid {
        self.publisher_id
    }

    pub fn publisher_name(&self) -> &String {
        &self.publisher_name
    }

    pub fn publisher_shortname(&self) -> Option<&String> {
        self.publisher_shortname.as_ref()
    }

    pub fn publisher_url(&self) -> Option<&String> {
        self.publisher_url.as_ref()
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at.clone()
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at.clone()
    }

    #[graphql(
        description = "Get imprints linked to this publisher",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on imprint_name and imprint_url"
            ),
            order(
                default = {
                    ImprintOrderBy {
                        field: ImprintField::ImprintName,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
        )
    )]
    pub fn imprints(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: ImprintOrderBy,
    ) -> FieldResult<Vec<Imprint>> {
        Imprint::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            vec![],
            Some(self.publisher_id),
            None,
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "The brand under which a publisher issues works.")]
impl Imprint {
    pub fn imprint_id(&self) -> Uuid {
        self.imprint_id
    }

    pub fn publisher_id(&self) -> Uuid {
        self.publisher_id
    }

    pub fn imprint_name(&self) -> &String {
        &self.imprint_name
    }

    pub fn imprint_url(&self) -> Option<&String> {
        self.imprint_url.as_ref()
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at.clone()
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at.clone()
    }

    pub fn publisher(&self, context: &Context) -> FieldResult<Publisher> {
        Publisher::from_id(&context.db, &self.publisher_id).map_err(|e| e.into())
    }

    #[graphql(
    description="Get works linked to this imprint",
    arguments(
        limit(
            default = 100,
            description = "The number of items to return"
        ),
        offset(
            default = 0,
            description = "The number of items to skip"
        ),
        filter(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_title, doi, reference, short_abstract, long_abstract, and landing_page"
        ),
        order(
            default = {
                WorkOrderBy {
                    field: WorkField::FullTitle,
                    direction: Direction::Asc,
                }
            },
            description = "The order in which to sort the results",
        ),
        work_types(
            default = vec![],
            description = "Specific types to filter by",
        ),
        work_status(description = "A specific status to filter by"),
    )
  )]
    pub fn works(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: WorkOrderBy,
        work_types: Vec<WorkType>,
        work_status: Option<WorkStatus>,
    ) -> FieldResult<Vec<Work>> {
        Work::all(
            &context.db,
            limit,
            offset,
            Some(filter),
            order,
            vec![],
            Some(self.imprint_id),
            None,
            work_types,
            work_status,
        )
        .map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "A person who has been involved in the production of a written text.")]
impl Contributor {
    pub fn contributor_id(&self) -> Uuid {
        self.contributor_id
    }

    pub fn first_name(&self) -> Option<&String> {
        self.first_name.as_ref()
    }

    pub fn last_name(&self) -> &String {
        &self.last_name
    }

    pub fn full_name(&self) -> &String {
        &self.full_name
    }

    pub fn orcid(&self) -> Option<&Orcid> {
        self.orcid.as_ref()
    }

    pub fn website(&self) -> Option<&String> {
        self.website.as_ref()
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at.clone()
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at.clone()
    }

    #[graphql(
        description = "Get contributions linked to this contributor",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    ContributionOrderBy {
                        field: ContributionField::ContributionType,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            contribution_types(
                default = vec![],
                description = "Specific types to filter by",
            ),
        )
    )]
    pub fn contributions(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        order: ContributionOrderBy,
        contribution_types: Vec<ContributionType>,
    ) -> FieldResult<Vec<Contribution>> {
        Contribution::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            vec![],
            None,
            Some(self.contributor_id),
            contribution_types,
            None,
        )
        .map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "A person's involvement in the production of a written text.")]
impl Contribution {
    pub fn contribution_id(&self) -> Uuid {
        self.contribution_id
    }

    pub fn contributor_id(&self) -> Uuid {
        self.contributor_id
    }

    pub fn work_id(&self) -> Uuid {
        self.work_id
    }

    pub fn contribution_type(&self) -> &ContributionType {
        &self.contribution_type
    }

    pub fn main_contribution(&self) -> bool {
        self.main_contribution
    }

    pub fn biography(&self) -> Option<&String> {
        self.biography.as_ref()
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at.clone()
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at.clone()
    }

    pub fn first_name(&self) -> Option<&String> {
        self.first_name.as_ref()
    }

    pub fn last_name(&self) -> &String {
        &self.last_name
    }

    pub fn full_name(&self) -> &String {
        &self.full_name
    }

    pub fn contribution_ordinal(&self) -> &i32 {
        &self.contribution_ordinal
    }

    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(|e| e.into())
    }

    pub fn contributor(&self, context: &Context) -> FieldResult<Contributor> {
        Contributor::from_id(&context.db, &self.contributor_id).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get affiliations linked to this contribution",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    AffiliationOrderBy {
                        field: AffiliationField::AffiliationOrdinal,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
        )
    )]
    pub fn affiliations(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        order: AffiliationOrderBy,
    ) -> FieldResult<Vec<Affiliation>> {
        Affiliation::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            vec![],
            None,
            Some(self.contribution_id),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "A periodical of publications about a particular subject.")]
impl Series {
    pub fn series_id(&self) -> Uuid {
        self.series_id
    }

    pub fn series_type(&self) -> &SeriesType {
        &self.series_type
    }

    pub fn series_name(&self) -> &String {
        &self.series_name
    }

    pub fn issn_print(&self) -> &String {
        &self.issn_print
    }

    pub fn issn_digital(&self) -> &String {
        &self.issn_digital
    }

    #[graphql(description = "URL of the series' landing page")]
    pub fn series_url(&self) -> Option<&String> {
        self.series_url.as_ref()
    }

    pub fn series_description(&self) -> Option<&String> {
        self.series_description.as_ref()
    }

    #[graphql(description = "URL of the series' call for proposals page")]
    pub fn series_cfp_url(&self) -> Option<&String> {
        self.series_cfp_url.as_ref()
    }

    pub fn imprint_id(&self) -> Uuid {
        self.imprint_id
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at.clone()
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at.clone()
    }

    pub fn imprint(&self, context: &Context) -> FieldResult<Imprint> {
        Imprint::from_id(&context.db, &self.imprint_id).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get issues linked to this series",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    IssueOrderBy {
                        field: IssueField::IssueOrdinal,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
        )
    )]
    pub fn issues(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        order: IssueOrderBy,
    ) -> FieldResult<Vec<Issue>> {
        Issue::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            vec![],
            None,
            Some(self.series_id),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "A work published as a number in a periodical.")]
impl Issue {
    pub fn issue_id(&self) -> Uuid {
        self.issue_id
    }

    pub fn work_id(&self) -> Uuid {
        self.work_id
    }

    pub fn series_id(&self) -> Uuid {
        self.series_id
    }

    pub fn issue_ordinal(&self) -> &i32 {
        &self.issue_ordinal
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at.clone()
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at.clone()
    }

    pub fn series(&self, context: &Context) -> FieldResult<Series> {
        Series::from_id(&context.db, &self.series_id).map_err(|e| e.into())
    }

    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "Description of a work's language.")]
impl Language {
    pub fn language_id(&self) -> Uuid {
        self.language_id
    }

    pub fn work_id(&self) -> Uuid {
        self.work_id
    }

    pub fn language_code(&self) -> &LanguageCode {
        &self.language_code
    }

    pub fn language_relation(&self) -> &LanguageRelation {
        &self.language_relation
    }

    pub fn main_language(&self) -> bool {
        self.main_language
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at.clone()
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at.clone()
    }

    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "A location, such as a web shop or distribution platform, where a publication can be acquired or viewed.")]
impl Location {
    pub fn location_id(&self) -> Uuid {
        self.location_id
    }

    pub fn publication_id(&self) -> Uuid {
        self.publication_id
    }

    pub fn landing_page(&self) -> Option<&String> {
        self.landing_page.as_ref()
    }

    pub fn full_text_url(&self) -> Option<&String> {
        self.full_text_url.as_ref()
    }

    pub fn location_platform(&self) -> &LocationPlatform {
        &self.location_platform
    }

    pub fn canonical(&self) -> bool {
        self.canonical
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at.clone()
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at.clone()
    }

    pub fn publication(&self, context: &Context) -> FieldResult<Publication> {
        Publication::from_id(&context.db, &self.publication_id).map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "The amount of money, in any currency, that a publication costs.")]
impl Price {
    pub fn price_id(&self) -> Uuid {
        self.price_id
    }

    pub fn publication_id(&self) -> Uuid {
        self.publication_id
    }

    pub fn currency_code(&self) -> &CurrencyCode {
        &self.currency_code
    }

    pub fn unit_price(&self) -> f64 {
        self.unit_price
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at.clone()
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at.clone()
    }

    pub fn publication(&self, context: &Context) -> FieldResult<Publication> {
        Publication::from_id(&context.db, &self.publication_id).map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "A significant discipline or term related to a work.")]
impl Subject {
    pub fn subject_id(&self) -> &Uuid {
        &self.subject_id
    }

    pub fn work_id(&self) -> &Uuid {
        &self.work_id
    }

    pub fn subject_type(&self) -> &SubjectType {
        &self.subject_type
    }

    pub fn subject_code(&self) -> &String {
        &self.subject_code
    }

    pub fn subject_ordinal(&self) -> &i32 {
        &self.subject_ordinal
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at.clone()
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at.clone()
    }

    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "An organisation with which contributors may be affiliated or by which works may be funded.")]
impl Institution {
    pub fn institution_id(&self) -> &Uuid {
        &self.institution_id
    }

    pub fn institution_name(&self) -> &String {
        &self.institution_name
    }

    #[graphql(
        description = "Digital Object Identifier of the organisation as full URL. It must use the HTTPS scheme and the doi.org domain (e.g. https://doi.org/10.13039/100014013)"
    )]
    pub fn institution_doi(&self) -> Option<&Doi> {
        self.institution_doi.as_ref()
    }

    pub fn country_code(&self) -> Option<&CountryCode> {
        self.country_code.as_ref()
    }

    #[graphql(
        description = "Research Organisation Registry identifier of the organisation as full URL. It must use the HTTPS scheme and the ror.org domain (e.g. https://ror.org/051z6e826)"
    )]
    pub fn ror(&self) -> Option<&Ror> {
        self.ror.as_ref()
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at.clone()
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at.clone()
    }

    #[graphql(
        description = "Get fundings linked to this institution",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    FundingOrderBy {
                        field: FundingField::Program,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
        )
    )]
    pub fn fundings(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        order: FundingOrderBy,
    ) -> FieldResult<Vec<Funding>> {
        Funding::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            vec![],
            None,
            Some(self.institution_id),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(
        description = "Get affiliations linked to this institution",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            order(
                default = {
                    AffiliationOrderBy {
                        field: AffiliationField::AffiliationOrdinal,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
        )
    )]
    pub fn affiliations(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        order: AffiliationOrderBy,
    ) -> FieldResult<Vec<Affiliation>> {
        Affiliation::all(
            &context.db,
            limit,
            offset,
            None,
            order,
            vec![],
            Some(self.institution_id),
            None,
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "A grant awarded to the publication of a work by an institution.")]
impl Funding {
    pub fn funding_id(&self) -> &Uuid {
        &self.funding_id
    }

    pub fn work_id(&self) -> &Uuid {
        &self.work_id
    }

    pub fn institution_id(&self) -> &Uuid {
        &self.institution_id
    }

    pub fn program(&self) -> Option<&String> {
        self.program.as_ref()
    }

    pub fn project_name(&self) -> Option<&String> {
        self.project_name.as_ref()
    }

    pub fn project_shortname(&self) -> Option<&String> {
        self.project_shortname.as_ref()
    }

    pub fn grant_number(&self) -> Option<&String> {
        self.grant_number.as_ref()
    }

    pub fn jurisdiction(&self) -> Option<&String> {
        self.jurisdiction.as_ref()
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at.clone()
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at.clone()
    }

    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(|e| e.into())
    }

    pub fn institution(&self, context: &Context) -> FieldResult<Institution> {
        Institution::from_id(&context.db, &self.institution_id).map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "An association between a person and an institution for a specific contribution.")]
impl Affiliation {
    pub fn affiliation_id(&self) -> Uuid {
        self.affiliation_id
    }

    pub fn contribution_id(&self) -> Uuid {
        self.contribution_id
    }

    pub fn institution_id(&self) -> Uuid {
        self.institution_id
    }

    pub fn affiliation_ordinal(&self) -> &i32 {
        &self.affiliation_ordinal
    }

    pub fn position(&self) -> Option<&String> {
        self.position.as_ref()
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at.clone()
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at.clone()
    }

    pub fn institution(&self, context: &Context) -> FieldResult<Institution> {
        Institution::from_id(&context.db, &self.institution_id).map_err(|e| e.into())
    }

    pub fn contribution(&self, context: &Context) -> FieldResult<Contribution> {
        Contribution::from_id(&context.db, &self.contribution_id).map_err(|e| e.into())
    }
}

#[juniper::object(Context = Context, description = "A relationship between two works, e.g. a book and one of its chapters, or an original and its translation.")]
impl WorkRelation {
    pub fn work_relation_id(&self) -> &Uuid {
        &self.work_relation_id
    }

    pub fn relator_work_id(&self) -> &Uuid {
        &self.relator_work_id
    }

    pub fn related_work_id(&self) -> &Uuid {
        &self.related_work_id
    }

    pub fn relation_type(&self) -> &RelationType {
        &self.relation_type
    }

    pub fn relation_ordinal(&self) -> &i32 {
        &self.relation_ordinal
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at.clone()
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at.clone()
    }

    pub fn related_work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.related_work_id).map_err(|e| e.into())
    }
}

#[juniper::object(
    Context = Context,
    description = "A citation to a written text. References must always include the DOI of the cited work, the unstructured citation, or both.",
)]
impl Reference {
    #[graphql(description = "UUID of the reference.")]
    pub fn reference_id(&self) -> Uuid {
        self.reference_id
    }

    #[graphql(description = "UUID of the citing work.")]
    pub fn work_id(&self) -> Uuid {
        self.work_id
    }

    #[graphql(description = "Number used to order references within a work's bibliography.")]
    pub fn reference_ordinal(&self) -> &i32 {
        self.reference_ordinal
    }

    #[graphql(description = "Digital Object Identifier of the cited work as full URL.")]
    pub fn doi(&self) -> Option<&Doi> {
        self.doi.as_ref()
    }

    #[graphql(
        description = "Full reference text. When the DOI of the cited work is not known this field is required, and may be used in conjunction with other structured data to help identify the cited work."
    )]
    pub fn unstructured_citation(&self) -> Option<&String> {
        self.unstructured_citation.as_ref()
    }

    #[graphql(description = "ISSN of a series.")]
    pub fn issn(&self) -> Option<&String> {
        self.issn.as_ref()
    }

    #[graphql(description = "Book ISBN, when the cited work is a book or a chapter.")]
    pub fn isbn(&self) -> Option<&Isbn> {
        self.isbn.as_ref()
    }

    #[graphql(description = "Title of a journal, when the cited work is an article.")]
    pub fn journal_title(&self) -> Option<&String> {
        self.journal_title.as_ref()
    }

    #[graphql(description = "Journal article, conference paper, or book chapter title.")]
    pub fn article_title(&self) -> Option<&String> {
        self.article_title.as_ref()
    }

    #[graphql(description = "Title of a book or conference series.")]
    pub fn series_title(&self) -> Option<&String> {
        self.series_title.as_ref()
    }

    #[graphql(description = "Title of a book or conference proceeding.")]
    pub fn volume_title(&self) -> Option<&String> {
        self.volume_title.as_ref()
    }

    #[graphql(description = "Book edition number.")]
    pub fn edition(&self) -> Option<&i32> {
        self.edition.as_ref()
    }

    #[graphql(description = "First author of the cited work.")]
    pub fn author(&self) -> Option<&String> {
        self.author.as_ref()
    }

    #[graphql(description = "Volume number of a journal or book set.")]
    pub fn volume(&self) -> Option<&String> {
        self.volume.as_ref()
    }

    #[graphql(description = "Journal issue, when the cited work is an article.")]
    pub fn issue(&self) -> Option<&String> {
        self.issue.as_ref()
    }

    #[graphql(description = "First page of the cited page range.")]
    pub fn first_page(&self) -> Option<&String> {
        self.first_page.as_ref()
    }

    #[graphql(
        description = "The chapter, section or part number, when the cited work is a component of a book."
    )]
    pub fn component_number(&self) -> Option<&String> {
        self.component_number.as_ref()
    }

    #[graphql(
        description = "Standard identifier (e.g. \"14064-1\"), when the cited work is a standard."
    )]
    pub fn standard_designator(&self) -> Option<&String> {
        self.standard_designator.as_ref()
    }

    #[graphql(
        description = "Full name of the standards organisation (e.g. \"International Organization for Standardization\"), when the cited work is a standard."
    )]
    pub fn standards_body_name(&self) -> Option<&String> {
        self.standards_body_name.as_ref()
    }

    #[graphql(
        description = "Acronym of the standards organisation (e.g. \"ISO\"), when the cited work is a standard."
    )]
    pub fn standards_body_acronym(&self) -> Option<&String> {
        self.standards_body_acronym.as_ref()
    }

    #[graphql(description = "URL of the cited work.")]
    pub fn reference_url(&self) -> Option<&String> {
        self.reference_url.as_ref()
    }

    #[graphql(
        description = "Publication date of the cited work. Day and month should be set to \"01\" when only the publication year is known."
    )]
    pub fn publication_date(&self) -> Option<NaiveDate> {
        self.publication_date
    }

    #[graphql(
        description = "Date the cited work was accessed, when citing a website or online article."
    )]
    pub fn retrieval_date(&self) -> Option<NaiveDate> {
        self.retrieval_date
    }

    #[graphql(description = "Timestamp of the creation of this record within Thoth.")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at.clone()
    }

    #[graphql(description = "Timestamp of the last update to this record within Thoth.")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at.clone()
    }

    #[graphql(description = "The citing work.")]
    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(|e| e.into())
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}

fn publisher_id_from_imprint_id(db: &crate::db::PgPool, imprint_id: Uuid) -> ThothResult<Uuid> {
    Ok(Imprint::from_id(db, &imprint_id)?.publisher_id)
}

fn publisher_id_from_work_id(db: &crate::db::PgPool, work_id: Uuid) -> ThothResult<Uuid> {
    Work::from_id(db, &work_id)?.publisher_id(db)
}

fn publisher_id_from_publication_id(
    db: &crate::db::PgPool,
    publication_id: Uuid,
) -> ThothResult<Uuid> {
    Publication::from_id(db, &publication_id)?.publisher_id(db)
}

fn publisher_id_from_contribution_id(
    db: &crate::db::PgPool,
    contribution_id: Uuid,
) -> ThothResult<Uuid> {
    Contribution::from_id(db, &contribution_id)?.publisher_id(db)
}
