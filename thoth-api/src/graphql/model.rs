use std::sync::Arc;

use chrono::naive::NaiveDate;
use juniper::{FieldError, FieldResult};
use uuid::Uuid;
use zitadel::actix::introspection::IntrospectedUser;

use super::types::inputs::{
    ContributionOrderBy, Convert, Direction, FundingOrderBy, IssueOrderBy, LanguageOrderBy,
    LengthUnit, PriceOrderBy, SubjectOrderBy, TimeExpression, WeightUnit,
};
use crate::db::PgPool;
use crate::markup::{convert_from_jats, ConversionLimit, MarkupFormat};
use crate::model::{
    additional_resource::{AdditionalResource, AdditionalResourceOrderBy},
    affiliation::{Affiliation, AffiliationOrderBy},
    award::{Award, AwardOrderBy, AwardRole},
    biography::{Biography, BiographyOrderBy},
    book_review::{BookReview, BookReviewOrderBy},
    contact::{Contact, ContactOrderBy, ContactType},
    contribution::{Contribution, ContributionType},
    contributor::Contributor,
    endorsement::{Endorsement, EndorsementOrderBy},
    file::{File, FileType},
    funding::Funding,
    imprint::{Imprint, ImprintField, ImprintOrderBy},
    institution::{CountryCode, Institution},
    issue::Issue,
    language::{Language, LanguageCode, LanguageRelation},
    locale::LocaleCode,
    location::{Location, LocationOrderBy, LocationPlatform},
    price::{CurrencyCode, Price},
    publication::{
        AccessibilityException, AccessibilityStandard, Publication, PublicationOrderBy,
        PublicationType,
    },
    publisher::Publisher,
    r#abstract::{Abstract, AbstractOrderBy, AbstractType},
    reference::{Reference, ReferenceOrderBy},
    series::{Series, SeriesType},
    subject::{Subject, SubjectType},
    title::{Title, TitleOrderBy},
    work::{Work, WorkOrderBy, WorkStatus, WorkType},
    work_featured_video::WorkFeaturedVideo,
    work_relation::{RelationType, WorkRelation, WorkRelationOrderBy},
    Crud, Doi, Isbn, Orcid, Ror, Timestamp,
};
use crate::policy::PolicyContext;
use crate::storage::{CloudFrontClient, S3Client};
use thoth_errors::ThothError;

impl juniper::Context for Context {}

pub struct Context {
    pub db: Arc<PgPool>,
    pub user: Option<IntrospectedUser>,
    pub s3_client: Arc<S3Client>,
    pub cloudfront_client: Arc<CloudFrontClient>,
}

impl Context {
    pub fn new(
        pool: Arc<PgPool>,
        user: Option<IntrospectedUser>,
        s3_client: Arc<S3Client>,
        cloudfront_client: Arc<CloudFrontClient>,
    ) -> Self {
        Self {
            db: pool,
            user,
            s3_client,
            cloudfront_client,
        }
    }

    pub fn s3_client(&self) -> &S3Client {
        self.s3_client.as_ref()
    }

    pub fn cloudfront_client(&self) -> &CloudFrontClient {
        self.cloudfront_client.as_ref()
    }
}

impl PolicyContext for Context {
    fn db(&self) -> &PgPool {
        &self.db
    }
    fn user(&self) -> Option<&IntrospectedUser> {
        self.user.as_ref()
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
        Ok(Title::canonical_from_work_id(&ctx.db, &self.work_id)?.full_title)
    }

