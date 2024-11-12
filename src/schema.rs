// @generated automatically by Diesel CLI..

pub mod sql_types {
    #[derive(diesel::sql_types::*, diesel::query_builder::QueryId, crate::model::contribution::Contribution_type, crate::model::work::Work_type, crate::model::work::Work_status, crate::model::publication::Publication_type, crate::model::language::Language_relation, crate::model::language::Language_code, crate::model::series::Series_type, crate::model::price::Currency_code, crate::model::subject::Subject_type, crate::model::institution::Country_code, crate::model::work_relation::Relation_type, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "contribution_type"))]
    pub struct ContributionType;

    #[derive(diesel::sql_types::*, diesel::query_builder::QueryId, crate::model::contribution::Contribution_type, crate::model::work::Work_type, crate::model::work::Work_status, crate::model::publication::Publication_type, crate::model::language::Language_relation, crate::model::language::Language_code, crate::model::series::Series_type, crate::model::price::Currency_code, crate::model::subject::Subject_type, crate::model::institution::Country_code, crate::model::work_relation::Relation_type, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "country_code"))]
    pub struct CountryCode;

    #[derive(diesel::sql_types::*, diesel::query_builder::QueryId, crate::model::contribution::Contribution_type, crate::model::work::Work_type, crate::model::work::Work_status, crate::model::publication::Publication_type, crate::model::language::Language_relation, crate::model::language::Language_code, crate::model::series::Series_type, crate::model::price::Currency_code, crate::model::subject::Subject_type, crate::model::institution::Country_code, crate::model::work_relation::Relation_type, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "currency_code"))]
    pub struct CurrencyCode;

    #[derive(diesel::sql_types::*, diesel::query_builder::QueryId, crate::model::contribution::Contribution_type, crate::model::work::Work_type, crate::model::work::Work_status, crate::model::publication::Publication_type, crate::model::language::Language_relation, crate::model::language::Language_code, crate::model::series::Series_type, crate::model::price::Currency_code, crate::model::subject::Subject_type, crate::model::institution::Country_code, crate::model::work_relation::Relation_type, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "language_code"))]
    pub struct LanguageCode;

    #[derive(diesel::sql_types::*, diesel::query_builder::QueryId, crate::model::contribution::Contribution_type, crate::model::work::Work_type, crate::model::work::Work_status, crate::model::publication::Publication_type, crate::model::language::Language_relation, crate::model::language::Language_code, crate::model::series::Series_type, crate::model::price::Currency_code, crate::model::subject::Subject_type, crate::model::institution::Country_code, crate::model::work_relation::Relation_type, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "language_relation"))]
    pub struct LanguageRelation;

    #[derive(diesel::sql_types::*, diesel::query_builder::QueryId, crate::model::contribution::Contribution_type, crate::model::work::Work_type, crate::model::work::Work_status, crate::model::publication::Publication_type, crate::model::language::Language_relation, crate::model::language::Language_code, crate::model::series::Series_type, crate::model::price::Currency_code, crate::model::subject::Subject_type, crate::model::institution::Country_code, crate::model::work_relation::Relation_type, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "location_platform"))]
    pub struct LocationPlatform;

    #[derive(diesel::sql_types::*, diesel::query_builder::QueryId, crate::model::contribution::Contribution_type, crate::model::work::Work_type, crate::model::work::Work_status, crate::model::publication::Publication_type, crate::model::language::Language_relation, crate::model::language::Language_code, crate::model::series::Series_type, crate::model::price::Currency_code, crate::model::subject::Subject_type, crate::model::institution::Country_code, crate::model::work_relation::Relation_type, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "publication_type"))]
    pub struct PublicationType;

    #[derive(diesel::sql_types::*, diesel::query_builder::QueryId, crate::model::contribution::Contribution_type, crate::model::work::Work_type, crate::model::work::Work_status, crate::model::publication::Publication_type, crate::model::language::Language_relation, crate::model::language::Language_code, crate::model::series::Series_type, crate::model::price::Currency_code, crate::model::subject::Subject_type, crate::model::institution::Country_code, crate::model::work_relation::Relation_type, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "relation_type"))]
    pub struct RelationType;

