use chrono::naive::NaiveDate;
use diesel::prelude::*;
use juniper::FieldError;
use juniper::FieldResult;
use juniper::RootNode;
use uuid::Uuid;

use crate::db::PgPool;
use crate::models::contributor::*;
use crate::models::funder::*;
use crate::models::language::*;
use crate::models::price::*;
use crate::models::publication::*;
use crate::models::publisher::*;
use crate::models::series::*;
use crate::models::subject::*;
use crate::models::work::*;
use crate::schema::*;

#[derive(Clone)]
pub struct Context {
    pub db: PgPool,
}

impl juniper::Context for Context {}

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
    )
  )]
    fn works(context: &Context, limit: i32, offset: i32, filter: String) -> Vec<Work> {
        use crate::schema::work::dsl::*;
        let connection = context.db.get().unwrap();
        work.filter(full_title.ilike(format!("%{}%", filter)))
            .or_filter(doi.ilike(format!("%{}%", filter)))
            .or_filter(reference.ilike(format!("%{}%", filter)))
            .or_filter(short_abstract.ilike(format!("%{}%", filter)))
            .or_filter(long_abstract.ilike(format!("%{}%", filter)))
            .or_filter(landing_page.ilike(format!("%{}%", filter)))
            .order(full_title.asc())
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

    #[graphql(
        description = "Query the full list of publications",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip")
        )
    )]
    fn publications(context: &Context, limit: i32, offset: i32) -> Vec<Publication> {
        use crate::schema::publication::dsl::*;
        let connection = context.db.get().unwrap();
        publication
            .order(publication_type.asc())
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
    )
  )]
    fn publishers(context: &Context, limit: i32, offset: i32, filter: String) -> Vec<Publisher> {
        use crate::schema::publisher::dsl::*;
        let connection = context.db.get().unwrap();
        publisher
            .filter(publisher_name.ilike(format!("%{}%", filter)))
            .or_filter(publisher_shortname.ilike(format!("%{}%", filter)))
            .order(publisher_name.asc())
            .limit(limit.into())
            .offset(offset.into())
            .load::<Publisher>(&connection)
            .expect("Error loading publishers")
    }

    #[graphql(description = "Query a publication work using its id")]
    fn publication(context: &Context, publisher_id: Uuid) -> FieldResult<Publisher> {
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
        description = "Query the full list of imprints",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip")
        )
    )]
    fn imprints(context: &Context, limit: i32, offset: i32) -> Vec<Imprint> {
        use crate::schema::imprint::dsl::*;
        let connection = context.db.get().unwrap();
        imprint
            .order(imprint_name.asc())
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
        description = "Query the full list of contributors",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip"),
            filter(
                default = "".to_string(),
                description = "A query string to search. This argument is a test, do not rely on it. At present it simply searches for case insensitive literals on full_name and orcid"
            ),
        )
    )]
    fn contributors(context: &Context, limit: i32, offset: i32, filter: String) -> Vec<Contributor> {
        use crate::schema::contributor::dsl::*;
        let connection = context.db.get().unwrap();
        contributor.filter(full_name.ilike(format!("%{}%", filter)))
            .or_filter(orcid.ilike(format!("%{}%", filter)))
            .order(full_name.asc())
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
        description = "Query the full list of contributions",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip")
        )
    )]
    fn contributions(context: &Context, limit: i32, offset: i32) -> Vec<Contribution> {
        use crate::schema::contribution::dsl::*;
        let connection = context.db.get().unwrap();
        contribution
            .order(contribution_type.asc())
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

    #[graphql(
        description = "Query the full list of series",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip")
        )
    )]
    fn serieses(context: &Context, limit: i32, offset: i32) -> Vec<Series> {
        use crate::schema::series::dsl::*;
        let connection = context.db.get().unwrap();
        series
            .order(series_name.asc())
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
        description = "Query the full list of issues",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip")
        )
    )]
    fn issues(context: &Context, limit: i32, offset: i32) -> Vec<Issue> {
        use crate::schema::issue::dsl::*;
        let connection = context.db.get().unwrap();
        issue
            .order(issue_ordinal.asc())
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

    #[graphql(
        description = "Query the full list of languages",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip")
        )
    )]
    fn languages(context: &Context, limit: i32, offset: i32) -> Vec<Language> {
        use crate::schema::language::dsl::*;
        let connection = context.db.get().unwrap();
        language
            .order(language_code.asc())
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

    #[graphql(
        description = "Query the full list of prices",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip")
        )
    )]
    fn prices(context: &Context, limit: i32, offset: i32) -> Vec<Price> {
        use crate::schema::price::dsl::*;
        let connection = context.db.get().unwrap();
        price
            .order(currency_code.asc())
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

    #[graphql(
        description = "Query the full list of subjects",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip")
        )
    )]
    fn subjects(context: &Context, limit: i32, offset: i32) -> Vec<Subject> {
        use crate::schema::subject::dsl::*;
        let connection = context.db.get().unwrap();
        subject
            .order(subject_type.asc())
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

    #[graphql(
        description = "Query the full list of funders",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip")
        )
    )]
    fn funders(context: &Context, limit: i32, offset: i32) -> Vec<Funder> {
        use crate::schema::funder::dsl::*;
        let connection = context.db.get().unwrap();
        funder
            .order(funder_name.asc())
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
        description = "Query the full list of fundings",
        arguments(
            limit(default = 100, description = "The number of items to return"),
            offset(default = 0, description = "The number of items to skip")
        )
    )]
    fn fundings(context: &Context, limit: i32, offset: i32) -> Vec<Funding> {
        use crate::schema::funding::dsl::*;
        let connection = context.db.get().unwrap();
        funding
            .order(program.asc())
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
}

