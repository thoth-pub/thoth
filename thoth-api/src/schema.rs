pub mod sql_types {
    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "award_role"))]
    pub struct AwardRole;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "contribution_type"))]
    pub struct ContributionType;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "country_code"))]
    pub struct CountryCode;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "language_relation"))]
    pub struct LanguageRelation;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "language_code"))]
    pub struct LanguageCode;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "location_platform"))]
    pub struct LocationPlatform;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "currency_code"))]
    pub struct CurrencyCode;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "publication_type"))]
    pub struct PublicationType;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "series_type"))]
    pub struct SeriesType;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "subject_type"))]
    pub struct SubjectType;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "work_type"))]
    pub struct WorkType;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "work_status"))]
    pub struct WorkStatus;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "relation_type"))]
    pub struct RelationType;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "resource_type"))]
    pub struct ResourceType;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "locale_code"))]
    pub struct LocaleCode;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "abstract_type"))]
    pub struct AbstractType;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "markup_format"))]
    pub struct MarkupFormat;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "file_type"))]
    pub struct FileType;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "contact_type"))]
    pub struct ContactType;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "accessibility_standard"))]
    pub struct AccessibilityStandard;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "accessibility_exception"))]
    pub struct AccessibilityException;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "checksum_algorithm"))]
    pub struct ChecksumAlgorithm;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "thoth_package"))]
    pub struct ThothPackage;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "distribution_platform"))]
    pub struct DistributionPlatform;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "publisher_service_configuration_source"))]
    pub struct PublisherServiceConfigurationSource;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "distribution_job_kind"))]
    pub struct DistributionJobKind;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "distribution_job_status"))]
    pub struct DistributionJobStatus;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "distribution_job_attempt_result"))]
    pub struct DistributionJobAttemptResult;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "distribution_job_cancellation_reason"))]
    pub struct DistributionJobCancellationReason;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "crossref_write_route"))]
    pub struct CrossrefWriteRoute;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "crossref_write_permit_state"))]
    pub struct CrossrefWritePermitState;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "crossref_reconciliation_state"))]
    pub struct CrossrefReconciliationState;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "crossref_write_scope"))]
    pub struct CrossrefWriteScope;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "crossref_void_reason"))]
    pub struct CrossrefVoidReason;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "xid8"))]
    pub struct Xid8;
}

use diesel::{allow_tables_to_appear_in_same_query, joinable, table};