    #[graphql(description = "Main title of the work (excluding subtitle)")]
    #[graphql(
        deprecated = "Please use Work `titles` field instead to get the correct title in a multilingual manner"
    )]
    pub fn title(&self, ctx: &Context) -> FieldResult<String> {
        Ok(Title::canonical_from_work_id(&ctx.db, &self.work_id)?.title)
    }

    #[graphql(description = "Secondary title of the work (excluding main title)")]
    #[graphql(
        deprecated = "Please use Work `titles` field instead to get the correct sub_title in a multilingual manner"
    )]
    pub fn subtitle(&self, ctx: &Context) -> FieldResult<Option<String>> {
        Ok(Title::canonical_from_work_id(&ctx.db, &self.work_id)?.subtitle)
    }

    #[graphql(
        description = "Short abstract of the work. Where a work has two different versions of the abstract, the truncated version should be entered here. Otherwise, it can be left blank. This field is not output in metadata formats; where relevant, Long Abstract is used instead."
    )]
    #[graphql(
        deprecated = "Please use Work `abstracts` field instead to get the correct short abstract in a multilingual manner"
    )]
    pub fn short_abstract(&self, ctx: &Context) -> FieldResult<Option<String>> {
        Ok(
            Abstract::short_canonical_from_work_id(&ctx.db, &self.work_id)
                .map(|a| a.content)
                .ok(),
        )
    }

    #[graphql(
        description = "Abstract of the work. Where a work has only one abstract, it should be entered here, and Short Abstract can be left blank. Long Abstract is output in metadata formats, and Short Abstract is not."
    )]
    #[graphql(
        deprecated = "Please use Work `abstracts` field instead to get the correct long abstract in a multilingual manner"
    )]
    pub fn long_abstract(&self, ctx: &Context) -> FieldResult<Option<String>> {
        Ok(
            Abstract::long_canonical_from_work_id(&ctx.db, &self.work_id)
                .map(|a| a.content)
                .ok(),
        )
    }

    #[allow(clippy::too_many_arguments)]
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
        #[graphql(
            default = MarkupFormat::JatsXml,
            description = "If set, only shows results with this markup format"
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
            Some(self.work_id),
            None,
            locale_codes.unwrap_or_default(),
            vec![],
            None,
            None,
        )
        .map_err(FieldError::from)?;

        let markup = markup_format.ok_or(ThothError::MissingMarkupFormat)?;
        for title in titles.iter_mut() {
            title.title = convert_from_jats(&title.title, markup, ConversionLimit::Title)?;
            title.subtitle = title
                .subtitle
                .as_ref()
                .map(|subtitle| convert_from_jats(subtitle, markup, ConversionLimit::Title))
                .transpose()?;
            title.full_title =
                convert_from_jats(&title.full_title, markup, ConversionLimit::Title)?;
        }

        Ok(titles)
    }

    #[allow(clippy::too_many_arguments)]
    #[graphql(description = "Query abstracts by work ID")]
    fn abstracts(
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
            default = AbstractOrderBy::default(),
            description = "The order in which to sort the results"
        )]
        order: Option<AbstractOrderBy>,
        #[graphql(
            default = vec![],
            description = "If set, only shows results with these locale codes"
        )]
        locale_codes: Option<Vec<LocaleCode>>,
        #[graphql(
            default = MarkupFormat::JatsXml,
            description = "If set, only shows results with this markup format"
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
            Some(*self.work_id()),
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

    #[graphql(description = "Description of additional resources linked to this work")]
    pub fn resources_description(
        &self,
        #[graphql(
            default = MarkupFormat::JatsXml,
            description = "Markup format used for rendering resources description",
        )]
        markup_format: Option<MarkupFormat>,
    ) -> FieldResult<Option<String>> {
        self.resources_description
            .as_ref()
            .map(|value| {
                convert_from_jats(
                    value,
                    markup_format.ok_or(ThothError::MissingMarkupFormat)?,
                    ConversionLimit::Abstract,
                )
            })
            .transpose()
            .map_err(Into::into)
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
        Imprint::from_id(&context.db, &self.imprint_id).map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
    }
    #[graphql(description = "Get the front cover file for this work")]
    pub fn frontcover(&self, context: &Context) -> FieldResult<Option<File>> {
        File::from_work_id(&context.db, &self.work_id).map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Get additional resources linked to this work")]
    pub fn additional_resources(
        &self,
        context: &Context,
        #[graphql(default = 50, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
        #[graphql(
            default = MarkupFormat::JatsXml,
            description = "Markup format used for rendering textual fields"
        )]
        markup_format: Option<MarkupFormat>,
    ) -> FieldResult<Vec<AdditionalResource>> {
        let mut additional_resources = AdditionalResource::all(
            &context.db,
            limit.unwrap_or(50),
            offset.unwrap_or_default(),
            None,
            AdditionalResourceOrderBy::default(),
            vec![],
            Some(self.work_id),
            None,
            vec![],
            vec![],
            None,
            None,
        )?;

        let markup = markup_format.ok_or(ThothError::MissingMarkupFormat)?;
        for additional_resource in &mut additional_resources {
            additional_resource.title =
                convert_from_jats(&additional_resource.title, markup, ConversionLimit::Title)?;
            additional_resource.description = additional_resource
                .description
                .as_ref()
                .map(|description| {
                    convert_from_jats(description, markup, ConversionLimit::Abstract)
                })
                .transpose()?;
        }

        Ok(additional_resources)
    }

    #[graphql(description = "Get awards linked to this work")]
    pub fn awards(
        &self,
        context: &Context,
        #[graphql(default = 50, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
    ) -> FieldResult<Vec<Award>> {
        Award::all(
            &context.db,
            limit.unwrap_or(50),
            offset.unwrap_or_default(),
            None,
            AwardOrderBy::default(),
            vec![],
            Some(self.work_id),
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Get endorsements linked to this work")]
    pub fn endorsements(
        &self,
        context: &Context,
        #[graphql(default = 50, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
    ) -> FieldResult<Vec<Endorsement>> {
        Endorsement::all(
            &context.db,
            limit.unwrap_or(50),
            offset.unwrap_or_default(),
            None,
            EndorsementOrderBy::default(),
            vec![],
            Some(self.work_id),
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Get book reviews linked to this work")]
    pub fn book_reviews(
        &self,
        context: &Context,
        #[graphql(default = 50, description = "The number of items to return")] limit: Option<i32>,
        #[graphql(default = 0, description = "The number of items to skip")] offset: Option<i32>,
    ) -> FieldResult<Vec<BookReview>> {
        BookReview::all(
            &context.db,
            limit.unwrap_or(50),
            offset.unwrap_or_default(),
            None,
            BookReviewOrderBy::default(),
            vec![],
            Some(self.work_id),
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Get the featured video linked to this work")]
    pub fn featured_video(&self, context: &Context) -> FieldResult<Option<WorkFeaturedVideo>> {
        WorkFeaturedVideo::from_work_id(&context.db, &self.work_id).map_err(Into::into)
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

    #[graphql(description = "WCAG standard accessibility level met by this publication (if any)")]
    pub fn accessibility_standard(&self) -> Option<&AccessibilityStandard> {
        self.accessibility_standard.as_ref()
    }

    #[graphql(
        description = "EPUB- or PDF-specific standard accessibility level met by this publication, if applicable"
    )]
    pub fn accessibility_additional_standard(&self) -> Option<&AccessibilityStandard> {
        self.accessibility_additional_standard.as_ref()
    }

    #[graphql(
        description = "Reason for this publication not being required to comply with accessibility standards (if any)"
    )]
    pub fn accessibility_exception(&self) -> Option<&AccessibilityException> {
        self.accessibility_exception.as_ref()
    }

    #[graphql(
        description = "Link to a web page showing detailed accessibility information for this publication"
    )]
    pub fn accessibility_report_url(&self) -> Option<&String> {
        self.accessibility_report_url.as_ref()
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
            None,
        )
        .map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Get the publication file for this publication")]
    pub fn file(&self, context: &Context) -> FieldResult<Option<File>> {
        File::from_publication_id(&context.db, &self.publication_id).map_err(Into::into)
    }

    #[graphql(description = "Get the work to which this publication belongs")]
    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(Into::into)
    }
}

