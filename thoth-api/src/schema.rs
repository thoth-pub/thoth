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
    use crate::model::contribution::Contribution_type;

    contribution (contribution_id) {
        contribution_id -> Uuid,
        work_id -> Uuid,
        contributor_id -> Uuid,
        contribution_type -> Contribution_type,
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
    use crate::model::institution::Country_code;

    institution (institution_id) {
        institution_id -> Uuid,
        institution_name -> Text,
        institution_doi -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        ror -> Nullable<Text>,
        country_code -> Nullable<Country_code>,
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
    use crate::model::language::Language_relation;
    use crate::model::language::Language_code;

    language (language_id) {
        language_id -> Uuid,
        work_id -> Uuid,
        language_code -> Language_code,
        language_relation -> Language_relation,
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
    use crate::model::price::Currency_code;

    price (price_id) {
        price_id -> Uuid,
        publication_id -> Uuid,
        currency_code -> Currency_code,
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
    use crate::model::publication::Publication_type;

    publication (publication_id) {
        publication_id -> Uuid,
        publication_type -> Publication_type,
        work_id -> Uuid,
        isbn -> Nullable<Text>,
        publication_url -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
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
    use crate::model::series::Series_type;

    series (series_id) {
        series_id -> Uuid,
        series_type -> Series_type,
        series_name -> Text,
        issn_print -> Text,
        issn_digital -> Text,
        series_url -> Nullable<Text>,
        imprint_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
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
    use crate::model::subject::Subject_type;

    subject (subject_id) {
        subject_id -> Uuid,
        work_id -> Uuid,
        subject_type -> Subject_type,
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
    use crate::model::work::Work_type;
    use crate::model::work::Work_status;

    work (work_id) {
        work_id -> Uuid,
        work_type -> Work_type,
        work_status -> Work_status,
        full_title -> Text,
        title -> Text,
        subtitle -> Nullable<Text>,
        reference -> Nullable<Text>,
        edition -> Int4,
        imprint_id -> Uuid,
        doi -> Nullable<Text>,
        publication_date -> Nullable<Date>,
        place -> Nullable<Text>,
        width -> Nullable<Float8>,
        height -> Nullable<Float8>,
        page_count -> Nullable<Int4>,
        page_breakdown -> Nullable<Text>,
        image_count -> Nullable<Int4>,
        table_count -> Nullable<Int4>,
        audio_count -> Nullable<Int4>,
        video_count -> Nullable<Int4>,
        license -> Nullable<Text>,
        copyright_holder -> Text,
        landing_page -> Nullable<Text>,
        lccn -> Nullable<Text>,
        oclc -> Nullable<Text>,
        short_abstract -> Nullable<Text>,
        long_abstract -> Nullable<Text>,
        general_note -> Nullable<Text>,
        toc -> Nullable<Text>,
        cover_url -> Nullable<Text>,
        cover_caption -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
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
joinable!(series -> imprint (imprint_id));
joinable!(series_history -> account (account_id));
joinable!(series_history -> series (series_id));
joinable!(subject -> work (work_id));
joinable!(subject_history -> account (account_id));
joinable!(subject_history -> subject (subject_id));
joinable!(work -> imprint (imprint_id));
joinable!(work_history -> account (account_id));
joinable!(work_history -> work (work_id));

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
    price,
    price_history,
    publication,
    publication_history,
    publisher,
    publisher_account,
    publisher_history,
    series,
    series_history,
    subject,
    subject_history,
    work,
    work_history,
);
