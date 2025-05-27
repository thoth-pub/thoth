use crate::model::title::{PatchTitle, Title};
use crate::schema::work_title;
use chrono::naive::NaiveDate;
use diesel::dsl::Limit;
use diesel::prelude::*;
use juniper::RootNode;
use juniper::{EmptySubscription, FieldError, FieldResult, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::account::model::AccountAccess;
use crate::account::model::DecodedToken;
use crate::db::PgPool;
use crate::model::contributor::*;
use crate::model::funding::*;
use crate::model::imprint::*;
use crate::model::institution::*;
use crate::model::issue::*;
use crate::model::language::*;
use crate::model::location::*;
use crate::model::price::*;
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
use crate::model::{affiliation::*, TitleOrderBy};
use crate::model::{contribution::*, NewTitle};
use thoth_errors::{ThothError, ThothResult};

use super::utils::{Direction, Expression};
use crate::model::LocaleCode;

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

impl Default for ContributionOrderBy {
    fn default() -> ContributionOrderBy {
        ContributionOrderBy {
            field: ContributionField::ContributionType,
            direction: Default::default(),
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting issues list")]
pub struct IssueOrderBy {
    pub field: IssueField,
    pub direction: Direction,
}

impl Default for IssueOrderBy {
    fn default() -> IssueOrderBy {
        IssueOrderBy {
            field: IssueField::IssueOrdinal,
            direction: Default::default(),
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting languages list")]
pub struct LanguageOrderBy {
    pub field: LanguageField,
    pub direction: Direction,
}

impl Default for LanguageOrderBy {
    fn default() -> LanguageOrderBy {
        LanguageOrderBy {
            field: LanguageField::LanguageCode,
            direction: Default::default(),
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting prices list")]
pub struct PriceOrderBy {
    pub field: PriceField,
    pub direction: Direction,
}

impl Default for PriceOrderBy {
    fn default() -> PriceOrderBy {
        PriceOrderBy {
            field: PriceField::CurrencyCode,
            direction: Default::default(),
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting subjects list")]
pub struct SubjectOrderBy {
    pub field: SubjectField,
    pub direction: Direction,
}

impl Default for SubjectOrderBy {
    fn default() -> SubjectOrderBy {
        SubjectOrderBy {
            field: SubjectField::SubjectType,
            direction: Default::default(),
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting fundings list")]
pub struct FundingOrderBy {
    pub field: FundingField,
    pub direction: Direction,
}

impl Default for FundingOrderBy {
    fn default() -> FundingOrderBy {
        FundingOrderBy {
            field: FundingField::Program,
            direction: Default::default(),
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(
    description = "Timestamp and choice out of greater than/less than to use when filtering by a time field (e.g. updated_at)"
)]
pub struct TimeExpression {
    pub timestamp: Timestamp,
    pub expression: Expression,
}

pub struct QueryRoot;

#[juniper::graphql_object(Context = Context)]
impl QueryRoot {
    #[allow(clippy::too_many_arguments)]
    #[graphql(description = "Query the full list of works")]
    fn works(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_title, doi, reference, short_abstract, long_abstract, and landing_page"
        )]
        filter: Option<String>,
        #[graphql(
            default = WorkOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<WorkOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
        #[graphql(
            default = vec![],
            description = "Specific types to filter by",
        )]
        work_types: Option<Vec<WorkType>>,
        #[graphql(description = "(deprecated) A specific status to filter by")] work_status: Option<
            WorkStatus,
        >,
        #[graphql(
            default = vec![],
            description = "Specific statuses to filter by"
        )]
        work_statuses: Option<Vec<WorkStatus>>,
        #[graphql(
            description = "Only show results updated either before (less than) or after (greater than) the specified timestamp"
        )]
        updated_at_with_relations: Option<TimeExpression>,
    ) -> FieldResult<Vec<Work>> {
        let mut statuses = work_statuses.unwrap_or_default();
        if let Some(status) = work_status {
            statuses.push(status);
        }
        Work::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            filter,
            order.unwrap_or_default(),
            publishers.unwrap_or_default(),
            None,
            None,
            work_types.unwrap_or_default(),
            statuses,
            updated_at_with_relations,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single work using its ID")]
    fn work(
        context: &Context,
        #[graphql(description = "Thoth work ID to search on")] work_id: Uuid,
    ) -> FieldResult<Work> {
        Work::from_id(&context.db, &work_id).map_err(|e| e.into())
    }

    #[graphql(description = "Query a single work using its DOI")]
    fn work_by_doi(
        context: &Context,
        #[graphql(description = "Work DOI to search on")] doi: Doi,
    ) -> FieldResult<Work> {
        Work::from_doi(&context.db, doi, vec![]).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of works")]
    fn work_count(
        context: &Context,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_title, doi, reference, short_abstract, long_abstract, and landing_page",
        )]
        filter: Option<String>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs",
        )]
        publishers: Option<Vec<Uuid>>,
        #[graphql(
            default = vec![],
            description = "Specific types to filter by",
        )]
        work_types: Option<Vec<WorkType>>,
        #[graphql(description = "(deprecated) A specific status to filter by")] work_status: Option<
            WorkStatus,
        >,
        #[graphql(
            default = vec![],
            description = "Specific statuses to filter by"
        )]
        work_statuses: Option<Vec<WorkStatus>>,
        #[graphql(
            description = "Only show results updated either before (less than) or after (greater than) the specified timestamp"
        )]
        updated_at_with_relations: Option<TimeExpression>,
    ) -> FieldResult<i32> {
        let mut statuses = work_statuses.unwrap_or_default();
        if let Some(status) = work_status {
            statuses.push(status);
        }
        Work::count(
            &context.db,
            filter,
            publishers.unwrap_or_default(),
            work_types.unwrap_or_default(),
            statuses,
            updated_at_with_relations,
        )
        .map_err(|e| e.into())
    }

    #[allow(clippy::too_many_arguments)]
    #[graphql(description = "Query the full list of books (a subset of the full list of works)")]
    fn books(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_title, doi, reference, short_abstract, long_abstract, and landing_page"
        )]
        filter: Option<String>,
        #[graphql(
            default = WorkOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<WorkOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
        #[graphql(description = "(deprecated) A specific status to filter by")] work_status: Option<
            WorkStatus,
        >,
        #[graphql(
            default = vec![],
            description = "Specific statuses to filter by"
        )]
        work_statuses: Option<Vec<WorkStatus>>,
        #[graphql(
            description = "Only show results updated either before (less than) or after (greater than) the specified timestamp"
        )]
        updated_at_with_relations: Option<TimeExpression>,
    ) -> FieldResult<Vec<Work>> {
        let mut statuses = work_statuses.unwrap_or_default();
        if let Some(status) = work_status {
            statuses.push(status);
        }
        Work::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            filter,
            order.unwrap_or_default(),
            publishers.unwrap_or_default(),
            None,
            None,
            vec![
                WorkType::Monograph,
                WorkType::EditedBook,
                WorkType::Textbook,
                WorkType::JournalIssue,
            ],
            statuses,
            updated_at_with_relations,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single book using its DOI")]
    fn book_by_doi(
        context: &Context,
        #[graphql(description = "Book DOI to search on")] doi: Doi,
    ) -> FieldResult<Work> {
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
        description = "Get the total number of books (a subset of the total number of works)"
    )]
    fn book_count(
        context: &Context,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_title, doi, reference, short_abstract, long_abstract, and landing_page"
        )]
        filter: Option<String>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
        #[graphql(description = "(deprecated) A specific status to filter by")] work_status: Option<
            WorkStatus,
        >,
        #[graphql(
            default = vec![],
            description = "Specific statuses to filter by"
        )]
        work_statuses: Option<Vec<WorkStatus>>,
        #[graphql(
            description = "Only show results updated either before (less than) or after (greater than) the specified timestamp"
        )]
        updated_at_with_relations: Option<TimeExpression>,
    ) -> FieldResult<i32> {
        let mut statuses = work_statuses.unwrap_or_default();
        if let Some(status) = work_status {
            statuses.push(status);
        }
        Work::count(
            &context.db,
            filter,
            publishers.unwrap_or_default(),
            vec![
                WorkType::Monograph,
                WorkType::EditedBook,
                WorkType::Textbook,
                WorkType::JournalIssue,
            ],
            statuses,
            updated_at_with_relations,
        )
        .map_err(|e| e.into())
    }

    #[allow(clippy::too_many_arguments)]
    #[graphql(description = "Query the full list of chapters (a subset of the full list of works)")]
    fn chapters(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_title, doi, reference, short_abstract, long_abstract, and landing_page"
        )]
        filter: Option<String>,
        #[graphql(
            default = WorkOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<WorkOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
        #[graphql(description = "(deprecated) A specific status to filter by")] work_status: Option<
            WorkStatus,
        >,
        #[graphql(
            default = vec![],
            description = "Specific statuses to filter by"
        )]
        work_statuses: Option<Vec<WorkStatus>>,
        #[graphql(
            description = "Only show results updated either before (less than) or after (greater than) the specified timestamp"
        )]
        updated_at_with_relations: Option<TimeExpression>,
    ) -> FieldResult<Vec<Work>> {
        let mut statuses = work_statuses.unwrap_or_default();
        if let Some(status) = work_status {
            statuses.push(status);
        }
        Work::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            filter,
            order.unwrap_or_default(),
            publishers.unwrap_or_default(),
            None,
            None,
            vec![WorkType::BookChapter],
            statuses,
            updated_at_with_relations,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single chapter using its DOI")]
    fn chapter_by_doi(
        context: &Context,
        #[graphql(description = "Chapter DOI to search on")] doi: Doi,
    ) -> FieldResult<Work> {
        Work::from_doi(&context.db, doi, vec![WorkType::BookChapter]).map_err(|e| e.into())
    }

    #[graphql(
        description = "Get the total number of chapters (a subset of the total number of works)"
    )]
    fn chapter_count(
        context: &Context,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_title, doi, reference, short_abstract, long_abstract, and landing_page"
        )]
        filter: Option<String>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
        #[graphql(description = "(deprecated) A specific status to filter by")] work_status: Option<
            WorkStatus,
        >,
        #[graphql(
            default = vec![],
            description = "Specific statuses to filter by"
        )]
        work_statuses: Option<Vec<WorkStatus>>,
        #[graphql(
            description = "Only show results updated either before (less than) or after (greater than) the specified timestamp"
        )]
        updated_at_with_relations: Option<TimeExpression>,
    ) -> FieldResult<i32> {
        let mut statuses = work_statuses.unwrap_or_default();
        if let Some(status) = work_status {
            statuses.push(status);
        }
        Work::count(
            &context.db,
            filter,
            publishers.unwrap_or_default(),
            vec![WorkType::BookChapter],
            statuses,
            updated_at_with_relations,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query the full list of publications")]
    fn publications(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on isbn"
        )]
        filter: Option<String>,
        #[graphql(
            default = PublicationOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<PublicationOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
        #[graphql(
            default = vec![],
            description = "Specific types to filter by",
        )]
        publication_types: Option<Vec<PublicationType>>,
    ) -> FieldResult<Vec<Publication>> {
        Publication::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            filter,
            order.unwrap_or_default(),
            publishers.unwrap_or_default(),
            None,
            None,
            publication_types.unwrap_or_default(),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single publication using its ID")]
    fn publication(
        context: &Context,
        #[graphql(description = "Thoth publication ID to search on")] publication_id: Uuid,
    ) -> FieldResult<Publication> {
        Publication::from_id(&context.db, &publication_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of publications")]
    fn publication_count(
        context: &Context,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on isbn"
        )]
        filter: Option<String>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
        #[graphql(
            default = vec![],
            description = "Specific types to filter by",
        )]
        publication_types: Option<Vec<PublicationType>>,
    ) -> FieldResult<i32> {
        Publication::count(
            &context.db,
            filter,
            publishers.unwrap_or_default(),
            publication_types.unwrap_or_default(),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query the full list of publishers")]
    fn publishers(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on publisher_name and publisher_shortname"
        )]
        filter: Option<String>,
        #[graphql(
            default = PublisherOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<PublisherOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
    ) -> FieldResult<Vec<Publisher>> {
        Publisher::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            filter,
            order.unwrap_or_default(),
            publishers.unwrap_or_default(),
            None,
            None,
            vec![],
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single publisher using its ID")]
    fn publisher(
        context: &Context,
        #[graphql(description = "Thoth publisher ID to search on")] publisher_id: Uuid,
    ) -> FieldResult<Publisher> {
        Publisher::from_id(&context.db, &publisher_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of publishers")]
    fn publisher_count(
        context: &Context,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on publisher_name and publisher_shortname"
        )]
        filter: Option<String>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
    ) -> FieldResult<i32> {
        Publisher::count(
            &context.db,
            filter,
            publishers.unwrap_or_default(),
            vec![],
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query the full list of imprints")]
    fn imprints(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on imprint_name and imprint_url"
        )]
        filter: Option<String>,
        #[graphql(
            default = ImprintOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<ImprintOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
    ) -> FieldResult<Vec<Imprint>> {
        Imprint::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            filter,
            order.unwrap_or_default(),
            publishers.unwrap_or_default(),
            None,
            None,
            vec![],
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single imprint using its ID")]
    fn imprint(
        context: &Context,
        #[graphql(description = "Thoth imprint ID to search on")] imprint_id: Uuid,
    ) -> FieldResult<Imprint> {
        Imprint::from_id(&context.db, &imprint_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of imprints")]
    fn imprint_count(
        context: &Context,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on imprint_name and imprint_url"
        )]
        filter: Option<String>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
    ) -> FieldResult<i32> {
        Imprint::count(
            &context.db,
            filter,
            publishers.unwrap_or_default(),
            vec![],
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query the full list of contributors")]
    fn contributors(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_name, last_name and orcid"
        )]
        filter: Option<String>,
        #[graphql(
            default = ContributorOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<ContributorOrderBy>,
    ) -> FieldResult<Vec<Contributor>> {
        Contributor::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            filter,
            order.unwrap_or_default(),
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single contributor using its ID")]
    fn contributor(
        context: &Context,
        #[graphql(description = "Thoth contributor ID to search on")] contributor_id: Uuid,
    ) -> FieldResult<Contributor> {
        Contributor::from_id(&context.db, &contributor_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of contributors")]
    fn contributor_count(
        context: &Context,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_name, last_name and orcid"
        )]
        filter: Option<String>,
    ) -> FieldResult<i32> {
        Contributor::count(&context.db, filter, vec![], vec![], vec![], None).map_err(|e| e.into())
    }

    #[graphql(description = "Query the full list of contributions")]
    fn contributions(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = ContributionOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<ContributionOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
        #[graphql(
            default = vec![],
            description = "Specific types to filter by",
        )]
        contribution_types: Option<Vec<ContributionType>>,
    ) -> FieldResult<Vec<Contribution>> {
        Contribution::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            None,
            order.unwrap_or_default(),
            publishers.unwrap_or_default(),
            None,
            None,
            contribution_types.unwrap_or_default(),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single contribution using its ID")]
    fn contribution(
        context: &Context,
        #[graphql(description = "Thoth contribution ID to search on")] contribution_id: Uuid,
    ) -> FieldResult<Contribution> {
        Contribution::from_id(&context.db, &contribution_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of contributions")]
    fn contribution_count(
        context: &Context,
        #[graphql(
            default = vec![],
            description = "Specific types to filter by",
        )]
        contribution_types: Option<Vec<ContributionType>>,
    ) -> FieldResult<i32> {
        Contribution::count(
            &context.db,
            None,
            vec![],
            contribution_types.unwrap_or_default(),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query the full list of series")]
    fn serieses(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on series_name, issn_print, issn_digital, series_url and series_description"
        )]
        filter: Option<String>,
        #[graphql(
            default = SeriesOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<SeriesOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
        #[graphql(
            default = vec![],
            description = "Specific types to filter by",
        )]
        series_types: Option<Vec<SeriesType>>,
    ) -> FieldResult<Vec<Series>> {
        Series::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            filter,
            order.unwrap_or_default(),
            publishers.unwrap_or_default(),
            None,
            None,
            series_types.unwrap_or_default(),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single series using its ID")]
    fn series(
        context: &Context,
        #[graphql(description = "Thoth series ID to search on")] series_id: Uuid,
    ) -> FieldResult<Series> {
        Series::from_id(&context.db, &series_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of series")]
    fn series_count(
        context: &Context,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on series_name, issn_print, issn_digital, series_url and series_description"
        )]
        filter: Option<String>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
        #[graphql(
            default = vec![],
            description = "Specific types to filter by",
        )]
        series_types: Option<Vec<SeriesType>>,
    ) -> FieldResult<i32> {
        Series::count(
            &context.db,
            filter,
            publishers.unwrap_or_default(),
            series_types.unwrap_or_default(),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query the full list of issues")]
    fn issues(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = IssueOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<IssueOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
    ) -> FieldResult<Vec<Issue>> {
        Issue::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            None,
            order.unwrap_or_default(),
            publishers.unwrap_or_default(),
            None,
            None,
            vec![],
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single issue using its ID")]
    fn issue(
        context: &Context,
        #[graphql(description = "Thoth issue ID to search on")] issue_id: Uuid,
    ) -> FieldResult<Issue> {
        Issue::from_id(&context.db, &issue_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of issues")]
    fn issue_count(context: &Context) -> FieldResult<i32> {
        Issue::count(&context.db, None, vec![], vec![], vec![], None).map_err(|e| e.into())
    }

    #[allow(clippy::too_many_arguments)]
    #[graphql(description = "Query the full list of languages")]
    fn languages(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = LanguageOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<LanguageOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
        #[graphql(
            default = vec![],
            description = "Specific languages to filter by"
        )]
        language_codes: Option<Vec<LanguageCode>>,
        #[graphql(
            description = "(deprecated) A specific relation to filter by"
        )]
        language_relation: Option<LanguageRelation>,
        #[graphql(
            default = vec![],
            description = "Specific relations to filter by"
        )]
        language_relations: Option<Vec<LanguageRelation>>,
    ) -> FieldResult<Vec<Language>> {
        let mut relations = language_relations.unwrap_or_default();
        if let Some(relation) = language_relation {
            relations.push(relation);
        }
        Language::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            None,
            order.unwrap_or_default(),
            publishers.unwrap_or_default(),
            None,
            None,
            language_codes.unwrap_or_default(),
            relations,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single language using its ID")]
    fn language(
        context: &Context,
        #[graphql(description = "Thoth language ID to search on")] language_id: Uuid,
    ) -> FieldResult<Language> {
        Language::from_id(&context.db, &language_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of languages associated to works")]
    fn language_count(
        context: &Context,
        #[graphql(
            default = vec![],
            description = "Specific languages to filter by"
        )]
        language_codes: Option<Vec<LanguageCode>>,
        #[graphql(
            description = "(deprecated) A specific relation to filter by"
        )]
        language_relation: Option<LanguageRelation>,
        #[graphql(
            default = vec![],
            description = "Specific relations to filter by"
        )]
        language_relations: Option<Vec<LanguageRelation>>,
    ) -> FieldResult<i32> {
        let mut relations = language_relations.unwrap_or_default();
        if let Some(relation) = language_relation {
            relations.push(relation);
        }
        Language::count(
            &context.db,
            None,
            vec![],
            language_codes.unwrap_or_default(),
            relations,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query the full list of locations")]
    fn locations(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = LocationOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<LocationOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
        #[graphql(
            default = vec![],
            description = "Specific platforms to filter by"
        )]
        location_platforms: Option<Vec<LocationPlatform>>,
    ) -> FieldResult<Vec<Location>> {
        Location::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            None,
            order.unwrap_or_default(),
            publishers.unwrap_or_default(),
            None,
            None,
            location_platforms.unwrap_or_default(),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single location using its ID")]
    fn location(
        context: &Context,
        #[graphql(description = "Thoth location ID to search on")] location_id: Uuid,
    ) -> FieldResult<Location> {
        Location::from_id(&context.db, &location_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of locations associated to works")]
    fn location_count(
        context: &Context,
        #[graphql(
            default = vec![],
            description = "Specific platforms to filter by"
        )]
        location_platforms: Option<Vec<LocationPlatform>>,
    ) -> FieldResult<i32> {
        Location::count(
            &context.db,
            None,
            vec![],
            location_platforms.unwrap_or_default(),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query the full list of prices")]
    fn prices(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = PriceOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<PriceOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
        #[graphql(
            default = vec![],
            description = "Specific currencies to filter by"
        )]
        currency_codes: Option<Vec<CurrencyCode>>,
    ) -> FieldResult<Vec<Price>> {
        Price::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            None,
            order.unwrap_or_default(),
            publishers.unwrap_or_default(),
            None,
            None,
            currency_codes.unwrap_or_default(),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single price using its ID")]
    fn price(
        context: &Context,
        #[graphql(description = "Thoth price ID to search on")] price_id: Uuid,
    ) -> FieldResult<Price> {
        Price::from_id(&context.db, &price_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of prices associated to works")]
    fn price_count(
        context: &Context,
        #[graphql(
            default = vec![],
            description = "Specific currencies to filter by"
        )]
        currency_codes: Option<Vec<CurrencyCode>>,
    ) -> FieldResult<i32> {
        Price::count(
            &context.db,
            None,
            vec![],
            currency_codes.unwrap_or_default(),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query the full list of subjects")]
    fn subjects(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on subject_code"
        )]
        filter: Option<String>,
        #[graphql(
            default = SubjectOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<SubjectOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
        #[graphql(
            default = vec![],
            description = "Specific types to filter by",
        )]
        subject_types: Option<Vec<SubjectType>>,
    ) -> FieldResult<Vec<Subject>> {
        Subject::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            filter,
            order.unwrap_or_default(),
            publishers.unwrap_or_default(),
            None,
            None,
            subject_types.unwrap_or_default(),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single subject using its ID")]
    fn subject(
        context: &Context,
        #[graphql(description = "Thoth subject ID to search on")] subject_id: Uuid,
    ) -> FieldResult<Subject> {
        Subject::from_id(&context.db, &subject_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of subjects associated to works")]
    fn subject_count(
        context: &Context,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on subject_code"
        )]
        filter: Option<String>,
        #[graphql(
            default = vec![],
            description = "Specific types to filter by",
        )]
        subject_types: Option<Vec<SubjectType>>,
    ) -> FieldResult<i32> {
        Subject::count(
            &context.db,
            filter,
            vec![],
            subject_types.unwrap_or_default(),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query the full list of institutions")]
    fn institutions(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on institution_name, ror and institution_doi"
        )]
        filter: Option<String>,
        #[graphql(
            default = InstitutionOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<InstitutionOrderBy>,
    ) -> FieldResult<Vec<Institution>> {
        Institution::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            filter,
            order.unwrap_or_default(),
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single institution using its ID")]
    fn institution(
        context: &Context,
        #[graphql(description = "Thoth institution ID to search on")] institution_id: Uuid,
    ) -> FieldResult<Institution> {
        Institution::from_id(&context.db, &institution_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of institutions")]
    fn institution_count(
        context: &Context,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on institution_name, ror and institution_doi"
        )]
        filter: Option<String>,
    ) -> FieldResult<i32> {
        Institution::count(&context.db, filter, vec![], vec![], vec![], None).map_err(|e| e.into())
    }

    #[graphql(description = "Query the full list of fundings")]
    fn fundings(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = FundingOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<FundingOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
    ) -> FieldResult<Vec<Funding>> {
        Funding::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            None,
            order.unwrap_or_default(),
            publishers.unwrap_or_default(),
            None,
            None,
            vec![],
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single funding using its ID")]
    fn funding(
        context: &Context,
        #[graphql(description = "Thoth funding ID to search on")] funding_id: Uuid,
    ) -> FieldResult<Funding> {
        Funding::from_id(&context.db, &funding_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of funding instances associated to works")]
    fn funding_count(context: &Context) -> FieldResult<i32> {
        Funding::count(&context.db, None, vec![], vec![], vec![], None).map_err(|e| e.into())
    }

    #[graphql(description = "Query the full list of affiliations")]
    fn affiliations(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = AffiliationOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<AffiliationOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
    ) -> FieldResult<Vec<Affiliation>> {
        Affiliation::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            None,
            order.unwrap_or_default(),
            publishers.unwrap_or_default(),
            None,
            None,
            vec![],
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single affiliation using its ID")]
    fn affiliation(
        context: &Context,
        #[graphql(description = "Thoth affiliation ID to search on")] affiliation_id: Uuid,
    ) -> FieldResult<Affiliation> {
        Affiliation::from_id(&context.db, &affiliation_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of affiliations")]
    fn affiliation_count(context: &Context) -> FieldResult<i32> {
        Affiliation::count(&context.db, None, vec![], vec![], vec![], None).map_err(|e| e.into())
    }

    #[graphql(description = "Query the full list of references")]
    fn references(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = ReferenceOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<ReferenceOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
    ) -> FieldResult<Vec<Reference>> {
        Reference::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            None,
            order.unwrap_or_default(),
            publishers.unwrap_or_default(),
            None,
            None,
            vec![],
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Query a single reference using its ID")]
    fn reference(
        context: &Context,
        #[graphql(description = "Thoth reference ID to search on")] reference_id: Uuid,
    ) -> FieldResult<Reference> {
        Reference::from_id(&context.db, &reference_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the total number of references")]
    fn reference_count(context: &Context) -> FieldResult<i32> {
        Reference::count(&context.db, None, vec![], vec![], vec![], None).map_err(|e| e.into())
    }

    #[graphql(description = "Query a title by its ID")]
    fn title(context: &Context, title_id: Uuid) -> FieldResult<Title> {
        Title::from_id(&context.db, &title_id).map_err(|e| e.into())
    }

    #[graphql(description = "Query titles by work ID")]
    fn titles(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on title_, subtitle, full_title fields"
        )]
        filter: Option<String>,
        #[graphql(
            default = TitleOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<TitleOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results with these locale codes"
        )]
        locale_codes: Option<Vec<LocaleCode>>,
    ) -> FieldResult<Vec<Title>> {
        Title::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            filter,
            order.unwrap_or_default(),
            vec![],
            None,
            None,
            locale_codes.unwrap_or_default(),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }
}

pub struct MutationRoot;

#[juniper::graphql_object(Context = Context)]
impl MutationRoot {
    #[graphql(description = "Create a new work with the specified values")]
    fn create_work(
        context: &Context,
        #[graphql(description = "Values for work to be created")] data: NewWork,
    ) -> FieldResult<Work> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_imprint_id(&context.db, data.imprint_id)?)?;

        data.validate()?;

        Work::create(&context.db, &data).map_err(|e| e.into())
    }

    #[graphql(description = "Create a new publisher with the specified values")]
    fn create_publisher(
        context: &Context,
        #[graphql(description = "Values for publisher to be created")] data: NewPublisher,
    ) -> FieldResult<Publisher> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        // Only superusers can create new publishers - NewPublisher has no ID field
        if !context.account_access.is_superuser {
            return Err(ThothError::Unauthorised.into());
        }

        Publisher::create(&context.db, &data).map_err(|e| e.into())
    }

    #[graphql(description = "Create a new imprint with the specified values")]
    fn create_imprint(
        context: &Context,
        #[graphql(description = "Values for imprint to be created")] data: NewImprint,
    ) -> FieldResult<Imprint> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context.account_access.can_edit(data.publisher_id)?;

        Imprint::create(&context.db, &data).map_err(|e| e.into())
    }

    #[graphql(description = "Create a new contributor with the specified values")]
    fn create_contributor(
        context: &Context,
        #[graphql(description = "Values for contributor to be created")] data: NewContributor,
    ) -> FieldResult<Contributor> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        Contributor::create(&context.db, &data).map_err(|e| e.into())
    }

    #[graphql(description = "Create a new contribution with the specified values")]
    fn create_contribution(
        context: &Context,
        #[graphql(description = "Values for contribution to be created")] data: NewContribution,
    ) -> FieldResult<Contribution> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;

        Contribution::create(&context.db, &data).map_err(|e| e.into())
    }

    #[graphql(description = "Create a new publication with the specified values")]
    fn create_publication(
        context: &Context,
        #[graphql(description = "Values for publication to be created")] data: NewPublication,
    ) -> FieldResult<Publication> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;

        data.validate(&context.db)?;

        Publication::create(&context.db, &data).map_err(|e| e.into())
    }

    #[graphql(description = "Create a new series with the specified values")]
    fn create_series(
        context: &Context,
        #[graphql(description = "Values for series to be created")] data: NewSeries,
    ) -> FieldResult<Series> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_imprint_id(&context.db, data.imprint_id)?)?;

        Series::create(&context.db, &data).map_err(|e| e.into())
    }

    #[graphql(description = "Create a new issue with the specified values")]
    fn create_issue(
        context: &Context,
        #[graphql(description = "Values for issue to be created")] data: NewIssue,
    ) -> FieldResult<Issue> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;

        data.imprints_match(&context.db)?;

        Issue::create(&context.db, &data).map_err(|e| e.into())
    }

    #[graphql(description = "Create a new language with the specified values")]
    fn create_language(
        context: &Context,
        #[graphql(description = "Values for language to be created")] data: NewLanguage,
    ) -> FieldResult<Language> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;

        Language::create(&context.db, &data).map_err(|e| e.into())
    }

    #[graphql(description = "Create a new title with the specified values")]
    fn create_title(
        context: &Context,
        #[graphql(description = "Values for title to be created")] data: NewTitle,
    ) -> FieldResult<Title> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;

        let has_canonical_title = match Work::from_id(&context.db, &data.work_id)?.title(context) {
            Ok(_) => true,
            Err(_) => false,
        };

        // Only superusers can update the canonical location when a Thoth Location Platform canonical location already exists
        if has_canonical_title && data.canonical && !context.account_access.is_superuser {
            return Err(ThothError::ThothUpdateCanonicalError.into());
        }

        Title::create(&context.db, &data).map_err(|e| e.into())
    }

    #[graphql(description = "Create a new institution with the specified values")]
    fn create_institution(
        context: &Context,
        #[graphql(description = "Values for institution to be created")] data: NewInstitution,
    ) -> FieldResult<Institution> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        Institution::create(&context.db, &data).map_err(|e| e.into())
    }

    #[graphql(description = "Create a new funding with the specified values")]
    fn create_funding(
        context: &Context,
        #[graphql(description = "Values for funding to be created")] data: NewFunding,
    ) -> FieldResult<Funding> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;

        Funding::create(&context.db, &data).map_err(|e| e.into())
    }

    #[graphql(description = "Create a new location with the specified values")]
    fn create_location(
        context: &Context,
        #[graphql(description = "Values for location to be created")] data: NewLocation,
    ) -> FieldResult<Location> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        // Only superusers can create new locations where Location Platform is Thoth
        if !context.account_access.is_superuser && data.location_platform == LocationPlatform::Thoth
        {
            return Err(ThothError::ThothLocationError.into());
        }
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

    #[graphql(description = "Create a new price with the specified values")]
    fn create_price(
        context: &Context,
        #[graphql(description = "Values for price to be created")] data: NewPrice,
    ) -> FieldResult<Price> {
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

    #[graphql(description = "Create a new subject with the specified values")]
    fn create_subject(
        context: &Context,
        #[graphql(description = "Values for subject to be created")] data: NewSubject,
    ) -> FieldResult<Subject> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;

        check_subject(&data.subject_type, &data.subject_code)?;

        Subject::create(&context.db, &data).map_err(|e| e.into())
    }

    #[graphql(description = "Create a new affiliation with the specified values")]
    fn create_affiliation(
        context: &Context,
        #[graphql(description = "Values for affiliation to be created")] data: NewAffiliation,
    ) -> FieldResult<Affiliation> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_contribution_id(
                &context.db,
                data.contribution_id,
            )?)?;

        Affiliation::create(&context.db, &data).map_err(|e| e.into())
    }

    #[graphql(description = "Create a new work relation with the specified values")]
    fn create_work_relation(
        context: &Context,
        #[graphql(description = "Values for work relation to be created")] data: NewWorkRelation,
    ) -> FieldResult<WorkRelation> {
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

    #[graphql(description = "Create a new reference with the specified values")]
    fn create_reference(
        context: &Context,
        #[graphql(description = "Values for reference to be created")] data: NewReference,
    ) -> FieldResult<Reference> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context
            .account_access
            .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;

        Reference::create(&context.db, &data).map_err(|e| e.into())
    }

    #[graphql(description = "Update an existing work with the specified values")]
    fn update_work(
        context: &Context,
        #[graphql(description = "Values to apply to existing work")] data: PatchWork,
    ) -> FieldResult<Work> {
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

        data.validate()?;

        if work.is_published() && !data.is_published() && !context.account_access.is_superuser {
            return Err(ThothError::ThothSetWorkStatusError.into());
        }

        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        // update the work and, if it succeeds, synchronise its children statuses and pub. date
        match work.update(&context.db, &data, &account_id) {
            Ok(w) => {
                // update chapters if their pub. data, withdrawn_date or work_status doesn't match the parent's
                for child in work.children(&context.db)? {
                    if child.publication_date != w.publication_date
                        || child.work_status != w.work_status
                        || child.withdrawn_date != w.withdrawn_date
                    {
                        let mut data: PatchWork = child.clone().into();
                        data.publication_date = w.publication_date;
                        data.withdrawn_date = w.withdrawn_date;
                        data.work_status = w.work_status;
                        child.update(&context.db, &data, &account_id)?;
                    }
                }
                Ok(w)
            }
            Err(e) => Err(e.into()),
        }
    }

    #[graphql(description = "Update an existing publisher with the specified values")]
    fn update_publisher(
        context: &Context,
        #[graphql(description = "Values to apply to existing publisher")] data: PatchPublisher,
    ) -> FieldResult<Publisher> {
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

    #[graphql(description = "Update an existing imprint with the specified values")]
    fn update_imprint(
        context: &Context,
        #[graphql(description = "Values to apply to existing imprint")] data: PatchImprint,
    ) -> FieldResult<Imprint> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let imprint = Imprint::from_id(&context.db, &data.imprint_id).unwrap();
        context.account_access.can_edit(imprint.publisher_id())?;

        if data.publisher_id != imprint.publisher_id {
            context.account_access.can_edit(data.publisher_id)?;
        }
        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        imprint
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    #[graphql(description = "Update an existing contributor with the specified values")]
    fn update_contributor(
        context: &Context,
        #[graphql(description = "Values to apply to existing contributor")] data: PatchContributor,
    ) -> FieldResult<Contributor> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        Contributor::from_id(&context.db, &data.contributor_id)
            .unwrap()
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    #[graphql(description = "Update an existing contribution with the specified values")]
    fn update_contribution(
        context: &Context,
        #[graphql(description = "Values to apply to existing contribution")]
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

    #[graphql(description = "Update an existing publication with the specified values")]
    fn update_publication(
        context: &Context,
        #[graphql(description = "Values to apply to existing publication")] data: PatchPublication,
    ) -> FieldResult<Publication> {
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

    #[graphql(description = "Update an existing series with the specified values")]
    fn update_series(
        context: &Context,
        #[graphql(description = "Values to apply to existing series")] data: PatchSeries,
    ) -> FieldResult<Series> {
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

    #[graphql(description = "Update an existing issue with the specified values")]
    fn update_issue(
        context: &Context,
        #[graphql(description = "Values to apply to existing issue")] data: PatchIssue,
    ) -> FieldResult<Issue> {
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

    #[graphql(description = "Update an existing language with the specified values")]
    fn update_language(
        context: &Context,
        #[graphql(description = "Values to apply to existing language")] data: PatchLanguage,
    ) -> FieldResult<Language> {
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

    #[graphql(description = "Update an existing title with the specified values")]
    fn update_title(
        context: &Context,
        #[graphql(description = "Values to apply to existing title")] data: PatchTitle,
    ) -> FieldResult<Title> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let title = Title::from_id(&context.db, &data.title_id).unwrap();
        context
            .account_access
            .can_edit(title.publisher_id(&context.db)?)?;

        if data.work_id != title.work_id {
            context
                .account_access
                .can_edit(publisher_id_from_work_id(&context.db, data.work_id)?)?;
        }

        let has_canonical_title = match Work::from_id(&context.db, &data.work_id)?.title(context) {
            Ok(_) => true,
            Err(_) => false,
        };

        // Only superusers can update the canonical location when a Thoth Location Platform canonical location already exists
        if has_canonical_title && data.canonical && !context.account_access.is_superuser {
            return Err(ThothError::ThothUpdateCanonicalError.into());
        }

        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        title
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    #[graphql(description = "Update an existing institution with the specified values")]
    fn update_institution(
        context: &Context,
        #[graphql(description = "Values to apply to existing institution")] data: PatchInstitution,
    ) -> FieldResult<Institution> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        Institution::from_id(&context.db, &data.institution_id)
            .unwrap()
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    #[graphql(description = "Update an existing funding with the specified values")]
    fn update_funding(
        context: &Context,
        #[graphql(description = "Values to apply to existing funding")] data: PatchFunding,
    ) -> FieldResult<Funding> {
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

    #[graphql(description = "Update an existing location with the specified values")]
    fn update_location(
        context: &Context,
        #[graphql(description = "Values to apply to existing location")] data: PatchLocation,
    ) -> FieldResult<Location> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let current_location = Location::from_id(&context.db, &data.location_id).unwrap();
        let has_canonical_thoth_location = Publication::from_id(&context.db, &data.publication_id)?
            .locations(
                context,
                Some(1),
                None,
                None,
                Some(vec![LocationPlatform::Thoth]),
            )?
            .first()
            .is_some_and(|location| location.canonical);
        // Only superusers can update the canonical location when a Thoth Location Platform canonical location already exists
        if has_canonical_thoth_location && data.canonical && !context.account_access.is_superuser {
            return Err(ThothError::ThothUpdateCanonicalError.into());
        }

        // Only superusers can edit locations where Location Platform is Thoth
        if !context.account_access.is_superuser
            && current_location.location_platform == LocationPlatform::Thoth
        {
            return Err(ThothError::ThothLocationError.into());
        }
        context
            .account_access
            .can_edit(current_location.publisher_id(&context.db)?)?;

        if data.publication_id != current_location.publication_id {
            context
                .account_access
                .can_edit(publisher_id_from_publication_id(
                    &context.db,
                    data.publication_id,
                )?)?;
        }

        if data.canonical {
            data.canonical_record_complete(&context.db)?;
        }

        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        current_location
            .update(&context.db, &data, &account_id)
            .map_err(|e| e.into())
    }

    #[graphql(description = "Update an existing price with the specified values")]
    fn update_price(
        context: &Context,
        #[graphql(description = "Values to apply to existing price")] data: PatchPrice,
    ) -> FieldResult<Price> {
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

    #[graphql(description = "Update an existing subject with the specified values")]
    fn update_subject(
        context: &Context,
        #[graphql(description = "Values to apply to existing subject")] data: PatchSubject,
    ) -> FieldResult<Subject> {
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

    #[graphql(description = "Update an existing affiliation with the specified values")]
    fn update_affiliation(
        context: &Context,
        #[graphql(description = "Values to apply to existing affiliation")] data: PatchAffiliation,
    ) -> FieldResult<Affiliation> {
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

    #[graphql(description = "Update an existing work relation with the specified values")]
    fn update_work_relation(
        context: &Context,
        #[graphql(description = "Values to apply to existing work relation")]
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

    #[graphql(description = "Update an existing reference with the specified values")]
    fn update_reference(
        context: &Context,
        #[graphql(description = "Values to apply to existing reference")] data: PatchReference,
    ) -> FieldResult<Reference> {
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

    #[graphql(description = "Delete a single work using its ID")]
    fn delete_work(
        context: &Context,
        #[graphql(description = "Thoth ID of work to be deleted")] work_id: Uuid,
    ) -> FieldResult<Work> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let work = Work::from_id(&context.db, &work_id).unwrap();
        context
            .account_access
            .can_edit(work.publisher_id(&context.db)?)?;

        if work.is_published() && !context.account_access.is_superuser {
            return Err(ThothError::ThothDeleteWorkError.into());
        }

        work.delete(&context.db).map_err(|e| e.into())
    }

    #[graphql(description = "Delete a single publisher using its ID")]
    fn delete_publisher(
        context: &Context,
        #[graphql(description = "Thoth ID of publisher to be deleted")] publisher_id: Uuid,
    ) -> FieldResult<Publisher> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let publisher = Publisher::from_id(&context.db, &publisher_id).unwrap();
        context.account_access.can_edit(publisher_id)?;

        publisher.delete(&context.db).map_err(|e| e.into())
    }

    #[graphql(description = "Delete a single imprint using its ID")]
    fn delete_imprint(
        context: &Context,
        #[graphql(description = "Thoth ID of imprint to be deleted")] imprint_id: Uuid,
    ) -> FieldResult<Imprint> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let imprint = Imprint::from_id(&context.db, &imprint_id).unwrap();
        context.account_access.can_edit(imprint.publisher_id())?;

        imprint.delete(&context.db).map_err(|e| e.into())
    }

    #[graphql(description = "Delete a single contributor using its ID")]
    fn delete_contributor(
        context: &Context,
        #[graphql(description = "Thoth ID of contributor to be deleted")] contributor_id: Uuid,
    ) -> FieldResult<Contributor> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let contributor = Contributor::from_id(&context.db, &contributor_id).unwrap();
        for linked_publisher_id in contributor.linked_publisher_ids(&context.db)? {
            context.account_access.can_edit(linked_publisher_id)?;
        }

        contributor.delete(&context.db).map_err(|e| e.into())
    }

    #[graphql(description = "Delete a single contribution using its ID")]
    fn delete_contribution(
        context: &Context,
        #[graphql(description = "Thoth ID of contribution to be deleted")] contribution_id: Uuid,
    ) -> FieldResult<Contribution> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let contribution = Contribution::from_id(&context.db, &contribution_id).unwrap();
        context
            .account_access
            .can_edit(contribution.publisher_id(&context.db)?)?;

        contribution.delete(&context.db).map_err(|e| e.into())
    }

    #[graphql(description = "Delete a single publication using its ID")]
    fn delete_publication(
        context: &Context,
        #[graphql(description = "Thoth ID of publication to be deleted")] publication_id: Uuid,
    ) -> FieldResult<Publication> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let publication = Publication::from_id(&context.db, &publication_id).unwrap();
        context
            .account_access
            .can_edit(publication.publisher_id(&context.db)?)?;

        publication.delete(&context.db).map_err(|e| e.into())
    }

    #[graphql(description = "Delete a single series using its ID")]
    fn delete_series(
        context: &Context,
        #[graphql(description = "Thoth ID of series to be deleted")] series_id: Uuid,
    ) -> FieldResult<Series> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let series = Series::from_id(&context.db, &series_id).unwrap();
        context
            .account_access
            .can_edit(series.publisher_id(&context.db)?)?;

        series.delete(&context.db).map_err(|e| e.into())
    }

    #[graphql(description = "Delete a single issue using its ID")]
    fn delete_issue(
        context: &Context,
        #[graphql(description = "Thoth ID of issue to be deleted")] issue_id: Uuid,
    ) -> FieldResult<Issue> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let issue = Issue::from_id(&context.db, &issue_id).unwrap();
        context
            .account_access
            .can_edit(issue.publisher_id(&context.db)?)?;

        issue.delete(&context.db).map_err(|e| e.into())
    }

    #[graphql(description = "Delete a single language using its ID")]
    fn delete_language(
        context: &Context,
        #[graphql(description = "Thoth ID of language to be deleted")] language_id: Uuid,
    ) -> FieldResult<Language> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let language = Language::from_id(&context.db, &language_id).unwrap();
        context
            .account_access
            .can_edit(language.publisher_id(&context.db)?)?;

        language.delete(&context.db).map_err(|e| e.into())
    }

    #[graphql(description = "Delete a single title using its ID")]
    fn delete_title(
        context: &Context,
        #[graphql(description = "Thoth ID of title to be deleted")] title_id: Uuid,
    ) -> FieldResult<Title> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let title = Title::from_id(&context.db, &title_id).unwrap();
        context
            .account_access
            .can_edit(title.publisher_id(&context.db)?)?;

        title.delete(&context.db).map_err(|e| e.into())
    }

    #[graphql(description = "Delete a single institution using its ID")]
    fn delete_institution(
        context: &Context,
        #[graphql(description = "Thoth ID of institution to be deleted")] institution_id: Uuid,
    ) -> FieldResult<Institution> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let institution = Institution::from_id(&context.db, &institution_id).unwrap();
        for linked_publisher_id in institution.linked_publisher_ids(&context.db)? {
            context.account_access.can_edit(linked_publisher_id)?;
        }

        institution.delete(&context.db).map_err(|e| e.into())
    }

    #[graphql(description = "Delete a single funding using its ID")]
    fn delete_funding(
        context: &Context,
        #[graphql(description = "Thoth ID of funding to be deleted")] funding_id: Uuid,
    ) -> FieldResult<Funding> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let funding = Funding::from_id(&context.db, &funding_id).unwrap();
        context
            .account_access
            .can_edit(funding.publisher_id(&context.db)?)?;

        funding.delete(&context.db).map_err(|e| e.into())
    }

    #[graphql(description = "Delete a single location using its ID")]
    fn delete_location(
        context: &Context,
        #[graphql(description = "Thoth ID of location to be deleted")] location_id: Uuid,
    ) -> FieldResult<Location> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let location = Location::from_id(&context.db, &location_id).unwrap();
        // Only superusers can delete locations where Location Platform is Thoth
        if !context.account_access.is_superuser
            && location.location_platform == LocationPlatform::Thoth
        {
            return Err(ThothError::ThothLocationError.into());
        }
        context
            .account_access
            .can_edit(location.publisher_id(&context.db)?)?;

        location.delete(&context.db).map_err(|e| e.into())
    }

    #[graphql(description = "Delete a single price using its ID")]
    fn delete_price(
        context: &Context,
        #[graphql(description = "Thoth ID of price to be deleted")] price_id: Uuid,
    ) -> FieldResult<Price> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let price = Price::from_id(&context.db, &price_id).unwrap();
        context
            .account_access
            .can_edit(price.publisher_id(&context.db)?)?;

        price.delete(&context.db).map_err(|e| e.into())
    }

    #[graphql(description = "Delete a single subject using its ID")]
    fn delete_subject(
        context: &Context,
        #[graphql(description = "Thoth ID of subject to be deleted")] subject_id: Uuid,
    ) -> FieldResult<Subject> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let subject = Subject::from_id(&context.db, &subject_id).unwrap();
        context
            .account_access
            .can_edit(subject.publisher_id(&context.db)?)?;

        subject.delete(&context.db).map_err(|e| e.into())
    }

    #[graphql(description = "Delete a single affiliation using its ID")]
    fn delete_affiliation(
        context: &Context,
        #[graphql(description = "Thoth ID of affiliation to be deleted")] affiliation_id: Uuid,
    ) -> FieldResult<Affiliation> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let affiliation = Affiliation::from_id(&context.db, &affiliation_id).unwrap();
        context
            .account_access
            .can_edit(affiliation.publisher_id(&context.db)?)?;

        affiliation.delete(&context.db).map_err(|e| e.into())
    }

    #[graphql(description = "Delete a single work relation using its ID")]
    fn delete_work_relation(
        context: &Context,
        #[graphql(description = "Thoth ID of work relation to be deleted")] work_relation_id: Uuid,
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

    #[graphql(description = "Delete a single reference using its ID")]
    fn delete_reference(
        context: &Context,
        #[graphql(description = "Thoth ID of reference to be deleted")] reference_id: Uuid,
    ) -> FieldResult<Reference> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let reference = Reference::from_id(&context.db, &reference_id).unwrap();
        context
            .account_access
            .can_edit(reference.publisher_id(&context.db)?)?;

        reference.delete(&context.db).map_err(|e| e.into())
    }
}

