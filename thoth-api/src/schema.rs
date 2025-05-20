pub mod sql_types {
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
    #[diesel(postgres_type(name = "locale_code"))]
    pub struct LocaleCode;
}

table! {
    use diesel::sql_types::*;

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
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        token -> Nullable<Text>,
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

    affiliation_history (affiliation_history_id) {
        affiliation_history_id -> Uuid,
        affiliation_id -> Uuid,
        account_id -> Uuid,
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
        biography -> Nullable<Text>,
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
        account_id -> Uuid,
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
        account_id -> Uuid,
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
        jurisdiction -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    funding_history (funding_history_id) {
        funding_history_id -> Uuid,
        funding_id -> Uuid,
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    imprint (imprint_id) {
        imprint_id -> Uuid,
        publisher_id -> Uuid,
        imprint_name -> Text,
        imprint_url -> Nullable<Text>,
        crossmark_doi -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    imprint_history (imprint_history_id) {
        imprint_history_id -> Uuid,
        imprint_id -> Uuid,
        account_id -> Uuid,
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
        account_id -> Uuid,
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
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    issue_history (issue_history_id) {
        issue_history_id -> Uuid,
        issue_id -> Uuid,
        account_id -> Uuid,
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
        main_language -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    language_history (language_history_id) {
        language_history_id -> Uuid,
        language_id -> Uuid,
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::LocationPlatform;

    location (location_id) {
        location_id -> Uuid,
        publication_id -> Uuid,
        landing_page -> Nullable<Text>,
        full_text_url -> Nullable<Text>,
        location_platform -> LocationPlatform,
        canonical -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    location_history (location_history_id) {
        location_history_id -> Uuid,
        location_id -> Uuid,
        account_id -> Uuid,
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
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::PublicationType;

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
    }
}

table! {
    use diesel::sql_types::*;

    publication_history (publication_history_id) {
        publication_history_id -> Uuid,
        publication_id -> Uuid,
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    publisher (publisher_id) {
        publisher_id -> Uuid,
        publisher_name -> Text,
        publisher_shortname -> Nullable<Text>,
        publisher_url -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    publisher_account (account_id, publisher_id) {
        account_id -> Uuid,
        publisher_id -> Uuid,
        is_admin -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    publisher_history (publisher_history_id) {
        publisher_history_id -> Uuid,
        publisher_id -> Uuid,
        account_id -> Uuid,
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
        account_id -> Uuid,
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
        account_id -> Uuid,
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
        account_id -> Uuid,
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
        // full_title -> Text,
        // title -> Text,
        // subtitle -> Nullable<Text>,
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
        short_abstract -> Nullable<Text>,
        long_abstract -> Nullable<Text>,
        general_note -> Nullable<Text>,
        bibliography_note -> Nullable<Text>,
        toc -> Nullable<Text>,
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
        account_id -> Uuid,
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
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    
    locale (locale_id) {
        locale_id -> Uuid,
        code -> Text,
        name -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::LocaleCode;

    title (title_id) {
        title_id -> Uuid,
        work_id -> Uuid,
        locale_code -> LocaleCode,
        full_title -> Text,
        #[sql_name = "title"]
        title_ -> Text,
        subtitle -> Nullable<Text>,
        canonical -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    
    title_history (title_history_id) {
        title_history_id -> Uuid,
        title_id -> Uuid,
        account_id -> Uuid,
        data -> Jsonb,
        timestamp -> Timestamptz,
    }
}

joinable!(affiliation -> contribution (contribution_id));
joinable!(affiliation -> institution (institution_id));
joinable!(affiliation_history -> account (account_id));
joinable!(affiliation_history -> affiliation (affiliation_id));
joinable!(contribution -> contributor (contributor_id));
joinable!(contribution -> work (work_id));
joinable!(contribution_history -> account (account_id));
joinable!(contribution_history -> contribution (contribution_id));
joinable!(contributor_history -> account (account_id));
joinable!(contributor_history -> contributor (contributor_id));
joinable!(funding -> institution (institution_id));
joinable!(funding -> work (work_id));
joinable!(funding_history -> account (account_id));
joinable!(funding_history -> funding (funding_id));
joinable!(imprint -> publisher (publisher_id));
joinable!(imprint_history -> account (account_id));
joinable!(imprint_history -> imprint (imprint_id));
joinable!(institution_history -> account (account_id));
joinable!(institution_history -> institution (institution_id));
joinable!(issue -> series (series_id));
joinable!(issue -> work (work_id));
joinable!(issue_history -> account (account_id));
joinable!(issue_history -> issue (issue_id));
joinable!(language -> work (work_id));
joinable!(language_history -> account (account_id));
joinable!(language_history -> language (language_id));
joinable!(location -> publication (publication_id));
joinable!(location_history -> account (account_id));
joinable!(location_history -> location (location_id));
joinable!(price -> publication (publication_id));
joinable!(price_history -> account (account_id));
joinable!(price_history -> price (price_id));
joinable!(publication -> work (work_id));
joinable!(publication_history -> account (account_id));
joinable!(publication_history -> publication (publication_id));
joinable!(publisher_account -> account (account_id));
joinable!(publisher_account -> publisher (publisher_id));
joinable!(publisher_history -> account (account_id));
joinable!(publisher_history -> publisher (publisher_id));
joinable!(reference -> work (work_id));
joinable!(reference_history -> account (account_id));
joinable!(reference_history -> reference (reference_id));
joinable!(series -> imprint (imprint_id));
joinable!(series_history -> account (account_id));
joinable!(series_history -> series (series_id));
joinable!(subject -> work (work_id));
joinable!(subject_history -> account (account_id));
joinable!(subject_history -> subject (subject_id));
joinable!(work -> imprint (imprint_id));
joinable!(work_history -> account (account_id));
joinable!(work_history -> work (work_id));
joinable!(work_relation -> work (relator_work_id));
joinable!(work_relation_history -> account (account_id));
joinable!(work_relation_history -> work_relation (work_relation_id));
joinable!(title -> work (work_id));
joinable!(title_history -> title (title_id));
joinable!(title_history -> account (account_id));

allow_tables_to_appear_in_same_query!(
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
    title,
    locale,
    title_history,
);