pub struct MutationRoot;

#[juniper::object(Context = Context)]
impl MutationRoot {
    fn create_work(context: &Context, data: NewWork) -> FieldResult<Work> {
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
        let connection = context.db.get().unwrap();
        match diesel::insert_into(imprint::table)
            .values(&data)
            .get_result(&connection)
        {
            Ok(imprint) => Ok(imprint),
            Err(e) => Err(FieldError::from(e)),
        }
    }

    fn create_contributor(context: &Context, data: NewContributor) -> FieldResult<Contributor> {
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

    pub fn lccn(&self) -> Option<&i32> {
        self.lccn.as_ref()
    }

    pub fn oclc(&self) -> Option<&i32> {
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

    pub fn imprint(&self, context: &Context) -> Imprint {
        use crate::schema::imprint::dsl::*;
        let connection = context.db.get().unwrap();
        imprint
            .find(self.imprint_id)
            .first(&connection)
            .expect("Error loading imprint")
    }

    pub fn contributions(&self, context: &Context) -> Vec<Contribution> {
        use crate::schema::contribution::dsl::*;
        let connection = context.db.get().unwrap();
        contribution
            .filter(work_id.eq(self.work_id))
            .load::<Contribution>(&connection)
            .expect("Error loading contributions")
    }

    pub fn languages(&self, context: &Context) -> Vec<Language> {
        use crate::schema::language::dsl::*;
        let connection = context.db.get().unwrap();
        language
            .filter(work_id.eq(self.work_id))
            .load::<Language>(&connection)
            .expect("Error loading languages")
    }

    pub fn publications(&self, context: &Context) -> Vec<Publication> {
        use crate::schema::publication::dsl::*;
        let connection = context.db.get().unwrap();
        publication
            .filter(work_id.eq(self.work_id))
            .load::<Publication>(&connection)
            .expect("Error loading publications")
    }

    pub fn subjects(&self, context: &Context) -> Vec<Subject> {
        use crate::schema::subject::dsl::*;
        let connection = context.db.get().unwrap();
        subject
            .filter(work_id.eq(self.work_id))
            .load::<Subject>(&connection)
            .expect("Error loading subjects")
    }

    pub fn issues(&self, context: &Context) -> Vec<Issue> {
        use crate::schema::issue::dsl::*;
        let connection = context.db.get().unwrap();
        issue
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

    pub fn isbn(&self) -> Option<&String> {
        self.isbn.as_ref()
    }

    pub fn publication_url(&self) -> Option<&String> {
        self.publication_url.as_ref()
    }

    pub fn prices(&self, context: &Context) -> Vec<Price> {
        use crate::schema::price::dsl::*;
        let connection = context.db.get().unwrap();
        price
            .filter(publication_id.eq(self.publication_id))
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

    pub fn imprints(&self, context: &Context) -> Vec<Imprint> {
        use crate::schema::imprint::dsl::*;
        let connection = context.db.get().unwrap();
        imprint
            .filter(publisher_id.eq(self.publisher_id))
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

    pub fn publisher(&self, context: &Context) -> Publisher {
        use crate::schema::publisher::dsl::*;
        let connection = context.db.get().unwrap();
        publisher
            .find(self.publisher_id)
            .first(&connection)
            .expect("Error loading publisher")
    }

    pub fn works(&self, context: &Context) -> Vec<Work> {
        use crate::schema::work::dsl::*;
        let connection = context.db.get().unwrap();
        work.filter(imprint_id.eq(self.imprint_id))
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

    pub fn contributions(&self, context: &Context) -> Vec<Contribution> {
        use crate::schema::contribution::dsl::*;
        let connection = context.db.get().unwrap();
        contribution
            .filter(contributor_id.eq(self.contributor_id))
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

    pub fn imprint(&self, context: &Context) -> Imprint {
        use crate::schema::imprint::dsl::*;
        let connection = context.db.get().unwrap();
        imprint
            .find(self.imprint_id)
            .first(&connection)
            .expect("Error loading imprint")
    }

    pub fn issues(&self, context: &Context) -> Vec<Issue> {
        use crate::schema::issue::dsl::*;
        let connection = context.db.get().unwrap();
        issue
            .filter(series_id.eq(self.series_id))
            .load::<Issue>(&connection)
            .expect("Error loading issues")
    }
}

#[juniper::object(Context = Context, description = "A work published as a number in a periodical.")]
impl Issue {
    pub fn issue_ordinal(&self) -> &i32 {
        &self.issue_ordinal
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

    pub fn language_code(&self) -> &LanguageCode {
        &self.language_code
    }

    pub fn language_relation(&self) -> &LanguageRelation {
        &self.language_relation
    }

    pub fn main_language(&self) -> bool {
        self.main_language
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

    pub fn currency_code(&self) -> &CurrencyCode {
        &self.currency_code
    }

    pub fn unit_price(&self) -> f64 {
        self.unit_price
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

    pub fn subject_type(&self) -> &SubjectType {
        &self.subject_type
    }

    pub fn subject_code(&self) -> &String {
        &self.subject_code
    }

    pub fn subject_ordinal(&self) -> &i32 {
        &self.subject_ordinal
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

    pub fn fundings(&self, context: &Context) -> Vec<Funding> {
        use crate::schema::funding::dsl::*;
        let connection = context.db.get().unwrap();
        funding
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