    #[derive(diesel::sql_types::*, diesel::query_builder::QueryId, crate::model::contribution::Contribution_type, crate::model::work::Work_type, crate::model::work::Work_status, crate::model::publication::Publication_type, crate::model::language::Language_relation, crate::model::language::Language_code, crate::model::series::Series_type, crate::model::price::Currency_code, crate::model::subject::Subject_type, crate::model::institution::Country_code, crate::model::work_relation::Relation_type, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "series_type"))]
    pub struct SeriesType;

    #[derive(diesel::sql_types::*, diesel::query_builder::QueryId, crate::model::contribution::Contribution_type, crate::model::work::Work_type, crate::model::work::Work_status, crate::model::publication::Publication_type, crate::model::language::Language_relation, crate::model::language::Language_code, crate::model::series::Series_type, crate::model::price::Currency_code, crate::model::subject::Subject_type, crate::model::institution::Country_code, crate::model::work_relation::Relation_type, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "subject_type"))]
    pub struct SubjectType;

    #[derive(diesel::sql_types::*, diesel::query_builder::QueryId, crate::model::contribution::Contribution_type, crate::model::work::Work_type, crate::model::work::Work_status, crate::model::publication::Publication_type, crate::model::language::Language_relation, crate::model::language::Language_code, crate::model::series::Series_type, crate::model::price::Currency_code, crate::model::subject::Subject_type, crate::model::institution::Country_code, crate::model::work_relation::Relation_type, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "work_status"))]
    pub struct WorkStatus;

    #[derive(diesel::sql_types::*, diesel::query_builder::QueryId, crate::model::contribution::Contribution_type, crate::model::work::Work_type, crate::model::work::Work_status, crate::model::publication::Publication_type, crate::model::language::Language_relation, crate::model::language::Language_code, crate::model::series::Series_type, crate::model::price::Currency_code, crate::model::subject::Subject_type, crate::model::institution::Country_code, crate::model::work_relation::Relation_type, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "work_type"))]
    pub struct WorkType;
}