#[juniper::graphql_object(Context = Context, description = "A written text that can be published")]
impl Work {
    #[graphql(description = "Thoth ID of the work")]
    pub fn work_id(&self) -> &Uuid {
        &self.work_id
    }

    #[graphql(description = "Type of the work")]
    pub fn work_type(&self) -> &WorkType {
        &self.work_type
    }

    #[graphql(description = "Publication status of the work")]
    pub fn work_status(&self) -> &WorkStatus {
        &self.work_status
    }

    #[graphql(description = "Concatenation of title and subtitle with punctuation mark")]
    #[graphql(
        deprecated = "Please use Work `titles` field instead to get the correct full title in a multilingual manner"
    )]
    pub fn full_title(&self, ctx: &Context) -> FieldResult<String> {
        let mut connection = ctx.db.get()?;
        let title = work_title::table
            .filter(work_title::work_id.eq(&self.work_id))
            .filter(work_title::canonical.eq(true))
            .first::<Title>(&mut connection)?;
        Ok(title.full_title)
    }

    #[graphql(description = "Main title of the work (excluding subtitle)")]
    #[graphql(
        deprecated = "Please use Work `titles` field instead to get the correct title in a multilingual manner"
    )]
    pub fn title(&self, ctx: &Context) -> FieldResult<String> {
        let mut connection = ctx.db.get()?;
        let title = work_title::table
            .filter(work_title::work_id.eq(&self.work_id))
            .filter(work_title::canonical.eq(true))
            .first::<Title>(&mut connection)?;
        Ok(title.title)
    }

    #[graphql(description = "Query titles by work ID")]
    fn titles(
        &self,
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on title_, subtitle, full_title fields"
        )]
        filter: Option<String>,
        #[graphql(
            default = TitleOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<TitleOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results with these locale codes"
        )]
        locale_codes: Option<Vec<LocaleCode>>,
    ) -> FieldResult<Vec<Title>> {
        Title::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            filter,
            order.unwrap_or_default(),
            vec![],
            Some(self.work_id),
            None,
            locale_codes.unwrap_or_default(),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Secondary title of the work (excluding main title)")]
    pub fn subtitle(&self, ctx: &Context) -> FieldResult<Option<String>> {
        let mut connection = ctx.db.get()?;
        let title = work_title::table
            .filter(work_title::work_id.eq(&self.work_id))
            .filter(work_title::canonical.eq(true))
            .first::<Title>(&mut connection)
            .map_err(|e| FieldError::new(e.to_string(), Value::null()))?;
        Ok(title.subtitle)
    }

    #[graphql(description = "Internal reference code")]
    pub fn reference(&self) -> Option<&String> {
        self.reference.as_ref()
    }

    #[graphql(description = "Edition number of the work (not applicable to chapters)")]
    pub fn edition(&self) -> Option<&i32> {
        self.edition.as_ref()
    }

    #[graphql(description = "Thoth ID of the work's imprint")]
    pub fn imprint_id(&self) -> Uuid {
        self.imprint_id
    }

    #[graphql(
        description = "Digital Object Identifier of the work as full URL, using the HTTPS scheme and the doi.org domain (e.g. https://doi.org/10.11647/obp.0001)"
    )]
    pub fn doi(&self) -> Option<&Doi> {
        self.doi.as_ref()
    }

    #[graphql(description = "Date the work was published")]
    pub fn publication_date(&self) -> Option<NaiveDate> {
        self.publication_date
    }

    #[graphql(
        description = "Date the work was withdrawn from publication. Only applies to out of print and withdrawn works."
    )]
    pub fn withdrawn_date(&self) -> Option<NaiveDate> {
        self.withdrawn_date
    }

    #[graphql(description = "Place of publication of the work")]
    pub fn place(&self) -> Option<&String> {
        self.place.as_ref()
    }

    #[graphql(
        description = "Total number of pages in the work. In most cases, unnumbered pages (e.g. endpapers) should be omitted from this count."
    )]
    pub fn page_count(&self) -> Option<&i32> {
        self.page_count.as_ref()
    }

    #[graphql(
        description = "Breakdown of work's page count into front matter, main content, and/or back matter (e.g. 'xi + 140')"
    )]
    pub fn page_breakdown(&self) -> Option<&String> {
        self.page_breakdown.as_ref()
    }

    #[graphql(description = "Total number of images in the work")]
    pub fn image_count(&self) -> Option<&i32> {
        self.image_count.as_ref()
    }

    #[graphql(description = "Total number of tables in the work")]
    pub fn table_count(&self) -> Option<&i32> {
        self.table_count.as_ref()
    }

    #[graphql(description = "Total number of audio fragments in the work")]
    pub fn audio_count(&self) -> Option<&i32> {
        self.audio_count.as_ref()
    }

    #[graphql(description = "Total number of video fragments in the work")]
    pub fn video_count(&self) -> Option<&i32> {
        self.video_count.as_ref()
    }

    #[graphql(
        description = "URL of the license which applies to this work (frequently a Creative Commons license for open-access works)"
    )]
    pub fn license(&self) -> Option<&String> {
        self.license.as_ref()
    }

    #[graphql(description = "Copyright holder of the work")]
    pub fn copyright_holder(&self) -> Option<&String> {
        self.copyright_holder.as_ref()
    }

    #[graphql(description = "URL of the web page of the work")]
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

    #[graphql(
        description = "Short abstract of the work. Where a work has two different versions of the abstract, the truncated version should be entered here. Otherwise, it can be left blank. This field is not output in metadata formats; where relevant, Long Abstract is used instead."
    )]
    pub fn short_abstract(&self) -> Option<&String> {
        self.short_abstract.as_ref()
    }

    #[graphql(
        description = "Abstract of the work. Where a work has only one abstract, it should be entered here, and Short Abstract can be left blank. Long Abstract is output in metadata formats, and Short Abstract is not."
    )]
    pub fn long_abstract(&self) -> Option<&String> {
        self.long_abstract.as_ref()
    }

    #[graphql(
        description = "A general-purpose field used to include information that does not have a specific designated field"
    )]
    pub fn general_note(&self) -> Option<&String> {
        self.general_note.as_ref()
    }

    #[graphql(
        description = "Indicates that the work contains a bibliography or other similar information"
    )]
    pub fn bibliography_note(&self) -> Option<&String> {
        self.bibliography_note.as_ref()
    }

    #[graphql(description = "Table of contents of the work (not applicable to chapters)")]
    pub fn toc(&self) -> Option<&String> {
        self.toc.as_ref()
    }

    #[graphql(description = "URL of the work's cover image")]
    pub fn cover_url(&self) -> Option<&String> {
        self.cover_url.as_ref()
    }

    #[graphql(description = "Caption describing the work's cover image")]
    pub fn cover_caption(&self) -> Option<&String> {
        self.cover_caption.as_ref()
    }

    #[graphql(description = "Date and time at which the work record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the work record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
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

    #[graphql(
        description = "Date and time at which the work record or any of its linked records was last updated"
    )]
    pub fn updated_at_with_relations(&self) -> Timestamp {
        self.updated_at_with_relations
    }

    #[graphql(description = "Get this work's imprint")]
    pub fn imprint(&self, context: &Context) -> FieldResult<Imprint> {
        Imprint::from_id(&context.db, &self.imprint_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get contributions linked to this work")]
    pub fn contributions(
        &self,
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = ContributionOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<ContributionOrderBy>,
        #[graphql(
            default = vec![],
            description = "Specific types to filter by",
        )]
        contribution_types: Option<Vec<ContributionType>>,
    ) -> FieldResult<Vec<Contribution>> {
        Contribution::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            None,
            order.unwrap_or_default(),
            vec![],
            Some(self.work_id),
            None,
            contribution_types.unwrap_or_default(),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[allow(clippy::too_many_arguments)]
    #[graphql(description = "Get languages linked to this work")]
    pub fn languages(
        &self,
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = LanguageOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<LanguageOrderBy>,
        #[graphql(
            default = vec![],
            description = "Specific languages to filter by"
        )]
        language_codes: Option<Vec<LanguageCode>>,
        #[graphql(
            description = "(deprecated) A specific relation to filter by"
        )]
        language_relation: Option<LanguageRelation>,
        #[graphql(
            default = vec![],
            description = "Specific relations to filter by"
        )]
        language_relations: Option<Vec<LanguageRelation>>,
    ) -> FieldResult<Vec<Language>> {
        let mut relations = language_relations.unwrap_or_default();
        if let Some(relation) = language_relation {
            relations.push(relation);
        }
        Language::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            None,
            order.unwrap_or_default(),
            vec![],
            Some(self.work_id),
            None,
            language_codes.unwrap_or_default(),
            relations,
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Get publications linked to this work")]
    pub fn publications(
        &self,
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on isbn"
        )]
        filter: Option<String>,
        #[graphql(
            default = PublicationOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<PublicationOrderBy>,
        #[graphql(
            default = vec![],
            description = "Specific types to filter by",
        )]
        publication_types: Option<Vec<PublicationType>>,
    ) -> FieldResult<Vec<Publication>> {
        Publication::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            filter,
            order.unwrap_or_default(),
            vec![],
            Some(self.work_id),
            None,
            publication_types.unwrap_or_default(),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Get subjects linked to this work")]
    pub fn subjects(
        &self,
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on subject_code"
        )]
        filter: Option<String>,
        #[graphql(
            default = SubjectOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<SubjectOrderBy>,
        #[graphql(
            default = vec![],
            description = "Specific types to filter by",
        )]
        subject_types: Option<Vec<SubjectType>>,
    ) -> FieldResult<Vec<Subject>> {
        Subject::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            filter,
            order.unwrap_or_default(),
            vec![],
            Some(self.work_id),
            None,
            subject_types.unwrap_or_default(),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Get fundings linked to this work")]
    pub fn fundings(
        &self,
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = FundingOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<FundingOrderBy>,
    ) -> FieldResult<Vec<Funding>> {
        Funding::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            None,
            order.unwrap_or_default(),
            vec![],
            Some(self.work_id),
            None,
            vec![],
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Get issues linked to this work")]
    pub fn issues(
        &self,
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = IssueOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<IssueOrderBy>,
    ) -> FieldResult<Vec<Issue>> {
        Issue::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            None,
            order.unwrap_or_default(),
            vec![],
            Some(self.work_id),
            None,
            vec![],
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }
    #[graphql(description = "Get other works related to this work")]
    pub fn relations(
        &self,
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = WorkRelationOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<WorkRelationOrderBy>,
        #[graphql(
            default = vec![],
            description = "Specific types to filter by",
        )]
        relation_types: Option<Vec<RelationType>>,
    ) -> FieldResult<Vec<WorkRelation>> {
        WorkRelation::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            None,
            order.unwrap_or_default(),
            vec![],
            Some(self.work_id),
            None,
            relation_types.unwrap_or_default(),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }
    #[graphql(description = "Get references cited by this work")]
    pub fn references(
        &self,
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on doi, unstructured_citation, issn, isbn, journal_title, article_title, series_title, volume_title, author, standard_designator, standards_body_name, and standards_body_acronym"
        )]
        filter: Option<String>,
        #[graphql(
            default = ReferenceOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<ReferenceOrderBy>,
    ) -> FieldResult<Vec<Reference>> {
        Reference::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            filter,
            order.unwrap_or_default(),
            vec![],
            Some(self.work_id),
            None,
            vec![],
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }
}