#[juniper::graphql_object(
    Context = Context,
    description = "A file stored in the system (publication file, front cover, additional resource, or featured video)."
)]
impl File {
    #[graphql(description = "Thoth ID of the file")]
    pub fn file_id(&self) -> &Uuid {
        &self.file_id
    }

    #[graphql(
        description = "Type of file (publication, frontcover, additional_resource, or work_featured_video)"
    )]
    pub fn file_type(&self) -> &FileType {
        &self.file_type
    }

    #[graphql(description = "Thoth ID of the work (for frontcovers)")]
    pub fn work_id(&self) -> Option<&Uuid> {
        self.work_id.as_ref()
    }

    #[graphql(description = "Thoth ID of the publication (for publication files)")]
    pub fn publication_id(&self) -> Option<&Uuid> {
        self.publication_id.as_ref()
    }

    #[graphql(description = "Thoth ID of the additional resource (for additional resource files)")]
    pub fn additional_resource_id(&self) -> Option<&Uuid> {
        self.additional_resource_id.as_ref()
    }

    #[graphql(description = "Thoth ID of the featured video (for featured video files)")]
    pub fn work_featured_video_id(&self) -> Option<&Uuid> {
        self.work_featured_video_id.as_ref()
    }

    #[graphql(description = "S3 object key (canonical DOI-based path)")]
    pub fn object_key(&self) -> &String {
        &self.object_key
    }

    #[graphql(description = "Public CDN URL")]
    pub fn cdn_url(&self) -> &String {
        &self.cdn_url
    }

    #[graphql(description = "MIME type used when serving the file")]
    pub fn mime_type(&self) -> &String {
        &self.mime_type
    }

    #[graphql(description = "Size of the file in bytes")]
    pub fn bytes(&self) -> i32 {
        // GraphQL does not support i64; files larger than 2GB will overflow.
        self.bytes as i32
    }

    #[graphql(description = "SHA-256 checksum of the stored file")]
    pub fn sha256(&self) -> &String {
        &self.sha256
    }

    #[graphql(description = "Date and time at which the file record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the file record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
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

    #[graphql(description = "Zitadel organisation ID associated with the publisher")]
    pub fn zitadel_id(&self) -> Option<&String> {
        self.zitadel_id.as_ref()
    }

    #[graphql(
        description = "Statement from the publisher on the accessibility of its texts for readers with impairments"
    )]
    pub fn accessibility_statement(&self) -> Option<&String> {
        self.accessibility_statement.as_ref()
    }

    #[graphql(
        description = "URL of the publisher's report on the accessibility of its texts for readers with impairments"
    )]
    pub fn accessibility_report_url(&self) -> Option<&String> {
        self.accessibility_report_url.as_ref()
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
                    direction: Direction::Asc }
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
            None,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "Get contacts linked to this publisher")]
    pub fn contacts(
        &self,
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
            vec![],
            Some(self.publisher_id),
            None,
            contact_types.unwrap_or_default(),
            vec![],
            None,
            None,
        )
        .map_err(Into::into)
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

    #[graphql(description = "Default currency code for works under this imprint")]
    pub fn default_currency(&self) -> Option<&CurrencyCode> {
        self.default_currency.as_ref()
    }

    #[graphql(description = "Default publication place for works under this imprint")]
    pub fn default_place(&self) -> Option<&String> {
        self.default_place.as_ref()
    }

    #[graphql(description = "Default locale code for works under this imprint")]
    pub fn default_locale(&self) -> Option<&LocaleCode> {
        self.default_locale.as_ref()
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
        Publisher::from_id(&context.db, &self.publisher_id).map_err(Into::into)
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
        publication_date: Option<TimeExpression>,
        #[graphql(
            description = "Only show results with a publication date either before (less than) or after (greater than) the specified timestamp"
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
            publication_date,
            updated_at_with_relations,
        )
        .map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
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

    #[allow(clippy::too_many_arguments)]
    #[graphql(description = "Query the full list of biographies")]
    pub fn biographies(
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
            description = "If set, only shows results with this markup format"
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
            Some(self.contribution_id),
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

    #[graphql(description = "Biography of the contributor at the time of contribution")]
    #[graphql(
        deprecated = "Please use Contribution `biographies` field instead to get the correct biography in a multilingual manner"
    )]
    pub fn biography(&self, ctx: &Context) -> FieldResult<Option<String>> {
        Ok(
            Biography::canonical_from_contribution_id(&ctx.db, &self.contribution_id)
                .map(|a| a.content)
                .ok(),
        )
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
        Work::from_id(&context.db, &self.work_id).map_err(Into::into)
    }

    #[graphql(description = "Get the contributor who created the contribution")]
    pub fn contributor(&self, context: &Context) -> FieldResult<Contributor> {
        Contributor::from_id(&context.db, &self.contributor_id).map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
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
        Imprint::from_id(&context.db, &self.imprint_id).map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
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

    #[graphql(description = "Published issue number given to this issue within the series, if any")]
    pub fn issue_number(&self) -> Option<&i32> {
        self.issue_number.as_ref()
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
        Series::from_id(&context.db, &self.series_id).map_err(Into::into)
    }

    #[graphql(description = "Get the work represented by the issue")]
    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(Into::into)
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
        Work::from_id(&context.db, &self.work_id).map_err(Into::into)
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
        Publication::from_id(&context.db, &self.publication_id).map_err(Into::into)
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
        Publication::from_id(&context.db, &self.publication_id).map_err(Into::into)
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
        Work::from_id(&context.db, &self.work_id).map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
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
            None,
        )
        .map_err(Into::into)
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
        Work::from_id(&context.db, &self.work_id).map_err(Into::into)
    }

    #[graphql(description = "Get the funding institution")]
    pub fn institution(&self, context: &Context) -> FieldResult<Institution> {
        Institution::from_id(&context.db, &self.institution_id).map_err(Into::into)
    }
}