diesel::table! {
    account (account_id) {
        account_id -> Uuid,
        name -> Text,
        surname -> Text,
        email -> Text,
        hash -> Bytea,
        salt -> Text,
        is_superuser -> Bool,
        is_bot -> Bool,
        is_active -> Bool,
        created_at -> Timestamp,
        token -> Nullable<Text>,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    affiliation (affiliation_id) {
        affiliation_id -> Uuid,
        contribution_id -> Uuid,
        institution_id -> Uuid,
        affiliation_ordinal -> Int4,
        position -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    affiliation_history (affiliation_history_id) {
        affiliation_history_id -> Uuid,
        affiliation_id -> Uuid,
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ContributionType;

    contribution (contribution_id) {
        work_id -> Uuid,
        contributor_id -> Uuid,
        contribution_type -> ContributionType,
        main_contribution -> Bool,
        biography -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        first_name -> Nullable<Text>,
        last_name -> Text,
        full_name -> Text,
        contribution_id -> Uuid,
        contribution_ordinal -> Int4,
    }
}

diesel::table! {
    contribution_history (contribution_history_id) {
        contribution_history_id -> Uuid,
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamp,
        contribution_id -> Uuid,
    }
}

diesel::table! {
    contributor (contributor_id) {
        contributor_id -> Uuid,
        first_name -> Nullable<Text>,
        last_name -> Text,
        full_name -> Text,
        orcid -> Nullable<Text>,
        website -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    contributor_history (contributor_history_id) {
        contributor_history_id -> Uuid,
        contributor_id -> Uuid,
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    funding (funding_id) {
        funding_id -> Uuid,
        work_id -> Uuid,
        institution_id -> Uuid,
        program -> Nullable<Text>,
        project_name -> Nullable<Text>,
        project_shortname -> Nullable<Text>,
        grant_number -> Nullable<Text>,
        jurisdiction -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    funding_history (funding_history_id) {
        funding_history_id -> Uuid,
        funding_id -> Uuid,
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    imprint (imprint_id) {
        imprint_id -> Uuid,
        publisher_id -> Uuid,
        imprint_name -> Text,
        imprint_url -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        crossmark_doi -> Nullable<Text>,
    }
}

diesel::table! {
    imprint_history (imprint_history_id) {
        imprint_history_id -> Uuid,
        imprint_id -> Uuid,
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::CountryCode;

    institution (institution_id) {
        institution_id -> Uuid,
        institution_name -> Text,
        institution_doi -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        ror -> Nullable<Text>,
        country_code -> Nullable<CountryCode>,
    }
}

diesel::table! {
    institution_history (institution_history_id) {
        institution_history_id -> Uuid,
        institution_id -> Uuid,
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    issue (issue_id) {
        series_id -> Uuid,
        work_id -> Uuid,
        issue_ordinal -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        issue_id -> Uuid,
    }
}

diesel::table! {
    issue_history (issue_history_id) {
        issue_history_id -> Uuid,
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamp,
        issue_id -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::LanguageCode;
    use super::sql_types::LanguageRelation;

    language (language_id) {
        language_id -> Uuid,
        work_id -> Uuid,
        language_code -> LanguageCode,
        language_relation -> LanguageRelation,
        main_language -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    language_history (language_history_id) {
        language_history_id -> Uuid,
        language_id -> Uuid,
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::LocationPlatform;

    location (location_id) {
        location_id -> Uuid,
        publication_id -> Uuid,
        landing_page -> Nullable<Text>,
        full_text_url -> Nullable<Text>,
        location_platform -> LocationPlatform,
        canonical -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    location_history (location_history_id) {
        location_history_id -> Uuid,
        location_id -> Uuid,
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::CurrencyCode;

    price (price_id) {
        price_id -> Uuid,
        publication_id -> Uuid,
        currency_code -> CurrencyCode,
        unit_price -> Float8,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    price_history (price_history_id) {
        price_history_id -> Uuid,
        price_id -> Uuid,
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PublicationType;

    publication (publication_id) {
        publication_id -> Uuid,
        publication_type -> PublicationType,
        work_id -> Uuid,
        isbn -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        width_mm -> Nullable<Float8>,
        width_in -> Nullable<Float8>,
        height_mm -> Nullable<Float8>,
        height_in -> Nullable<Float8>,
        depth_mm -> Nullable<Float8>,
        depth_in -> Nullable<Float8>,
        weight_g -> Nullable<Float8>,
        weight_oz -> Nullable<Float8>,
    }
}

diesel::table! {
    publication_history (publication_history_id) {
        publication_history_id -> Uuid,
        publication_id -> Uuid,
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    publisher (publisher_id) {
        publisher_id -> Uuid,
        publisher_name -> Text,
        publisher_shortname -> Nullable<Text>,
        publisher_url -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    publisher_account (account_id, publisher_id) {
        account_id -> Uuid,
        publisher_id -> Uuid,
        is_admin -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    publisher_history (publisher_history_id) {
        publisher_history_id -> Uuid,
        publisher_id -> Uuid,
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamp,
    }
}

diesel::table! {
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
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    reference_history (reference_history_id) {
        reference_history_id -> Uuid,
        reference_id -> Uuid,
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamp,
    }
}

diesel::table! {
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
        created_at -> Timestamp,
        updated_at -> Timestamp,
        series_description -> Nullable<Text>,
        series_cfp_url -> Nullable<Text>,
    }
}

diesel::table! {
    series_history (series_history_id) {
        series_history_id -> Uuid,
        series_id -> Uuid,
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SubjectType;

    subject (subject_id) {
        subject_id -> Uuid,
        work_id -> Uuid,
        subject_type -> SubjectType,
        subject_code -> Text,
        subject_ordinal -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    subject_history (subject_history_id) {
        subject_history_id -> Uuid,
        subject_id -> Uuid,
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::WorkType;
    use super::sql_types::WorkStatus;

    work (work_id) {
        work_id -> Uuid,
        work_type -> WorkType,
        work_status -> WorkStatus,
        full_title -> Text,
        title -> Text,
        subtitle -> Nullable<Text>,
        reference -> Nullable<Text>,
        edition -> Nullable<Int4>,
        imprint_id -> Uuid,
        doi -> Nullable<Text>,
        publication_date -> Nullable<Date>,
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
        short_abstract -> Nullable<Text>,
        long_abstract -> Nullable<Text>,
        general_note -> Nullable<Text>,
        toc -> Nullable<Text>,
        cover_url -> Nullable<Text>,
        cover_caption -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        first_page -> Nullable<Text>,
        last_page -> Nullable<Text>,
        page_interval -> Nullable<Text>,
        updated_at_with_relations -> Timestamp,
        bibliography_note -> Nullable<Text>,
        withdrawn_date -> Nullable<Date>,
    }
}

diesel::table! {
    work_history (work_history_id) {
        work_history_id -> Uuid,
        work_id -> Uuid,
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RelationType;

    work_relation (work_relation_id) {
        work_relation_id -> Uuid,
        relator_work_id -> Uuid,
        related_work_id -> Uuid,
        relation_type -> RelationType,
        relation_ordinal -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    work_relation_history (work_relation_history_id) {
        work_relation_history_id -> Uuid,
        work_relation_id -> Uuid,
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamp,
    }
}

diesel::joinable!(affiliation -> contribution (contribution_id));
diesel::joinable!(affiliation -> institution (institution_id));
diesel::joinable!(affiliation_history -> account (account_id));
diesel::joinable!(affiliation_history -> affiliation (affiliation_id));
diesel::joinable!(contribution -> contributor (contributor_id));
diesel::joinable!(contribution -> work (work_id));
diesel::joinable!(contribution_history -> account (account_id));
diesel::joinable!(contribution_history -> contribution (contribution_id));
diesel::joinable!(contributor_history -> account (account_id));
diesel::joinable!(contributor_history -> contributor (contributor_id));
diesel::joinable!(funding -> institution (institution_id));
diesel::joinable!(funding -> work (work_id));
diesel::joinable!(funding_history -> account (account_id));
diesel::joinable!(funding_history -> funding (funding_id));
diesel::joinable!(imprint -> publisher (publisher_id));
diesel::joinable!(imprint_history -> account (account_id));
diesel::joinable!(imprint_history -> imprint (imprint_id));
diesel::joinable!(institution_history -> account (account_id));
diesel::joinable!(institution_history -> institution (institution_id));
diesel::joinable!(issue -> series (series_id));
diesel::joinable!(issue -> work (work_id));
diesel::joinable!(issue_history -> account (account_id));
diesel::joinable!(issue_history -> issue (issue_id));
diesel::joinable!(language -> work (work_id));
diesel::joinable!(language_history -> account (account_id));
diesel::joinable!(language_history -> language (language_id));
diesel::joinable!(location -> publication (publication_id));
diesel::joinable!(location_history -> account (account_id));
diesel::joinable!(location_history -> location (location_id));
diesel::joinable!(price -> publication (publication_id));
diesel::joinable!(price_history -> account (account_id));
diesel::joinable!(price_history -> price (price_id));
diesel::joinable!(publication -> work (work_id));
diesel::joinable!(publication_history -> account (account_id));
diesel::joinable!(publication_history -> publication (publication_id));
diesel::joinable!(publisher_account -> account (account_id));
diesel::joinable!(publisher_account -> publisher (publisher_id));
diesel::joinable!(publisher_history -> account (account_id));
diesel::joinable!(publisher_history -> publisher (publisher_id));
diesel::joinable!(reference -> work (work_id));
diesel::joinable!(reference_history -> account (account_id));
diesel::joinable!(reference_history -> reference (reference_id));
diesel::joinable!(series -> imprint (imprint_id));
diesel::joinable!(series_history -> account (account_id));
diesel::joinable!(series_history -> series (series_id));
diesel::joinable!(subject -> work (work_id));
diesel::joinable!(subject_history -> account (account_id));
diesel::joinable!(subject_history -> subject (subject_id));
diesel::joinable!(work -> imprint (imprint_id));
diesel::joinable!(work_history -> account (account_id));
diesel::joinable!(work_history -> work (work_id));
diesel::joinable!(work_relation_history -> account (account_id));
diesel::joinable!(work_relation_history -> work_relation (work_relation_id));

diesel::allow_tables_to_appear_in_same_query!(
    account,
    affiliation,
    affiliation_history,
    contribution,
    contribution_history,
    contributor,
    contributor_history,
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
    publisher_account,
    publisher_history,
    reference,
    reference_history,
    series,
    series_history,
    subject,
    subject_history,
    work,
    work_history,
    work_relation,
    work_relation_history,
);
