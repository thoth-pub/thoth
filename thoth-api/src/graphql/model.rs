use chrono::naive::NaiveDate;
use chrono::DateTime;
use chrono::Utc;
use diesel::prelude::*;
use juniper::FieldError;
use juniper::FieldResult;
use juniper::RootNode;
use std::sync::Arc;
use uuid::Uuid;

use crate::account::model::AccountAccess;
use crate::account::model::DecodedToken;
use crate::contribution::model::*;
use crate::contributor::model::*;
use crate::db::PgPool;
use crate::errors::ThothError;
use crate::errors::ThothResult;
use crate::funder::model::*;
use crate::funding::model::*;
use crate::imprint::model::*;
use crate::issue::model::*;
use crate::language::model::*;
use crate::price::model::*;
use crate::publication::model::*;
use crate::publisher::model::*;
use crate::schema::*;
use crate::series::model::*;
use crate::subject::model::*;
use crate::work::model::*;

use super::utils::Direction;
use crate::imprint::crud::Crud;

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
        work_type(description = "A specific type to filter by"),
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
        work_type: Option<WorkType>,
        work_status: Option<WorkStatus>,
    ) -> Vec<Work> {
        use crate::schema::work::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::work
            .inner_join(crate::schema::imprint::table)
            .select((
                dsl::work_id,
                dsl::work_type,
                dsl::work_status,
                dsl::full_title,
                dsl::title,
                dsl::subtitle,
                dsl::reference,
                dsl::edition,
                dsl::imprint_id,
                dsl::doi,
                dsl::publication_date,
                dsl::place,
                dsl::width,
                dsl::height,
                dsl::page_count,
                dsl::page_breakdown,
                dsl::image_count,
                dsl::table_count,
                dsl::audio_count,
                dsl::video_count,
                dsl::license,
                dsl::copyright_holder,
                dsl::landing_page,
                dsl::lccn,
                dsl::oclc,
                dsl::short_abstract,
                dsl::long_abstract,
                dsl::general_note,
                dsl::toc,
                dsl::cover_url,
                dsl::cover_caption,
                dsl::created_at,
                dsl::updated_at,
            ))
            .into_boxed();
        match order.field {
            WorkField::WorkId => match order.direction {
                Direction::Asc => query = query.order(dsl::work_id.asc()),
                Direction::Desc => query = query.order(dsl::work_id.desc()),
            },
            WorkField::WorkType => match order.direction {
                Direction::Asc => query = query.order(dsl::work_type.asc()),
                Direction::Desc => query = query.order(dsl::work_type.desc()),
            },
            WorkField::WorkStatus => match order.direction {
                Direction::Asc => query = query.order(dsl::work_status.asc()),
                Direction::Desc => query = query.order(dsl::work_status.desc()),
            },
            WorkField::FullTitle => match order.direction {
                Direction::Asc => query = query.order(dsl::full_title.asc()),
                Direction::Desc => query = query.order(dsl::full_title.desc()),
            },
            WorkField::Title => match order.direction {
                Direction::Asc => query = query.order(dsl::title.asc()),
                Direction::Desc => query = query.order(dsl::title.desc()),
            },
            WorkField::Subtitle => match order.direction {
                Direction::Asc => query = query.order(dsl::subtitle.asc()),
                Direction::Desc => query = query.order(dsl::subtitle.desc()),
            },
            WorkField::Reference => match order.direction {
                Direction::Asc => query = query.order(dsl::reference.asc()),
                Direction::Desc => query = query.order(dsl::reference.desc()),
            },
            WorkField::Edition => match order.direction {
                Direction::Asc => query = query.order(dsl::edition.asc()),
                Direction::Desc => query = query.order(dsl::edition.desc()),
            },
            WorkField::Doi => match order.direction {
                Direction::Asc => query = query.order(dsl::doi.asc()),
                Direction::Desc => query = query.order(dsl::doi.desc()),
            },
            WorkField::PublicationDate => match order.direction {
                Direction::Asc => query = query.order(dsl::publication_date.asc()),
                Direction::Desc => query = query.order(dsl::publication_date.desc()),
            },
            WorkField::Place => match order.direction {
                Direction::Asc => query = query.order(dsl::place.asc()),
                Direction::Desc => query = query.order(dsl::place.desc()),
            },
            WorkField::Width => match order.direction {
                Direction::Asc => query = query.order(dsl::width.asc()),
                Direction::Desc => query = query.order(dsl::width.desc()),
            },
            WorkField::Height => match order.direction {
                Direction::Asc => query = query.order(dsl::height.asc()),
                Direction::Desc => query = query.order(dsl::height.desc()),
            },
            WorkField::PageCount => match order.direction {
                Direction::Asc => query = query.order(dsl::page_count.asc()),
                Direction::Desc => query = query.order(dsl::page_count.desc()),
            },
            WorkField::PageBreakdown => match order.direction {
                Direction::Asc => query = query.order(dsl::page_breakdown.asc()),
                Direction::Desc => query = query.order(dsl::page_breakdown.desc()),
            },
            WorkField::ImageCount => match order.direction {
                Direction::Asc => query = query.order(dsl::image_count.asc()),
                Direction::Desc => query = query.order(dsl::image_count.desc()),
            },
            WorkField::TableCount => match order.direction {
                Direction::Asc => query = query.order(dsl::table_count.asc()),
                Direction::Desc => query = query.order(dsl::table_count.desc()),
            },
            WorkField::AudioCount => match order.direction {
                Direction::Asc => query = query.order(dsl::audio_count.asc()),
                Direction::Desc => query = query.order(dsl::audio_count.desc()),
            },
            WorkField::VideoCount => match order.direction {
                Direction::Asc => query = query.order(dsl::video_count.asc()),
                Direction::Desc => query = query.order(dsl::video_count.desc()),
            },
            WorkField::License => match order.direction {
                Direction::Asc => query = query.order(dsl::license.asc()),
                Direction::Desc => query = query.order(dsl::license.desc()),
            },
            WorkField::CopyrightHolder => match order.direction {
                Direction::Asc => query = query.order(dsl::copyright_holder.asc()),
                Direction::Desc => query = query.order(dsl::copyright_holder.desc()),
            },
            WorkField::LandingPage => match order.direction {
                Direction::Asc => query = query.order(dsl::landing_page.asc()),
                Direction::Desc => query = query.order(dsl::landing_page.desc()),
            },
            WorkField::Lccn => match order.direction {
                Direction::Asc => query = query.order(dsl::lccn.asc()),
                Direction::Desc => query = query.order(dsl::lccn.desc()),
            },
            WorkField::Oclc => match order.direction {
                Direction::Asc => query = query.order(dsl::oclc.asc()),
                Direction::Desc => query = query.order(dsl::oclc.desc()),
            },
            WorkField::ShortAbstract => match order.direction {
                Direction::Asc => query = query.order(dsl::short_abstract.asc()),
                Direction::Desc => query = query.order(dsl::short_abstract.desc()),
            },
            WorkField::LongAbstract => match order.direction {
                Direction::Asc => query = query.order(dsl::long_abstract.asc()),
                Direction::Desc => query = query.order(dsl::long_abstract.desc()),
            },
            WorkField::GeneralNote => match order.direction {
                Direction::Asc => query = query.order(dsl::general_note.asc()),
                Direction::Desc => query = query.order(dsl::general_note.desc()),
            },
            WorkField::Toc => match order.direction {
                Direction::Asc => query = query.order(dsl::toc.asc()),
                Direction::Desc => query = query.order(dsl::toc.desc()),
            },
            WorkField::CoverUrl => match order.direction {
                Direction::Asc => query = query.order(dsl::cover_url.asc()),
                Direction::Desc => query = query.order(dsl::cover_url.desc()),
            },
            WorkField::CoverCaption => match order.direction {
                Direction::Asc => query = query.order(dsl::cover_caption.asc()),
                Direction::Desc => query = query.order(dsl::cover_caption.desc()),
            },
            WorkField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::created_at.asc()),
                Direction::Desc => query = query.order(dsl::created_at.desc()),
            },
            WorkField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::updated_at.asc()),
                Direction::Desc => query = query.order(dsl::updated_at.desc()),
            },
        }
        // Ordering and construction of filters is important here: result needs to be
        // `WHERE (x = $1 [OR x = $2...]) AND (y ILIKE $3 [OR z ILIKE $3...])`.
        // Interchanging .filter, .or, and .or_filter would result in different bracketing.
        for pub_id in publishers {
            query = query.or_filter(crate::schema::imprint::publisher_id.eq(pub_id));
        }
        if let Some(wk_type) = work_type {
            query = query.filter(dsl::work_type.eq(wk_type))
        }
        if let Some(wk_status) = work_status {
            query = query.filter(dsl::work_status.eq(wk_status))
        }
        query
            .filter(
                dsl::full_title
                    .ilike(format!("%{}%", filter))
                    .or(dsl::doi.ilike(format!("%{}%", filter)))
                    .or(dsl::reference.ilike(format!("%{}%", filter)))
                    .or(dsl::short_abstract.ilike(format!("%{}%", filter)))
                    .or(dsl::long_abstract.ilike(format!("%{}%", filter)))
                    .or(dsl::landing_page.ilike(format!("%{}%", filter))),
            )
            .limit(limit.into())
            .offset(offset.into())
            .load::<Work>(&connection)
            .expect("Error loading works")
    }

    #[graphql(description = "Query a single work using its id")]
    fn work(context: &Context, work_id: Uuid) -> FieldResult<Work> {
        let connection = context.db.get().unwrap();
        match crate::schema::work::dsl::work
            .find(work_id)
            .get_result::<Work>(&connection)
        {
            Ok(work) => Ok(work),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    #[graphql(description = "Query a single work using its DOI")]
    fn work_by_doi(context: &Context, doi: String) -> FieldResult<Work> {
        let connection = context.db.get().unwrap();
        use diesel::sql_types::Nullable;
        use diesel::sql_types::Text;
        // Allow case-insensitive searching (DOIs in database may have mixed casing)
        sql_function!(fn lower(x: Nullable<Text>) -> Nullable<Text>);
        match crate::schema::work::dsl::work
            .filter(lower(crate::schema::work::dsl::doi).eq(doi.to_lowercase()))
            .get_result::<Work>(&connection)
        {
            Ok(work) => Ok(work),
            Err(e) => Err(FieldError::from(e)),
        }
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
            work_type(description = "A specific type to filter by"),
            work_status(description = "A specific status to filter by"),
        )
    )]
    fn work_count(
        context: &Context,
        filter: String,
        publishers: Vec<Uuid>,
        work_type: Option<WorkType>,
        work_status: Option<WorkStatus>,
    ) -> i32 {
        use crate::schema::work::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::work
            .inner_join(crate::schema::imprint::table)
            .select((
                dsl::work_id,
                dsl::work_type,
                dsl::work_status,
                dsl::full_title,
                dsl::title,
                dsl::subtitle,
                dsl::reference,
                dsl::edition,
                dsl::imprint_id,
                dsl::doi,
                dsl::publication_date,
                dsl::place,
                dsl::width,
                dsl::height,
                dsl::page_count,
                dsl::page_breakdown,
                dsl::image_count,
                dsl::table_count,
                dsl::audio_count,
                dsl::video_count,
                dsl::license,
                dsl::copyright_holder,
                dsl::landing_page,
                dsl::lccn,
                dsl::oclc,
                dsl::short_abstract,
                dsl::long_abstract,
                dsl::general_note,
                dsl::toc,
                dsl::cover_url,
                dsl::cover_caption,
                dsl::created_at,
                dsl::updated_at,
            ))
            .into_boxed();
        // Ordering and construction of filters is important here: result needs to be
        // `WHERE (x = $1 [OR x = $2...]) AND (y ILIKE $3 [OR z ILIKE $3...])`.
        // Interchanging .filter, .or, and .or_filter would result in different bracketing.
        for pub_id in publishers {
            query = query.or_filter(crate::schema::imprint::publisher_id.eq(pub_id));
        }
        if let Some(wk_type) = work_type {
            query = query.filter(dsl::work_type.eq(wk_type))
        }
        if let Some(wk_status) = work_status {
            query = query.filter(dsl::work_status.eq(wk_status))
        }
        // `SELECT COUNT(*)` in postgres returns a BIGINT, which diesel parses as i64. Juniper does
        // not implement i64 yet, only i32. The only sensible way, albeit shameful, to solve this
        // is converting i64 to string and then parsing it as i32. This should work until we reach
        // 2147483647 records - if you are fixing this bug, congratulations on book number 2147483647!
        query
            .filter(
                dsl::full_title
                    .ilike(format!("%{}%", filter))
                    .or(dsl::doi.ilike(format!("%{}%", filter)))
                    .or(dsl::reference.ilike(format!("%{}%", filter)))
                    .or(dsl::short_abstract.ilike(format!("%{}%", filter)))
                    .or(dsl::long_abstract.ilike(format!("%{}%", filter)))
                    .or(dsl::landing_page.ilike(format!("%{}%", filter))),
            )
            .count()
            .get_result::<i64>(&connection)
            .expect("Error loading work count")
            .to_string()
            .parse::<i32>()
            .unwrap()
    }

    #[graphql(
        description = "Query the full list of publications",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on isbn and publication_url"
            ),
            order(
                default = PublicationOrderBy::default(),
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            publication_type(description = "A specific type to filter by"),
        )
    )]
    fn publications(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: PublicationOrderBy,
        publishers: Vec<Uuid>,
        publication_type: Option<PublicationType>,
    ) -> Vec<Publication> {
        use crate::schema::publication::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::publication
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select((
                dsl::publication_id,
                dsl::publication_type,
                dsl::work_id,
                dsl::isbn,
                dsl::publication_url,
                dsl::created_at,
                dsl::updated_at,
            ))
            .into_boxed();
        match order.field {
            PublicationField::PublicationId => match order.direction {
                Direction::Asc => query = query.order(dsl::publication_id.asc()),
                Direction::Desc => query = query.order(dsl::publication_id.desc()),
            },
            PublicationField::PublicationType => match order.direction {
                Direction::Asc => query = query.order(dsl::publication_type.asc()),
                Direction::Desc => query = query.order(dsl::publication_type.desc()),
            },
            PublicationField::WorkId => match order.direction {
                Direction::Asc => query = query.order(dsl::work_id.asc()),
                Direction::Desc => query = query.order(dsl::work_id.desc()),
            },
            PublicationField::Isbn => match order.direction {
                Direction::Asc => query = query.order(dsl::isbn.asc()),
                Direction::Desc => query = query.order(dsl::isbn.desc()),
            },
            PublicationField::PublicationUrl => match order.direction {
                Direction::Asc => query = query.order(dsl::publication_url.asc()),
                Direction::Desc => query = query.order(dsl::publication_url.desc()),
            },
            PublicationField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::created_at.asc()),
                Direction::Desc => query = query.order(dsl::created_at.desc()),
            },
            PublicationField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::updated_at.asc()),
                Direction::Desc => query = query.order(dsl::updated_at.desc()),
            },
        }
        // Ordering and construction of filters is important here: result needs to be
        // `WHERE (x = $1 [OR x = $2...]) AND (y ILIKE $3 [OR z ILIKE $3...])`.
        // Interchanging .filter, .or, and .or_filter would result in different bracketing.
        for pub_id in publishers {
            query = query.or_filter(crate::schema::imprint::publisher_id.eq(pub_id));
        }
        // ISBN and URL fields are both nullable, so searching with an empty filter could fail
        if !filter.is_empty() {
            query = query.filter(
                dsl::isbn
                    .ilike(format!("%{}%", filter))
                    .or(dsl::publication_url.ilike(format!("%{}%", filter))),
            );
        }
        if let Some(pub_type) = publication_type {
            query = query.filter(dsl::publication_type.eq(pub_type))
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Publication>(&connection)
            .expect("Error loading publications")
    }

    #[graphql(description = "Query a single publication using its id")]
    fn publication(context: &Context, publication_id: Uuid) -> FieldResult<Publication> {
        let connection = context.db.get().unwrap();
        match crate::schema::publication::dsl::publication
            .find(publication_id)
            .get_result::<Publication>(&connection)
        {
            Ok(publication) => Ok(publication),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    #[graphql(
        description = "Get the total number of publications",
        arguments(
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on isbn and publication_url",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            publication_type(description = "A specific type to filter by"),
        )
    )]
    fn publication_count(
        context: &Context,
        filter: String,
        publishers: Vec<Uuid>,
        publication_type: Option<PublicationType>,
    ) -> i32 {
        use crate::schema::publication::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::publication
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select((
                dsl::publication_id,
                dsl::publication_type,
                dsl::work_id,
                dsl::isbn,
                dsl::publication_url,
                dsl::created_at,
                dsl::updated_at,
            ))
            .into_boxed();
        // Ordering and construction of filters is important here: result needs to be
        // `WHERE (x = $1 [OR x = $2...]) AND (y ILIKE $3 [OR z ILIKE $3...])`.
        // Interchanging .filter, .or, and .or_filter would result in different bracketing.
        for pub_id in publishers {
            query = query.or_filter(crate::schema::imprint::publisher_id.eq(pub_id));
        }
        // ISBN and URL fields are both nullable, so searching with an empty filter could fail
        if !filter.is_empty() {
            query = query.filter(
                dsl::isbn
                    .ilike(format!("%{}%", filter))
                    .or(dsl::publication_url.ilike(format!("%{}%", filter))),
            );
        }
        if let Some(pub_type) = publication_type {
            query = query.filter(dsl::publication_type.eq(pub_type))
        }
        // see comment in work_count()
        query
            .count()
            .get_result::<i64>(&connection)
            .expect("Error loading publication count")
            .to_string()
            .parse::<i32>()
            .unwrap()
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
    ) -> Vec<Publisher> {
        use crate::schema::publisher::dsl::*;
        let connection = context.db.get().unwrap();
        let mut query = publisher.into_boxed();
        match order.field {
            PublisherField::PublisherId => match order.direction {
                Direction::Asc => query = query.order(publisher_id.asc()),
                Direction::Desc => query = query.order(publisher_id.desc()),
            },
            PublisherField::PublisherName => match order.direction {
                Direction::Asc => query = query.order(publisher_name.asc()),
                Direction::Desc => query = query.order(publisher_name.desc()),
            },
            PublisherField::PublisherShortname => match order.direction {
                Direction::Asc => query = query.order(publisher_shortname.asc()),
                Direction::Desc => query = query.order(publisher_shortname.desc()),
            },
            PublisherField::PublisherUrl => match order.direction {
                Direction::Asc => query = query.order(publisher_url.asc()),
                Direction::Desc => query = query.order(publisher_url.desc()),
            },
            PublisherField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(created_at.asc()),
                Direction::Desc => query = query.order(created_at.desc()),
            },
            PublisherField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(updated_at.asc()),
                Direction::Desc => query = query.order(updated_at.desc()),
            },
        }
        // Ordering and construction of filters is important here: result needs to be
        // `WHERE (x = $1 [OR x = $2...]) AND (y ILIKE $3 [OR z ILIKE $3...])`.
        // Interchanging .filter, .or, and .or_filter would result in different bracketing.
        for pub_id in publishers {
            query = query.or_filter(publisher_id.eq(pub_id));
        }
        query
            .filter(
                publisher_name
                    .ilike(format!("%{}%", filter))
                    .or(publisher_shortname.ilike(format!("%{}%", filter))),
            )
            .limit(limit.into())
            .offset(offset.into())
            .load::<Publisher>(&connection)
            .expect("Error loading publishers")
    }

    #[graphql(description = "Query a publisher using its id")]
    fn publisher(context: &Context, publisher_id: Uuid) -> FieldResult<Publisher> {
        let connection = context.db.get().unwrap();
        match crate::schema::publisher::dsl::publisher
            .find(publisher_id)
            .get_result::<Publisher>(&connection)
        {
            Ok(publisher) => Ok(publisher),
            Err(e) => Err(FieldError::from(e)),
        }
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
    fn publisher_count(context: &Context, filter: String, publishers: Vec<Uuid>) -> i32 {
        use crate::schema::publisher::dsl::*;
        let connection = context.db.get().unwrap();
        let mut query = publisher.into_boxed();
        // Ordering and construction of filters is important here: result needs to be
        // `WHERE (x = $1 [OR x = $2...]) AND (y ILIKE $3 [OR z ILIKE $3...])`.
        // Interchanging .filter, .or, and .or_filter would result in different bracketing.
        for pub_id in publishers {
            query = query.or_filter(publisher_id.eq(pub_id));
        }
        // see comment in work_count()
        query
            .filter(
                publisher_name
                    .ilike(format!("%{}%", filter))
                    .or(publisher_shortname.ilike(format!("%{}%", filter))),
            )
            .count()
            .get_result::<i64>(&connection)
            .expect("Error loading publisher count")
            .to_string()
            .parse::<i32>()
            .unwrap()
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
    ) -> Vec<Imprint> {
        use crate::schema::imprint::dsl::*;
        let connection = context.db.get().unwrap();
        let mut query = imprint.into_boxed();
        match order.field {
            ImprintField::ImprintId => match order.direction {
                Direction::Asc => query = query.order(imprint_id.asc()),
                Direction::Desc => query = query.order(imprint_id.desc()),
            },
            ImprintField::ImprintName => match order.direction {
                Direction::Asc => query = query.order(imprint_name.asc()),
                Direction::Desc => query = query.order(imprint_name.desc()),
            },
            ImprintField::ImprintUrl => match order.direction {
                Direction::Asc => query = query.order(imprint_url.asc()),
                Direction::Desc => query = query.order(imprint_url.desc()),
            },
            ImprintField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(created_at.asc()),
                Direction::Desc => query = query.order(created_at.desc()),
            },
            ImprintField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(updated_at.asc()),
                Direction::Desc => query = query.order(updated_at.desc()),
            },
        }
        // Ordering and construction of filters is important here: result needs to be
        // `WHERE (x = $1 [OR x = $2...]) AND (y ILIKE $3 [OR z ILIKE $3...])`.
        // Interchanging .filter, .or, and .or_filter would result in different bracketing.
        for pub_id in publishers {
            query = query.or_filter(publisher_id.eq(pub_id));
        }
        query
            .filter(
                imprint_name
                    .ilike(format!("%{}%", filter))
                    .or(imprint_url.ilike(format!("%{}%", filter))),
            )
            .limit(limit.into())
            .offset(offset.into())
            .load::<Imprint>(&connection)
            .expect("Error loading imprints")
    }

    #[graphql(description = "Query a single imprint using its id")]
    fn imprint(context: &Context, imprint_id: Uuid) -> FieldResult<Imprint> {
        let connection = context.db.get().unwrap();
        match crate::schema::imprint::dsl::imprint
            .find(imprint_id)
            .get_result::<Imprint>(&connection)
        {
            Ok(imprint) => Ok(imprint),
            Err(e) => Err(FieldError::from(e)),
        }
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
    fn imprint_count(context: &Context, filter: String, publishers: Vec<Uuid>) -> i32 {
        use crate::schema::imprint::dsl::*;
        let connection = context.db.get().unwrap();
        let mut query = imprint.into_boxed();
        // Ordering and construction of filters is important here: result needs to be
        // `WHERE (x = $1 [OR x = $2...]) AND (y ILIKE $3 [OR z ILIKE $3...])`.
        // Interchanging .filter, .or, and .or_filter would result in different bracketing.
        for pub_id in publishers {
            query = query.or_filter(publisher_id.eq(pub_id));
        }
        // see comment in work_count()
        query
            .filter(
                imprint_name
                    .ilike(format!("%{}%", filter))
                    .or(imprint_url.ilike(format!("%{}%", filter))),
            )
            .count()
            .get_result::<i64>(&connection)
            .expect("Error loading imprint count")
            .to_string()
            .parse::<i32>()
            .unwrap()
    }

    #[graphql(
        description = "Query the full list of contributors",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_name and orcid"
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
    ) -> Vec<Contributor> {
        use crate::schema::contributor::dsl::*;
        let connection = context.db.get().unwrap();
        let mut query = contributor.into_boxed();
        match order.field {
            ContributorField::ContributorId => match order.direction {
                Direction::Asc => query = query.order(contributor_id.asc()),
                Direction::Desc => query = query.order(contributor_id.desc()),
            },
            ContributorField::FirstName => match order.direction {
                Direction::Asc => query = query.order(first_name.asc()),
                Direction::Desc => query = query.order(first_name.desc()),
            },
            ContributorField::LastName => match order.direction {
                Direction::Asc => query = query.order(last_name.asc()),
                Direction::Desc => query = query.order(last_name.desc()),
            },
            ContributorField::FullName => match order.direction {
                Direction::Asc => query = query.order(full_name.asc()),
                Direction::Desc => query = query.order(full_name.desc()),
            },
            ContributorField::Orcid => match order.direction {
                Direction::Asc => query = query.order(orcid.asc()),
                Direction::Desc => query = query.order(orcid.desc()),
            },
            ContributorField::Website => match order.direction {
                Direction::Asc => query = query.order(website.asc()),
                Direction::Desc => query = query.order(website.desc()),
            },
            ContributorField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(created_at.asc()),
                Direction::Desc => query = query.order(created_at.desc()),
            },
            ContributorField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(updated_at.asc()),
                Direction::Desc => query = query.order(updated_at.desc()),
            },
        }
        query
            .filter(full_name.ilike(format!("%{}%", filter)))
            .or_filter(orcid.ilike(format!("%{}%", filter)))
            .limit(limit.into())
            .offset(offset.into())
            .load::<Contributor>(&connection)
            .expect("Error loading contributors")
    }

    #[graphql(description = "Query a single contributor using its id")]
    fn contributor(context: &Context, contributor_id: Uuid) -> FieldResult<Contributor> {
        let connection = context.db.get().unwrap();
        match crate::schema::contributor::dsl::contributor
            .find(contributor_id)
            .get_result::<Contributor>(&connection)
        {
            Ok(contributor) => Ok(contributor),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    #[graphql(
        description = "Get the total number of contributors",
        arguments(
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_name and orcid",
            ),
        )
    )]
    fn contributor_count(context: &Context, filter: String) -> i32 {
        use crate::schema::contributor::dsl::*;
        let connection = context.db.get().unwrap();
        // see comment in work_count()
        contributor
            .filter(full_name.ilike(format!("%{}%", filter)))
            .or_filter(orcid.ilike(format!("%{}%", filter)))
            .count()
            .get_result::<i64>(&connection)
            .expect("Error loading contributor count")
            .to_string()
            .parse::<i32>()
            .unwrap()
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
            contribution_type(description = "A specific type to filter by"),
        )
    )]
    fn contributions(
        context: &Context,
        limit: i32,
        offset: i32,
        order: ContributionOrderBy,
        publishers: Vec<Uuid>,
        contribution_type: Option<ContributionType>,
    ) -> Vec<Contribution> {
        use crate::schema::contribution::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::contribution
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select((
                dsl::work_id,
                dsl::contributor_id,
                dsl::contribution_type,
                dsl::main_contribution,
                dsl::biography,
                dsl::institution,
                dsl::created_at,
                dsl::updated_at,
                dsl::first_name,
                dsl::last_name,
                dsl::full_name,
            ))
            .into_boxed();
        match order.field {
            ContributionField::WorkId => match order.direction {
                Direction::Asc => query = query.order(dsl::work_id.asc()),
                Direction::Desc => query = query.order(dsl::work_id.desc()),
            },
            ContributionField::ContributorId => match order.direction {
                Direction::Asc => query = query.order(dsl::contributor_id.asc()),
                Direction::Desc => query = query.order(dsl::contributor_id.desc()),
            },
            ContributionField::ContributionType => match order.direction {
                Direction::Asc => query = query.order(dsl::contribution_type.asc()),
                Direction::Desc => query = query.order(dsl::contribution_type.desc()),
            },
            ContributionField::MainContribution => match order.direction {
                Direction::Asc => query = query.order(dsl::main_contribution.asc()),
                Direction::Desc => query = query.order(dsl::main_contribution.desc()),
            },
            ContributionField::Biography => match order.direction {
                Direction::Asc => query = query.order(dsl::biography.asc()),
                Direction::Desc => query = query.order(dsl::biography.desc()),
            },
            ContributionField::Institution => match order.direction {
                Direction::Asc => query = query.order(dsl::institution.asc()),
                Direction::Desc => query = query.order(dsl::institution.desc()),
            },
            ContributionField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::created_at.asc()),
                Direction::Desc => query = query.order(dsl::created_at.desc()),
            },
            ContributionField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::updated_at.asc()),
                Direction::Desc => query = query.order(dsl::updated_at.desc()),
            },
            ContributionField::FirstName => match order.direction {
                Direction::Asc => query = query.order(dsl::first_name.asc()),
                Direction::Desc => query = query.order(dsl::first_name.desc()),
            },
            ContributionField::LastName => match order.direction {
                Direction::Asc => query = query.order(dsl::last_name.asc()),
                Direction::Desc => query = query.order(dsl::last_name.desc()),
            },
            ContributionField::FullName => match order.direction {
                Direction::Asc => query = query.order(dsl::full_name.asc()),
                Direction::Desc => query = query.order(dsl::full_name.desc()),
            },
        }
        // Ordering and construction of filters is important here: result needs to be
        // `WHERE (x = $1 [OR x = $2...]) AND (y ILIKE $3 [OR z ILIKE $3...])`.
        // Interchanging .filter, .or, and .or_filter would result in different bracketing.
        for pub_id in publishers {
            query = query.or_filter(crate::schema::imprint::publisher_id.eq(pub_id));
        }
        if let Some(cont_type) = contribution_type {
            query = query.filter(dsl::contribution_type.eq(cont_type))
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Contribution>(&connection)
            .expect("Error loading contributions")
    }

    #[graphql(description = "Query a single contribution using its identifiers")]
    fn contribution(
        context: &Context,
        work_id: Uuid,
        contributor_id: Uuid,
        contribution_type: ContributionType,
    ) -> FieldResult<Contribution> {
        let connection = context.db.get().unwrap();
        match crate::schema::contribution::dsl::contribution
            .filter(crate::schema::contribution::dsl::work_id.eq(work_id))
            .filter(crate::schema::contribution::dsl::contributor_id.eq(contributor_id))
            .filter(crate::schema::contribution::dsl::contribution_type.eq(contribution_type))
            .get_result::<Contribution>(&connection)
        {
            Ok(contribution) => Ok(contribution),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    #[graphql(description = "Get the total number of contributions")]
    fn contribution_count(context: &Context, contribution_type: Option<ContributionType>) -> i32 {
        use crate::schema::contribution::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::contribution.into_boxed();
        if let Some(cont_type) = contribution_type {
            query = query.filter(dsl::contribution_type.eq(cont_type))
        }
        // see comment in work_count()
        query
            .count()
            .get_result::<i64>(&connection)
            .expect("Error loading contribution count")
            .to_string()
            .parse::<i32>()
            .unwrap()
    }

    #[graphql(
        description = "Query the full list of series",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on series_name, issn_print, issn_digital and series_url"
            ),
            order(
                default = SeriesOrderBy::default(),
                description = "The order in which to sort the results",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            series_type(description = "A specific type to filter by"),
        ),
    )]
    fn serieses(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: SeriesOrderBy,
        publishers: Vec<Uuid>,
        series_type: Option<SeriesType>,
    ) -> Vec<Series> {
        use crate::schema::series::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::series
            .inner_join(crate::schema::imprint::table)
            .select((
                dsl::series_id,
                dsl::series_type,
                dsl::series_name,
                dsl::issn_print,
                dsl::issn_digital,
                dsl::series_url,
                dsl::imprint_id,
                dsl::created_at,
                dsl::updated_at,
            ))
            .into_boxed();
        match order.field {
            SeriesField::SeriesId => match order.direction {
                Direction::Asc => query = query.order(dsl::series_id.asc()),
                Direction::Desc => query = query.order(dsl::series_id.desc()),
            },
            SeriesField::SeriesType => match order.direction {
                Direction::Asc => query = query.order(dsl::series_type.asc()),
                Direction::Desc => query = query.order(dsl::series_type.desc()),
            },
            SeriesField::SeriesName => match order.direction {
                Direction::Asc => query = query.order(dsl::series_name.asc()),
                Direction::Desc => query = query.order(dsl::series_name.desc()),
            },
            SeriesField::IssnPrint => match order.direction {
                Direction::Asc => query = query.order(dsl::issn_print.asc()),
                Direction::Desc => query = query.order(dsl::issn_print.desc()),
            },
            SeriesField::IssnDigital => match order.direction {
                Direction::Asc => query = query.order(dsl::issn_digital.asc()),
                Direction::Desc => query = query.order(dsl::issn_digital.desc()),
            },
            SeriesField::SeriesUrl => match order.direction {
                Direction::Asc => query = query.order(dsl::series_url.asc()),
                Direction::Desc => query = query.order(dsl::series_url.desc()),
            },
            SeriesField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::created_at.asc()),
                Direction::Desc => query = query.order(dsl::created_at.desc()),
            },
            SeriesField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::updated_at.asc()),
                Direction::Desc => query = query.order(dsl::updated_at.desc()),
            },
        }
        // Ordering and construction of filters is important here: result needs to be
        // `WHERE (x = $1 [OR x = $2...]) AND (y ILIKE $3 [OR z ILIKE $3...])`.
        // Interchanging .filter, .or, and .or_filter would result in different bracketing.
        for pub_id in publishers {
            query = query.or_filter(crate::schema::imprint::publisher_id.eq(pub_id));
        }
        if let Some(ser_type) = series_type {
            query = query.filter(dsl::series_type.eq(ser_type))
        }
        query
            .filter(
                dsl::series_name
                    .ilike(format!("%{}%", filter))
                    .or(dsl::issn_print.ilike(format!("%{}%", filter)))
                    .or(dsl::issn_digital.ilike(format!("%{}%", filter)))
                    .or(dsl::series_url.ilike(format!("%{}%", filter))),
            )
            .limit(limit.into())
            .offset(offset.into())
            .load::<Series>(&connection)
            .expect("Error loading series")
    }

    #[graphql(description = "Query a single series using its id")]
    fn series(context: &Context, series_id: Uuid) -> FieldResult<Series> {
        let connection = context.db.get().unwrap();
        match crate::schema::series::dsl::series
            .find(series_id)
            .get_result::<Series>(&connection)
        {
            Ok(series) => Ok(series),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    #[graphql(
        description = "Get the total number of series",
        arguments(
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on series_name, issn_print, issn_digital and series_url",
            ),
            publishers(
                default = vec![],
                description = "If set, only shows results connected to publishers with these IDs",
            ),
            series_type(description = "A specific type to filter by"),
        )
    )]
    fn series_count(
        context: &Context,
        filter: String,
        publishers: Vec<Uuid>,
        series_type: Option<SeriesType>,
    ) -> i32 {
        use crate::schema::series::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::series
            .inner_join(crate::schema::imprint::table)
            .select((
                dsl::series_id,
                dsl::series_type,
                dsl::series_name,
                dsl::issn_print,
                dsl::issn_digital,
                dsl::series_url,
                dsl::imprint_id,
                dsl::created_at,
                dsl::updated_at,
            ))
            .into_boxed();
        // Ordering and construction of filters is important here: result needs to be
        // `WHERE (x = $1 [OR x = $2...]) AND (y ILIKE $3 [OR z ILIKE $3...])`.
        // Interchanging .filter, .or, and .or_filter would result in different bracketing.
        for pub_id in publishers {
            query = query.or_filter(crate::schema::imprint::publisher_id.eq(pub_id));
        }
        if let Some(ser_type) = series_type {
            query = query.filter(dsl::series_type.eq(ser_type))
        }
        // see comment in work_count()
        query
            .filter(
                dsl::series_name
                    .ilike(format!("%{}%", filter))
                    .or(dsl::issn_print.ilike(format!("%{}%", filter)))
                    .or(dsl::issn_digital.ilike(format!("%{}%", filter)))
                    .or(dsl::series_url.ilike(format!("%{}%", filter))),
            )
            .count()
            .get_result::<i64>(&connection)
            .expect("Error loading series count")
            .to_string()
            .parse::<i32>()
            .unwrap()
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
    ) -> Vec<Issue> {
        use crate::schema::issue::dsl::*;
        let connection = context.db.get().unwrap();
        let mut query = issue
            .inner_join(crate::schema::series::table.inner_join(crate::schema::imprint::table))
            .select((series_id, work_id, issue_ordinal, created_at, updated_at))
            .into_boxed();
        match order.field {
            IssueField::SeriesId => match order.direction {
                Direction::Asc => query = query.order(series_id.asc()),
                Direction::Desc => query = query.order(series_id.desc()),
            },
            IssueField::WorkId => match order.direction {
                Direction::Asc => query = query.order(work_id.asc()),
                Direction::Desc => query = query.order(work_id.desc()),
            },
            IssueField::IssueOrdinal => match order.direction {
                Direction::Asc => query = query.order(issue_ordinal.asc()),
                Direction::Desc => query = query.order(issue_ordinal.desc()),
            },
            IssueField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(created_at.asc()),
                Direction::Desc => query = query.order(created_at.desc()),
            },
            IssueField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(updated_at.asc()),
                Direction::Desc => query = query.order(updated_at.desc()),
            },
        }
        for pub_id in publishers {
            query = query.or_filter(crate::schema::imprint::publisher_id.eq(pub_id));
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Issue>(&connection)
            .expect("Error loading issues")
    }

    #[graphql(description = "Query a single issue using its identifiers")]
    fn issue(context: &Context, series_id: Uuid, work_id: Uuid) -> FieldResult<Issue> {
        let connection = context.db.get().unwrap();
        match crate::schema::issue::dsl::issue
            .filter(crate::schema::issue::dsl::series_id.eq(series_id))
            .filter(crate::schema::issue::dsl::work_id.eq(work_id))
            .get_result::<Issue>(&connection)
        {
            Ok(issue) => Ok(issue),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    #[graphql(description = "Get the total number of issues")]
    fn issue_count(context: &Context) -> i32 {
        use crate::schema::issue::dsl::*;
        let connection = context.db.get().unwrap();
        // see comment in work_count()
        issue
            .count()
            .get_result::<i64>(&connection)
            .expect("Error loading issue count")
            .to_string()
            .parse::<i32>()
            .unwrap()
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
            language_code(description = "A specific language to filter by"),
            language_relation(description = "A specific relation to filter by"),
        )
    )]
    fn languages(
        context: &Context,
        limit: i32,
        offset: i32,
        order: LanguageOrderBy,
        publishers: Vec<Uuid>,
        language_code: Option<LanguageCode>,
        language_relation: Option<LanguageRelation>,
    ) -> Vec<Language> {
        use crate::schema::language::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::language
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select((
                dsl::language_id,
                dsl::work_id,
                dsl::language_code,
                dsl::language_relation,
                dsl::main_language,
                dsl::created_at,
                dsl::updated_at,
            ))
            .into_boxed();
        match order.field {
            LanguageField::LanguageId => match order.direction {
                Direction::Asc => query = query.order(dsl::language_id.asc()),
                Direction::Desc => query = query.order(dsl::language_id.desc()),
            },
            LanguageField::WorkId => match order.direction {
                Direction::Asc => query = query.order(dsl::work_id.asc()),
                Direction::Desc => query = query.order(dsl::work_id.desc()),
            },
            LanguageField::LanguageCode => match order.direction {
                Direction::Asc => query = query.order(dsl::language_code.asc()),
                Direction::Desc => query = query.order(dsl::language_code.desc()),
            },
            LanguageField::LanguageRelation => match order.direction {
                Direction::Asc => query = query.order(dsl::language_relation.asc()),
                Direction::Desc => query = query.order(dsl::language_relation.desc()),
            },
            LanguageField::MainLanguage => match order.direction {
                Direction::Asc => query = query.order(dsl::main_language.asc()),
                Direction::Desc => query = query.order(dsl::main_language.desc()),
            },
            LanguageField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::created_at.asc()),
                Direction::Desc => query = query.order(dsl::created_at.desc()),
            },
            LanguageField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::updated_at.asc()),
                Direction::Desc => query = query.order(dsl::updated_at.desc()),
            },
        }
        // Ordering and construction of filters is important here: result needs to be
        // `WHERE (x = $1 [OR x = $2...]) AND (y ILIKE $3 [OR z ILIKE $3...])`.
        // Interchanging .filter, .or, and .or_filter would result in different bracketing.
        for pub_id in publishers {
            query = query.or_filter(crate::schema::imprint::publisher_id.eq(pub_id));
        }
        if let Some(lang_code) = language_code {
            query = query.filter(dsl::language_code.eq(lang_code))
        }
        if let Some(lang_relation) = language_relation {
            query = query.filter(dsl::language_relation.eq(lang_relation))
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Language>(&connection)
            .expect("Error loading languages")
    }

    #[graphql(description = "Query a single language using its id")]
    fn language(context: &Context, language_id: Uuid) -> FieldResult<Language> {
        let connection = context.db.get().unwrap();
        match crate::schema::language::dsl::language
            .find(language_id)
            .get_result::<Language>(&connection)
        {
            Ok(language) => Ok(language),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    #[graphql(description = "Get the total number of languages associated to works")]
    fn language_count(
        context: &Context,
        language_code: Option<LanguageCode>,
        language_relation: Option<LanguageRelation>,
    ) -> i32 {
        use crate::schema::language::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::language.into_boxed();
        if let Some(lang_code) = language_code {
            query = query.filter(dsl::language_code.eq(lang_code))
        }
        if let Some(lang_relation) = language_relation {
            query = query.filter(dsl::language_relation.eq(lang_relation))
        }
        // see comment in work_count()
        query
            .count()
            .get_result::<i64>(&connection)
            .expect("Error loading language count")
            .to_string()
            .parse::<i32>()
            .unwrap()
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
            currency_code(description = "A specific currency to filter by"),
        )
    )]
    fn prices(
        context: &Context,
        limit: i32,
        offset: i32,
        order: PriceOrderBy,
        publishers: Vec<Uuid>,
        currency_code: Option<CurrencyCode>,
    ) -> Vec<Price> {
        use crate::schema::price::dsl;
        let connection = context.db.get().unwrap();
        let mut query =
            dsl::price
                .inner_join(crate::schema::publication::table.inner_join(
                    crate::schema::work::table.inner_join(crate::schema::imprint::table),
                ))
                .select((
                    dsl::price_id,
                    dsl::publication_id,
                    dsl::currency_code,
                    dsl::unit_price,
                    dsl::created_at,
                    dsl::updated_at,
                ))
                .into_boxed();
        match order.field {
            PriceField::PriceId => match order.direction {
                Direction::Asc => query = query.order(dsl::price_id.asc()),
                Direction::Desc => query = query.order(dsl::price_id.desc()),
            },
            PriceField::PublicationId => match order.direction {
                Direction::Asc => query = query.order(dsl::publication_id.asc()),
                Direction::Desc => query = query.order(dsl::publication_id.desc()),
            },
            PriceField::CurrencyCode => match order.direction {
                Direction::Asc => query = query.order(dsl::currency_code.asc()),
                Direction::Desc => query = query.order(dsl::currency_code.desc()),
            },
            PriceField::UnitPrice => match order.direction {
                Direction::Asc => query = query.order(dsl::unit_price.asc()),
                Direction::Desc => query = query.order(dsl::unit_price.desc()),
            },
            PriceField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::created_at.asc()),
                Direction::Desc => query = query.order(dsl::created_at.desc()),
            },
            PriceField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::updated_at.asc()),
                Direction::Desc => query = query.order(dsl::updated_at.desc()),
            },
        }
        // Ordering and construction of filters is important here: result needs to be
        // `WHERE (x = $1 [OR x = $2...]) AND (y ILIKE $3 [OR z ILIKE $3...])`.
        // Interchanging .filter, .or, and .or_filter would result in different bracketing.
        for pub_id in publishers {
            query = query.or_filter(crate::schema::imprint::publisher_id.eq(pub_id));
        }
        if let Some(curr_code) = currency_code {
            query = query.filter(dsl::currency_code.eq(curr_code))
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Price>(&connection)
            .expect("Error loading prices")
    }

    #[graphql(description = "Query a single price using its id")]
    fn price(context: &Context, price_id: Uuid) -> FieldResult<Price> {
        let connection = context.db.get().unwrap();
        match crate::schema::price::dsl::price
            .find(price_id)
            .get_result::<Price>(&connection)
        {
            Ok(price) => Ok(price),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    #[graphql(description = "Get the total number of prices associated to works")]
    fn price_count(context: &Context, currency_code: Option<CurrencyCode>) -> i32 {
        use crate::schema::price::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::price.into_boxed();
        if let Some(curr_code) = currency_code {
            query = query.filter(dsl::currency_code.eq(curr_code))
        }
        // see comment in work_count()
        query
            .count()
            .get_result::<i64>(&connection)
            .expect("Error loading price count")
            .to_string()
            .parse::<i32>()
            .unwrap()
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
            subject_type(description = "A specific type to filter by"),
        )
    )]
    fn subjects(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: SubjectOrderBy,
        publishers: Vec<Uuid>,
        subject_type: Option<SubjectType>,
    ) -> Vec<Subject> {
        use crate::schema::subject::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::subject
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select((
                dsl::subject_id,
                dsl::work_id,
                dsl::subject_type,
                dsl::subject_code,
                dsl::subject_ordinal,
                dsl::created_at,
                dsl::updated_at,
            ))
            .into_boxed();
        match order.field {
            SubjectField::SubjectId => match order.direction {
                Direction::Asc => query = query.order(dsl::subject_id.asc()),
                Direction::Desc => query = query.order(dsl::subject_id.desc()),
            },
            SubjectField::WorkId => match order.direction {
                Direction::Asc => query = query.order(dsl::work_id.asc()),
                Direction::Desc => query = query.order(dsl::work_id.desc()),
            },
            SubjectField::SubjectType => match order.direction {
                Direction::Asc => query = query.order(dsl::subject_type.asc()),
                Direction::Desc => query = query.order(dsl::subject_type.desc()),
            },
            SubjectField::SubjectCode => match order.direction {
                Direction::Asc => query = query.order(dsl::subject_code.asc()),
                Direction::Desc => query = query.order(dsl::subject_code.desc()),
            },
            SubjectField::SubjectOrdinal => match order.direction {
                Direction::Asc => query = query.order(dsl::subject_ordinal.asc()),
                Direction::Desc => query = query.order(dsl::subject_ordinal.desc()),
            },
            SubjectField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::created_at.asc()),
                Direction::Desc => query = query.order(dsl::created_at.desc()),
            },
            SubjectField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::updated_at.asc()),
                Direction::Desc => query = query.order(dsl::updated_at.desc()),
            },
        }
        // Ordering and construction of filters is important here: result needs to be
        // `WHERE (x = $1 [OR x = $2...]) AND (y ILIKE $3 [OR z ILIKE $3...])`.
        // Interchanging .filter, .or, and .or_filter would result in different bracketing.
        for pub_id in publishers {
            query = query.or_filter(crate::schema::imprint::publisher_id.eq(pub_id));
        }
        if let Some(sub_type) = subject_type {
            query = query.filter(dsl::subject_type.eq(sub_type))
        }
        query
            .filter(dsl::subject_code.ilike(format!("%{}%", filter)))
            .limit(limit.into())
            .offset(offset.into())
            .load::<Subject>(&connection)
            .expect("Error loading subjects")
    }

    #[graphql(description = "Query a single subject using its id")]
    fn subject(context: &Context, subject_id: Uuid) -> FieldResult<Subject> {
        let connection = context.db.get().unwrap();
        match crate::schema::subject::dsl::subject
            .find(subject_id)
            .get_result::<Subject>(&connection)
        {
            Ok(subject) => Ok(subject),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    #[graphql(description = "Get the total number of subjects associated to works")]
    fn subject_count(context: &Context, filter: String, subject_type: Option<SubjectType>) -> i32 {
        use crate::schema::subject::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::subject.into_boxed();
        if let Some(sub_type) = subject_type {
            query = query.filter(dsl::subject_type.eq(sub_type))
        }
        // see comment in work_count()
        query
            .filter(dsl::subject_code.ilike(format!("%{}%", filter)))
            .count()
            .get_result::<i64>(&connection)
            .expect("Error loading subject count")
            .to_string()
            .parse::<i32>()
            .unwrap()
    }

    #[graphql(
        description = "Query the full list of funders",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on funderName and funderDoi",
            ),
            order(
                default = FunderOrderBy::default(),
                description = "The order in which to sort the results",
            ),
        )
    )]
    fn funders(
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: FunderOrderBy,
    ) -> Vec<Funder> {
        use crate::schema::funder::dsl::*;
        let connection = context.db.get().unwrap();
        let mut query = funder.into_boxed();
        match order.field {
            FunderField::FunderId => match order.direction {
                Direction::Asc => query = query.order(funder_id.asc()),
                Direction::Desc => query = query.order(funder_id.desc()),
            },
            FunderField::FunderName => match order.direction {
                Direction::Asc => query = query.order(funder_name.asc()),
                Direction::Desc => query = query.order(funder_name.desc()),
            },
            FunderField::FunderDoi => match order.direction {
                Direction::Asc => query = query.order(funder_doi.asc()),
                Direction::Desc => query = query.order(funder_doi.desc()),
            },
            FunderField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(created_at.asc()),
                Direction::Desc => query = query.order(created_at.desc()),
            },
            FunderField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(updated_at.asc()),
                Direction::Desc => query = query.order(updated_at.desc()),
            },
        }
        query
            .filter(funder_name.ilike(format!("%{}%", filter)))
            .or_filter(funder_doi.ilike(format!("%{}%", filter)))
            .limit(limit.into())
            .offset(offset.into())
            .load::<Funder>(&connection)
            .expect("Error loading funders")
    }

    #[graphql(description = "Query a single funder using its id")]
    fn funder(context: &Context, funder_id: Uuid) -> FieldResult<Funder> {
        let connection = context.db.get().unwrap();
        match crate::schema::funder::dsl::funder
            .find(funder_id)
            .get_result::<Funder>(&connection)
        {
            Ok(funder) => Ok(funder),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    #[graphql(
        description = "Get the total number of funders",
        arguments(
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on funderName and funderDoi",
            ),
        )
    )]
    fn funder_count(context: &Context, filter: String) -> i32 {
        use crate::schema::funder::dsl::*;
        let connection = context.db.get().unwrap();
        // see comment in work_count()
        funder
            .filter(funder_name.ilike(format!("%{}%", filter)))
            .or_filter(funder_doi.ilike(format!("%{}%", filter)))
            .count()
            .get_result::<i64>(&connection)
            .expect("Error loading funder count")
            .to_string()
            .parse::<i32>()
            .unwrap()
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
    ) -> Vec<Funding> {
        use crate::schema::funding::dsl::*;
        let connection = context.db.get().unwrap();
        let mut query = funding
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select((
                funding_id,
                work_id,
                funder_id,
                program,
                project_name,
                project_shortname,
                grant_number,
                jurisdiction,
                created_at,
                updated_at,
            ))
            .into_boxed();
        match order.field {
            FundingField::FundingId => match order.direction {
                Direction::Asc => query = query.order(funding_id.asc()),
                Direction::Desc => query = query.order(funding_id.desc()),
            },
            FundingField::WorkId => match order.direction {
                Direction::Asc => query = query.order(work_id.asc()),
                Direction::Desc => query = query.order(work_id.desc()),
            },
            FundingField::FunderId => match order.direction {
                Direction::Asc => query = query.order(funder_id.asc()),
                Direction::Desc => query = query.order(funder_id.desc()),
            },
            FundingField::Program => match order.direction {
                Direction::Asc => query = query.order(program.asc()),
                Direction::Desc => query = query.order(program.desc()),
            },
            FundingField::ProjectName => match order.direction {
                Direction::Asc => query = query.order(project_name.asc()),
                Direction::Desc => query = query.order(project_name.desc()),
            },
            FundingField::ProjectShortname => match order.direction {
                Direction::Asc => query = query.order(project_shortname.asc()),
                Direction::Desc => query = query.order(project_shortname.desc()),
            },
            FundingField::GrantNumber => match order.direction {
                Direction::Asc => query = query.order(grant_number.asc()),
                Direction::Desc => query = query.order(grant_number.desc()),
            },
            FundingField::Jurisdiction => match order.direction {
                Direction::Asc => query = query.order(jurisdiction.asc()),
                Direction::Desc => query = query.order(jurisdiction.desc()),
            },
            FundingField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(created_at.asc()),
                Direction::Desc => query = query.order(created_at.desc()),
            },
            FundingField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(updated_at.asc()),
                Direction::Desc => query = query.order(updated_at.desc()),
            },
        }
        for pub_id in publishers {
            query = query.or_filter(crate::schema::imprint::publisher_id.eq(pub_id));
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Funding>(&connection)
            .expect("Error loading fundings")
    }

    #[graphql(description = "Query a single funding using its id")]
    fn funding(context: &Context, funding_id: Uuid) -> FieldResult<Funding> {
        let connection = context.db.get().unwrap();
        match crate::schema::funding::dsl::funding
            .find(funding_id)
            .get_result::<Funding>(&connection)
        {
            Ok(funding) => Ok(funding),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    #[graphql(description = "Get the total number of funding instances associated to works")]
    fn funding_count(context: &Context) -> i32 {
        use crate::schema::funding::dsl::*;
        let connection = context.db.get().unwrap();
        // see comment in work_count()
        funding
            .count()
            .get_result::<i64>(&connection)
            .expect("Error loading funding count")
            .to_string()
            .parse::<i32>()
            .unwrap()
    }
}

pub struct MutationRoot;

#[juniper::object(Context = Context)]
impl MutationRoot {
    fn create_work(context: &Context, data: NewWork) -> FieldResult<Work> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_imprint(data.imprint_id, context)?;

        let connection = context.db.get().unwrap();
        match diesel::insert_into(work::table)
            .values(&data)
            .get_result(&connection)
        {
            Ok(work) => Ok(work),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn create_publisher(context: &Context, data: NewPublisher) -> FieldResult<Publisher> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        // Only superusers can create new publishers - NewPublisher has no ID field
        if !context.account_access.is_superuser {
            return Err(ThothError::Unauthorised.into());
        }

        let connection = context.db.get().unwrap();
        match diesel::insert_into(publisher::table)
            .values(&data)
            .get_result(&connection)
        {
            Ok(publisher) => Ok(publisher),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn create_imprint(context: &Context, data: NewImprint) -> FieldResult<Imprint> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context.account_access.can_edit(data.publisher_id)?;

        match Imprint::create(&context.db, &data) {
            Ok(imprint) => Ok(imprint),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn create_contributor(context: &Context, data: NewContributor) -> FieldResult<Contributor> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;

        let connection = context.db.get().unwrap();
        match diesel::insert_into(contributor::table)
            .values(&data)
            .get_result(&connection)
        {
            Ok(contributor) => Ok(contributor),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn create_contribution(context: &Context, data: NewContribution) -> FieldResult<Contribution> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;

        let connection = context.db.get().unwrap();
        match diesel::insert_into(contribution::table)
            .values(&data)
            .get_result(&connection)
        {
            Ok(contribution) => Ok(contribution),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn create_publication(context: &Context, data: NewPublication) -> FieldResult<Publication> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;

        let connection = context.db.get().unwrap();
        match diesel::insert_into(publication::table)
            .values(&data)
            .get_result(&connection)
        {
            Ok(publication) => Ok(publication),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn create_series(context: &Context, data: NewSeries) -> FieldResult<Series> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_imprint(data.imprint_id, context)?;

        let connection = context.db.get().unwrap();
        match diesel::insert_into(series::table)
            .values(&data)
            .get_result(&connection)
        {
            Ok(series) => Ok(series),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn create_issue(context: &Context, data: NewIssue) -> FieldResult<Issue> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;
        issue_imprints_match(data.work_id, data.series_id, context)?;

        let connection = context.db.get().unwrap();
        match diesel::insert_into(issue::table)
            .values(&data)
            .get_result(&connection)
        {
            Ok(issue) => Ok(issue),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn create_language(context: &Context, data: NewLanguage) -> FieldResult<Language> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;

        let connection = context.db.get().unwrap();
        match diesel::insert_into(language::table)
            .values(&data)
            .get_result(&connection)
        {
            Ok(language) => Ok(language),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn create_funder(context: &Context, data: NewFunder) -> FieldResult<Funder> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;

        let connection = context.db.get().unwrap();
        match diesel::insert_into(funder::table)
            .values(&data)
            .get_result(&connection)
        {
            Ok(funder) => Ok(funder),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn create_funding(context: &Context, data: NewFunding) -> FieldResult<Funding> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;

        let connection = context.db.get().unwrap();
        match diesel::insert_into(funding::table)
            .values(&data)
            .get_result(&connection)
        {
            Ok(funding) => Ok(funding),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn create_price(context: &Context, data: NewPrice) -> FieldResult<Price> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_publication(data.publication_id, context)?;

        let connection = context.db.get().unwrap();
        match diesel::insert_into(price::table)
            .values(&data)
            .get_result(&connection)
        {
            Ok(price) => Ok(price),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn create_subject(context: &Context, data: NewSubject) -> FieldResult<Subject> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;

        check_subject(&data.subject_type, &data.subject_code)?;

        let connection = context.db.get().unwrap();
        match diesel::insert_into(subject::table)
            .values(&data)
            .get_result(&connection)
        {
            Ok(subject) => Ok(subject),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn update_work(context: &Context, data: PatchWork) -> FieldResult<Work> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_imprint(data.imprint_id, context)?;

        let connection = context.db.get().unwrap();
        let target = crate::schema::work::dsl::work.find(data.work_id);
        let work = target.get_result::<Work>(&connection).unwrap();
        if !(data.imprint_id == work.imprint_id) {
            user_can_edit_imprint(work.imprint_id, context)?;
            can_update_work_imprint(work.work_id, context)?;
        }

        connection.transaction(
            || match diesel::update(target).set(&data).get_result(&connection) {
                Ok(c) => {
                    let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
                    match NewWorkHistory::new(work, account_id).insert(&connection) {
                        Ok(_) => Ok(c),
                        Err(e) => Err(FieldError::from(e)),
                    }
                }
                Err(e) => Err(FieldError::from(e)),
            },
        )
    }

    fn update_publisher(context: &Context, data: PatchPublisher) -> FieldResult<Publisher> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context.account_access.can_edit(data.publisher_id)?;

        let connection = context.db.get().unwrap();
        let target = crate::schema::publisher::dsl::publisher.find(&data.publisher_id);
        let publisher = target.get_result::<Publisher>(&connection).unwrap();
        if !(data.publisher_id == publisher.publisher_id) {
            context.account_access.can_edit(publisher.publisher_id)?;
        }

        connection.transaction(
            || match diesel::update(target).set(&data).get_result(&connection) {
                Ok(c) => {
                    let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
                    match NewPublisherHistory::new(publisher, account_id).insert(&connection) {
                        Ok(_) => Ok(c),
                        Err(e) => Err(FieldError::from(e)),
                    }
                }
                Err(e) => Err(FieldError::from(e)),
            },
        )
    }

    fn update_imprint(context: &Context, data: PatchImprint) -> FieldResult<Imprint> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context.account_access.can_edit(data.publisher_id)?;
        let imprint = Imprint::from_id(&context.db, &data.imprint_id).unwrap();
        if !(data.publisher_id == imprint.publisher_id) {
            context.account_access.can_edit(imprint.publisher_id)?;
        }
        let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
        match imprint.update(&context.db, &data, &account_id) {
            Ok(c) => Ok(c),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn update_contributor(context: &Context, data: PatchContributor) -> FieldResult<Contributor> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let connection = context.db.get().unwrap();

        let target = crate::schema::contributor::dsl::contributor.find(&data.contributor_id);
        let contributor = target.get_result::<Contributor>(&connection).unwrap();

        connection.transaction(
            || match diesel::update(target).set(&data).get_result(&connection) {
                Ok(c) => {
                    let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
                    match NewContributorHistory::new(contributor, account_id).insert(&connection) {
                        Ok(_) => Ok(c),
                        Err(e) => Err(FieldError::from(e)),
                    }
                }
                Err(e) => Err(FieldError::from(e)),
            },
        )
    }

    fn update_contribution(
        context: &Context,
        data: PatchContribution,
    ) -> FieldResult<Contribution> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;

        let connection = context.db.get().unwrap();

        use crate::schema::contribution::dsl::*;
        // need to duplicate these otherwise the query gets moved
        let target_contribution = contribution
            .filter(work_id.eq(&data.work_id))
            .filter(contributor_id.eq(&data.contributor_id))
            .filter(contribution_type.eq(&data.contribution_type))
            .get_result::<Contribution>(&connection)
            .unwrap();
        let target = contribution
            .filter(work_id.eq(&data.work_id))
            .filter(contributor_id.eq(&data.contributor_id))
            .filter(contribution_type.eq(&data.contribution_type));

        connection.transaction(
            || match diesel::update(target).set(&data).get_result(&connection) {
                Ok(c) => {
                    let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
                    match NewContributionHistory::new(target_contribution, account_id)
                        .insert(&connection)
                    {
                        Ok(_) => Ok(c),
                        Err(e) => Err(FieldError::from(e)),
                    }
                }
                Err(e) => Err(FieldError::from(e)),
            },
        )
    }

    fn update_publication(context: &Context, data: PatchPublication) -> FieldResult<Publication> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;

        let connection = context.db.get().unwrap();
        let target = crate::schema::publication::dsl::publication.find(&data.publication_id);
        let publication = target.get_result::<Publication>(&connection).unwrap();
        if !(data.work_id == publication.work_id) {
            user_can_edit_work(publication.work_id, context)?;
        }

        connection.transaction(
            || match diesel::update(target).set(&data).get_result(&connection) {
                Ok(c) => {
                    let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
                    match NewPublicationHistory::new(publication, account_id).insert(&connection) {
                        Ok(_) => Ok(c),
                        Err(e) => Err(FieldError::from(e)),
                    }
                }
                Err(e) => Err(FieldError::from(e)),
            },
        )
    }

    fn update_series(context: &Context, data: PatchSeries) -> FieldResult<Series> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_imprint(data.imprint_id, context)?;

        let connection = context.db.get().unwrap();
        let target = crate::schema::series::dsl::series.find(&data.series_id);
        let series = target.get_result::<Series>(&connection).unwrap();
        if !(data.imprint_id == series.imprint_id) {
            user_can_edit_imprint(series.imprint_id, context)?;
        }

        connection.transaction(
            || match diesel::update(target).set(&data).get_result(&connection) {
                Ok(c) => {
                    let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
                    match NewSeriesHistory::new(series, account_id).insert(&connection) {
                        Ok(_) => Ok(c),
                        Err(e) => Err(FieldError::from(e)),
                    }
                }
                Err(e) => Err(FieldError::from(e)),
            },
        )
    }

    fn update_issue(context: &Context, data: PatchIssue) -> FieldResult<Issue> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;
        issue_imprints_match(data.work_id, data.series_id, context)?;

        let connection = context.db.get().unwrap();

        use crate::schema::issue::dsl::*;
        let target = issue
            .filter(series_id.eq(&data.series_id))
            .filter(work_id.eq(&data.work_id));
        let target_issue = target.get_result::<Issue>(&connection).unwrap();

        connection.transaction(
            || match diesel::update(target).set(&data).get_result(&connection) {
                Ok(c) => {
                    let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
                    match NewIssueHistory::new(target_issue, account_id).insert(&connection) {
                        Ok(_) => Ok(c),
                        Err(e) => Err(FieldError::from(e)),
                    }
                }
                Err(e) => Err(FieldError::from(e)),
            },
        )
    }

    fn update_language(context: &Context, data: PatchLanguage) -> FieldResult<Language> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;

        let connection = context.db.get().unwrap();
        let target = crate::schema::language::dsl::language.find(&data.language_id);
        let language = target.get_result::<Language>(&connection).unwrap();
        if !(data.work_id == language.work_id) {
            user_can_edit_work(language.work_id, context)?;
        }

        connection.transaction(
            || match diesel::update(target).set(&data).get_result(&connection) {
                Ok(c) => {
                    let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
                    match NewLanguageHistory::new(language, account_id).insert(&connection) {
                        Ok(_) => Ok(c),
                        Err(e) => Err(FieldError::from(e)),
                    }
                }
                Err(e) => Err(FieldError::from(e)),
            },
        )
    }

    fn update_funder(context: &Context, data: PatchFunder) -> FieldResult<Funder> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let connection = context.db.get().unwrap();
        let target = crate::schema::funder::dsl::funder.find(&data.funder_id);
        let funder = target.get_result::<Funder>(&connection).unwrap();

        connection.transaction(
            || match diesel::update(target).set(&data).get_result(&connection) {
                Ok(c) => {
                    let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
                    match NewFunderHistory::new(funder, account_id).insert(&connection) {
                        Ok(_) => Ok(c),
                        Err(e) => Err(FieldError::from(e)),
                    }
                }
                Err(e) => Err(FieldError::from(e)),
            },
        )
    }

    fn update_funding(context: &Context, data: PatchFunding) -> FieldResult<Funding> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;

        let connection = context.db.get().unwrap();
        let target = crate::schema::funding::dsl::funding.find(&data.funding_id);
        let funding = target.get_result::<Funding>(&connection).unwrap();
        if !(data.work_id == funding.work_id) {
            user_can_edit_work(funding.work_id, context)?;
        }

        connection.transaction(
            || match diesel::update(target).set(&data).get_result(&connection) {
                Ok(c) => {
                    let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
                    match NewFundingHistory::new(funding, account_id).insert(&connection) {
                        Ok(_) => Ok(c),
                        Err(e) => Err(FieldError::from(e)),
                    }
                }
                Err(e) => Err(FieldError::from(e)),
            },
        )
    }

    fn update_price(context: &Context, data: PatchPrice) -> FieldResult<Price> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_publication(data.publication_id, context)?;

        let connection = context.db.get().unwrap();
        let target = crate::schema::price::dsl::price.find(&data.price_id);
        let result = target.get_result::<Price>(&connection);
        let price = result.unwrap();
        if !(data.publication_id == price.publication_id) {
            user_can_edit_publication(price.publication_id, context)?;
        }

        connection.transaction(
            || match diesel::update(target).set(&data).get_result(&connection) {
                Ok(c) => {
                    let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
                    match NewPriceHistory::new(price, account_id).insert(&connection) {
                        Ok(_) => Ok(c),
                        Err(e) => Err(FieldError::from(e)),
                    }
                }
                Err(e) => Err(FieldError::from(e)),
            },
        )
    }

    fn update_subject(context: &Context, data: PatchSubject) -> FieldResult<Subject> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(data.work_id, context)?;

        let connection = context.db.get().unwrap();
        let target = crate::schema::subject::dsl::subject.find(&data.subject_id);
        let subject = target.get_result::<Subject>(&connection).unwrap();
        if !(data.work_id == subject.work_id) {
            user_can_edit_work(subject.work_id, context)?;
        }

        check_subject(&data.subject_type, &data.subject_code)?;

        connection.transaction(
            || match diesel::update(target).set(&data).get_result(&connection) {
                Ok(c) => {
                    let account_id = context.token.jwt.as_ref().unwrap().account_id(&context.db);
                    match NewSubjectHistory::new(subject, account_id).insert(&connection) {
                        Ok(_) => Ok(c),
                        Err(e) => Err(FieldError::from(e)),
                    }
                }
                Err(e) => Err(FieldError::from(e)),
            },
        )
    }

    fn delete_work(context: &Context, work_id: Uuid) -> FieldResult<Work> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(work_id, context)?;

        let connection = context.db.get().unwrap();
        let target = crate::schema::work::dsl::work.find(work_id);
        let result = target.get_result::<Work>(&connection);
        match diesel::delete(target).execute(&connection) {
            Ok(c) => Ok(result.unwrap()),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn delete_publisher(context: &Context, publisher_id: Uuid) -> FieldResult<Publisher> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        context.account_access.can_edit(publisher_id)?;

        let connection = context.db.get().unwrap();
        let target = crate::schema::publisher::dsl::publisher.find(publisher_id);
        let result = target.get_result::<Publisher>(&connection);
        match diesel::delete(target).execute(&connection) {
            Ok(c) => Ok(result.unwrap()),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn delete_imprint(context: &Context, imprint_id: Uuid) -> FieldResult<Imprint> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let imprint = Imprint::from_id(&context.db, &imprint_id).unwrap();
        context.account_access.can_edit(imprint.publisher_id)?;

        match imprint.delete(&context.db) {
            Ok(imprint) => Ok(imprint),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn delete_contributor(context: &Context, contributor_id: Uuid) -> FieldResult<Contributor> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let connection = context.db.get().unwrap();

        let target = crate::schema::contributor::dsl::contributor.find(contributor_id);
        let result = target.get_result::<Contributor>(&connection);
        match diesel::delete(target).execute(&connection) {
            Ok(c) => Ok(result.unwrap()),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn delete_contribution(
        context: &Context,
        work_id: Uuid,
        contributor_id: Uuid,
        contribution_type: ContributionType,
    ) -> FieldResult<Contribution> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(work_id, context)?;

        let connection = context.db.get().unwrap();

        use crate::schema::contribution::dsl;
        let target = dsl::contribution
            .filter(dsl::work_id.eq(&work_id))
            .filter(dsl::contributor_id.eq(&contributor_id))
            .filter(dsl::contribution_type.eq(&contribution_type));
        let result = dsl::contribution
            .filter(dsl::work_id.eq(&work_id))
            .filter(dsl::contributor_id.eq(&contributor_id))
            .filter(dsl::contribution_type.eq(&contribution_type))
            .get_result::<Contribution>(&connection);
        match diesel::delete(target).execute(&connection) {
            Ok(c) => Ok(result.unwrap()),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn delete_publication(context: &Context, publication_id: Uuid) -> FieldResult<Publication> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_publication(publication_id, context)?;

        let connection = context.db.get().unwrap();
        let target = crate::schema::publication::dsl::publication.find(publication_id);
        let result = target.get_result::<Publication>(&connection);

        match diesel::delete(target).execute(&connection) {
            Ok(c) => Ok(result.unwrap()),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn delete_series(context: &Context, series_id: Uuid) -> FieldResult<Series> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let connection = context.db.get().unwrap();

        let target = crate::schema::series::dsl::series.find(series_id);
        let result = target.get_result::<Series>(&connection);
        let series = result.unwrap();
        user_can_edit_imprint(series.imprint_id, context)?;

        match diesel::delete(target).execute(&connection) {
            Ok(c) => Ok(series),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn delete_issue(context: &Context, series_id: Uuid, work_id: Uuid) -> FieldResult<Issue> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        user_can_edit_work(work_id, context)?;

        let connection = context.db.get().unwrap();

        use crate::schema::issue::dsl;
        let target = dsl::issue
            .filter(dsl::series_id.eq(&series_id))
            .filter(dsl::work_id.eq(&work_id));
        let result = dsl::issue
            .filter(dsl::series_id.eq(&series_id))
            .filter(dsl::work_id.eq(&work_id))
            .get_result::<Issue>(&connection);
        match diesel::delete(target).execute(&connection) {
            Ok(c) => Ok(result.unwrap()),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn delete_language(context: &Context, language_id: Uuid) -> FieldResult<Language> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let connection = context.db.get().unwrap();

        let target = crate::schema::language::dsl::language.find(language_id);
        let result = target.get_result::<Language>(&connection);
        let language = result.unwrap();
        user_can_edit_work(language.work_id, context)?;

        match diesel::delete(target).execute(&connection) {
            Ok(c) => Ok(language),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn delete_funder(context: &Context, funder_id: Uuid) -> FieldResult<Funder> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let connection = context.db.get().unwrap();

        let target = crate::schema::funder::dsl::funder.find(funder_id);
        let result = target.get_result::<Funder>(&connection);
        match diesel::delete(target).execute(&connection) {
            Ok(c) => Ok(result.unwrap()),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn delete_funding(context: &Context, funding_id: Uuid) -> FieldResult<Funding> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let connection = context.db.get().unwrap();

        let target = crate::schema::funding::dsl::funding.find(funding_id);
        let result = target.get_result::<Funding>(&connection);
        let funding = result.unwrap();
        user_can_edit_work(funding.work_id, context)?;

        match diesel::delete(target).execute(&connection) {
            Ok(c) => Ok(funding),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn delete_price(context: &Context, price_id: Uuid) -> FieldResult<Price> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let connection = context.db.get().unwrap();

        let target = crate::schema::price::dsl::price.find(price_id);
        let result = target.get_result::<Price>(&connection);
        let price = result.unwrap();
        user_can_edit_publication(price.publication_id, context)?;

        match diesel::delete(target).execute(&connection) {
            Ok(c) => Ok(price),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn delete_subject(context: &Context, subject_id: Uuid) -> FieldResult<Subject> {
        context.token.jwt.as_ref().ok_or(ThothError::Unauthorised)?;
        let connection = context.db.get().unwrap();

        let target = crate::schema::subject::dsl::subject.find(subject_id);
        let result = target.get_result::<Subject>(&connection);
        let subject = result.unwrap();
        user_can_edit_work(subject.work_id, context)?;

        match diesel::delete(target).execute(&connection) {
            Ok(c) => Ok(subject),
            Err(e) => Err(FieldError::from(e)),
        }
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

    pub fn edition(&self) -> &i32 {
        &self.edition
    }

    #[graphql(
        description = "Digital Object Identifier of the work as full URL. It must use the HTTPS scheme and the doi.org domain (e.g. https://doi.org/10.11647/obp.0001)"
    )]
    pub fn doi(&self) -> Option<&String> {
        self.doi.as_ref()
    }

    pub fn publication_date(&self) -> Option<NaiveDate> {
        self.publication_date
    }

    pub fn place(&self) -> Option<&String> {
        self.place.as_ref()
    }

    pub fn width(&self) -> Option<&i32> {
        self.width.as_ref()
    }

    pub fn height(&self) -> Option<&i32> {
        self.height.as_ref()
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

    pub fn copyright_holder(&self) -> &str {
        self.copyright_holder.as_str()
    }

    pub fn landing_page(&self) -> Option<&String> {
        self.landing_page.as_ref()
    }

    pub fn lccn(&self) -> Option<&String> {
        self.lccn.as_ref()
    }

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

    pub fn toc(&self) -> Option<&String> {
        self.toc.as_ref()
    }

    pub fn cover_url(&self) -> Option<&String> {
        self.cover_url.as_ref()
    }

    pub fn cover_caption(&self) -> Option<&String> {
        self.cover_caption.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn imprint(&self, context: &Context) -> Imprint {
        use crate::schema::imprint::dsl::*;
        let connection = context.db.get().unwrap();
        imprint
            .find(self.imprint_id)
            .first(&connection)
            .expect("Error loading imprint")
    }

    #[graphql(
        description = "Get contributions linked to this work",
        arguments(
            order(
                default = {
                    ContributionOrderBy {
                        field: ContributionField::ContributionType,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            contribution_type(description = "A specific type to filter by"),
        )
    )]
    pub fn contributions(
        &self,
        context: &Context,
        order: ContributionOrderBy,
        contribution_type: Option<ContributionType>,
    ) -> Vec<Contribution> {
        use crate::schema::contribution::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::contribution.into_boxed();
        match order.field {
            ContributionField::WorkId => match order.direction {
                Direction::Asc => query = query.order(dsl::work_id.asc()),
                Direction::Desc => query = query.order(dsl::work_id.desc()),
            },
            ContributionField::ContributorId => match order.direction {
                Direction::Asc => query = query.order(dsl::contributor_id.asc()),
                Direction::Desc => query = query.order(dsl::contributor_id.desc()),
            },
            ContributionField::ContributionType => match order.direction {
                Direction::Asc => query = query.order(dsl::contribution_type.asc()),
                Direction::Desc => query = query.order(dsl::contribution_type.desc()),
            },
            ContributionField::MainContribution => match order.direction {
                Direction::Asc => query = query.order(dsl::main_contribution.asc()),
                Direction::Desc => query = query.order(dsl::main_contribution.desc()),
            },
            ContributionField::Biography => match order.direction {
                Direction::Asc => query = query.order(dsl::biography.asc()),
                Direction::Desc => query = query.order(dsl::biography.desc()),
            },
            ContributionField::Institution => match order.direction {
                Direction::Asc => query = query.order(dsl::institution.asc()),
                Direction::Desc => query = query.order(dsl::institution.desc()),
            },
            ContributionField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::created_at.asc()),
                Direction::Desc => query = query.order(dsl::created_at.desc()),
            },
            ContributionField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::updated_at.asc()),
                Direction::Desc => query = query.order(dsl::updated_at.desc()),
            },
            ContributionField::FirstName => match order.direction {
                Direction::Asc => query = query.order(dsl::first_name.asc()),
                Direction::Desc => query = query.order(dsl::first_name.desc()),
            },
            ContributionField::LastName => match order.direction {
                Direction::Asc => query = query.order(dsl::last_name.asc()),
                Direction::Desc => query = query.order(dsl::last_name.desc()),
            },
            ContributionField::FullName => match order.direction {
                Direction::Asc => query = query.order(dsl::full_name.asc()),
                Direction::Desc => query = query.order(dsl::full_name.desc()),
            },
        }
        if let Some(cont_type) = contribution_type {
            query = query.filter(dsl::contribution_type.eq(cont_type))
        }
        query
            .filter(dsl::work_id.eq(self.work_id))
            .load::<Contribution>(&connection)
            .expect("Error loading contributions")
    }

    #[graphql(
        description = "Get languages linked to this work",
        arguments(
            order(
                default = {
                    LanguageOrderBy {
                        field: LanguageField::LanguageCode,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            language_code(description = "A specific language to filter by"),
            language_relation(description = "A specific relation to filter by"),
        )
    )]
    pub fn languages(
        &self,
        context: &Context,
        order: LanguageOrderBy,
        language_code: Option<LanguageCode>,
        language_relation: Option<LanguageRelation>,
    ) -> Vec<Language> {
        use crate::schema::language::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::language.into_boxed();
        match order.field {
            LanguageField::LanguageId => match order.direction {
                Direction::Asc => query = query.order(dsl::language_id.asc()),
                Direction::Desc => query = query.order(dsl::language_id.desc()),
            },
            LanguageField::WorkId => match order.direction {
                Direction::Asc => query = query.order(dsl::work_id.asc()),
                Direction::Desc => query = query.order(dsl::work_id.desc()),
            },
            LanguageField::LanguageCode => match order.direction {
                Direction::Asc => query = query.order(dsl::language_code.asc()),
                Direction::Desc => query = query.order(dsl::language_code.desc()),
            },
            LanguageField::LanguageRelation => match order.direction {
                Direction::Asc => query = query.order(dsl::language_relation.asc()),
                Direction::Desc => query = query.order(dsl::language_relation.desc()),
            },
            LanguageField::MainLanguage => match order.direction {
                Direction::Asc => query = query.order(dsl::main_language.asc()),
                Direction::Desc => query = query.order(dsl::main_language.desc()),
            },
            LanguageField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::created_at.asc()),
                Direction::Desc => query = query.order(dsl::created_at.desc()),
            },
            LanguageField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::updated_at.asc()),
                Direction::Desc => query = query.order(dsl::updated_at.desc()),
            },
        }
        if let Some(lang_code) = language_code {
            query = query.filter(dsl::language_code.eq(lang_code))
        }
        if let Some(lang_relation) = language_relation {
            query = query.filter(dsl::language_relation.eq(lang_relation))
        }
        query
            .filter(dsl::work_id.eq(self.work_id))
            .load::<Language>(&connection)
            .expect("Error loading languages")
    }

    #[graphql(
        description = "Get publications linked to this work",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on isbn and publication_url"
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
            publication_type(description = "A specific type to filter by"),
        )
    )]
    pub fn publications(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: PublicationOrderBy,
        publication_type: Option<PublicationType>,
    ) -> Vec<Publication> {
        let connection = context.db.get().unwrap();
        use crate::schema::publication::dsl;
        let mut query = dsl::publication.into_boxed();
        match order.field {
            PublicationField::PublicationId => match order.direction {
                Direction::Asc => query = query.order(dsl::publication_id.asc()),
                Direction::Desc => query = query.order(dsl::publication_id.desc()),
            },
            PublicationField::PublicationType => match order.direction {
                Direction::Asc => query = query.order(dsl::publication_type.asc()),
                Direction::Desc => query = query.order(dsl::publication_type.desc()),
            },
            PublicationField::WorkId => match order.direction {
                Direction::Asc => query = query.order(dsl::work_id.asc()),
                Direction::Desc => query = query.order(dsl::work_id.desc()),
            },
            PublicationField::Isbn => match order.direction {
                Direction::Asc => query = query.order(dsl::isbn.asc()),
                Direction::Desc => query = query.order(dsl::isbn.desc()),
            },
            PublicationField::PublicationUrl => match order.direction {
                Direction::Asc => query = query.order(dsl::publication_url.asc()),
                Direction::Desc => query = query.order(dsl::publication_url.desc()),
            },
            PublicationField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::created_at.asc()),
                Direction::Desc => query = query.order(dsl::created_at.desc()),
            },
            PublicationField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::updated_at.asc()),
                Direction::Desc => query = query.order(dsl::updated_at.desc()),
            },
        }
        if let Some(pub_type) = publication_type {
            query = query.filter(dsl::publication_type.eq(pub_type));
        }
        // ISBN and URL fields are both nullable, so searching with an empty filter could fail
        if !filter.is_empty() {
            query = query.filter(
                dsl::isbn
                    .ilike(format!("%{}%", filter))
                    .or(dsl::publication_url.ilike(format!("%{}%", filter))),
            );
        }
        query
            .filter(dsl::work_id.eq(self.work_id))
            .limit(limit.into())
            .offset(offset.into())
            .load::<Publication>(&connection)
            .expect("Error loading publications")
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
            subject_type(description = "A specific type to filter by"),
        )
    )]
    pub fn subjects(
        &self,
        context: &Context,
        limit: i32,
        offset: i32,
        filter: String,
        order: SubjectOrderBy,
        subject_type: Option<SubjectType>,
    ) -> Vec<Subject> {
        use crate::schema::subject::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::subject.into_boxed();
        match order.field {
            SubjectField::SubjectId => match order.direction {
                Direction::Asc => query = query.order(dsl::subject_id.asc()),
                Direction::Desc => query = query.order(dsl::subject_id.desc()),
            },
            SubjectField::WorkId => match order.direction {
                Direction::Asc => query = query.order(dsl::work_id.asc()),
                Direction::Desc => query = query.order(dsl::work_id.desc()),
            },
            SubjectField::SubjectType => match order.direction {
                Direction::Asc => query = query.order(dsl::subject_type.asc()),
                Direction::Desc => query = query.order(dsl::subject_type.desc()),
            },
            SubjectField::SubjectCode => match order.direction {
                Direction::Asc => query = query.order(dsl::subject_code.asc()),
                Direction::Desc => query = query.order(dsl::subject_code.desc()),
            },
            SubjectField::SubjectOrdinal => match order.direction {
                Direction::Asc => query = query.order(dsl::subject_ordinal.asc()),
                Direction::Desc => query = query.order(dsl::subject_ordinal.desc()),
            },
            SubjectField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::created_at.asc()),
                Direction::Desc => query = query.order(dsl::created_at.desc()),
            },
            SubjectField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::updated_at.asc()),
                Direction::Desc => query = query.order(dsl::updated_at.desc()),
            },
        }
        if let Some(sub_type) = subject_type {
            query = query.filter(dsl::subject_type.eq(sub_type))
        }
        query
            .filter(dsl::work_id.eq(self.work_id))
            .filter(dsl::subject_code.ilike(format!("%{}%", filter)))
            .then_order_by(dsl::subject_code.asc())
            .limit(limit.into())
            .offset(offset.into())
            .load::<Subject>(&connection)
            .expect("Error loading subjects")
    }

    #[graphql(
        description = "Get fundings linked to this work",
        arguments(
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
    pub fn fundings(&self, context: &Context, order: FundingOrderBy) -> Vec<Funding> {
        use crate::schema::funding::dsl::*;
        let connection = context.db.get().unwrap();
        let mut query = funding.into_boxed();
        match order.field {
            FundingField::FundingId => match order.direction {
                Direction::Asc => query = query.order(funding_id.asc()),
                Direction::Desc => query = query.order(funding_id.desc()),
            },
            FundingField::WorkId => match order.direction {
                Direction::Asc => query = query.order(work_id.asc()),
                Direction::Desc => query = query.order(work_id.desc()),
            },
            FundingField::FunderId => match order.direction {
                Direction::Asc => query = query.order(funder_id.asc()),
                Direction::Desc => query = query.order(funder_id.desc()),
            },
            FundingField::Program => match order.direction {
                Direction::Asc => query = query.order(program.asc()),
                Direction::Desc => query = query.order(program.desc()),
            },
            FundingField::ProjectName => match order.direction {
                Direction::Asc => query = query.order(project_name.asc()),
                Direction::Desc => query = query.order(project_name.desc()),
            },
            FundingField::ProjectShortname => match order.direction {
                Direction::Asc => query = query.order(project_shortname.asc()),
                Direction::Desc => query = query.order(project_shortname.desc()),
            },
            FundingField::GrantNumber => match order.direction {
                Direction::Asc => query = query.order(grant_number.asc()),
                Direction::Desc => query = query.order(grant_number.desc()),
            },
            FundingField::Jurisdiction => match order.direction {
                Direction::Asc => query = query.order(jurisdiction.asc()),
                Direction::Desc => query = query.order(jurisdiction.desc()),
            },
            FundingField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(created_at.asc()),
                Direction::Desc => query = query.order(created_at.desc()),
            },
            FundingField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(updated_at.asc()),
                Direction::Desc => query = query.order(updated_at.desc()),
            },
        }
        query
            .filter(work_id.eq(self.work_id))
            .load::<Funding>(&connection)
            .expect("Error loading fundings")
    }

    #[graphql(
        description = "Get issues linked to this work",
        arguments(
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
    pub fn issues(&self, context: &Context, order: IssueOrderBy) -> Vec<Issue> {
        use crate::schema::issue::dsl::*;
        let connection = context.db.get().unwrap();
        let mut query = issue.into_boxed();
        match order.field {
            IssueField::SeriesId => match order.direction {
                Direction::Asc => query = query.order(series_id.asc()),
                Direction::Desc => query = query.order(series_id.desc()),
            },
            IssueField::WorkId => match order.direction {
                Direction::Asc => query = query.order(work_id.asc()),
                Direction::Desc => query = query.order(work_id.desc()),
            },
            IssueField::IssueOrdinal => match order.direction {
                Direction::Asc => query = query.order(issue_ordinal.asc()),
                Direction::Desc => query = query.order(issue_ordinal.desc()),
            },
            IssueField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(created_at.asc()),
                Direction::Desc => query = query.order(created_at.desc()),
            },
            IssueField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(updated_at.asc()),
                Direction::Desc => query = query.order(updated_at.desc()),
            },
        }
        query
            .filter(work_id.eq(self.work_id))
            .load::<Issue>(&connection)
            .expect("Error loading issues")
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

    pub fn isbn(&self) -> Option<&String> {
        self.isbn.as_ref()
    }

    pub fn publication_url(&self) -> Option<&String> {
        self.publication_url.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    #[graphql(
        description = "Get prices linked to this publication",
        arguments(
            order(
                default = {
                    PriceOrderBy {
                        field: PriceField::CurrencyCode,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            currency_code(description = "A specific currency to filter by"),
        )
    )]
    pub fn prices(
        &self,
        context: &Context,
        order: PriceOrderBy,
        currency_code: Option<CurrencyCode>,
    ) -> Vec<Price> {
        use crate::schema::price::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::price.into_boxed();
        match order.field {
            PriceField::PriceId => match order.direction {
                Direction::Asc => query = query.order(dsl::price_id.asc()),
                Direction::Desc => query = query.order(dsl::price_id.desc()),
            },
            PriceField::PublicationId => match order.direction {
                Direction::Asc => query = query.order(dsl::publication_id.asc()),
                Direction::Desc => query = query.order(dsl::publication_id.desc()),
            },
            PriceField::CurrencyCode => match order.direction {
                Direction::Asc => query = query.order(dsl::currency_code.asc()),
                Direction::Desc => query = query.order(dsl::currency_code.desc()),
            },
            PriceField::UnitPrice => match order.direction {
                Direction::Asc => query = query.order(dsl::unit_price.asc()),
                Direction::Desc => query = query.order(dsl::unit_price.desc()),
            },
            PriceField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::created_at.asc()),
                Direction::Desc => query = query.order(dsl::created_at.desc()),
            },
            PriceField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::updated_at.asc()),
                Direction::Desc => query = query.order(dsl::updated_at.desc()),
            },
        }
        if let Some(curr_code) = currency_code {
            query = query.filter(dsl::currency_code.eq(curr_code))
        }
        query
            .filter(dsl::publication_id.eq(self.publication_id))
            .load::<Price>(&connection)
            .expect("Error loading price")
    }

    pub fn work(&self, context: &Context) -> Work {
        use crate::schema::work::dsl::*;
        let connection = context.db.get().unwrap();
        work.find(self.work_id)
            .first(&connection)
            .expect("Error loading work")
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

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    #[graphql(
        description = "Get imprints linked to this publisher",
        arguments(
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
        filter: String,
        order: ImprintOrderBy,
    ) -> Vec<Imprint> {
        use crate::schema::imprint::dsl::*;
        let connection = context.db.get().unwrap();
        let mut query = imprint.into_boxed();
        match order.field {
            ImprintField::ImprintId => match order.direction {
                Direction::Asc => query = query.order(imprint_id.asc()),
                Direction::Desc => query = query.order(imprint_id.desc()),
            },
            ImprintField::ImprintName => match order.direction {
                Direction::Asc => query = query.order(imprint_name.asc()),
                Direction::Desc => query = query.order(imprint_name.desc()),
            },
            ImprintField::ImprintUrl => match order.direction {
                Direction::Asc => query = query.order(imprint_url.asc()),
                Direction::Desc => query = query.order(imprint_url.desc()),
            },
            ImprintField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(created_at.asc()),
                Direction::Desc => query = query.order(created_at.desc()),
            },
            ImprintField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(updated_at.asc()),
                Direction::Desc => query = query.order(updated_at.desc()),
            },
        }
        query
            .filter(publisher_id.eq(self.publisher_id))
            .filter(
                imprint_name
                    .ilike(format!("%{}%", filter))
                    .or(imprint_url.ilike(format!("%{}%", filter))),
            )
            .load::<Imprint>(&connection)
            .expect("Error loading imprints")
    }
}