#[juniper::graphql_object(Context = Context, description = "A manifestation of a written text")]
impl Publication {
    #[graphql(description = "Thoth ID of the publication")]
    pub fn publication_id(&self) -> Uuid {
        self.publication_id
    }

    #[graphql(description = "Format of this publication")]
    pub fn publication_type(&self) -> &PublicationType {
        &self.publication_type
    }

    #[graphql(description = "Thoth ID of the work to which this publication belongs")]
    pub fn work_id(&self) -> Uuid {
        self.work_id
    }

    #[graphql(
        description = "International Standard Book Number of the publication, in ISBN-13 format"
    )]
    pub fn isbn(&self) -> Option<&Isbn> {
        self.isbn.as_ref()
    }

    #[graphql(description = "Date and time at which the publication record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the publication record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    #[graphql(
        description = "Width of the physical Publication (in mm, cm or in) (only applicable to non-Chapter Paperbacks and Hardbacks)"
    )]
    pub fn width(
        &self,
        #[graphql(
            default = LengthUnit::default(),
            description = "Unit of measurement in which to represent the width (mm, cm or in)",
        )]
        units: LengthUnit,
    ) -> Option<f64> {
        match units {
            LengthUnit::Mm => self.width_mm,
            LengthUnit::Cm => self
                .width_mm
                .map(|w| w.convert_length_from_to(&LengthUnit::Mm, &LengthUnit::Cm)),
            LengthUnit::In => self.width_in,
        }
    }

    #[graphql(
        description = "Height of the physical Publication (in mm, cm or in) (only applicable to non-Chapter Paperbacks and Hardbacks)"
    )]
    pub fn height(
        &self,
        #[graphql(
            default = LengthUnit::default(),
            description = "Unit of measurement in which to represent the height (mm, cm or in)",
        )]
        units: LengthUnit,
    ) -> Option<f64> {
        match units {
            LengthUnit::Mm => self.height_mm,
            LengthUnit::Cm => self
                .height_mm
                .map(|w| w.convert_length_from_to(&LengthUnit::Mm, &LengthUnit::Cm)),
            LengthUnit::In => self.height_in,
        }
    }

    #[graphql(
        description = "Depth of the physical Publication (in mm, cm or in) (only applicable to non-Chapter Paperbacks and Hardbacks)"
    )]
    pub fn depth(
        &self,
        #[graphql(
            default = LengthUnit::default(),
            description = "Unit of measurement in which to represent the depth (mm, cm or in)",
        )]
        units: LengthUnit,
    ) -> Option<f64> {
        match units {
            LengthUnit::Mm => self.depth_mm,
            LengthUnit::Cm => self
                .depth_mm
                .map(|w| w.convert_length_from_to(&LengthUnit::Mm, &LengthUnit::Cm)),
            LengthUnit::In => self.depth_in,
        }
    }

    #[graphql(
        description = "Weight of the physical Publication (in g or oz) (only applicable to non-Chapter Paperbacks and Hardbacks)"
    )]
    pub fn weight(
        &self,
        #[graphql(
            default = WeightUnit::default(),
            description = "Unit of measurement in which to represent the weight (grams or ounces)",
        )]
        units: WeightUnit,
    ) -> Option<f64> {
        match units {
            WeightUnit::G => self.weight_g,
            WeightUnit::Oz => self.weight_oz,
        }
    }

    #[graphql(description = "Get prices linked to this publication")]
    pub fn prices(
        &self,
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = PriceOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<PriceOrderBy>,
        #[graphql(
            default = vec![],
            description = "Specific currencies to filter by"
        )]
        currency_codes: Option<Vec<CurrencyCode>>,
    ) -> FieldResult<Vec<Price>> {
        Price::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            None,
            order.unwrap_or_default(),
            vec![],
            Some(self.publication_id),
            None,
            currency_codes.unwrap_or_default(),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Get locations linked to this publication")]
    pub fn locations(
        &self,
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = LocationOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<LocationOrderBy>,
        #[graphql(
            default = vec![],
            description = "Specific platforms to filter by"
        )]
        location_platforms: Option<Vec<LocationPlatform>>,
    ) -> FieldResult<Vec<Location>> {
        Location::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            None,
            order.unwrap_or_default(),
            vec![],
            Some(self.publication_id),
            None,
            location_platforms.unwrap_or_default(),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Get the work to which this publication belongs")]
    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(|e| e.into())
    }
}