#[juniper::graphql_object(
    Context = Context,
    name = "WorkResource",
    description = "A resource linked to a work but not embedded in the work text."
)]
impl AdditionalResource {
    #[graphql(description = "Thoth ID of the work resource")]
    pub fn work_resource_id(&self) -> Uuid {
        self.additional_resource_id
    }

    #[graphql(description = "Thoth ID of the work to which this resource belongs")]
    pub fn work_id(&self) -> Uuid {
        self.work_id
    }

    #[graphql(description = "Title of the additional resource")]
    pub fn title(
        &self,
        #[graphql(description = "Markup format used for rendering title")] markup_format: Option<
            MarkupFormat,
        >,
    ) -> FieldResult<String> {
        match markup_format {
            Some(markup) => {
                convert_from_jats(&self.title, markup, ConversionLimit::Title).map_err(Into::into)
            }
            None => Ok(self.title.clone()),
        }
    }

    #[graphql(description = "Description of the additional resource")]
    pub fn description(
        &self,
        #[graphql(description = "Markup format used for rendering description")]
        markup_format: Option<MarkupFormat>,
    ) -> FieldResult<Option<String>> {
        match (&self.description, markup_format) {
            (Some(description), Some(markup)) => {
                convert_from_jats(description, markup, ConversionLimit::Abstract)
                    .map(Some)
                    .map_err(Into::into)
            }
            (Some(description), None) => Ok(Some(description.clone())),
            (None, _) => Ok(None),
        }
    }