#[juniper::object(Context = Context, description = "The brand under which a publisher issues works.")]
impl Imprint {
    pub fn imprint_id(&self) -> Uuid {
        self.imprint_id
    }

    pub fn imprint_name(&self) -> &String {
        &self.imprint_name
    }

    pub fn imprint_url(&self) -> Option<&String> {
        self.imprint_url.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn publisher(&self, context: &Context) -> Publisher {
        use crate::schema::publisher::dsl::*;
        let connection = context.db.get().unwrap();
        publisher
            .find(self.publisher_id)
            .first(&connection)
            .expect("Error loading publisher")
    }

    #[graphql(
    description="Get works linked to this imprint",
    arguments(
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
        work_type(description = "A specific type to filter by"),
        work_status(description = "A specific status to filter by"),
    )
  )]
    pub fn works(
        context: &Context,
        filter: String,
        order: WorkOrderBy,
        work_type: Option<WorkType>,
        work_status: Option<WorkStatus>,
    ) -> Vec<Work> {
        use crate::schema::work::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::work.into_boxed();
        match order.field {
            WorkField::WorkId => match order.direction {
                Direction::Asc => query = query.order(dsl::work_id.asc()),
                Direction::Desc => query = query.order(dsl::work_id.desc()),
            },
            WorkField::WorkType => match order.direction {
                Direction::Asc => query = query.order(dsl::work_type.asc()),
                Direction::Desc => query = query.order(dsl::work_type.desc()),
            },
            WorkField::WorkStatus => match order.direction {
                Direction::Asc => query = query.order(dsl::work_status.asc()),
                Direction::Desc => query = query.order(dsl::work_status.desc()),
            },
            WorkField::FullTitle => match order.direction {
                Direction::Asc => query = query.order(dsl::full_title.asc()),
                Direction::Desc => query = query.order(dsl::full_title.desc()),
            },
            WorkField::Title => match order.direction {
                Direction::Asc => query = query.order(dsl::title.asc()),
                Direction::Desc => query = query.order(dsl::title.desc()),
            },
            WorkField::Subtitle => match order.direction {
                Direction::Asc => query = query.order(dsl::subtitle.asc()),
                Direction::Desc => query = query.order(dsl::subtitle.desc()),
            },
            WorkField::Reference => match order.direction {
                Direction::Asc => query = query.order(dsl::reference.asc()),
                Direction::Desc => query = query.order(dsl::reference.desc()),
            },
            WorkField::Edition => match order.direction {
                Direction::Asc => query = query.order(dsl::edition.asc()),
                Direction::Desc => query = query.order(dsl::edition.desc()),
            },
            WorkField::Doi => match order.direction {
                Direction::Asc => query = query.order(dsl::doi.asc()),
                Direction::Desc => query = query.order(dsl::doi.desc()),
            },
            WorkField::PublicationDate => match order.direction {
                Direction::Asc => query = query.order(dsl::publication_date.asc()),
                Direction::Desc => query = query.order(dsl::publication_date.desc()),
            },
            WorkField::Place => match order.direction {
                Direction::Asc => query = query.order(dsl::place.asc()),
                Direction::Desc => query = query.order(dsl::place.desc()),
            },
            WorkField::Width => match order.direction {
                Direction::Asc => query = query.order(dsl::width.asc()),
                Direction::Desc => query = query.order(dsl::width.desc()),
            },
            WorkField::Height => match order.direction {
                Direction::Asc => query = query.order(dsl::height.asc()),
                Direction::Desc => query = query.order(dsl::height.desc()),
            },
            WorkField::PageCount => match order.direction {
                Direction::Asc => query = query.order(dsl::page_count.asc()),
                Direction::Desc => query = query.order(dsl::page_count.desc()),
            },
            WorkField::PageBreakdown => match order.direction {
                Direction::Asc => query = query.order(dsl::page_breakdown.asc()),
                Direction::Desc => query = query.order(dsl::page_breakdown.desc()),
            },
            WorkField::ImageCount => match order.direction {
                Direction::Asc => query = query.order(dsl::image_count.asc()),
                Direction::Desc => query = query.order(dsl::image_count.desc()),
            },
            WorkField::TableCount => match order.direction {
                Direction::Asc => query = query.order(dsl::table_count.asc()),
                Direction::Desc => query = query.order(dsl::table_count.desc()),
            },
            WorkField::AudioCount => match order.direction {
                Direction::Asc => query = query.order(dsl::audio_count.asc()),
                Direction::Desc => query = query.order(dsl::audio_count.desc()),
            },
            WorkField::VideoCount => match order.direction {
                Direction::Asc => query = query.order(dsl::video_count.asc()),
                Direction::Desc => query = query.order(dsl::video_count.desc()),
            },
            WorkField::License => match order.direction {
                Direction::Asc => query = query.order(dsl::license.asc()),
                Direction::Desc => query = query.order(dsl::license.desc()),
            },
            WorkField::CopyrightHolder => match order.direction {
                Direction::Asc => query = query.order(dsl::copyright_holder.asc()),
                Direction::Desc => query = query.order(dsl::copyright_holder.desc()),
            },
            WorkField::LandingPage => match order.direction {
                Direction::Asc => query = query.order(dsl::landing_page.asc()),
                Direction::Desc => query = query.order(dsl::landing_page.desc()),
            },
            WorkField::Lccn => match order.direction {
                Direction::Asc => query = query.order(dsl::lccn.asc()),
                Direction::Desc => query = query.order(dsl::lccn.desc()),
            },
            WorkField::Oclc => match order.direction {
                Direction::Asc => query = query.order(dsl::oclc.asc()),
                Direction::Desc => query = query.order(dsl::oclc.desc()),
            },
            WorkField::ShortAbstract => match order.direction {
                Direction::Asc => query = query.order(dsl::short_abstract.asc()),
                Direction::Desc => query = query.order(dsl::short_abstract.desc()),
            },
            WorkField::LongAbstract => match order.direction {
                Direction::Asc => query = query.order(dsl::long_abstract.asc()),
                Direction::Desc => query = query.order(dsl::long_abstract.desc()),
            },
            WorkField::GeneralNote => match order.direction {
                Direction::Asc => query = query.order(dsl::general_note.asc()),
                Direction::Desc => query = query.order(dsl::general_note.desc()),
            },
            WorkField::Toc => match order.direction {
                Direction::Asc => query = query.order(dsl::toc.asc()),
                Direction::Desc => query = query.order(dsl::toc.desc()),
            },
            WorkField::CoverUrl => match order.direction {
                Direction::Asc => query = query.order(dsl::cover_url.asc()),
                Direction::Desc => query = query.order(dsl::cover_url.desc()),
            },
            WorkField::CoverCaption => match order.direction {
                Direction::Asc => query = query.order(dsl::cover_caption.asc()),
                Direction::Desc => query = query.order(dsl::cover_caption.desc()),
            },
            WorkField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::created_at.asc()),
                Direction::Desc => query = query.order(dsl::created_at.desc()),
            },
            WorkField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::updated_at.asc()),
                Direction::Desc => query = query.order(dsl::updated_at.desc()),
            },
        }
        if let Some(wk_type) = work_type {
            query = query.filter(dsl::work_type.eq(wk_type))
        }
        if let Some(wk_status) = work_status {
            query = query.filter(dsl::work_status.eq(wk_status))
        }
        query
            .filter(dsl::imprint_id.eq(self.imprint_id))
            .filter(
                dsl::full_title
                    .ilike(format!("%{}%", filter))
                    .or(dsl::doi.ilike(format!("%{}%", filter)))
                    .or(dsl::reference.ilike(format!("%{}%", filter)))
                    .or(dsl::short_abstract.ilike(format!("%{}%", filter)))
                    .or(dsl::long_abstract.ilike(format!("%{}%", filter)))
                    .or(dsl::landing_page.ilike(format!("%{}%", filter))),
            )
            .load::<Work>(&connection)
            .expect("Error loading works")
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

    pub fn orcid(&self) -> Option<&String> {
        self.orcid.as_ref()
    }

    pub fn website(&self) -> Option<&String> {
        self.website.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    #[graphql(
        description = "Get contributions linked to this contributor",
        arguments(
            order(
                default = {
                    ContributionOrderBy {
                        field: ContributionField::ContributionType,
                        direction: Direction::Asc,
                    }
                },
                description = "The order in which to sort the results",
            ),
            contribution_type(description = "A specific type to filter by"),
        )
    )]
    pub fn contributions(
        &self,
        context: &Context,
        order: ContributionOrderBy,
        contribution_type: Option<ContributionType>,
    ) -> Vec<Contribution> {
        use crate::schema::contribution::dsl;
        let connection = context.db.get().unwrap();
        let mut query = dsl::contribution.into_boxed();
        match order.field {
            ContributionField::WorkId => match order.direction {
                Direction::Asc => query = query.order(dsl::work_id.asc()),
                Direction::Desc => query = query.order(dsl::work_id.desc()),
            },
            ContributionField::ContributorId => match order.direction {
                Direction::Asc => query = query.order(dsl::contributor_id.asc()),
                Direction::Desc => query = query.order(dsl::contributor_id.desc()),
            },
            ContributionField::ContributionType => match order.direction {
                Direction::Asc => query = query.order(dsl::contribution_type.asc()),
                Direction::Desc => query = query.order(dsl::contribution_type.desc()),
            },
            ContributionField::MainContribution => match order.direction {
                Direction::Asc => query = query.order(dsl::main_contribution.asc()),
                Direction::Desc => query = query.order(dsl::main_contribution.desc()),
            },
            ContributionField::Biography => match order.direction {
                Direction::Asc => query = query.order(dsl::biography.asc()),
                Direction::Desc => query = query.order(dsl::biography.desc()),
            },
            ContributionField::Institution => match order.direction {
                Direction::Asc => query = query.order(dsl::institution.asc()),
                Direction::Desc => query = query.order(dsl::institution.desc()),
            },
            ContributionField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::created_at.asc()),
                Direction::Desc => query = query.order(dsl::created_at.desc()),
            },
            ContributionField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::updated_at.asc()),
                Direction::Desc => query = query.order(dsl::updated_at.desc()),
            },
            ContributionField::FirstName => match order.direction {
                Direction::Asc => query = query.order(dsl::first_name.asc()),
                Direction::Desc => query = query.order(dsl::first_name.desc()),
            },
            ContributionField::LastName => match order.direction {
                Direction::Asc => query = query.order(dsl::last_name.asc()),
                Direction::Desc => query = query.order(dsl::last_name.desc()),
            },
            ContributionField::FullName => match order.direction {
                Direction::Asc => query = query.order(dsl::full_name.asc()),
                Direction::Desc => query = query.order(dsl::full_name.desc()),
            },
        }
        if let Some(cont_type) = contribution_type {
            query = query.filter(dsl::contribution_type.eq(cont_type))
        }
        query
            .filter(dsl::contributor_id.eq(self.contributor_id))
            .load::<Contribution>(&connection)
            .expect("Error loading contributions")
    }
}