#[juniper::graphql_object(Context = Context, description = "An organisation that produces and distributes written texts.")]
impl Publisher {
    #[graphql(description = "Thoth ID of the publisher")]
    pub fn publisher_id(&self) -> Uuid {
        self.publisher_id
    }

    #[graphql(description = "Name of the publisher")]
    pub fn publisher_name(&self) -> &String {
        &self.publisher_name
    }

    #[graphql(description = "Short name of the publisher, if any (e.g. an abbreviation)")]
    pub fn publisher_shortname(&self) -> Option<&String> {
        self.publisher_shortname.as_ref()
    }

    #[graphql(description = "URL of the publisher's website")]
    pub fn publisher_url(&self) -> Option<&String> {
        self.publisher_url.as_ref()
    }

    #[graphql(description = "Date and time at which the publisher record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the publisher record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    #[graphql(description = "Get imprints linked to this publisher")]
    pub fn imprints(
        &self,
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on imprint_name and imprint_url"
        )]
        filter: Option<String>,
        #[graphql(
           default = {
                ImprintOrderBy {
                    field: ImprintField::ImprintName,
                    direction: Direction::Asc,
                }
            },
            description = "The order in which to sort the results"
        )]
        order: Option<ImprintOrderBy>,
    ) -> FieldResult<Vec<Imprint>> {
        Imprint::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            filter,
            order.unwrap_or_default(),
            vec![],
            Some(self.publisher_id),
            None,
            vec![],
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }
}