    #[graphql(description = "Attribution for the resource source/author")]
    pub fn attribution(&self) -> Option<&String> {
        self.attribution.as_ref()
    }

    #[graphql(description = "Type of additional resource")]
    pub fn resource_type(&self) -> String {
        self.resource_type.to_string()
    }

    #[graphql(
        description = "DOI of the resource as full URL, using the HTTPS scheme and the doi.org domain"
    )]
    pub fn doi(&self) -> Option<&Doi> {
        self.doi.as_ref()
    }

    #[graphql(description = "Handle identifier of the resource")]
    pub fn handle(&self) -> Option<&String> {
        self.handle.as_ref()
    }

    #[graphql(description = "URL of the additional resource")]
    pub fn url(&self) -> Option<&String> {
        self.url.as_ref()
    }

    #[graphql(description = "Date associated with the additional resource")]
    pub fn date(&self) -> Option<NaiveDate> {
        self.date
    }

    #[graphql(
        description = "Number representing this resource's position in an ordered list of resources within the work"
    )]
    pub fn resource_ordinal(&self) -> i32 {
        self.resource_ordinal
    }

    #[graphql(description = "Date and time at which the resource record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the resource record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    #[graphql(description = "Get the work linked to this resource")]
    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(Into::into)
    }

    #[graphql(description = "Get the hosted file linked to this resource")]
    pub fn file(&self, context: &Context) -> FieldResult<Option<File>> {
        File::from_additional_resource_id(&context.db, &self.additional_resource_id)
            .map_err(Into::into)
    }
}

#[juniper::graphql_object(
    Context = Context,
    description = "An award linked to a work."
)]
impl Award {
    #[graphql(description = "Thoth ID of the award")]
    pub fn award_id(&self) -> Uuid {
        self.award_id
    }

    #[graphql(description = "Thoth ID of the work to which this award belongs")]
    pub fn work_id(&self) -> Uuid {
        self.work_id
    }

    #[graphql(description = "Title of the award")]
    pub fn title(
        &self,
        #[graphql(
            default = MarkupFormat::JatsXml,
            description = "Markup format used for rendering title",
        )]
        markup_format: Option<MarkupFormat>,
    ) -> FieldResult<String> {
        convert_from_jats(
            &self.title,
            markup_format.ok_or(ThothError::MissingMarkupFormat)?,
            ConversionLimit::Title,
        )
        .map_err(Into::into)
    }

    #[graphql(description = "URL of the award page")]
    pub fn url(&self) -> Option<&String> {
        self.url.as_ref()
    }

    #[graphql(description = "Category of the award")]
    pub fn category(&self) -> Option<&String> {
        self.category.as_ref()
    }

    #[graphql(description = "Role of the work in this award")]
    pub fn role(&self) -> Option<AwardRole> {
        self.role
    }

    #[graphql(description = "Prize statement for this award")]
    pub fn prize_statement(
        &self,
        #[graphql(
            default = MarkupFormat::JatsXml,
            description = "Markup format used for rendering prize statement",
        )]
        markup_format: Option<MarkupFormat>,
    ) -> FieldResult<Option<String>> {
        self.prize_statement
            .as_ref()
            .map(|prize_statement| {
                convert_from_jats(
                    prize_statement,
                    markup_format.ok_or(ThothError::MissingMarkupFormat)?,
                    ConversionLimit::Abstract,
                )
            })
            .transpose()
            .map_err(Into::into)
    }

    #[graphql(
        description = "Number representing this award's position in an ordered list of awards within the work"
    )]
    pub fn award_ordinal(&self) -> i32 {
        self.award_ordinal
    }

    #[graphql(description = "Date and time at which the award record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the award record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    #[graphql(description = "Get the work linked to this award")]
    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(Into::into)
    }
}