table! {
    use diesel::sql_types::*;
    use super::sql_types::{LocaleCode, MarkupFormat, AbstractType};

    #[sql_name = "abstract"]
    work_abstract (abstract_id) {
        abstract_id -> Uuid,
        work_id -> Uuid,
        content -> Text,
        locale_code -> LocaleCode,
        abstract_type -> AbstractType,
        canonical -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::ResourceType;

    additional_resource (additional_resource_id) {
        additional_resource_id -> Uuid,
        work_id -> Uuid,
        title -> Text,
        description -> Nullable<Text>,
        attribution -> Nullable<Text>,
        resource_type -> ResourceType,
        doi -> Nullable<Text>,
        handle -> Nullable<Text>,
        url -> Nullable<Text>,
        date -> Nullable<Date>,
        resource_ordinal -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    additional_resource_history (additional_resource_history_id) {
        additional_resource_history_id -> Uuid,
        additional_resource_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    affiliation (affiliation_id) {
        affiliation_id -> Uuid,
        contribution_id -> Uuid,
        institution_id -> Uuid,
        affiliation_ordinal -> Int4,
        position -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::LocaleCode;

    biography (biography_id) {
        biography_id -> Uuid,
        contribution_id -> Uuid,
        content -> Text,
        canonical -> Bool,
        locale_code -> LocaleCode,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::{AwardRole, CountryCode};

    award (award_id) {
        award_id -> Uuid,
        work_id -> Uuid,
        title -> Text,
        url -> Nullable<Text>,
        category -> Nullable<Text>,
        year -> Nullable<Text>,
        jury -> Nullable<Text>,
        country -> Nullable<CountryCode>,
        prize_statement -> Nullable<Text>,
        role -> Nullable<AwardRole>,
        award_ordinal -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    award_history (award_history_id) {
        award_history_id -> Uuid,
        award_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    affiliation_history (affiliation_history_id) {
        affiliation_history_id -> Uuid,
        affiliation_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::ContactType;

    contact (contact_id) {
        contact_id -> Uuid,
        publisher_id -> Uuid,
        contact_type -> ContactType,
        email -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    contact_history (contact_history_id) {
        contact_history_id -> Uuid,
        contact_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::ContributionType;

    contribution (contribution_id) {
        contribution_id -> Uuid,
        work_id -> Uuid,
        contributor_id -> Uuid,
        contribution_type -> ContributionType,
        main_contribution -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        first_name -> Nullable<Text>,
        last_name -> Text,
        full_name -> Text,
        contribution_ordinal -> Int4,
    }
}

table! {
    use diesel::sql_types::*;

    contribution_history (contribution_history_id) {
        contribution_history_id -> Uuid,
        contribution_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    contributor (contributor_id) {
        contributor_id -> Uuid,
        first_name -> Nullable<Text>,
        last_name -> Text,
        full_name -> Text,
        orcid -> Nullable<Text>,
        website -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    contributor_history (contributor_history_id) {
        contributor_history_id -> Uuid,
        contributor_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    book_review (book_review_id) {
        book_review_id -> Uuid,
        work_id -> Uuid,
        title -> Nullable<Text>,
        author_name -> Nullable<Text>,
        reviewer_orcid -> Nullable<Text>,
        reviewer_institution_id -> Nullable<Uuid>,
        url -> Nullable<Text>,
        doi -> Nullable<Text>,
        review_date -> Nullable<Date>,
        journal_name -> Nullable<Text>,
        journal_volume -> Nullable<Text>,
        journal_number -> Nullable<Text>,
        journal_issn -> Nullable<Text>,
        page_range -> Nullable<Text>,
        text -> Nullable<Text>,
        review_ordinal -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    book_review_history (book_review_history_id) {
        book_review_history_id -> Uuid,
        book_review_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::{
        DistributionJobCancellationReason, DistributionJobKind, DistributionJobStatus,
        DistributionPlatform,
    };

    distribution_job (distribution_job_id) {
        distribution_job_id -> Uuid,
        kind -> DistributionJobKind,
        publisher_id -> Uuid,
        work_id -> Nullable<Uuid>,
        activation_id -> Uuid,
        status -> DistributionJobStatus,
        deduplication_key -> Text,
        attempt_count -> Int4,
        available_at -> Timestamptz,
        claim_token -> Nullable<Uuid>,
        claimed_by -> Nullable<Text>,
        claimed_at -> Nullable<Timestamptz>,
        lease_expires_at -> Nullable<Timestamptz>,
        completed_at -> Nullable<Timestamptz>,
        cancellation_reason -> Nullable<DistributionJobCancellationReason>,
        last_error_code -> Nullable<Text>,
        last_error_detail -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        execution_profile -> Nullable<DistributionPlatform>,
        work_identity -> Nullable<Uuid>,
        created_generation -> Nullable<Int8>,
        job_ordinal -> Nullable<Int4>,
        predecessor_job_id -> Nullable<Uuid>,
        superseded_by_job_id -> Nullable<Uuid>,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::DistributionJobAttemptResult;

    distribution_job_attempt (distribution_job_attempt_id) {
        distribution_job_attempt_id -> Uuid,
        distribution_job_id -> Uuid,
        attempt_number -> Int4,
        claim_token -> Uuid,
        claimed_by -> Text,
        started_at -> Timestamptz,
        finished_at -> Nullable<Timestamptz>,
        result -> Nullable<DistributionJobAttemptResult>,
        error_code -> Nullable<Text>,
        error_detail -> Nullable<Text>,
        claimed_generation -> Nullable<Int8>,
        fenced_at -> Nullable<Timestamptz>,
        recovery_cleared_at -> Nullable<Timestamptz>,
        recovery_clearance_reference -> Nullable<Text>,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::DistributionPlatform;

    distribution_job_target (distribution_job_id, platform) {
        distribution_job_id -> Uuid,
        platform -> DistributionPlatform,
        created_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    endorsement (endorsement_id) {
        endorsement_id -> Uuid,
        work_id -> Uuid,
        author_name -> Text,
        author_role -> Nullable<Text>,
        author_orcid -> Nullable<Text>,
        author_institution_id -> Nullable<Uuid>,
        url -> Nullable<Text>,
        text -> Nullable<Text>,
        endorsement_ordinal -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    endorsement_history (endorsement_history_id) {
        endorsement_history_id -> Uuid,
        endorsement_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    funding (funding_id) {
        funding_id -> Uuid,
        work_id -> Uuid,
        institution_id -> Uuid,
        program -> Nullable<Text>,
        project_name -> Nullable<Text>,
        project_shortname -> Nullable<Text>,
        grant_number -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    funding_history (funding_history_id) {
        funding_history_id -> Uuid,
        funding_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::CurrencyCode;
    use super::sql_types::LocaleCode;

    imprint (imprint_id) {
        imprint_id -> Uuid,
        publisher_id -> Uuid,
        imprint_name -> Text,
        imprint_url -> Nullable<Text>,
        crossmark_doi -> Nullable<Text>,
        s3_bucket -> Nullable<Text>,
        cdn_domain -> Nullable<Text>,
        cloudfront_dist_id -> Nullable<Text>,
        default_currency -> Nullable<CurrencyCode>,
        default_place -> Nullable<Text>,
        default_locale -> Nullable<LocaleCode>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    imprint_history (imprint_history_id) {
        imprint_history_id -> Uuid,
        imprint_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
use diesel::sql_types::*;
    use super::sql_types::CountryCode;

    institution (institution_id) {
        institution_id -> Uuid,
        institution_name -> Text,
        institution_doi -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        ror -> Nullable<Text>,
        country_code -> Nullable<CountryCode>,
    }
}

table! {
    use diesel::sql_types::*;

    institution_history (institution_history_id) {
        institution_history_id -> Uuid,
        institution_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    issue (issue_id) {
        issue_id -> Uuid,
        series_id -> Uuid,
        work_id -> Uuid,
        issue_ordinal -> Int4,
        issue_number -> Nullable<Int4>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    issue_history (issue_history_id) {
        issue_history_id -> Uuid,
        issue_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::LanguageRelation;
    use super::sql_types::LanguageCode;

    language (language_id) {
        language_id -> Uuid,
        work_id -> Uuid,
        language_code -> LanguageCode,
        language_relation -> LanguageRelation,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    language_history (language_history_id) {
        language_history_id -> Uuid,
        language_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::ChecksumAlgorithm;
    use super::sql_types::LocationPlatform;

    location (location_id) {
        location_id -> Uuid,
        publication_id -> Uuid,
        landing_page -> Nullable<Text>,
        full_text_url -> Nullable<Text>,
        location_platform -> LocationPlatform,
        canonical -> Bool,
        checksum -> Nullable<Text>,
        checksum_algorithm -> Nullable<ChecksumAlgorithm>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    location_history (location_history_id) {
        location_history_id -> Uuid,
        location_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::CurrencyCode;

    price (price_id) {
        price_id -> Uuid,
        publication_id -> Uuid,
        currency_code -> CurrencyCode,
        unit_price -> Float8,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    price_history (price_history_id) {
        price_history_id -> Uuid,
        price_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::PublicationType;
    use super::sql_types::AccessibilityStandard;
    use super::sql_types::AccessibilityException;

    publication (publication_id) {
        publication_id -> Uuid,
        publication_type -> PublicationType,
        work_id -> Uuid,
        isbn -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        width_mm -> Nullable<Float8>,
        width_in -> Nullable<Float8>,
        height_mm -> Nullable<Float8>,
        height_in -> Nullable<Float8>,
        depth_mm -> Nullable<Float8>,
        depth_in -> Nullable<Float8>,
        weight_g -> Nullable<Float8>,
        weight_oz -> Nullable<Float8>,
        accessibility_standard -> Nullable<AccessibilityStandard>,
        accessibility_additional_standard -> Nullable<AccessibilityStandard>,
        accessibility_exception -> Nullable<AccessibilityException>,
        accessibility_report_url -> Nullable<Text>,
    }
}

table! {
    use diesel::sql_types::*;

    publication_history (publication_history_id) {
        publication_history_id -> Uuid,
        publication_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::ThothPackage;

    publisher (publisher_id) {
        publisher_id -> Uuid,
        publisher_name -> Text,
        publisher_shortname -> Nullable<Text>,
        publisher_url -> Nullable<Text>,
        zitadel_id -> Nullable<Text>,
        accessibility_statement -> Nullable<Text>,
        accessibility_report_url -> Nullable<Text>,
        subscription_package -> ThothPackage,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        service_configuration_updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::DistributionPlatform;

    publisher_distribution_platform (publisher_id, platform) {
        publisher_id -> Uuid,
        platform -> DistributionPlatform,
        enabled -> Bool,
        activation_id -> Uuid,
        enabled_at -> Timestamptz,
        disabled_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::PublisherServiceConfigurationSource;

    publisher_service_configuration_history (publisher_service_configuration_history_id) {
        publisher_service_configuration_history_id -> Uuid,
        publisher_id -> Uuid,
        actor -> Text,
        source -> PublisherServiceConfigurationSource,
        before_state -> Jsonb,
        after_state -> Jsonb,
        created_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    publisher_history (publisher_history_id) {
        publisher_history_id -> Uuid,
        publisher_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    reference (reference_id) {
        reference_id -> Uuid,
        work_id -> Uuid,
        reference_ordinal -> Int4,
        doi -> Nullable<Text>,
        unstructured_citation -> Nullable<Text>,
        issn -> Nullable<Text>,
        isbn -> Nullable<Text>,
        journal_title -> Nullable<Text>,
        article_title -> Nullable<Text>,
        series_title -> Nullable<Text>,
        volume_title -> Nullable<Text>,
        edition -> Nullable<Int4>,
        author -> Nullable<Text>,
        volume -> Nullable<Text>,
        issue -> Nullable<Text>,
        first_page -> Nullable<Text>,
        component_number -> Nullable<Text>,
        standard_designator -> Nullable<Text>,
        standards_body_name -> Nullable<Text>,
        standards_body_acronym -> Nullable<Text>,
        url -> Nullable<Text>,
        publication_date -> Nullable<Date>,
        retrieval_date -> Nullable<Date>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    reference_history (reference_history_id) {
        reference_history_id -> Uuid,
        reference_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::SeriesType;

    series (series_id) {
        series_id -> Uuid,
        series_type -> SeriesType,
        series_name -> Text,
        issn_print -> Nullable<Text>,
        issn_digital -> Nullable<Text>,
        series_url -> Nullable<Text>,
        imprint_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        series_description -> Nullable<Text>,
        series_cfp_url -> Nullable<Text>,
    }
}

table! {
    use diesel::sql_types::*;

    series_history (series_history_id) {
        series_history_id -> Uuid,
        series_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::SubjectType;

    subject (subject_id) {
        subject_id -> Uuid,
        work_id -> Uuid,
        subject_type -> SubjectType,
        subject_code -> Text,
        subject_ordinal -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    subject_history (subject_history_id) {
        subject_history_id -> Uuid,
        subject_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::WorkType;
    use super::sql_types::WorkStatus;

    work (work_id) {
        work_id -> Uuid,
        work_type -> WorkType,
        work_status -> WorkStatus,
        reference -> Nullable<Text>,
        edition -> Nullable<Int4>,
        imprint_id -> Uuid,
        doi -> Nullable<Text>,
        publication_date -> Nullable<Date>,
        withdrawn_date -> Nullable<Date>,
        place -> Nullable<Text>,
        page_count -> Nullable<Int4>,
        page_breakdown -> Nullable<Text>,
        image_count -> Nullable<Int4>,
        table_count -> Nullable<Int4>,
        audio_count -> Nullable<Int4>,
        video_count -> Nullable<Int4>,
        license -> Nullable<Text>,
        copyright_holder -> Nullable<Text>,
        landing_page -> Nullable<Text>,
        lccn -> Nullable<Text>,
        oclc -> Nullable<Text>,
        general_note -> Nullable<Text>,
        bibliography_note -> Nullable<Text>,
        toc -> Nullable<Text>,
        resources_description -> Nullable<Text>,
        cover_url -> Nullable<Text>,
        cover_caption -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        first_page -> Nullable<Text>,
        last_page -> Nullable<Text>,
        page_interval -> Nullable<Text>,
        updated_at_with_relations -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    work_history (work_history_id) {
        work_history_id -> Uuid,
        work_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    work_featured_video (work_featured_video_id) {
        work_featured_video_id -> Uuid,
        work_id -> Uuid,
        title -> Text,
        url -> Nullable<Text>,
        width -> Int4,
        height -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    work_featured_video_history (work_featured_video_history_id) {
        work_featured_video_history_id -> Uuid,
        work_featured_video_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::RelationType;

    work_relation (work_relation_id) {
        work_relation_id -> Uuid,
        relator_work_id -> Uuid,
        related_work_id -> Uuid,
        relation_type -> RelationType,
        relation_ordinal -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    work_relation_history (work_relation_history_id) {
        work_relation_history_id -> Uuid,
        work_relation_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::LocaleCode;
    use super::sql_types::MarkupFormat;

    #[sql_name = "title"]
    work_title (title_id) {
        title_id -> Uuid,
        work_id -> Uuid,
        full_title -> Text,
        title -> Text,
        subtitle -> Nullable<Text>,
        canonical -> Bool,
        locale_code -> LocaleCode,
    }
}

table! {
    use diesel::sql_types::*;

    title_history (title_history_id) {
        title_history_id -> Uuid,
        title_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::FileType;

    file (file_id) {
        file_id -> Uuid,
        file_type -> FileType,
        work_id -> Nullable<Uuid>,
        publication_id -> Nullable<Uuid>,
        additional_resource_id -> Nullable<Uuid>,
        work_featured_video_id -> Nullable<Uuid>,
        object_key -> Text,
        cdn_url -> Text,
        mime_type -> Text,
        bytes -> Int8,
        sha256 -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::FileType;

    file_upload (file_upload_id) {
        file_upload_id -> Uuid,
        file_type -> FileType,
        work_id -> Nullable<Uuid>,
        publication_id -> Nullable<Uuid>,
        additional_resource_id -> Nullable<Uuid>,
        work_featured_video_id -> Nullable<Uuid>,
        declared_mime_type -> Text,
        declared_extension -> Text,
        declared_sha256 -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    abstract_history (abstract_history_id) {
        abstract_history_id -> Uuid,
        abstract_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    biography_history (biography_history_id) {
        biography_history_id -> Uuid,
        biography_id -> Uuid,
        user_id -> Text,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::{
        CrossrefReconciliationState, CrossrefVoidReason, CrossrefWritePermitState,
        CrossrefWriteRoute, CrossrefWriteScope,
    };

    crossref_write_permit (permit_id) {
        permit_id -> Uuid,
        route -> CrossrefWriteRoute,
        scope -> CrossrefWriteScope,
        state -> CrossrefWritePermitState,
        reconciliation_state -> Nullable<CrossrefReconciliationState>,
        reservation_token -> Uuid,
        publisher_id -> Nullable<Uuid>,
        publisher_identity -> Uuid,
        root_work_identity -> Uuid,
        distribution_job_id -> Nullable<Uuid>,
        distribution_job_attempt_id -> Nullable<Uuid>,
        job_identity -> Nullable<Uuid>,
        attempt_identity -> Nullable<Uuid>,
        permit_generation -> Nullable<Int8>,
        source_generation_witness -> Int8,
        doi_set_digest -> Text,
        doi_set_cardinality -> Int4,
        crossref_timestamp -> Int8,
        doi_batch_id -> Text,
        payload_digest -> Nullable<Text>,
        operator_authorization_reference -> Nullable<Text>,
        reconciliation_annotation_reference -> Nullable<Text>,
        reconciliation_authorization_reference -> Nullable<Text>,
        void_reason -> Nullable<CrossrefVoidReason>,
        void_detail -> Nullable<Text>,
        void_authorization_reference -> Nullable<Text>,
        issued_at -> Timestamptz,
        authorized_at -> Nullable<Timestamptz>,
        provider_reported_at -> Nullable<Timestamptz>,
        reconciliation_annotated_at -> Nullable<Timestamptz>,
        reconciled_at -> Nullable<Timestamptz>,
        closed_at -> Nullable<Timestamptz>,
    }
}

table! {
    use diesel::sql_types::*;

    crossref_write_permit_doi (permit_id, doi) {
        permit_id -> Uuid,
        doi -> Text,
    }
}

table! {
    use diesel::sql_types::*;

    crossref_version_floor_audit (audit_id) {
        audit_id -> Uuid,
        mutation_kind -> Text,
        before_value -> Nullable<Int8>,
        after_value -> Nullable<Int8>,
        g6_attempt_id -> Nullable<Uuid>,
        observation_id -> Nullable<Uuid>,
        g7_authorization_reference -> Nullable<Text>,
        authorization_register_digest -> Nullable<Text>,
        actor -> Text,
        occurred_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    work_crossref_version_floor (floor_id) {
        floor_id -> Bool,
        floor_value -> Int8,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::DistributionPlatform;

    work_upsert_admission (execution_profile, publisher_id, activation_id) {
        execution_profile -> DistributionPlatform,
        publisher_id -> Uuid,
        activation_id -> Uuid,
        evidence_reference -> Text,
        admitted_at -> Timestamptz,
        actor -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::Xid8;

    work_upsert_capture_queue (entry_id) {
        entry_id -> Int8,
        txid -> Xid8,
        entry_kind -> Text,
        work_ids -> Nullable<Array<Uuid>>,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::DistributionPlatform;

    work_upsert_control (execution_profile) {
        execution_profile -> DistributionPlatform,
        capture_enabled -> Bool,
        execution_enabled -> Bool,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::DistributionPlatform;

    work_upsert_generation (work_id, execution_profile) {
        work_id -> Uuid,
        execution_profile -> DistributionPlatform,
        source_generation -> Int8,
        updated_at -> Timestamptz,
    }
}

joinable!(abstract_history -> work_abstract (abstract_id));
joinable!(additional_resource -> work (work_id));
joinable!(additional_resource_history -> additional_resource (additional_resource_id));
joinable!(affiliation -> contribution (contribution_id));
joinable!(affiliation -> institution (institution_id));
joinable!(affiliation_history -> affiliation (affiliation_id));
joinable!(award -> work (work_id));
joinable!(award_history -> award (award_id));
joinable!(biography_history -> biography (biography_id));
joinable!(book_review -> institution (reviewer_institution_id));
joinable!(book_review -> work (work_id));
joinable!(book_review_history -> book_review (book_review_id));
joinable!(contact -> publisher (publisher_id));
joinable!(contact_history -> contact (contact_id));
joinable!(contribution -> contributor (contributor_id));
joinable!(contribution -> work (work_id));
joinable!(contribution_history -> contribution (contribution_id));
joinable!(contributor_history -> contributor (contributor_id));
joinable!(crossref_write_permit -> distribution_job (distribution_job_id));
joinable!(crossref_write_permit -> distribution_job_attempt (distribution_job_attempt_id));
joinable!(crossref_write_permit -> publisher (publisher_id));
joinable!(crossref_write_permit_doi -> crossref_write_permit (permit_id));
joinable!(distribution_job -> publisher (publisher_id));
joinable!(distribution_job -> work (work_id));
joinable!(distribution_job_attempt -> distribution_job (distribution_job_id));
joinable!(distribution_job_target -> distribution_job (distribution_job_id));
joinable!(endorsement -> institution (author_institution_id));
joinable!(endorsement -> work (work_id));
joinable!(endorsement_history -> endorsement (endorsement_id));
joinable!(file -> work (work_id));
joinable!(file -> publication (publication_id));
joinable!(file -> additional_resource (additional_resource_id));
joinable!(file -> work_featured_video (work_featured_video_id));
joinable!(file_upload -> work (work_id));
joinable!(file_upload -> publication (publication_id));
joinable!(file_upload -> additional_resource (additional_resource_id));
joinable!(file_upload -> work_featured_video (work_featured_video_id));
joinable!(funding -> institution (institution_id));
joinable!(funding -> work (work_id));
joinable!(funding_history -> funding (funding_id));
joinable!(imprint -> publisher (publisher_id));
joinable!(imprint_history -> imprint (imprint_id));
joinable!(institution_history -> institution (institution_id));
joinable!(issue -> series (series_id));
joinable!(issue -> work (work_id));
joinable!(issue_history -> issue (issue_id));
joinable!(language -> work (work_id));
joinable!(language_history -> language (language_id));
joinable!(location -> publication (publication_id));
joinable!(location_history -> location (location_id));
joinable!(price -> publication (publication_id));
joinable!(price_history -> price (price_id));
joinable!(publication -> work (work_id));
joinable!(publication_history -> publication (publication_id));
joinable!(publisher_distribution_platform -> publisher (publisher_id));
joinable!(publisher_history -> publisher (publisher_id));
joinable!(publisher_service_configuration_history -> publisher (publisher_id));
joinable!(reference -> work (work_id));
joinable!(reference_history -> reference (reference_id));
joinable!(series -> imprint (imprint_id));
joinable!(series_history -> series (series_id));
joinable!(subject -> work (work_id));
joinable!(subject_history -> subject (subject_id));
joinable!(title_history -> work_title (title_id));
joinable!(work -> imprint (imprint_id));
joinable!(work_abstract -> work (work_id));
joinable!(work_history -> work (work_id));
joinable!(work_featured_video -> work (work_id));
joinable!(work_featured_video_history -> work_featured_video (work_featured_video_id));
joinable!(work_relation -> work (relator_work_id));
joinable!(work_relation_history -> work_relation (work_relation_id));
joinable!(work_title -> work (work_id));
joinable!(work_upsert_admission -> publisher (publisher_id));

allow_tables_to_appear_in_same_query!(
    abstract_history,
    additional_resource,
    additional_resource_history,
    affiliation,
    affiliation_history,
    award,
    award_history,
    biography,
    biography_history,
    book_review,
    book_review_history,
    contact,
    contact_history,
    contribution,
    contribution_history,
    contributor,
    contributor_history,
    crossref_version_floor_audit,
    crossref_write_permit,
    crossref_write_permit_doi,
    distribution_job,
    distribution_job_attempt,
    distribution_job_target,
    endorsement,
    endorsement_history,
    file,
    file_upload,
    funding,
    funding_history,
    imprint,
    imprint_history,
    institution,
    institution_history,
    issue,
    issue_history,
    language,
    language_history,
    location,
    location_history,
    price,
    price_history,
    publication,
    publication_history,
    publisher,
    publisher_distribution_platform,
    publisher_history,
    publisher_service_configuration_history,
    reference,
    reference_history,
    series,
    series_history,
    subject,
    subject_history,
    title_history,
    work,
    work_abstract,
    work_history,
    work_featured_video,
    work_featured_video_history,
    work_relation,
    work_relation_history,
    work_title,
    work_crossref_version_floor,
    work_upsert_admission,
    work_upsert_capture_queue,
    work_upsert_control,
    work_upsert_generation,
);