#[juniper::graphql_object(Context = Context, description = "The brand under which a publisher issues works.")]
impl Imprint {
    #[graphql(description = "Thoth ID of the imprint")]
    pub fn imprint_id(&self) -> Uuid {
        self.imprint_id
    }

    #[graphql(description = "Thoth ID of the publisher to which this imprint belongs")]
    pub fn publisher_id(&self) -> Uuid {
        self.publisher_id
    }

    #[graphql(description = "Name of the imprint")]
    pub fn imprint_name(&self) -> &String {
        &self.imprint_name
    }

    #[graphql(description = "URL of the imprint's landing page")]
    pub fn imprint_url(&self) -> Option<&String> {
        self.imprint_url.as_ref()
    }

    #[graphql(
        description = "DOI of the imprint's Crossmark policy page, if publisher participates. Crossmark 'gives readers quick and easy access to the
    current status of an item of content, including any corrections, retractions, or updates'. More: https://www.crossref.org/services/crossmark/"
    )]
    pub fn crossmark_doi(&self) -> Option<&Doi> {
        self.crossmark_doi.as_ref()
    }

    #[graphql(description = "Date and time at which the imprint record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the imprint record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    #[graphql(description = "Get the publisher to which this imprint belongs")]
    pub fn publisher(&self, context: &Context) -> FieldResult<Publisher> {
        Publisher::from_id(&context.db, &self.publisher_id).map_err(|e| e.into())
    }

    #[allow(clippy::too_many_arguments)]
    #[graphql(description = "Get works linked to this imprint")]
    pub fn works(
        &self,
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_title, doi, reference, short_abstract, long_abstract, and landing_page"
        )]
        filter: Option<String>,
        #[graphql(
            default = WorkOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<WorkOrderBy>,
        #[graphql(
            default = vec![],
            description = "Specific types to filter by",
        )]
        work_types: Option<Vec<WorkType>>,
        #[graphql(description = "(deprecated) A specific status to filter by")] work_status: Option<
            WorkStatus,
        >,
        #[graphql(
            default = vec![],
            description = "Specific statuses to filter by"
        )]
        work_statuses: Option<Vec<WorkStatus>>,
        #[graphql(
            description = "Only show results updated either before (less than) or after (greater than) the specified timestamp"
        )]
        updated_at_with_relations: Option<TimeExpression>,
    ) -> FieldResult<Vec<Work>> {
        let mut statuses = work_statuses.unwrap_or_default();
        if let Some(status) = work_status {
            statuses.push(status);
        }
        Work::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            filter,
            order.unwrap_or_default(),
            vec![],
            Some(self.imprint_id),
            None,
            work_types.unwrap_or_default(),
            statuses,
            updated_at_with_relations,
        )
        .map_err(|e| e.into())
    }
}