#[juniper::graphql_object(
    Context = Context,
    description = "An endorsement linked to a work."
)]
impl Endorsement {
    #[graphql(description = "Thoth ID of the endorsement")]
    pub fn endorsement_id(&self) -> Uuid {
        self.endorsement_id
    }

    #[graphql(description = "Thoth ID of the work to which this endorsement belongs")]
    pub fn work_id(&self) -> Uuid {
        self.work_id
    }

    #[graphql(description = "Name of the endorsement author")]
    pub fn author_name(&self) -> Option<&String> {
        self.author_name.as_ref()
    }

    #[graphql(description = "Role of the endorsement author")]
    pub fn author_role(&self) -> Option<&String> {
        self.author_role.as_ref()
    }

    #[graphql(
        description = "ORCID (Open Researcher and Contributor ID) of the endorsement author as full URL, using the HTTPS scheme and the orcid.org domain"
    )]
    pub fn author_orcid(&self) -> Option<&Orcid> {
        self.author_orcid.as_ref()
    }

    #[graphql(description = "Thoth ID of the endorsement author's institution")]
    pub fn author_institution_id(&self) -> Option<&Uuid> {
        self.author_institution_id.as_ref()
    }

    #[graphql(description = "URL associated with this endorsement")]
    pub fn url(&self) -> Option<&String> {
        self.url.as_ref()
    }

    #[graphql(description = "Text of the endorsement")]
    pub fn text(
        &self,
        #[graphql(
            default = MarkupFormat::JatsXml,
            description = "Markup format used for rendering endorsement text",
        )]
        markup_format: Option<MarkupFormat>,
    ) -> FieldResult<Option<String>> {
        self.text
            .as_ref()
            .map(|text| {
                convert_from_jats(
                    text,
                    markup_format.ok_or(ThothError::MissingMarkupFormat)?,
                    ConversionLimit::Abstract,
                )
            })
            .transpose()
            .map_err(Into::into)
    }

    #[graphql(
        description = "Number representing this endorsement's position in an ordered list of endorsements within the work"
    )]
    pub fn endorsement_ordinal(&self) -> i32 {
        self.endorsement_ordinal
    }

    #[graphql(description = "Date and time at which the endorsement record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the endorsement record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    #[graphql(description = "Get the work linked to this endorsement")]
    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(Into::into)
    }

    #[graphql(description = "Get the endorsement author's institution")]
    pub fn author_institution(&self, context: &Context) -> FieldResult<Option<Institution>> {
        self.author_institution_id
            .as_ref()
            .map(|institution_id| Institution::from_id(&context.db, institution_id))
            .transpose()
            .map_err(Into::into)
    }
}

#[juniper::graphql_object(
    Context = Context,
    description = "A review of a work."
)]
impl BookReview {
    #[graphql(description = "Thoth ID of the book review")]
    pub fn book_review_id(&self) -> Uuid {
        self.book_review_id
    }

    #[graphql(description = "Thoth ID of the work to which this review belongs")]
    pub fn work_id(&self) -> Uuid {
        self.work_id
    }

