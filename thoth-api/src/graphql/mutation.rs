use juniper::FieldResult;
use uuid::Uuid;

use crate::graphql::Context;
use crate::markup::{convert_to_jats, ConversionLimit, MarkupFormat};
use crate::model::{
    additional_resource::{
        AdditionalResource, AdditionalResourcePolicy, NewAdditionalResource, PatchAdditionalResource,
        ResourceType,
    },
    affiliation::{Affiliation, AffiliationPolicy, NewAffiliation, PatchAffiliation},
    award::{Award, AwardPolicy, NewAward, PatchAward},
    biography::{Biography, BiographyPolicy, NewBiography, PatchBiography},
    book_review::{BookReview, BookReviewPolicy, NewBookReview, PatchBookReview},
    contact::{Contact, ContactPolicy, NewContact, PatchContact},
    contribution::{Contribution, ContributionPolicy, NewContribution, PatchContribution},
    contributor::{Contributor, ContributorPolicy, NewContributor, PatchContributor},
    endorsement::{Endorsement, EndorsementPolicy, NewEndorsement, PatchEndorsement},
    file::{
        CompleteFileUpload, File, FilePolicy, FileUpload, FileUploadResponse, NewFileUpload,
        NewAdditionalResourceFileUpload, NewFrontcoverFileUpload, NewPublicationFileUpload,
        NewWorkFeaturedVideoFileUpload,
    },
    funding::{Funding, FundingPolicy, NewFunding, PatchFunding},
    imprint::{Imprint, ImprintPolicy, NewImprint, PatchImprint},
    institution::{Institution, InstitutionPolicy, NewInstitution, PatchInstitution},
    issue::{Issue, IssuePolicy, NewIssue, PatchIssue},
    language::{Language, LanguagePolicy, NewLanguage, PatchLanguage},
    location::{Location, LocationPolicy, NewLocation, PatchLocation},
    price::{NewPrice, PatchPrice, Price, PricePolicy},
    publication::{NewPublication, PatchPublication, Publication, PublicationPolicy},
    publisher::{NewPublisher, PatchPublisher, Publisher, PublisherPolicy},
    r#abstract::{Abstract, AbstractPolicy, NewAbstract, PatchAbstract},
    reference::{NewReference, PatchReference, Reference, ReferencePolicy},
    series::{NewSeries, PatchSeries, Series, SeriesPolicy},
    subject::{NewSubject, PatchSubject, Subject, SubjectPolicy},
    title::{convert_title_to_jats, NewTitle, PatchTitle, Title, TitlePolicy},
    work::{NewWork, PatchWork, Work, WorkPolicy},
    work_featured_video::{
        NewWorkFeaturedVideo, PatchWorkFeaturedVideo, WorkFeaturedVideo, WorkFeaturedVideoPolicy,
    },
    work_relation::{NewWorkRelation, PatchWorkRelation, WorkRelation, WorkRelationPolicy},
    Crud, Reorder,
};
use crate::policy::{CreatePolicy, DeletePolicy, MovePolicy, PolicyContext, UpdatePolicy};
use crate::storage::{
    build_cdn_url, copy_temp_object_to_final, delete_object, head_object,
    reconcile_replaced_object, temp_key, StorageConfig,
};
use thoth_errors::ThothError;

pub struct MutationRoot;