#[juniper::graphql_object(Context = Context, description = "A person who has been involved in the production of a written text.")]
impl Contributor {
    #[graphql(description = "Thoth ID of the contributor")]
    pub fn contributor_id(&self) -> Uuid {
        self.contributor_id
    }

    #[graphql(description = "Given or first name(s) of the contributor")]
    pub fn first_name(&self) -> Option<&String> {
        self.first_name.as_ref()
    }

    #[graphql(description = "Family or surname of the contributor")]
    pub fn last_name(&self) -> &String {
        &self.last_name
    }

    #[graphql(
        description = "Full, serialized name of the contributor. Serialization is often culturally determined."
    )]
    pub fn full_name(&self) -> &String {
        &self.full_name
    }

    #[graphql(
        description = "ORCID (Open Researcher and Contributor ID) of the contributor as full URL, using the HTTPS scheme and the orcid.org domain (e.g. https://orcid.org/0000-0002-1825-0097)"
    )]
    pub fn orcid(&self) -> Option<&Orcid> {
        self.orcid.as_ref()
    }

    #[graphql(description = "URL of the contributor's website")]
    pub fn website(&self) -> Option<&String> {
        self.website.as_ref()
    }

    #[graphql(description = "Date and time at which the contributor record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the contributor record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    #[graphql(description = "Get contributions linked to this contributor")]
    pub fn contributions(
        &self,
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = ContributionOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<ContributionOrderBy>,
        #[graphql(
            default = vec![],
            description = "Specific types to filter by",
        )]
        contribution_types: Option<Vec<ContributionType>>,
    ) -> FieldResult<Vec<Contribution>> {
        Contribution::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            None,
            order.unwrap_or_default(),
            vec![],
            None,
            Some(self.contributor_id),
            contribution_types.unwrap_or_default(),
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }
}