    #[graphql(description = "Title of the review")]
    pub fn title(
        &self,
        #[graphql(
            default = MarkupFormat::JatsXml,
            description = "Markup format used for rendering review title",
        )]
        markup_format: Option<MarkupFormat>,
    ) -> FieldResult<Option<String>> {
        self.title
            .as_ref()
            .map(|title| {
                convert_from_jats(
                    title,
                    markup_format.ok_or(ThothError::MissingMarkupFormat)?,
                    ConversionLimit::Title,
                )
            })
            .transpose()
            .map_err(Into::into)
    }

    #[graphql(description = "Name of the review author")]
    pub fn author_name(&self) -> Option<&String> {
        self.author_name.as_ref()
    }

    #[graphql(
        description = "ORCID (Open Researcher and Contributor ID) of the reviewer as full URL, using the HTTPS scheme and the orcid.org domain"
    )]
    pub fn reviewer_orcid(&self) -> Option<&Orcid> {
        self.reviewer_orcid.as_ref()
    }

    #[graphql(description = "Thoth ID of the reviewer's institution")]
    pub fn reviewer_institution_id(&self) -> Option<&Uuid> {
        self.reviewer_institution_id.as_ref()
    }

    #[graphql(description = "URL of the review publication")]
    pub fn url(&self) -> Option<&String> {
        self.url.as_ref()
    }

    #[graphql(
        description = "DOI of the review as full URL, using the HTTPS scheme and the doi.org domain"
    )]
    pub fn doi(&self) -> Option<&Doi> {
        self.doi.as_ref()
    }

    #[graphql(description = "Publication date of the review")]
    pub fn review_date(&self) -> Option<NaiveDate> {
        self.review_date
    }

    #[graphql(description = "Name of the journal where the review was published")]
    pub fn journal_name(&self) -> Option<&String> {
        self.journal_name.as_ref()
    }

    #[graphql(description = "Volume of the journal where the review was published")]
    pub fn journal_volume(&self) -> Option<&String> {
        self.journal_volume.as_ref()
    }

    #[graphql(description = "Number of the journal where the review was published")]
    pub fn journal_number(&self) -> Option<&String> {
        self.journal_number.as_ref()
    }

    #[graphql(description = "ISSN of the journal where the review was published")]
    pub fn journal_issn(&self) -> Option<&String> {
        self.journal_issn.as_ref()
    }

    #[graphql(description = "Page range of the review")]
    pub fn page_range(&self) -> Option<&String> {
        self.page_range.as_ref()
    }

    #[graphql(description = "Text of the review")]
    pub fn text(
        &self,
        #[graphql(
            default = MarkupFormat::JatsXml,
            description = "Markup format used for rendering review text",
        )]
        markup_format: Option<MarkupFormat>,
    ) -> FieldResult<Option<String>> {
        self.text
            .as_ref()
            .map(|text| {
                convert_from_jats(
                    text,
                    markup_format.ok_or(ThothError::MissingMarkupFormat)?,
                    ConversionLimit::Abstract,
                )
            })
            .transpose()
            .map_err(Into::into)
    }

    #[graphql(
        description = "Number representing this review's position in an ordered list of reviews within the work"
    )]
    pub fn review_ordinal(&self) -> i32 {
        self.review_ordinal
    }

    #[graphql(description = "Date and time at which the review record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the review record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    #[graphql(description = "Get the work linked to this review")]
    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(Into::into)
    }

    #[graphql(description = "Get the reviewer's institution")]
    pub fn reviewer_institution(&self, context: &Context) -> FieldResult<Option<Institution>> {
        self.reviewer_institution_id
            .as_ref()
            .map(|institution_id| Institution::from_id(&context.db, institution_id))
            .transpose()
            .map_err(Into::into)
    }
}

#[juniper::graphql_object(
    Context = Context,
    description = "A featured video linked to a work."
)]
impl WorkFeaturedVideo {
    #[graphql(description = "Thoth ID of the featured video")]
    pub fn work_featured_video_id(&self) -> Uuid {
        self.work_featured_video_id
    }

    #[graphql(description = "Thoth ID of the work to which this featured video belongs")]
    pub fn work_id(&self) -> Uuid {
        self.work_id
    }

    #[graphql(description = "Title or caption of the featured video")]
    pub fn title(&self) -> Option<&String> {
        self.title.as_ref()
    }

    #[graphql(description = "CDN URL of the featured video")]
    pub fn url(&self) -> Option<&String> {
        self.url.as_ref()
    }

    #[graphql(description = "Rendered width of the featured video embed")]
    pub fn width(&self) -> i32 {
        self.width
    }

    #[graphql(description = "Rendered height of the featured video embed")]
    pub fn height(&self) -> i32 {
        self.height
    }