#[juniper::object(Context = Context, description = "A person's involvement in the production of a written text.")]
impl Contribution {
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

    pub fn institution(&self) -> Option<&String> {
        self.institution.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
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

    pub fn work(&self, context: &Context) -> Work {
        use crate::schema::work::dsl::*;
        let connection = context.db.get().unwrap();
        work.find(self.work_id)
            .first(&connection)
            .expect("Error loading work")
    }

    pub fn contributor(&self, context: &Context) -> Contributor {
        use crate::schema::contributor::dsl::*;
        let connection = context.db.get().unwrap();
        contributor
            .find(self.contributor_id)
            .first(&connection)
            .expect("Error loading contributions")
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

    pub fn series_url(&self) -> Option<&String> {
        self.series_url.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn imprint(&self, context: &Context) -> Imprint {
        use crate::schema::imprint::dsl::*;
        let connection = context.db.get().unwrap();
        imprint
            .find(self.imprint_id)
            .first(&connection)
            .expect("Error loading imprint")
    }

    #[graphql(
        description = "Get issues linked to this series",
        arguments(
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
    pub fn issues(&self, context: &Context, order: IssueOrderBy) -> Vec<Issue> {
        use crate::schema::issue::dsl::*;
        let connection = context.db.get().unwrap();
        let mut query = issue.into_boxed();
        match order.field {
            IssueField::SeriesId => match order.direction {
                Direction::Asc => query = query.order(series_id.asc()),
                Direction::Desc => query = query.order(series_id.desc()),
            },
            IssueField::WorkId => match order.direction {
                Direction::Asc => query = query.order(work_id.asc()),
                Direction::Desc => query = query.order(work_id.desc()),
            },
            IssueField::IssueOrdinal => match order.direction {
                Direction::Asc => query = query.order(issue_ordinal.asc()),
                Direction::Desc => query = query.order(issue_ordinal.desc()),
            },
            IssueField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(created_at.asc()),
                Direction::Desc => query = query.order(created_at.desc()),
            },
            IssueField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(updated_at.asc()),
                Direction::Desc => query = query.order(updated_at.desc()),
            },
        }
        query
            .filter(series_id.eq(self.series_id))
            .load::<Issue>(&connection)
            .expect("Error loading issues")
    }
}

#[juniper::object(Context = Context, description = "A work published as a number in a periodical.")]
impl Issue {
    pub fn work_id(&self) -> Uuid {
        self.work_id
    }

    pub fn series_id(&self) -> Uuid {
        self.series_id
    }

    pub fn issue_ordinal(&self) -> &i32 {
        &self.issue_ordinal
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn series(&self, context: &Context) -> Series {
        use crate::schema::series::dsl::*;
        let connection = context.db.get().unwrap();
        series
            .find(self.series_id)
            .first(&connection)
            .expect("Error loading series")
    }

    pub fn work(&self, context: &Context) -> Work {
        use crate::schema::work::dsl::*;
        let connection = context.db.get().unwrap();
        work.find(self.work_id)
            .first(&connection)
            .expect("Error loading work")
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

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn work(&self, context: &Context) -> Work {
        use crate::schema::work::dsl::*;
        let connection = context.db.get().unwrap();
        work.find(self.work_id)
            .first(&connection)
            .expect("Error loading work")
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

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn publication(&self, context: &Context) -> Publication {
        use crate::schema::publication::dsl::*;
        let connection = context.db.get().unwrap();
        publication
            .find(self.publication_id)
            .first(&connection)
            .expect("Error loading publication")
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

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn work(&self, context: &Context) -> Work {
        use crate::schema::work::dsl::*;
        let connection = context.db.get().unwrap();
        work.find(self.work_id)
            .first(&connection)
            .expect("Error loading work")
    }
}

#[juniper::object(Context = Context, description = "An organisation that provides the money to pay for the publication of a work.")]
impl Funder {
    pub fn funder_id(&self) -> &Uuid {
        &self.funder_id
    }

    pub fn funder_name(&self) -> &String {
        &self.funder_name
    }

    pub fn funder_doi(&self) -> Option<&String> {
        self.funder_doi.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    #[graphql(
        description = "Get fundings linked to this funder",
        arguments(
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
    pub fn fundings(&self, context: &Context, order: FundingOrderBy) -> Vec<Funding> {
        use crate::schema::funding::dsl::*;
        let connection = context.db.get().unwrap();
        let mut query = funding.into_boxed();
        match order.field {
            FundingField::FundingId => match order.direction {
                Direction::Asc => query = query.order(funding_id.asc()),
                Direction::Desc => query = query.order(funding_id.desc()),
            },
            FundingField::WorkId => match order.direction {
                Direction::Asc => query = query.order(work_id.asc()),
                Direction::Desc => query = query.order(work_id.desc()),
            },
            FundingField::FunderId => match order.direction {
                Direction::Asc => query = query.order(funder_id.asc()),
                Direction::Desc => query = query.order(funder_id.desc()),
            },
            FundingField::Program => match order.direction {
                Direction::Asc => query = query.order(program.asc()),
                Direction::Desc => query = query.order(program.desc()),
            },
            FundingField::ProjectName => match order.direction {
                Direction::Asc => query = query.order(project_name.asc()),
                Direction::Desc => query = query.order(project_name.desc()),
            },
            FundingField::ProjectShortname => match order.direction {
                Direction::Asc => query = query.order(project_shortname.asc()),
                Direction::Desc => query = query.order(project_shortname.desc()),
            },
            FundingField::GrantNumber => match order.direction {
                Direction::Asc => query = query.order(grant_number.asc()),
                Direction::Desc => query = query.order(grant_number.desc()),
            },
            FundingField::Jurisdiction => match order.direction {
                Direction::Asc => query = query.order(jurisdiction.asc()),
                Direction::Desc => query = query.order(jurisdiction.desc()),
            },
            FundingField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(created_at.asc()),
                Direction::Desc => query = query.order(created_at.desc()),
            },
            FundingField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(updated_at.asc()),
                Direction::Desc => query = query.order(updated_at.desc()),
            },
        }
        query
            .filter(funder_id.eq(self.funder_id))
            .load::<Funding>(&connection)
            .expect("Error loading fundings")
    }
}

#[juniper::object(Context = Context, description = "A grant awarded to the publication of a work by a funder.")]
impl Funding {
    pub fn funding_id(&self) -> &Uuid {
        &self.funding_id
    }

    pub fn work_id(&self) -> &Uuid {
        &self.work_id
    }

    pub fn funder_id(&self) -> &Uuid {
        &self.funder_id
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

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn work(&self, context: &Context) -> Work {
        use crate::schema::work::dsl::*;
        let connection = context.db.get().unwrap();
        work.find(self.work_id)
            .first(&connection)
            .expect("Error loading work")
    }

    pub fn funder(&self, context: &Context) -> Funder {
        use crate::schema::funder::dsl::*;
        let connection = context.db.get().unwrap();
        funder
            .find(self.funder_id)
            .first(&connection)
            .expect("Error loading funder")
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}

fn user_can_edit_imprint(imprint_id: Uuid, context: &Context) -> ThothResult<()> {
    use crate::schema::imprint::dsl;
    let pub_id = dsl::imprint
        .select(dsl::publisher_id)
        .filter(dsl::imprint_id.eq(imprint_id))
        .first::<Uuid>(&context.db.get().unwrap())
        .expect("Error checking permissions");
    context.account_access.can_edit(pub_id)
}

fn user_can_edit_work(work_id: Uuid, context: &Context) -> ThothResult<()> {
    use crate::schema::imprint::dsl::*;
    let pub_id = imprint
        .inner_join(crate::schema::work::table)
        .select(publisher_id)
        .filter(crate::schema::work::work_id.eq(work_id))
        .first::<Uuid>(&context.db.get().unwrap())
        .expect("Error checking permissions");
    context.account_access.can_edit(pub_id)
}

fn user_can_edit_publication(publication_id: Uuid, context: &Context) -> ThothResult<()> {
    use crate::schema::imprint::dsl::*;
    let pub_id = imprint
        .inner_join(crate::schema::work::table.inner_join(crate::schema::publication::table))
        .select(publisher_id)
        .filter(crate::schema::publication::publication_id.eq(publication_id))
        .first::<Uuid>(&context.db.get().unwrap())
        .expect("Error checking permissions");
    context.account_access.can_edit(pub_id)
}

fn issue_imprints_match(work_id: Uuid, series_id: Uuid, context: &Context) -> ThothResult<()> {
    let series_imprint = crate::schema::series::table
        .select(crate::schema::series::imprint_id)
        .filter(crate::schema::series::series_id.eq(series_id))
        .first::<Uuid>(&context.db.get().unwrap())
        .expect("Error loading series for issue");
    let work_imprint = crate::schema::work::table
        .select(crate::schema::work::imprint_id)
        .filter(crate::schema::work::work_id.eq(work_id))
        .first::<Uuid>(&context.db.get().unwrap())
        .expect("Error loading work for issue");
    if work_imprint == series_imprint {
        Ok(())
    } else {
        Err(ThothError::IssueImprintsError)
    }
}

fn can_update_work_imprint(work_id: Uuid, context: &Context) -> ThothResult<()> {
    use crate::schema::issue::dsl;
    // see comment in work_count()
    let issue_count = dsl::issue
        .filter(dsl::work_id.eq(work_id))
        .count()
        .get_result::<i64>(&context.db.get().unwrap())
        .expect("Error loading issue count for work")
        .to_string()
        .parse::<i32>()
        .unwrap();
    // If a work has any related issues, its imprint cannot be changed,
    // because an issue's series and work must both have the same imprint.
    if issue_count == 0 {
        Ok(())
    } else {
        Err(ThothError::IssueImprintsError)
    }
}