#[juniper::graphql_object(Context = Context, description = "A person's involvement in the production of a written text.")]
impl Contribution {
    #[graphql(description = "Thoth ID of the contribution")]
    pub fn contribution_id(&self) -> Uuid {
        self.contribution_id
    }

    #[graphql(description = "Thoth ID of the contributor who created the contribution")]
    pub fn contributor_id(&self) -> Uuid {
        self.contributor_id
    }

    #[graphql(description = "Thoth ID of the work in which the contribution appears")]
    pub fn work_id(&self) -> Uuid {
        self.work_id
    }

    #[graphql(description = "Nature of the contribution")]
    pub fn contribution_type(&self) -> &ContributionType {
        &self.contribution_type
    }

    #[graphql(
        description = "Whether this is a main contribution to the work (e.g. contributor credited on title page)"
    )]
    pub fn main_contribution(&self) -> bool {
        self.main_contribution
    }

    #[graphql(description = "Biography of the contributor at the time of contribution")]
    pub fn biography(&self) -> Option<&String> {
        self.biography.as_ref()
    }

    #[graphql(description = "Date and time at which the contribution record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the contribution record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    #[graphql(
        description = "Given or first name(s) of the contributor, as credited in this contribution"
    )]
    pub fn first_name(&self) -> Option<&String> {
        self.first_name.as_ref()
    }

    #[graphql(
        description = "Family or surname of the contributor, as credited in this contribution"
    )]
    pub fn last_name(&self) -> &String {
        &self.last_name
    }

    #[graphql(
        description = "Full, serialized name of the contributor, as credited in this contribution"
    )]
    pub fn full_name(&self) -> &String {
        &self.full_name
    }

    #[graphql(
        description = "Number representing this contribution's position in an ordered list of contributions within the work"
    )]
    pub fn contribution_ordinal(&self) -> &i32 {
        &self.contribution_ordinal
    }

    #[graphql(description = "Get the work in which the contribution appears")]
    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the contributor who created the contribution")]
    pub fn contributor(&self, context: &Context) -> FieldResult<Contributor> {
        Contributor::from_id(&context.db, &self.contributor_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get affiliations linked to this contribution")]
    pub fn affiliations(
        &self,
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = AffiliationOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<AffiliationOrderBy>,
    ) -> FieldResult<Vec<Affiliation>> {
        Affiliation::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            None,
            order.unwrap_or_default(),
            vec![],
            None,
            Some(self.contribution_id),
            vec![],
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }
}

#[juniper::graphql_object(Context = Context, description = "A periodical of publications about a particular subject.")]
impl Series {
    #[graphql(description = "Thoth ID of the series")]
    pub fn series_id(&self) -> Uuid {
        self.series_id
    }

    #[graphql(description = "Type of the series")]
    pub fn series_type(&self) -> &SeriesType {
        &self.series_type
    }

    #[graphql(description = "Name of the series")]
    pub fn series_name(&self) -> &String {
        &self.series_name
    }

    #[graphql(
        description = "Print ISSN (International Standard Serial Number) of the series. This represents the print media version."
    )]
    pub fn issn_print(&self) -> Option<&String> {
        self.issn_print.as_ref()
    }

    #[graphql(
        description = "Electronic ISSN (International Standard Serial Number) of the series. This represents the online version."
    )]
    pub fn issn_digital(&self) -> Option<&String> {
        self.issn_digital.as_ref()
    }

    #[graphql(description = "URL of the series' landing page")]
    pub fn series_url(&self) -> Option<&String> {
        self.series_url.as_ref()
    }

    #[graphql(description = "Description of the series")]
    pub fn series_description(&self) -> Option<&String> {
        self.series_description.as_ref()
    }

    #[graphql(description = "URL of the series' call for proposals page")]
    pub fn series_cfp_url(&self) -> Option<&String> {
        self.series_cfp_url.as_ref()
    }

    #[graphql(description = "Thoth ID of the imprint to which this series belongs")]
    pub fn imprint_id(&self) -> Uuid {
        self.imprint_id
    }

    #[graphql(description = "Date and time at which the series record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the series record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    #[graphql(description = "Get the imprint linked to this series")]
    pub fn imprint(&self, context: &Context) -> FieldResult<Imprint> {
        Imprint::from_id(&context.db, &self.imprint_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get issues linked to this series")]
    pub fn issues(
        &self,
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = IssueOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<IssueOrderBy>,
    ) -> FieldResult<Vec<Issue>> {
        Issue::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            None,
            order.unwrap_or_default(),
            vec![],
            None,
            Some(self.series_id),
            vec![],
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }
}

#[juniper::graphql_object(Context = Context, description = "A work published as a number in a periodical.")]
impl Issue {
    #[graphql(description = "Thoth ID of the issue")]
    pub fn issue_id(&self) -> Uuid {
        self.issue_id
    }

    #[graphql(description = "Thoth ID of the work represented by the issue")]
    pub fn work_id(&self) -> Uuid {
        self.work_id
    }

    #[graphql(description = "Thoth ID of the series to which the issue belongs")]
    pub fn series_id(&self) -> Uuid {
        self.series_id
    }

    #[graphql(
        description = "Number representing this issue's position in an ordered list of issues within the series (does not have to correspond to published issue number)"
    )]
    pub fn issue_ordinal(&self) -> &i32 {
        &self.issue_ordinal
    }

    #[graphql(description = "Date and time at which the issue record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the issue record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    #[graphql(description = "Get the series to which the issue belongs")]
    pub fn series(&self, context: &Context) -> FieldResult<Series> {
        Series::from_id(&context.db, &self.series_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the work represented by the issue")]
    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(|e| e.into())
    }
}

#[juniper::graphql_object(Context = Context, description = "Description of a work's language.")]
impl Language {
    #[graphql(description = "Thoth ID of the language")]
    pub fn language_id(&self) -> Uuid {
        self.language_id
    }

    #[graphql(description = "Thoth ID of the work which has this language")]
    pub fn work_id(&self) -> Uuid {
        self.work_id
    }

    #[graphql(description = "Three-letter ISO 639 code representing the language")]
    pub fn language_code(&self) -> &LanguageCode {
        &self.language_code
    }

    #[graphql(description = "Relation between this language and the original language of the text")]
    pub fn language_relation(&self) -> &LanguageRelation {
        &self.language_relation
    }

    #[graphql(
        description = "Whether this is a main language of the work (e.g. used for large sections of the text rather than just isolated quotations)"
    )]
    pub fn main_language(&self) -> bool {
        self.main_language
    }

    #[graphql(description = "Date and time at which the language record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the language record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    #[graphql(description = "Get the work which has this language")]
    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(|e| e.into())
    }
}

#[juniper::graphql_object(Context = Context, description = "A location, such as a web shop or distribution platform, where a publication can be acquired or viewed.")]
impl Location {
    #[graphql(description = "Thoth ID of the location")]
    pub fn location_id(&self) -> Uuid {
        self.location_id
    }

    #[graphql(description = "Thoth ID of the publication linked to this location")]
    pub fn publication_id(&self) -> Uuid {
        self.publication_id
    }