    #[graphql(description = "Date and time at which the featured video record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the featured video record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    #[graphql(description = "Get the work linked to this featured video")]
    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(Into::into)
    }

    #[graphql(description = "Get the hosted file linked to this featured video")]
    pub fn file(&self, context: &Context) -> FieldResult<Option<File>> {
        File::from_work_featured_video_id(&context.db, &self.work_featured_video_id)
            .map_err(Into::into)
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
        Institution::from_id(&context.db, &self.institution_id).map_err(Into::into)
    }

    #[graphql(description = "Get the contribution linked to this affiliation")]
    pub fn contribution(&self, context: &Context) -> FieldResult<Contribution> {
        Contribution::from_id(&context.db, &self.contribution_id).map_err(Into::into)
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
        Work::from_id(&context.db, &self.related_work_id).map_err(Into::into)
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
        Work::from_id(&context.db, &self.work_id).map_err(Into::into)
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
        Work::from_id(&context.db, &self.work_id).map_err(Into::into)
    }
}

#[juniper::graphql_object(Context = Context, description = "An abstract associated with a work.")]
impl Abstract {
    #[graphql(description = "Thoth ID of the abstract")]
    pub fn abstract_id(&self) -> Uuid {
        self.abstract_id
    }
    #[graphql(description = "Thoth ID of the work to which the abstract is linked")]
    pub fn work_id(&self) -> Uuid {
        self.work_id
    }
    #[graphql(description = "Locale code of the abstract")]
    pub fn locale_code(&self) -> &LocaleCode {
        &self.locale_code
    }
    #[graphql(description = "Content of the abstract")]
    pub fn content(&self) -> &String {
        &self.content
    }
    #[graphql(description = "Whether this is the canonical abstract for the work")]
    pub fn canonical(&self) -> bool {
        self.canonical
    }
    #[graphql(description = "Type of the abstract")]
    pub fn abstract_type(&self) -> &AbstractType {
        &self.abstract_type
    }
    #[graphql(description = "Get the work to which the abstract is linked")]
    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        Work::from_id(&context.db, &self.work_id).map_err(Into::into)
    }
}

#[juniper::graphql_object(Context = Context, description = "A biography associated with a work and contribution.")]
impl Biography {
    #[graphql(description = "Thoth ID of the biography")]
    pub fn biography_id(&self) -> Uuid {
        self.biography_id
    }

    #[graphql(description = "Thoth ID of the contribution to which the biography is linked")]
    pub fn contribution_id(&self) -> Uuid {
        self.contribution_id
    }

    #[graphql(description = "Locale code of the biography")]
    pub fn locale_code(&self) -> &LocaleCode {
        &self.locale_code
    }

    #[graphql(description = "Content of the biography")]
    pub fn content(&self) -> &String {
        &self.content
    }

    #[graphql(description = "Whether this is the canonical biography for the contribution/work")]
    pub fn canonical(&self) -> bool {
        self.canonical
    }

    #[graphql(description = "Get the work to which the biography is linked via contribution")]
    pub fn work(&self, context: &Context) -> FieldResult<Work> {
        let contribution = Contribution::from_id(&context.db, &self.contribution_id)?;
        Work::from_id(&context.db, &contribution.work_id).map_err(Into::into)
    }

    #[graphql(description = "Get the contribution to which the biography is linked")]
    pub fn contribution(&self, context: &Context) -> FieldResult<Contribution> {
        Contribution::from_id(&context.db, &self.contribution_id).map_err(Into::into)
    }
}

#[juniper::graphql_object(Context = Context, description = "A way to get in touch with a publisher.")]
impl Contact {
    #[graphql(description = "Thoth ID of the contact")]
    pub fn contact_id(&self) -> Uuid {
        self.contact_id
    }

    #[graphql(description = "Thoth ID of the publisher to which this contact belongs")]
    pub fn publisher_id(&self) -> Uuid {
        self.publisher_id
    }

    #[graphql(description = "Type of the contact")]
    pub fn contact_type(&self) -> &ContactType {
        &self.contact_type
    }

    #[graphql(description = "Email address of the contact")]
    pub fn email(&self) -> &String {
        &self.email
    }

    #[graphql(description = "Date and time at which the contact record was created")]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    #[graphql(description = "Date and time at which the contact record was last updated")]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    #[graphql(description = "Get the publisher to which this contact belongs")]
    pub fn publisher(&self, context: &Context) -> FieldResult<Publisher> {
        Publisher::from_id(&context.db, &self.publisher_id).map_err(Into::into)
    }
}