#[juniper::graphql_object(Context = Context)]
impl MutationRoot {
    #[graphql(description = "Create a new work with the specified values")]
    fn create_work(
        context: &Context,
        #[graphql(description = "Values for work to be created")] data: NewWork,
    ) -> FieldResult<Work> {
        WorkPolicy::can_create(context, &data, ())?;
        Work::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new publisher with the specified values")]
    fn create_publisher(
        context: &Context,
        #[graphql(description = "Values for publisher to be created")] data: NewPublisher,
    ) -> FieldResult<Publisher> {
        PublisherPolicy::can_create(context, &data, ())?;
        Publisher::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new imprint with the specified values")]
    fn create_imprint(
        context: &Context,
        #[graphql(description = "Values for imprint to be created")] data: NewImprint,
    ) -> FieldResult<Imprint> {
        ImprintPolicy::can_create(context, &data, ())?;
        Imprint::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new contributor with the specified values")]
    fn create_contributor(
        context: &Context,
        #[graphql(description = "Values for contributor to be created")] data: NewContributor,
    ) -> FieldResult<Contributor> {
        ContributorPolicy::can_create(context, &data, ())?;
        Contributor::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new contribution with the specified values")]
    fn create_contribution(
        context: &Context,
        #[graphql(description = "Values for contribution to be created")] data: NewContribution,
    ) -> FieldResult<Contribution> {
        ContributionPolicy::can_create(context, &data, ())?;
        Contribution::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new publication with the specified values")]
    fn create_publication(
        context: &Context,
        #[graphql(description = "Values for publication to be created")] data: NewPublication,
    ) -> FieldResult<Publication> {
        PublicationPolicy::can_create(context, &data, ())?;
        Publication::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new series with the specified values")]
    fn create_series(
        context: &Context,
        #[graphql(description = "Values for series to be created")] data: NewSeries,
    ) -> FieldResult<Series> {
        SeriesPolicy::can_create(context, &data, ())?;
        Series::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new issue with the specified values")]
    fn create_issue(
        context: &Context,
        #[graphql(description = "Values for issue to be created")] data: NewIssue,
    ) -> FieldResult<Issue> {
        IssuePolicy::can_create(context, &data, ())?;
        Issue::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new language with the specified values")]
    fn create_language(
        context: &Context,
        #[graphql(description = "Values for language to be created")] data: NewLanguage,
    ) -> FieldResult<Language> {
        LanguagePolicy::can_create(context, &data, ())?;
        Language::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new title with the specified values")]
    fn create_title(
        context: &Context,
        #[graphql(description = "The markup format of the title")] markup_format: Option<
            MarkupFormat,
        >,
        #[graphql(description = "Values for title to be created")] mut data: NewTitle,
    ) -> FieldResult<Title> {
        TitlePolicy::can_create(context, &data, markup_format)?;

        let markup = markup_format.expect("Validated by policy");
        convert_title_to_jats(&mut data, markup)?;

        Title::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new abstract with the specified values")]
    fn create_abstract(
        context: &Context,
        #[graphql(description = "The markup format of the abstract")] markup_format: Option<
            MarkupFormat,
        >,
        #[graphql(description = "Values for abstract to be created")] mut data: NewAbstract,
    ) -> FieldResult<Abstract> {
        AbstractPolicy::can_create(context, &data, markup_format)?;

        let markup = markup_format.expect("Validated by policy");
        data.content = convert_to_jats(data.content, markup, ConversionLimit::Abstract)?;

        Abstract::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new biography with the specified values")]
    fn create_biography(
        context: &Context,
        #[graphql(description = "The markup format of the biography")] markup_format: Option<
            MarkupFormat,
        >,
        #[graphql(description = "Values for biography to be created")] mut data: NewBiography,
    ) -> FieldResult<Biography> {
        BiographyPolicy::can_create(context, &data, markup_format)?;

        let markup = markup_format.expect("Validated by policy");
        data.content = convert_to_jats(data.content, markup, ConversionLimit::Biography)?;

        Biography::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new institution with the specified values")]
    fn create_institution(
        context: &Context,
        #[graphql(description = "Values for institution to be created")] data: NewInstitution,
    ) -> FieldResult<Institution> {
        InstitutionPolicy::can_create(context, &data, ())?;
        Institution::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new funding with the specified values")]
    fn create_funding(
        context: &Context,
        #[graphql(description = "Values for funding to be created")] data: NewFunding,
    ) -> FieldResult<Funding> {
        FundingPolicy::can_create(context, &data, ())?;
        Funding::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new location with the specified values")]
    fn create_location(
        context: &Context,
        #[graphql(description = "Values for location to be created")] data: NewLocation,
    ) -> FieldResult<Location> {
        LocationPolicy::can_create(context, &data, ())?;
        Location::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new price with the specified values")]
    fn create_price(
        context: &Context,
        #[graphql(description = "Values for price to be created")] data: NewPrice,
    ) -> FieldResult<Price> {
        PricePolicy::can_create(context, &data, ())?;
        Price::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new subject with the specified values")]
    fn create_subject(
        context: &Context,
        #[graphql(description = "Values for subject to be created")] data: NewSubject,
    ) -> FieldResult<Subject> {
        SubjectPolicy::can_create(context, &data, ())?;
        Subject::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new affiliation with the specified values")]
    fn create_affiliation(
        context: &Context,
        #[graphql(description = "Values for affiliation to be created")] data: NewAffiliation,
    ) -> FieldResult<Affiliation> {
        AffiliationPolicy::can_create(context, &data, ())?;
        Affiliation::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new work relation with the specified values")]
    fn create_work_relation(
        context: &Context,
        #[graphql(description = "Values for work relation to be created")] data: NewWorkRelation,
    ) -> FieldResult<WorkRelation> {
        WorkRelationPolicy::can_create(context, &data, ())?;
        WorkRelation::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new reference with the specified values")]
    fn create_reference(
        context: &Context,
        #[graphql(description = "Values for reference to be created")] data: NewReference,
    ) -> FieldResult<Reference> {
        ReferencePolicy::can_create(context, &data, ())?;
        Reference::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new additional resource with the specified values")]
    fn create_additional_resource(
        context: &Context,
        #[graphql(description = "The markup format of the additional resource text fields")]
        markup_format: Option<MarkupFormat>,
        #[graphql(description = "Values for additional resource to be created")]
        mut data: NewAdditionalResource,
    ) -> FieldResult<AdditionalResource> {
        AdditionalResourcePolicy::can_create(context, &data, ())?;

        let markup = markup_format.unwrap_or(MarkupFormat::JatsXml);
        data.title = convert_to_jats(data.title, markup, ConversionLimit::Title)?;
        data.description = data
            .description
            .map(|description| convert_to_jats(description, markup, ConversionLimit::Abstract))
            .transpose()?;

        AdditionalResource::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new award with the specified values")]
    fn create_award(
        context: &Context,
        #[graphql(description = "The markup format of the award text fields")]
        markup_format: Option<MarkupFormat>,
        #[graphql(description = "Values for award to be created")] mut data: NewAward,
    ) -> FieldResult<Award> {
        AwardPolicy::can_create(context, &data, ())?;

        let markup = markup_format.unwrap_or(MarkupFormat::JatsXml);
        data.title = convert_to_jats(data.title, markup, ConversionLimit::Title)?;
        data.note = data
            .note
            .map(|note| convert_to_jats(note, markup, ConversionLimit::Abstract))
            .transpose()?;

        Award::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new endorsement with the specified values")]
    fn create_endorsement(
        context: &Context,
        #[graphql(description = "The markup format of the endorsement text field")]
        markup_format: Option<MarkupFormat>,
        #[graphql(description = "Values for endorsement to be created")] mut data: NewEndorsement,
    ) -> FieldResult<Endorsement> {
        EndorsementPolicy::can_create(context, &data, ())?;

        let markup = markup_format.unwrap_or(MarkupFormat::JatsXml);
        data.text = data
            .text
            .map(|text| convert_to_jats(text, markup, ConversionLimit::Abstract))
            .transpose()?;

        Endorsement::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new book review with the specified values")]
    fn create_book_review(
        context: &Context,
        #[graphql(description = "The markup format of the book review text field")]
        markup_format: Option<MarkupFormat>,
        #[graphql(description = "Values for book review to be created")] mut data: NewBookReview,
    ) -> FieldResult<BookReview> {
        BookReviewPolicy::can_create(context, &data, ())?;

        let markup = markup_format.unwrap_or(MarkupFormat::JatsXml);
        data.text = data
            .text
            .map(|text| convert_to_jats(text, markup, ConversionLimit::Abstract))
            .transpose()?;

        BookReview::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new featured video with the specified values")]
    fn create_work_featured_video(
        context: &Context,
        #[graphql(description = "Values for featured video to be created")] data: NewWorkFeaturedVideo,
    ) -> FieldResult<WorkFeaturedVideo> {
        WorkFeaturedVideoPolicy::can_create(context, &data, ())?;
        WorkFeaturedVideo::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Create a new contact with the specified values")]
    fn create_contact(
        context: &Context,
        #[graphql(description = "Values for contact to be created")] data: NewContact,
    ) -> FieldResult<Contact> {
        ContactPolicy::can_create(context, &data, ())?;
        Contact::create(&context.db, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing work with the specified values")]
    fn update_work(
        context: &Context,
        #[graphql(description = "Values to apply to existing work")] data: PatchWork,
    ) -> FieldResult<Work> {
        let work = context.load_current(&data.work_id)?;
        WorkPolicy::can_update(context, &work, &data, ())?;

        // update the work and, if it succeeds, synchronise its children statuses and pub. date
        let w = work.update(context, &data)?;
        for child in work.children(&context.db)? {
            if child.publication_date != w.publication_date
                || child.work_status != w.work_status
                || child.withdrawn_date != w.withdrawn_date
            {
                let mut data: PatchWork = child.clone().into();
                data.publication_date = w.publication_date;
                data.withdrawn_date = w.withdrawn_date;
                data.work_status = w.work_status;
                child.update(context, &data)?;
            }
        }
        Ok(w)
    }

    #[graphql(description = "Update an existing publisher with the specified values")]
    fn update_publisher(
        context: &Context,
        #[graphql(description = "Values to apply to existing publisher")] data: PatchPublisher,
    ) -> FieldResult<Publisher> {
        let publisher = context.load_current(&data.publisher_id)?;
        PublisherPolicy::can_update(context, &publisher, &data, ())?;

        publisher.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing imprint with the specified values")]
    fn update_imprint(
        context: &Context,
        #[graphql(description = "Values to apply to existing imprint")] data: PatchImprint,
    ) -> FieldResult<Imprint> {
        let imprint = context.load_current(&data.imprint_id)?;
        ImprintPolicy::can_update(context, &imprint, &data, ())?;

        imprint.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing contributor with the specified values")]
    fn update_contributor(
        context: &Context,
        #[graphql(description = "Values to apply to existing contributor")] data: PatchContributor,
    ) -> FieldResult<Contributor> {
        let contributor = context.load_current(&data.contributor_id)?;
        ContributorPolicy::can_update(context, &contributor, &data, ())?;

        contributor.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing contribution with the specified values")]
    fn update_contribution(
        context: &Context,
        #[graphql(description = "Values to apply to existing contribution")]
        data: PatchContribution,
    ) -> FieldResult<Contribution> {
        let contribution = context.load_current(&data.contribution_id)?;
        ContributionPolicy::can_update(context, &contribution, &data, ())?;

        contribution.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing publication with the specified values")]
    fn update_publication(
        context: &Context,
        #[graphql(description = "Values to apply to existing publication")] data: PatchPublication,
    ) -> FieldResult<Publication> {
        let publication = context.load_current(&data.publication_id)?;
        PublicationPolicy::can_update(context, &publication, &data, ())?;

        publication.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing series with the specified values")]
    fn update_series(
        context: &Context,
        #[graphql(description = "Values to apply to existing series")] data: PatchSeries,
    ) -> FieldResult<Series> {
        let series = context.load_current(&data.series_id)?;
        SeriesPolicy::can_update(context, &series, &data, ())?;

        series.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing issue with the specified values")]
    fn update_issue(
        context: &Context,
        #[graphql(description = "Values to apply to existing issue")] data: PatchIssue,
    ) -> FieldResult<Issue> {
        let issue = context.load_current(&data.issue_id)?;
        IssuePolicy::can_update(context, &issue, &data, ())?;

        issue.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing language with the specified values")]
    fn update_language(
        context: &Context,
        #[graphql(description = "Values to apply to existing language")] data: PatchLanguage,
    ) -> FieldResult<Language> {
        let language = context.load_current(&data.language_id)?;
        LanguagePolicy::can_update(context, &language, &data, ())?;

        language.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing institution with the specified values")]
    fn update_institution(
        context: &Context,
        #[graphql(description = "Values to apply to existing institution")] data: PatchInstitution,
    ) -> FieldResult<Institution> {
        let institution = context.load_current(&data.institution_id)?;
        InstitutionPolicy::can_update(context, &institution, &data, ())?;

        institution.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing funding with the specified values")]
    fn update_funding(
        context: &Context,
        #[graphql(description = "Values to apply to existing funding")] data: PatchFunding,
    ) -> FieldResult<Funding> {
        let funding = context.load_current(&data.funding_id)?;
        FundingPolicy::can_update(context, &funding, &data, ())?;

        funding.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing location with the specified values")]
    fn update_location(
        context: &Context,
        #[graphql(description = "Values to apply to existing location")] data: PatchLocation,
    ) -> FieldResult<Location> {
        let current_location = context.load_current(&data.location_id)?;
        LocationPolicy::can_update(context, &current_location, &data, ())?;

        current_location.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing price with the specified values")]
    fn update_price(
        context: &Context,
        #[graphql(description = "Values to apply to existing price")] data: PatchPrice,
    ) -> FieldResult<Price> {
        let price = context.load_current(&data.price_id)?;
        PricePolicy::can_update(context, &price, &data, ())?;

        price.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing subject with the specified values")]
    fn update_subject(
        context: &Context,
        #[graphql(description = "Values to apply to existing subject")] data: PatchSubject,
    ) -> FieldResult<Subject> {
        let subject = context.load_current(&data.subject_id)?;
        SubjectPolicy::can_update(context, &subject, &data, ())?;

        subject.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing affiliation with the specified values")]
    fn update_affiliation(
        context: &Context,
        #[graphql(description = "Values to apply to existing affiliation")] data: PatchAffiliation,
    ) -> FieldResult<Affiliation> {
        let affiliation = context.load_current(&data.affiliation_id)?;
        AffiliationPolicy::can_update(context, &affiliation, &data, ())?;

        affiliation.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing work relation with the specified values")]
    fn update_work_relation(
        context: &Context,
        #[graphql(description = "Values to apply to existing work relation")]
        data: PatchWorkRelation,
    ) -> FieldResult<WorkRelation> {
        let work_relation = context.load_current(&data.work_relation_id)?;
        WorkRelationPolicy::can_update(context, &work_relation, &data, ())?;

        work_relation.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing reference with the specified values")]
    fn update_reference(
        context: &Context,
        #[graphql(description = "Values to apply to existing reference")] data: PatchReference,
    ) -> FieldResult<Reference> {
        let reference = context.load_current(&data.reference_id)?;
        ReferencePolicy::can_update(context, &reference, &data, ())?;

        reference.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing additional resource with the specified values")]
    fn update_additional_resource(
        context: &Context,
        #[graphql(description = "The markup format of the additional resource text fields")]
        markup_format: Option<MarkupFormat>,
        #[graphql(description = "Values to apply to existing additional resource")]
        mut data: PatchAdditionalResource,
    ) -> FieldResult<AdditionalResource> {
        let additional_resource = context.load_current(&data.additional_resource_id)?;
        AdditionalResourcePolicy::can_update(context, &additional_resource, &data, ())?;

        let markup = markup_format.unwrap_or(MarkupFormat::JatsXml);
        data.title = convert_to_jats(data.title, markup, ConversionLimit::Title)?;
        data.description = data
            .description
            .map(|description| convert_to_jats(description, markup, ConversionLimit::Abstract))
            .transpose()?;

        additional_resource.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing award with the specified values")]
    fn update_award(
        context: &Context,
        #[graphql(description = "The markup format of the award text fields")]
        markup_format: Option<MarkupFormat>,
        #[graphql(description = "Values to apply to existing award")] mut data: PatchAward,
    ) -> FieldResult<Award> {
        let award = context.load_current(&data.award_id)?;
        AwardPolicy::can_update(context, &award, &data, ())?;

        let markup = markup_format.unwrap_or(MarkupFormat::JatsXml);
        data.title = convert_to_jats(data.title, markup, ConversionLimit::Title)?;
        data.note = data
            .note
            .map(|note| convert_to_jats(note, markup, ConversionLimit::Abstract))
            .transpose()?;

        award.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing endorsement with the specified values")]
    fn update_endorsement(
        context: &Context,
        #[graphql(description = "The markup format of the endorsement text field")]
        markup_format: Option<MarkupFormat>,
        #[graphql(description = "Values to apply to existing endorsement")]
        mut data: PatchEndorsement,
    ) -> FieldResult<Endorsement> {
        let endorsement = context.load_current(&data.endorsement_id)?;
        EndorsementPolicy::can_update(context, &endorsement, &data, ())?;

        let markup = markup_format.unwrap_or(MarkupFormat::JatsXml);
        data.text = data
            .text
            .map(|text| convert_to_jats(text, markup, ConversionLimit::Abstract))
            .transpose()?;

        endorsement.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing book review with the specified values")]
    fn update_book_review(
        context: &Context,
        #[graphql(description = "The markup format of the book review text field")]
        markup_format: Option<MarkupFormat>,
        #[graphql(description = "Values to apply to existing book review")]
        mut data: PatchBookReview,
    ) -> FieldResult<BookReview> {
        let book_review = context.load_current(&data.book_review_id)?;
        BookReviewPolicy::can_update(context, &book_review, &data, ())?;

        let markup = markup_format.unwrap_or(MarkupFormat::JatsXml);
        data.text = data
            .text
            .map(|text| convert_to_jats(text, markup, ConversionLimit::Abstract))
            .transpose()?;

        book_review.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing featured video with the specified values")]
    fn update_work_featured_video(
        context: &Context,
        #[graphql(description = "Values to apply to existing featured video")]
        data: PatchWorkFeaturedVideo,
    ) -> FieldResult<WorkFeaturedVideo> {
        let work_featured_video = context.load_current(&data.work_featured_video_id)?;
        WorkFeaturedVideoPolicy::can_update(context, &work_featured_video, &data, ())?;

        work_featured_video.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing contact with the specified values")]
    fn update_contact(
        context: &Context,
        #[graphql(description = "Values to apply to existing contact")] data: PatchContact,
    ) -> FieldResult<Contact> {
        let contact = context.load_current(&data.contact_id)?;
        ContactPolicy::can_update(context, &contact, &data, ())?;

        contact.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing title with the specified values")]
    fn update_title(
        context: &Context,
        #[graphql(description = "The markup format of the title")] markup_format: Option<
            MarkupFormat,
        >,
        #[graphql(description = "Values to apply to existing title")] mut data: PatchTitle,
    ) -> FieldResult<Title> {
        let title = context.load_current(&data.title_id)?;
        TitlePolicy::can_update(context, &title, &data, markup_format)?;

        let markup = markup_format.expect("Validated by policy");
        convert_title_to_jats(&mut data, markup)?;

        title.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing abstract with the specified values")]
    fn update_abstract(
        context: &Context,
        #[graphql(description = "The markup format of the abstract")] markup_format: Option<
            MarkupFormat,
        >,
        #[graphql(description = "Values to apply to existing abstract")] mut data: PatchAbstract,
    ) -> FieldResult<Abstract> {
        let r#abstract = context.load_current(&data.abstract_id)?;
        AbstractPolicy::can_update(context, &r#abstract, &data, markup_format)?;

        let markup = markup_format.expect("Validated by policy");
        data.content = convert_to_jats(data.content, markup, ConversionLimit::Abstract)?;

        r#abstract.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Update an existing biography with the specified values")]
    fn update_biography(
        context: &Context,
        #[graphql(description = "The markup format of the biography")] markup_format: Option<
            MarkupFormat,
        >,
        #[graphql(description = "Values to apply to existing biography")] mut data: PatchBiography,
    ) -> FieldResult<Biography> {
        let biography = context.load_current(&data.biography_id)?;
        BiographyPolicy::can_update(context, &biography, &data, markup_format)?;

        let markup = markup_format.expect("Validated by policy");
        data.content = convert_to_jats(data.content, markup, ConversionLimit::Biography)?;

        biography.update(context, &data).map_err(Into::into)
    }

    #[graphql(description = "Delete a single work using its ID")]
    fn delete_work(
        context: &Context,
        #[graphql(description = "Thoth ID of work to be deleted")] work_id: Uuid,
    ) -> FieldResult<Work> {
        let work = context.load_current(&work_id)?;
        WorkPolicy::can_delete(context, &work)?;

        work.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single publisher using its ID")]
    fn delete_publisher(
        context: &Context,
        #[graphql(description = "Thoth ID of publisher to be deleted")] publisher_id: Uuid,
    ) -> FieldResult<Publisher> {
        let publisher = context.load_current(&publisher_id)?;
        PublisherPolicy::can_delete(context, &publisher)?;

        publisher.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single imprint using its ID")]
    fn delete_imprint(
        context: &Context,
        #[graphql(description = "Thoth ID of imprint to be deleted")] imprint_id: Uuid,
    ) -> FieldResult<Imprint> {
        let imprint = context.load_current(&imprint_id)?;
        ImprintPolicy::can_delete(context, &imprint)?;

        imprint.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single contributor using its ID")]
    fn delete_contributor(
        context: &Context,
        #[graphql(description = "Thoth ID of contributor to be deleted")] contributor_id: Uuid,
    ) -> FieldResult<Contributor> {
        let contributor = context.load_current(&contributor_id)?;
        ContributorPolicy::can_delete(context, &contributor)?;

        contributor.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single contribution using its ID")]
    fn delete_contribution(
        context: &Context,
        #[graphql(description = "Thoth ID of contribution to be deleted")] contribution_id: Uuid,
    ) -> FieldResult<Contribution> {
        let contribution = context.load_current(&contribution_id)?;
        ContributionPolicy::can_delete(context, &contribution)?;

        contribution.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single publication using its ID")]
    fn delete_publication(
        context: &Context,
        #[graphql(description = "Thoth ID of publication to be deleted")] publication_id: Uuid,
    ) -> FieldResult<Publication> {
        let publication = context.load_current(&publication_id)?;
        PublicationPolicy::can_delete(context, &publication)?;

        publication.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single series using its ID")]
    fn delete_series(
        context: &Context,
        #[graphql(description = "Thoth ID of series to be deleted")] series_id: Uuid,
    ) -> FieldResult<Series> {
        let series = context.load_current(&series_id)?;
        SeriesPolicy::can_delete(context, &series)?;

        series.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single issue using its ID")]
    fn delete_issue(
        context: &Context,
        #[graphql(description = "Thoth ID of issue to be deleted")] issue_id: Uuid,
    ) -> FieldResult<Issue> {
        let issue = context.load_current(&issue_id)?;
        IssuePolicy::can_delete(context, &issue)?;

        issue.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single language using its ID")]
    fn delete_language(
        context: &Context,
        #[graphql(description = "Thoth ID of language to be deleted")] language_id: Uuid,
    ) -> FieldResult<Language> {
        let language = context.load_current(&language_id)?;
        LanguagePolicy::can_delete(context, &language)?;

        language.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single title using its ID")]
    fn delete_title(
        context: &Context,
        #[graphql(description = "Thoth ID of title to be deleted")] title_id: Uuid,
    ) -> FieldResult<Title> {
        let title = context.load_current(&title_id)?;
        TitlePolicy::can_delete(context, &title)?;

        title.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single institution using its ID")]
    fn delete_institution(
        context: &Context,
        #[graphql(description = "Thoth ID of institution to be deleted")] institution_id: Uuid,
    ) -> FieldResult<Institution> {
        let institution = context.load_current(&institution_id)?;
        InstitutionPolicy::can_delete(context, &institution)?;

        institution.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single funding using its ID")]
    fn delete_funding(
        context: &Context,
        #[graphql(description = "Thoth ID of funding to be deleted")] funding_id: Uuid,
    ) -> FieldResult<Funding> {
        let funding = context.load_current(&funding_id)?;
        FundingPolicy::can_delete(context, &funding)?;

        funding.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single location using its ID")]
    fn delete_location(
        context: &Context,
        #[graphql(description = "Thoth ID of location to be deleted")] location_id: Uuid,
    ) -> FieldResult<Location> {
        let location = context.load_current(&location_id)?;
        LocationPolicy::can_delete(context, &location)?;

        location.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single price using its ID")]
    fn delete_price(
        context: &Context,
        #[graphql(description = "Thoth ID of price to be deleted")] price_id: Uuid,
    ) -> FieldResult<Price> {
        let price = context.load_current(&price_id)?;
        PricePolicy::can_delete(context, &price)?;

        price.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single subject using its ID")]
    fn delete_subject(
        context: &Context,
        #[graphql(description = "Thoth ID of subject to be deleted")] subject_id: Uuid,
    ) -> FieldResult<Subject> {
        let subject = context.load_current(&subject_id)?;
        SubjectPolicy::can_delete(context, &subject)?;

        subject.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single affiliation using its ID")]
    fn delete_affiliation(
        context: &Context,
        #[graphql(description = "Thoth ID of affiliation to be deleted")] affiliation_id: Uuid,
    ) -> FieldResult<Affiliation> {
        let affiliation = context.load_current(&affiliation_id)?;
        AffiliationPolicy::can_delete(context, &affiliation)?;

        affiliation.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single work relation using its ID")]
    fn delete_work_relation(
        context: &Context,
        #[graphql(description = "Thoth ID of work relation to be deleted")] work_relation_id: Uuid,
    ) -> FieldResult<WorkRelation> {
        let work_relation = context.load_current(&work_relation_id)?;
        WorkRelationPolicy::can_delete(context, &work_relation)?;

        work_relation.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single reference using its ID")]
    fn delete_reference(
        context: &Context,
        #[graphql(description = "Thoth ID of reference to be deleted")] reference_id: Uuid,
    ) -> FieldResult<Reference> {
        let reference = context.load_current(&reference_id)?;
        ReferencePolicy::can_delete(context, &reference)?;

        reference.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single additional resource using its ID")]
    fn delete_additional_resource(
        context: &Context,
        #[graphql(description = "Thoth ID of additional resource to be deleted")]
        additional_resource_id: Uuid,
    ) -> FieldResult<AdditionalResource> {
        let additional_resource = context.load_current(&additional_resource_id)?;
        AdditionalResourcePolicy::can_delete(context, &additional_resource)?;

        additional_resource.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single award using its ID")]
    fn delete_award(
        context: &Context,
        #[graphql(description = "Thoth ID of award to be deleted")] award_id: Uuid,
    ) -> FieldResult<Award> {
        let award = context.load_current(&award_id)?;
        AwardPolicy::can_delete(context, &award)?;

        award.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single endorsement using its ID")]
    fn delete_endorsement(
        context: &Context,
        #[graphql(description = "Thoth ID of endorsement to be deleted")] endorsement_id: Uuid,
    ) -> FieldResult<Endorsement> {
        let endorsement = context.load_current(&endorsement_id)?;
        EndorsementPolicy::can_delete(context, &endorsement)?;

        endorsement.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single book review using its ID")]
    fn delete_book_review(
        context: &Context,
        #[graphql(description = "Thoth ID of book review to be deleted")] book_review_id: Uuid,
    ) -> FieldResult<BookReview> {
        let book_review = context.load_current(&book_review_id)?;
        BookReviewPolicy::can_delete(context, &book_review)?;

        book_review.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single featured video using its ID")]
    fn delete_work_featured_video(
        context: &Context,
        #[graphql(description = "Thoth ID of featured video to be deleted")] work_featured_video_id: Uuid,
    ) -> FieldResult<WorkFeaturedVideo> {
        let work_featured_video = context.load_current(&work_featured_video_id)?;
        WorkFeaturedVideoPolicy::can_delete(context, &work_featured_video)?;

        work_featured_video.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single abstract using its ID")]
    fn delete_abstract(
        context: &Context,
        #[graphql(description = "Thoth ID of abstract to be deleted")] abstract_id: Uuid,
    ) -> FieldResult<Abstract> {
        let r#abstract = context.load_current(&abstract_id)?;
        AbstractPolicy::can_delete(context, &r#abstract)?;

        r#abstract.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Delete a single biography using its ID")]
    fn delete_biography(
        context: &Context,
        #[graphql(description = "Thoth ID of biography to be deleted")] biography_id: Uuid,
    ) -> FieldResult<Biography> {
        let biography = context.load_current(&biography_id)?;
        BiographyPolicy::can_delete(context, &biography)?;

        biography.delete(&context.db).map_err(Into::into)
    }

    #[graphql(description = "Change the ordering of an affiliation within a contribution")]
    fn move_affiliation(
        context: &Context,
        #[graphql(description = "Thoth ID of affiliation to be moved")] affiliation_id: Uuid,
        #[graphql(
            description = "Ordinal representing position to which affiliation should be moved"
        )]
        new_ordinal: i32,
    ) -> FieldResult<Affiliation> {
        let affiliation = context.load_current(&affiliation_id)?;
        AffiliationPolicy::can_move(context, &affiliation)?;

        if new_ordinal == affiliation.affiliation_ordinal {
            // No action required
            return Ok(affiliation);
        }

        affiliation
            .change_ordinal(context, affiliation.affiliation_ordinal, new_ordinal)
            .map_err(Into::into)
    }

    #[graphql(description = "Change the ordering of a contribution within a work")]
    fn move_contribution(
        context: &Context,
        #[graphql(description = "Thoth ID of contribution to be moved")] contribution_id: Uuid,
        #[graphql(
            description = "Ordinal representing position to which contribution should be moved"
        )]
        new_ordinal: i32,
    ) -> FieldResult<Contribution> {
        let contribution = context.load_current(&contribution_id)?;
        ContributionPolicy::can_move(context, &contribution)?;

        if new_ordinal == contribution.contribution_ordinal {
            // No action required
            return Ok(contribution);
        }

        contribution
            .change_ordinal(context, contribution.contribution_ordinal, new_ordinal)
            .map_err(Into::into)
    }

    #[graphql(description = "Change the ordering of an issue within a series")]
    fn move_issue(
        context: &Context,
        #[graphql(description = "Thoth ID of issue to be moved")] issue_id: Uuid,
        #[graphql(description = "Ordinal representing position to which issue should be moved")]
        new_ordinal: i32,
    ) -> FieldResult<Issue> {
        let issue = context.load_current(&issue_id)?;
        IssuePolicy::can_move(context, &issue)?;

        if new_ordinal == issue.issue_ordinal {
            // No action required
            return Ok(issue);
        }

        issue
            .change_ordinal(context, issue.issue_ordinal, new_ordinal)
            .map_err(Into::into)
    }

    #[graphql(description = "Change the ordering of a reference within a work")]
    fn move_reference(
        context: &Context,
        #[graphql(description = "Thoth ID of reference to be moved")] reference_id: Uuid,
        #[graphql(
            description = "Ordinal representing position to which reference should be moved"
        )]
        new_ordinal: i32,
    ) -> FieldResult<Reference> {
        let reference = context.load_current(&reference_id)?;
        ReferencePolicy::can_move(context, &reference)?;

        if new_ordinal == reference.reference_ordinal {
            // No action required
            return Ok(reference);
        }

        reference
            .change_ordinal(context, reference.reference_ordinal, new_ordinal)
            .map_err(Into::into)
    }

    #[graphql(description = "Change the ordering of an additional resource within a work")]
    fn move_additional_resource(
        context: &Context,
        #[graphql(description = "Thoth ID of additional resource to be moved")]
        additional_resource_id: Uuid,
        #[graphql(
            description = "Ordinal representing position to which additional resource should be moved"
        )]
        new_ordinal: i32,
    ) -> FieldResult<AdditionalResource> {
        let additional_resource = context.load_current(&additional_resource_id)?;
        AdditionalResourcePolicy::can_move(context, &additional_resource)?;

        if new_ordinal == additional_resource.resource_ordinal {
            return Ok(additional_resource);
        }

        additional_resource
            .change_ordinal(context, additional_resource.resource_ordinal, new_ordinal)
            .map_err(Into::into)
    }

    #[graphql(description = "Change the ordering of an award within a work")]
    fn move_award(
        context: &Context,
        #[graphql(description = "Thoth ID of award to be moved")] award_id: Uuid,
        #[graphql(description = "Ordinal representing position to which award should be moved")]
        new_ordinal: i32,
    ) -> FieldResult<Award> {
        let award = context.load_current(&award_id)?;
        AwardPolicy::can_move(context, &award)?;

        if new_ordinal == award.award_ordinal {
            return Ok(award);
        }

        award
            .change_ordinal(context, award.award_ordinal, new_ordinal)
            .map_err(Into::into)
    }

    #[graphql(description = "Change the ordering of an endorsement within a work")]
    fn move_endorsement(
        context: &Context,
        #[graphql(description = "Thoth ID of endorsement to be moved")] endorsement_id: Uuid,
        #[graphql(
            description = "Ordinal representing position to which endorsement should be moved"
        )]
        new_ordinal: i32,
    ) -> FieldResult<Endorsement> {
        let endorsement = context.load_current(&endorsement_id)?;
        EndorsementPolicy::can_move(context, &endorsement)?;

        if new_ordinal == endorsement.endorsement_ordinal {
            return Ok(endorsement);
        }

        endorsement
            .change_ordinal(context, endorsement.endorsement_ordinal, new_ordinal)
            .map_err(Into::into)
    }

    #[graphql(description = "Change the ordering of a book review within a work")]
    fn move_book_review(
        context: &Context,
        #[graphql(description = "Thoth ID of book review to be moved")] book_review_id: Uuid,
        #[graphql(
            description = "Ordinal representing position to which book review should be moved"
        )]
        new_ordinal: i32,
    ) -> FieldResult<BookReview> {
        let book_review = context.load_current(&book_review_id)?;
        BookReviewPolicy::can_move(context, &book_review)?;

        if new_ordinal == book_review.review_ordinal {
            return Ok(book_review);
        }

        book_review
            .change_ordinal(context, book_review.review_ordinal, new_ordinal)
            .map_err(Into::into)
    }

    #[graphql(description = "Change the ordering of a subject within a work")]
    fn move_subject(
        context: &Context,
        #[graphql(description = "Thoth ID of subject to be moved")] subject_id: Uuid,
        #[graphql(description = "Ordinal representing position to which subject should be moved")]
        new_ordinal: i32,
    ) -> FieldResult<Subject> {
        let subject = context.load_current(&subject_id)?;
        SubjectPolicy::can_move(context, &subject)?;

        if new_ordinal == subject.subject_ordinal {
            // No action required
            return Ok(subject);
        }

        subject
            .change_ordinal(context, subject.subject_ordinal, new_ordinal)
            .map_err(Into::into)
    }

    #[graphql(description = "Change the ordering of a work relation within a work")]
    fn move_work_relation(
        context: &Context,
        #[graphql(description = "Thoth ID of work relation to be moved")] work_relation_id: Uuid,
        #[graphql(
            description = "Ordinal representing position to which work relation should be moved"
        )]
        new_ordinal: i32,
    ) -> FieldResult<WorkRelation> {
        let work_relation = context.load_current(&work_relation_id)?;
        WorkRelationPolicy::can_move(context, &work_relation)?;

        if new_ordinal == work_relation.relation_ordinal {
            // No action required
            return Ok(work_relation);
        }

        work_relation
            .change_ordinal(context, work_relation.relation_ordinal, new_ordinal)
            .map_err(Into::into)
    }

    #[graphql(
        description = "Start uploading a publication file (e.g. PDF, EPUB, XML) for a given publication. Returns an upload session ID, a presigned S3 PUT URL, and required PUT headers."
    )]
    async fn init_publication_file_upload(
        context: &Context,
        #[graphql(description = "Input for starting a publication file upload")]
        data: NewPublicationFileUpload,
    ) -> FieldResult<FileUploadResponse> {
        let publication: Publication = context.load_current(&data.publication_id)?;

        let new_upload: NewFileUpload = data.into();
        FilePolicy::can_create(context, &new_upload, Some(publication.publication_type))?;

        let work: Work = context.load_current(&publication.work_id)?;
        work.doi.ok_or(ThothError::WorkMissingDoiForFileUpload)?;

        let imprint: Imprint = context.load_current(&work.imprint_id)?;
        let storage_config = StorageConfig::from_imprint(&imprint)?;

        new_upload
            .create_upload_response(&context.db, context.s3_client(), &storage_config, 30)
            .await
            .map_err(Into::into)
    }

    #[graphql(
        description = "Start uploading a front cover image for a given work. Returns an upload session ID, a presigned S3 PUT URL, and required PUT headers."
    )]
    async fn init_frontcover_file_upload(
        context: &Context,
        #[graphql(description = "Input for starting a front cover upload")]
        data: NewFrontcoverFileUpload,
    ) -> FieldResult<FileUploadResponse> {
        let work: Work = context.load_current(&data.work_id)?;

        let new_upload: NewFileUpload = data.into();
        FilePolicy::can_create(context, &new_upload, None)?;

        work.doi.ok_or(ThothError::WorkMissingDoiForFileUpload)?;

        let imprint: Imprint = context.load_current(&work.imprint_id)?;
        let storage_config = StorageConfig::from_imprint(&imprint)?;

        new_upload
            .create_upload_response(&context.db, context.s3_client(), &storage_config, 30)
            .await
            .map_err(Into::into)
    }

    #[graphql(
        description = "Start uploading a file for an additional resource. Supported resource types include AUDIO, VIDEO, IMAGE, DOCUMENT, DATASET, and SPREADSHEET."
    )]
    async fn init_additional_resource_file_upload(
        context: &Context,
        #[graphql(description = "Input for starting an additional resource upload")]
        data: NewAdditionalResourceFileUpload,
    ) -> FieldResult<FileUploadResponse> {
        let additional_resource: AdditionalResource = context.load_current(&data.additional_resource_id)?;
        context.require_cdn_write_for(&additional_resource)?;

        let new_upload: NewFileUpload = data.into();
        FilePolicy::validate_resource_file_extension(
            &new_upload.declared_extension,
            additional_resource.resource_type,
        )?;
        FilePolicy::validate_resource_file_mime_type(
            additional_resource.resource_type,
            &new_upload.declared_mime_type,
        )?;

        let work: Work = context.load_current(&additional_resource.work_id)?;
        work.doi.ok_or(ThothError::WorkMissingDoiForFileUpload)?;

        let imprint: Imprint = context.load_current(&work.imprint_id)?;
        let storage_config = StorageConfig::from_imprint(&imprint)?;

        new_upload
            .create_upload_response(&context.db, context.s3_client(), &storage_config, 30)
            .await
            .map_err(Into::into)
    }

    #[graphql(
        description = "Start uploading a hosted featured video for a work. The uploaded file is promoted to a DOI-scoped resource path."
    )]
    async fn init_work_featured_video_file_upload(
        context: &Context,
        #[graphql(description = "Input for starting a featured video upload")]
        data: NewWorkFeaturedVideoFileUpload,
    ) -> FieldResult<FileUploadResponse> {
        let work_featured_video: WorkFeaturedVideo =
            context.load_current(&data.work_featured_video_id)?;
        context.require_cdn_write_for(&work_featured_video)?;

        let new_upload: NewFileUpload = data.into();
        FilePolicy::validate_resource_file_extension(
            &new_upload.declared_extension,
            ResourceType::Video,
        )?;
        FilePolicy::validate_resource_file_mime_type(
            ResourceType::Video,
            &new_upload.declared_mime_type,
        )?;

        let work: Work = context.load_current(&work_featured_video.work_id)?;
        work.doi.ok_or(ThothError::WorkMissingDoiForFileUpload)?;

        let imprint: Imprint = context.load_current(&work.imprint_id)?;
        let storage_config = StorageConfig::from_imprint(&imprint)?;

        new_upload
            .create_upload_response(&context.db, context.s3_client(), &storage_config, 30)
            .await
            .map_err(Into::into)
    }

    #[graphql(
        description = "Complete a file upload, validate it, and promote it to its final DOI-based location."
    )]
    async fn complete_file_upload(
        context: &Context,
        #[graphql(description = "Input for completing a file upload")] data: CompleteFileUpload,
    ) -> FieldResult<File> {
        let file_upload: FileUpload = context.load_current(&data.file_upload_id)?;
        FilePolicy::can_delete(context, &file_upload)?;

        let (work, publication, additional_resource, work_featured_video) =
            file_upload.load_scope(context)?;
        let doi = work
            .doi
            .as_ref()
            .ok_or(ThothError::WorkMissingDoiForFileUpload)?;

        let imprint: Imprint = context.load_current(&work.imprint_id)?;
        let storage_config = StorageConfig::from_imprint(&imprint)?;

        let s3_client = context.s3_client();
        let cloudfront_client = context.cloudfront_client();

        let temp_key = temp_key(&file_upload.file_upload_id);
        let (bytes, mime_type) =
            head_object(s3_client, &storage_config.s3_bucket, &temp_key).await?;
        match file_upload.file_type {
            crate::model::file::FileType::Frontcover | crate::model::file::FileType::Publication => {
                FilePolicy::can_complete_upload(
                    context,
                    &file_upload,
                    publication.as_ref().map(|pubn| pubn.publication_type),
                    bytes,
                    &mime_type,
                )?;
            }
            crate::model::file::FileType::AdditionalResource => {
                let resource_type = additional_resource
                    .as_ref()
                    .map(|resource| resource.resource_type)
                    .ok_or(ThothError::AdditionalResourceFileUploadMissingAdditionalResourceId)?;
                FilePolicy::can_complete_resource_upload(
                    context,
                    &file_upload,
                    resource_type,
                    bytes,
                    &mime_type,
                )?;
            }
            crate::model::file::FileType::WorkFeaturedVideo => {
                work_featured_video
                    .as_ref()
                    .ok_or(ThothError::WorkFeaturedVideoFileUploadMissingWorkFeaturedVideoId)?;
                FilePolicy::can_complete_resource_upload(
                    context,
                    &file_upload,
                    ResourceType::Video,
                    bytes,
                    &mime_type,
                )?;
            }
        }

        let canonical_key = file_upload.canonical_key(doi)?;

        copy_temp_object_to_final(
            s3_client,
            &storage_config.s3_bucket,
            &temp_key,
            &canonical_key,
        )
        .await?;

        let cdn_url = build_cdn_url(&storage_config.cdn_domain, &canonical_key);
        let (file, old_object_key) = file_upload.persist_file_record(
            context,
            &canonical_key,
            &cdn_url,
            &mime_type,
            bytes,
        )?;
        file_upload.sync_related_metadata(context, &work, &cdn_url)?;

        reconcile_replaced_object(
            s3_client,
            cloudfront_client,
            &storage_config.s3_bucket,
            &storage_config.cloudfront_dist_id,
            old_object_key.as_deref(),
            &canonical_key,
        )
        .await?;

        file_upload.clone().delete(&context.db)?;

        delete_object(s3_client, &storage_config.s3_bucket, &temp_key).await?;

        Ok(file)
    }

    #[graphql(description = "Delete a single contact using its ID")]
    fn delete_contact(
        context: &Context,
        #[graphql(description = "Thoth ID of contact to be deleted")] contact_id: Uuid,
    ) -> FieldResult<Contact> {
        let contact = context.load_current(&contact_id)?;
        ContactPolicy::can_delete(context, &contact)?;

        contact.delete(&context.db).map_err(Into::into)
    }
}