    #[graphql(description = "Public-facing URL via which the publication can be accessed")]
    pub fn landing_page(&self) -> Option<&String> {
        self.landing_page.as_ref()
    }

    #[graphql(description = "Direct link to the full text file")]
    pub fn full_text_url(&self) -> Option<&String> {
        self.full_text_url.as_ref()
    }

    #[graphql(description = "Platform where the publication is hosted or can be acquired")]
    pub fn location_platform(&self) -> &LocationPlatform {
        &self.location_platform
    }

    #[graphql(
        description = "Whether this is the canonical location for this specific publication (e.g. the main platform on which the print version is sold, or the official version of record hosted on the publisher's own web server)"
    )]
    pub fn canonical(&self) -> bool {
        self.canonical
    }

    #[graphql(description = "Date and time at which the location record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the location record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    #[graphql(description = "Get the publication linked to this location")]
    pub fn publication(&self, context: &Context) -> FieldResult<Publication> {
        Publication::from_id(&context.db, &self.publication_id).map_err(|e| e.into())
    }
}

#[juniper::graphql_object(Context = Context, description = "The amount of money, in any currency, that a publication costs.")]
impl Price {
    #[graphql(description = "Thoth ID of the price")]
    pub fn price_id(&self) -> Uuid {
        self.price_id
    }

    #[graphql(description = "Thoth ID of the publication linked to this price")]
    pub fn publication_id(&self) -> Uuid {
        self.publication_id
    }

    #[graphql(
        description = "Three-letter ISO 4217 code representing the currency used in this price"
    )]
    pub fn currency_code(&self) -> &CurrencyCode {
        &self.currency_code
    }

    #[graphql(description = "Value of the publication in the specified currency")]
    pub fn unit_price(&self) -> f64 {
        self.unit_price
    }

    #[graphql(description = "Date and time at which the price record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the price record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    #[graphql(description = "Get the publication linked to this price")]
    pub fn publication(&self, context: &Context) -> FieldResult<Publication> {
        Publication::from_id(&context.db, &self.publication_id).map_err(|e| e.into())
    }
}

#[juniper::graphql_object(Context = Context, description = "A significant discipline or term related to a work.")]
impl Subject {
    #[graphql(description = "Thoth ID of the subject")]
    pub fn subject_id(&self) -> &Uuid {
        &self.subject_id
    }

    #[graphql(description = "Thoth ID of the work to which the subject is linked")]
    pub fn work_id(&self) -> &Uuid {
        &self.work_id
    }

    #[graphql(description = "Type of the subject (e.g. the subject category scheme being used)")]
    pub fn subject_type(&self) -> &SubjectType {
        &self.subject_type
    }

    #[graphql(description = "Code representing the subject within the specified type")]
    pub fn subject_code(&self) -> &String {
        &self.subject_code
    }

    #[graphql(
        description = "Number representing this subject's position in an ordered list of subjects of the same type within the work (subjects of equal prominence can have the same number)"
    )]
    pub fn subject_ordinal(&self) -> &i32 {
        &self.subject_ordinal
    }

    #[graphql(description = "Date and time at which the subject record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the subject record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    #[graphql(description = "Get the work to which the subject is linked")]
    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(|e| e.into())
    }
}

#[juniper::graphql_object(Context = Context, description = "An organisation with which contributors may be affiliated or by which works may be funded.")]
impl Institution {
    #[graphql(description = "Thoth ID of the institution")]
    pub fn institution_id(&self) -> &Uuid {
        &self.institution_id
    }

    #[graphql(description = "Name of the institution")]
    pub fn institution_name(&self) -> &String {
        &self.institution_name
    }

    #[graphql(
        description = "Digital Object Identifier of the organisation as full URL, using the HTTPS scheme and the doi.org domain (e.g. https://doi.org/10.13039/100014013)"
    )]
    pub fn institution_doi(&self) -> Option<&Doi> {
        self.institution_doi.as_ref()
    }

    #[graphql(
        description = "Three-letter ISO 3166-1 code representing the country where this institution is based"
    )]
    pub fn country_code(&self) -> Option<&CountryCode> {
        self.country_code.as_ref()
    }

    #[graphql(
        description = "Research Organisation Registry identifier of the organisation as full URL, using the HTTPS scheme and the ror.org domain (e.g. https://ror.org/051z6e826)"
    )]
    pub fn ror(&self) -> Option<&Ror> {
        self.ror.as_ref()
    }

    #[graphql(description = "Date and time at which the institution record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the institution record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    #[graphql(description = "Get fundings linked to this institution")]
    pub fn fundings(
        &self,
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = FundingOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<FundingOrderBy>,
    ) -> FieldResult<Vec<Funding>> {
        Funding::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            None,
            order.unwrap_or_default(),
            vec![],
            None,
            Some(self.institution_id),
            vec![],
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }

    #[graphql(description = "Get affiliations linked to this institution")]
    pub fn affiliations(
        &self,
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = AffiliationOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<AffiliationOrderBy>,
    ) -> FieldResult<Vec<Affiliation>> {
        Affiliation::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            None,
            order.unwrap_or_default(),
            vec![],
            Some(self.institution_id),
            None,
            vec![],
            vec![],
            None,
        )
        .map_err(|e| e.into())
    }
}

#[juniper::graphql_object(Context = Context, description = "A grant awarded for the publication of a work by an institution.")]
impl Funding {
    #[graphql(description = "Thoth ID of the funding")]
    pub fn funding_id(&self) -> &Uuid {
        &self.funding_id
    }

    #[graphql(description = "Thoth ID of the funded work")]
    pub fn work_id(&self) -> &Uuid {
        &self.work_id
    }

    #[graphql(description = "Thoth ID of the funding institution")]
    pub fn institution_id(&self) -> &Uuid {
        &self.institution_id
    }

    #[graphql(description = "Name of the funding program")]
    pub fn program(&self) -> Option<&String> {
        self.program.as_ref()
    }

    #[graphql(description = "Name of the funding project")]
    pub fn project_name(&self) -> Option<&String> {
        self.project_name.as_ref()
    }

    #[graphql(description = "Short name of the funding project")]
    pub fn project_shortname(&self) -> Option<&String> {
        self.project_shortname.as_ref()
    }

    #[graphql(description = "Grant number of the award")]
    pub fn grant_number(&self) -> Option<&String> {
        self.grant_number.as_ref()
    }

    #[graphql(description = "Jurisdiction of the award")]
    pub fn jurisdiction(&self) -> Option<&String> {
        self.jurisdiction.as_ref()
    }

    #[graphql(description = "Date and time at which the funding record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the funding record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    #[graphql(description = "Get the funded work")]
    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the funding institution")]
    pub fn institution(&self, context: &Context) -> FieldResult<Institution> {
        Institution::from_id(&context.db, &self.institution_id).map_err(|e| e.into())
    }
}

#[juniper::graphql_object(Context = Context, description = "An association between a person and an institution for a specific contribution.")]
impl Affiliation {
    #[graphql(description = "Thoth ID of the affiliation")]
    pub fn affiliation_id(&self) -> Uuid {
        self.affiliation_id
    }

    #[graphql(description = "Thoth ID of the contribution linked to this affiliation")]
    pub fn contribution_id(&self) -> Uuid {
        self.contribution_id
    }

    #[graphql(description = "Thoth ID of the institution linked to this affiliation")]
    pub fn institution_id(&self) -> Uuid {
        self.institution_id
    }

    #[graphql(
        description = "Number representing this affiliation's position in an ordered list of affiliations within the contribution"
    )]
    pub fn affiliation_ordinal(&self) -> &i32 {
        &self.affiliation_ordinal
    }

    #[graphql(
        description = "Position of the contributor at the institution at the time of contribution"
    )]
    pub fn position(&self) -> Option<&String> {
        self.position.as_ref()
    }

    #[graphql(description = "Date and time at which the affiliation record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the affiliation record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    #[graphql(description = "Get the institution linked to this affiliation")]
    pub fn institution(&self, context: &Context) -> FieldResult<Institution> {
        Institution::from_id(&context.db, &self.institution_id).map_err(|e| e.into())
    }

    #[graphql(description = "Get the contribution linked to this affiliation")]
    pub fn contribution(&self, context: &Context) -> FieldResult<Contribution> {
        Contribution::from_id(&context.db, &self.contribution_id).map_err(|e| e.into())
    }
}

#[juniper::graphql_object(Context = Context, description = "A relationship between two works, e.g. a book and one of its chapters, or an original and its translation.")]
impl WorkRelation {
    #[graphql(description = "Thoth ID of the work relation")]
    pub fn work_relation_id(&self) -> &Uuid {
        &self.work_relation_id
    }

    #[graphql(description = "Thoth ID of the work to which this work relation belongs")]
    pub fn relator_work_id(&self) -> &Uuid {
        &self.relator_work_id
    }

    #[graphql(description = "Thoth ID of the other work in the relationship")]
    pub fn related_work_id(&self) -> &Uuid {
        &self.related_work_id
    }

    #[graphql(description = "Nature of the relationship")]
    pub fn relation_type(&self) -> &RelationType {
        &self.relation_type
    }

    #[graphql(
        description = "Number representing this work relation's position in an ordered list of relations of the same type within the work"
    )]
    pub fn relation_ordinal(&self) -> &i32 {
        &self.relation_ordinal
    }

    #[graphql(description = "Date and time at which the work relation record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the work relation record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    #[graphql(description = "Get the other work in the relationship")]
    pub fn related_work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.related_work_id).map_err(|e| e.into())
    }
}

#[juniper::graphql_object(
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
        &self.reference_ordinal
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
    pub fn url(&self) -> Option<&String> {
        self.url.as_ref()
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
        self.created_at
    }

    #[graphql(description = "Timestamp of the last update to this record within Thoth.")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    #[graphql(description = "The citing work.")]
    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(|e| e.into())
    }
}

#[juniper::graphql_object(Context = Context, description = "A title associated with a work.")]
impl Title {
    #[graphql(description = "Thoth ID of the title")]
    pub fn title_id(&self) -> Uuid {
        self.title_id
    }

    #[graphql(description = "Thoth ID of the work to which the title is linked")]
    pub fn work_id(&self) -> Uuid {
        self.work_id
    }

    #[graphql(description = "Locale code of the title")]
    pub fn locale_code(&self) -> &LocaleCode {
        &self.locale_code
    }

    #[graphql(description = "Full title including subtitle")]
    pub fn full_title(&self) -> &String {
        &self.full_title
    }

    #[graphql(description = "Main title (excluding subtitle)")]
    pub fn title(&self) -> &String {
        &self.title
    }

    #[graphql(description = "Subtitle of the work")]
    pub fn subtitle(&self) -> Option<&String> {
        self.subtitle.as_ref()
    }

    #[graphql(description = "Whether this is the canonical title for the work")]
    pub fn canonical(&self) -> bool {
        self.canonical
    }

    #[graphql(description = "Get the work to which the title is linked")]
    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(|e| e.into())
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<Context>>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {}, EmptySubscription::new())
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
