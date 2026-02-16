use juniper::{FieldError, FieldResult};
use uuid::Uuid;

use super::types::inputs::{
    ContributionOrderBy, FundingOrderBy, IssueOrderBy, LanguageOrderBy, PriceOrderBy,
    SubjectOrderBy, TimeExpression,
};
use crate::graphql::types::me::{Me, ToMe};
use crate::graphql::Context;
use crate::markup::{convert_from_jats, ConversionLimit, MarkupFormat};
use crate::model::{
    additional_resource::{AdditionalResource, AdditionalResourceOrderBy},
    affiliation::{Affiliation, AffiliationOrderBy},
    award::{Award, AwardOrderBy},
    biography::{Biography, BiographyOrderBy},
    book_review::{BookReview, BookReviewOrderBy},
    contact::{Contact, ContactOrderBy, ContactType},
    contribution::{Contribution, ContributionType},
    contributor::{Contributor, ContributorOrderBy},
    endorsement::{Endorsement, EndorsementOrderBy},
    file::File,
    funding::Funding,
    imprint::{Imprint, ImprintOrderBy},
    institution::{Institution, InstitutionOrderBy},
    issue::Issue,
    language::{Language, LanguageCode, LanguageRelation},
    locale::LocaleCode,
    location::{Location, LocationOrderBy, LocationPlatform},
    price::{CurrencyCode, Price},
    publication::{Publication, PublicationOrderBy, PublicationType},
    publisher::{Publisher, PublisherOrderBy},
    r#abstract::{Abstract, AbstractOrderBy},
    reference::{Reference, ReferenceOrderBy},
    series::{Series, SeriesOrderBy, SeriesType},
    subject::{Subject, SubjectType},
    title::{Title, TitleOrderBy},
    work::{Work, WorkOrderBy, WorkStatus, WorkType},
    work_featured_video::{WorkFeaturedVideo, WorkFeaturedVideoOrderBy},
    Crud, Doi,
};
use crate::policy::PolicyContext;
use thoth_errors::ThothError;

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
            description = "Only show results with a publication date either before (less than) or after (greater than) the specified timestamp"
        )]
        publication_date: Option<TimeExpression>,
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
            publication_date,
            updated_at_with_relations,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single work using its ID")]
    fn work(
        context: &Context,
        #[graphql(description = "Thoth work ID to search on")] work_id: Uuid,
    ) -> FieldResult<Work> {
        Work::from_id(&context.db, &work_id).map_err(Into::into)
    }

    #[graphql(description = "Query a single work using its DOI")]
    fn work_by_doi(
        context: &Context,
        #[graphql(description = "Work DOI to search on")] doi: Doi,
    ) -> FieldResult<Work> {
        Work::from_doi(&context.db, doi, vec![]).map_err(Into::into)
    }

    #[allow(clippy::too_many_arguments)]
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
            description = "Only show results with a publication date either before (less than) or after (greater than) the specified timestamp"
        )]
        publication_date: Option<TimeExpression>,
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
            publication_date,
            updated_at_with_relations,
        )
        .map_err(Into::into)
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
            description = "Only show results with a publication date either before (less than) or after (greater than) the specified timestamp"
        )]
        publication_date: Option<TimeExpression>,
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
            publication_date,
            updated_at_with_relations,
        )
        .map_err(Into::into)
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
        .map_err(Into::into)
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
            description = "Only show results with a publication date either before (less than) or after (greater than) the specified timestamp"
        )]
        publication_date: Option<TimeExpression>,
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
            publication_date,
            updated_at_with_relations,
        )
        .map_err(Into::into)
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
            description = "Only show results with a publication date either before (less than) or after (greater than) the specified timestamp"
        )]
        publication_date: Option<TimeExpression>,
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
            publication_date,
            updated_at_with_relations,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single chapter using its DOI")]
    fn chapter_by_doi(
        context: &Context,
        #[graphql(description = "Chapter DOI to search on")] doi: Doi,
    ) -> FieldResult<Work> {
        Work::from_doi(&context.db, doi, vec![WorkType::BookChapter]).map_err(Into::into)
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
            description = "Only show results with a publication date either before (less than) or after (greater than) the specified timestamp"
        )]
        publication_date: Option<TimeExpression>,
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
            publication_date,
            updated_at_with_relations,
        )
        .map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single publication using its ID")]
    fn publication(
        context: &Context,
        #[graphql(description = "Thoth publication ID to search on")] publication_id: Uuid,
    ) -> FieldResult<Publication> {
        Publication::from_id(&context.db, &publication_id).map_err(Into::into)
    }

    #[graphql(description = "Query a single file using its ID")]
    fn file(
        context: &Context,
        #[graphql(description = "Thoth file ID to search on")] file_id: Uuid,
    ) -> FieldResult<File> {
        File::from_id(&context.db, &file_id).map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single publisher using its ID")]
    fn publisher(
        context: &Context,
        #[graphql(description = "Thoth publisher ID to search on")] publisher_id: Uuid,
    ) -> FieldResult<Publisher> {
        Publisher::from_id(&context.db, &publisher_id).map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single imprint using its ID")]
    fn imprint(
        context: &Context,
        #[graphql(description = "Thoth imprint ID to search on")] imprint_id: Uuid,
    ) -> FieldResult<Imprint> {
        Imprint::from_id(&context.db, &imprint_id).map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single contributor using its ID")]
    fn contributor(
        context: &Context,
        #[graphql(description = "Thoth contributor ID to search on")] contributor_id: Uuid,
    ) -> FieldResult<Contributor> {
        Contributor::from_id(&context.db, &contributor_id).map_err(Into::into)
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
        Contributor::count(&context.db, filter, vec![], vec![], vec![], None, None)
            .map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single contribution using its ID")]
    fn contribution(
        context: &Context,
        #[graphql(description = "Thoth contribution ID to search on")] contribution_id: Uuid,
    ) -> FieldResult<Contribution> {
        Contribution::from_id(&context.db, &contribution_id).map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single series using its ID")]
    fn series(
        context: &Context,
        #[graphql(description = "Thoth series ID to search on")] series_id: Uuid,
    ) -> FieldResult<Series> {
        Series::from_id(&context.db, &series_id).map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single issue using its ID")]
    fn issue(
        context: &Context,
        #[graphql(description = "Thoth issue ID to search on")] issue_id: Uuid,
    ) -> FieldResult<Issue> {
        Issue::from_id(&context.db, &issue_id).map_err(Into::into)
    }

    #[graphql(description = "Get the total number of issues")]
    fn issue_count(context: &Context) -> FieldResult<i32> {
        Issue::count(&context.db, None, vec![], vec![], vec![], None, None).map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single language using its ID")]
    fn language(
        context: &Context,
        #[graphql(description = "Thoth language ID to search on")] language_id: Uuid,
    ) -> FieldResult<Language> {
        Language::from_id(&context.db, &language_id).map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single location using its ID")]
    fn location(
        context: &Context,
        #[graphql(description = "Thoth location ID to search on")] location_id: Uuid,
    ) -> FieldResult<Location> {
        Location::from_id(&context.db, &location_id).map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single price using its ID")]
    fn price(
        context: &Context,
        #[graphql(description = "Thoth price ID to search on")] price_id: Uuid,
    ) -> FieldResult<Price> {
        Price::from_id(&context.db, &price_id).map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single subject using its ID")]
    fn subject(
        context: &Context,
        #[graphql(description = "Thoth subject ID to search on")] subject_id: Uuid,
    ) -> FieldResult<Subject> {
        Subject::from_id(&context.db, &subject_id).map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single institution using its ID")]
    fn institution(
        context: &Context,
        #[graphql(description = "Thoth institution ID to search on")] institution_id: Uuid,
    ) -> FieldResult<Institution> {
        Institution::from_id(&context.db, &institution_id).map_err(Into::into)
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
        Institution::count(&context.db, filter, vec![], vec![], vec![], None, None)
            .map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single funding using its ID")]
    fn funding(
        context: &Context,
        #[graphql(description = "Thoth funding ID to search on")] funding_id: Uuid,
    ) -> FieldResult<Funding> {
        Funding::from_id(&context.db, &funding_id).map_err(Into::into)
    }

    #[graphql(description = "Get the total number of funding instances associated to works")]
    fn funding_count(context: &Context) -> FieldResult<i32> {
        Funding::count(&context.db, None, vec![], vec![], vec![], None, None).map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single affiliation using its ID")]
    fn affiliation(
        context: &Context,
        #[graphql(description = "Thoth affiliation ID to search on")] affiliation_id: Uuid,
    ) -> FieldResult<Affiliation> {
        Affiliation::from_id(&context.db, &affiliation_id).map_err(Into::into)
    }

    #[graphql(description = "Get the total number of affiliations")]
    fn affiliation_count(context: &Context) -> FieldResult<i32> {
        Affiliation::count(&context.db, None, vec![], vec![], vec![], None, None)
            .map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single reference using its ID")]
    fn reference(
        context: &Context,
        #[graphql(description = "Thoth reference ID to search on")] reference_id: Uuid,
    ) -> FieldResult<Reference> {
        Reference::from_id(&context.db, &reference_id).map_err(Into::into)
    }

    #[graphql(description = "Get the total number of references")]
    fn reference_count(context: &Context) -> FieldResult<i32> {
        Reference::count(&context.db, None, vec![], vec![], vec![], None, None).map_err(Into::into)
    }

    #[graphql(description = "Query the full list of additional resources")]
    fn additional_resources(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = AdditionalResourceOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<AdditionalResourceOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
    ) -> FieldResult<Vec<AdditionalResource>> {
        AdditionalResource::all(
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single additional resource using its ID")]
    fn additional_resource(
        context: &Context,
        #[graphql(description = "Thoth additional resource ID to search on")]
        additional_resource_id: Uuid,
    ) -> FieldResult<AdditionalResource> {
        AdditionalResource::from_id(&context.db, &additional_resource_id).map_err(Into::into)
    }

    #[graphql(description = "Get the total number of additional resources")]
    fn additional_resource_count(context: &Context) -> FieldResult<i32> {
        AdditionalResource::count(&context.db, None, vec![], vec![], vec![], None, None)
            .map_err(Into::into)
    }

    #[graphql(description = "Query the full list of awards")]
    fn awards(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = AwardOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<AwardOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
    ) -> FieldResult<Vec<Award>> {
        Award::all(
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single award using its ID")]
    fn award(
        context: &Context,
        #[graphql(description = "Thoth award ID to search on")] award_id: Uuid,
    ) -> FieldResult<Award> {
        Award::from_id(&context.db, &award_id).map_err(Into::into)
    }

    #[graphql(description = "Get the total number of awards")]
    fn award_count(context: &Context) -> FieldResult<i32> {
        Award::count(&context.db, None, vec![], vec![], vec![], None, None).map_err(Into::into)
    }

    #[graphql(description = "Query the full list of endorsements")]
    fn endorsements(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = EndorsementOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<EndorsementOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
    ) -> FieldResult<Vec<Endorsement>> {
        Endorsement::all(
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single endorsement using its ID")]
    fn endorsement(
        context: &Context,
        #[graphql(description = "Thoth endorsement ID to search on")] endorsement_id: Uuid,
    ) -> FieldResult<Endorsement> {
        Endorsement::from_id(&context.db, &endorsement_id).map_err(Into::into)
    }

    #[graphql(description = "Get the total number of endorsements")]
    fn endorsement_count(context: &Context) -> FieldResult<i32> {
        Endorsement::count(&context.db, None, vec![], vec![], vec![], None, None)
            .map_err(Into::into)
    }

    #[graphql(description = "Query the full list of book reviews")]
    fn book_reviews(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = BookReviewOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<BookReviewOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
    ) -> FieldResult<Vec<BookReview>> {
        BookReview::all(
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single book review using its ID")]
    fn book_review(
        context: &Context,
        #[graphql(description = "Thoth book review ID to search on")] book_review_id: Uuid,
    ) -> FieldResult<BookReview> {
        BookReview::from_id(&context.db, &book_review_id).map_err(Into::into)
    }

    #[graphql(description = "Get the total number of book reviews")]
    fn book_review_count(context: &Context) -> FieldResult<i32> {
        BookReview::count(&context.db, None, vec![], vec![], vec![], None, None)
            .map_err(Into::into)
    }

    #[graphql(description = "Query the full list of featured videos")]
    fn work_featured_videos(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = WorkFeaturedVideoOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<WorkFeaturedVideoOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
    ) -> FieldResult<Vec<WorkFeaturedVideo>> {
        WorkFeaturedVideo::all(
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single featured video using its ID")]
    fn work_featured_video(
        context: &Context,
        #[graphql(description = "Thoth featured video ID to search on")] work_featured_video_id: Uuid,
    ) -> FieldResult<WorkFeaturedVideo> {
        WorkFeaturedVideo::from_id(&context.db, &work_featured_video_id).map_err(Into::into)
    }

    #[graphql(description = "Get the total number of featured videos")]
    fn work_featured_video_count(context: &Context) -> FieldResult<i32> {
        WorkFeaturedVideo::count(&context.db, None, vec![], vec![], vec![], None, None)
            .map_err(Into::into)
    }

    #[graphql(description = "Query a title by its ID")]
    fn title(
        context: &Context,
        title_id: Uuid,
        markup_format: Option<MarkupFormat>,
    ) -> FieldResult<Title> {
        let mut title = Title::from_id(&context.db, &title_id).map_err(FieldError::from)?;
        let markup = markup_format.ok_or(ThothError::MissingMarkupFormat)?;
        title.title = convert_from_jats(&title.title, markup, ConversionLimit::Title)?;
        if let Some(subtitle) = &title.subtitle {
            title.subtitle = Some(convert_from_jats(subtitle, markup, ConversionLimit::Title)?);
        }
        title.full_title = convert_from_jats(&title.full_title, markup, ConversionLimit::Title)?;
        Ok(title)
    }

    #[graphql(description = "Query the full list of titles")]
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
        #[graphql(
            default = MarkupFormat::JatsXml,
            description = "If set shows result with this markup format"
        )]
        markup_format: Option<MarkupFormat>,
    ) -> FieldResult<Vec<Title>> {
        let mut titles = Title::all(
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
            None,
        )
        .map_err(FieldError::from)?;

        let markup = markup_format.ok_or(ThothError::MissingMarkupFormat)?;
        for title in &mut titles {
            title.title = convert_from_jats(&title.title, markup, ConversionLimit::Title)?;
            if let Some(subtitle) = &title.subtitle {
                title.subtitle = Some(convert_from_jats(subtitle, markup, ConversionLimit::Title)?);
            }
            title.full_title =
                convert_from_jats(&title.full_title, markup, ConversionLimit::Title)?;
        }
        Ok(titles)
    }

    #[graphql(description = "Query an abstract by its ID")]
    fn r#abstract(
        context: &Context,
        abstract_id: Uuid,
        #[graphql(
            default = MarkupFormat::JatsXml,
            description = "If set shows results with this markup format"
        )]
        markup_format: Option<MarkupFormat>,
    ) -> FieldResult<Abstract> {
        let mut r#abstract =
            Abstract::from_id(&context.db, &abstract_id).map_err(FieldError::from)?;
        let markup = markup_format.ok_or(ThothError::MissingMarkupFormat)?;
        r#abstract.content =
            convert_from_jats(&r#abstract.content, markup, ConversionLimit::Abstract)?;
        Ok(r#abstract)
    }

    #[allow(clippy::too_many_arguments)]
    #[graphql(description = "Query the full list of abstracts")]
    fn abstracts(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on content fields"
        )]
        filter: Option<String>,
        #[graphql(
            default = AbstractOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<AbstractOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set only shows results with these locale codes"
        )]
        locale_codes: Option<Vec<LocaleCode>>,
        #[graphql(
            default = MarkupFormat::JatsXml,
            description = "If set shows result with this markup format"
        )]
        markup_format: Option<MarkupFormat>,
    ) -> FieldResult<Vec<Abstract>> {
        let mut abstracts = Abstract::all(
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
            None,
        )
        .map_err(FieldError::from)?;

        let markup = markup_format.ok_or(ThothError::MissingMarkupFormat)?;
        for r#abstract in &mut abstracts {
            r#abstract.content =
                convert_from_jats(&r#abstract.content, markup, ConversionLimit::Abstract)?;
        }

        Ok(abstracts)
    }

    #[graphql(description = "Query an biography by it's ID")]
    fn biography(
        context: &Context,
        biography_id: Uuid,
        #[graphql(
            default = MarkupFormat::JatsXml,
            description = "If set shows result with this markup format"
        )]
        markup_format: Option<MarkupFormat>,
    ) -> FieldResult<Biography> {
        let mut biography =
            Biography::from_id(&context.db, &biography_id).map_err(FieldError::from)?;
        let markup = markup_format.ok_or(ThothError::MissingMarkupFormat)?;
        biography.content =
            convert_from_jats(&biography.content, markup, ConversionLimit::Biography)?;
        Ok(biography)
    }

    #[allow(clippy::too_many_arguments)]
    #[graphql(description = "Query biographies by work ID")]
    fn biographies(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = "".to_string(),
            description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on content fields"
        )]
        filter: Option<String>,
        #[graphql(
            default = BiographyOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<BiographyOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results with these locale codes"
        )]
        locale_codes: Option<Vec<LocaleCode>>,
        #[graphql(
            default = MarkupFormat::JatsXml,
            description = "If set shows result with this markup format"
        )]
        markup_format: Option<MarkupFormat>,
    ) -> FieldResult<Vec<Biography>> {
        let mut biographies = Biography::all(
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
            None,
        )
        .map_err(FieldError::from)?;

        let markup = markup_format.ok_or(ThothError::MissingMarkupFormat)?;
        for biography in &mut biographies {
            biography.content =
                convert_from_jats(&biography.content, markup, ConversionLimit::Biography)?;
        }

        Ok(biographies)
    }

    #[graphql(description = "Query the full list of contacts")]
    fn contacts(
        context: &Context,
        #[graphql(default = 100, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = ContactOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<ContactOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results connected to publishers with these IDs"
        )]
        publishers: Option<Vec<Uuid>>,
        #[graphql(
            default = vec![],
            description = "Specific types to filter by",
        )]
        contact_types: Option<Vec<ContactType>>,
    ) -> FieldResult<Vec<Contact>> {
        Contact::all(
            &context.db,
            limit.unwrap_or_default(),
            offset.unwrap_or_default(),
            None,
            order.unwrap_or_default(),
            publishers.unwrap_or_default(),
            None,
            None,
            contact_types.unwrap_or_default(),
            vec![],
            None,
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Query a single contact using its ID")]
    fn contact(
        context: &Context,
        #[graphql(description = "Thoth contact ID to search on")] contact_id: Uuid,
    ) -> FieldResult<Contact> {
        Contact::from_id(&context.db, &contact_id).map_err(Into::into)
    }

    #[graphql(description = "Get the total number of contacts")]
    fn contact_count(
        context: &Context,
        #[graphql(
            default = vec![],
            description = "Specific types to filter by"
        )]
        contact_types: Option<Vec<ContactType>>,
    ) -> FieldResult<i32> {
        Contact::count(
            &context.db,
            None,
            vec![],
            contact_types.unwrap_or_default(),
            vec![],
            None,
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Get the total number of contacts")]
    fn me(context: &Context) -> FieldResult<Me> {
        let user = context.require_authentication()?;
        user.to_me(context)
    }
}
