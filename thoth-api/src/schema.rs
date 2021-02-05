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
        created_at -> Timestamp,
        updated_at -> Timestamp,
        token -> Nullable<Text>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::contribution::model::Contribution_type;

    contribution (work_id, contributor_id, contribution_type) {
        work_id -> Uuid,
        contributor_id -> Uuid,
        contribution_type -> Contribution_type,
        main_contribution -> Bool,
        biography -> Nullable<Text>,
        institution -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        first_name -> Nullable<Text>,
        last_name -> Text,
        full_name -> Text,
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
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;

    funder (funder_id) {
        funder_id -> Uuid,
        funder_name -> Text,
        funder_doi -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;

    funding (funding_id) {
        funding_id -> Uuid,
        work_id -> Uuid,
        funder_id -> Uuid,
        program -> Nullable<Text>,
        project_name -> Nullable<Text>,
        project_shortname -> Nullable<Text>,
        grant_number -> Nullable<Text>,
        jurisdiction -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;

    imprint (imprint_id) {
        imprint_id -> Uuid,
        publisher_id -> Uuid,
        imprint_name -> Text,
        imprint_url -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;

    issue (series_id, work_id) {
        series_id -> Uuid,
        work_id -> Uuid,
        issue_ordinal -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::language::model::Language_relation;
    use crate::language::model::Language_code;

    language (language_id) {
        language_id -> Uuid,
        work_id -> Uuid,
        language_code -> Language_code,
        language_relation -> Language_relation,
        main_language -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::price::model::Currency_code;

    price (price_id) {
        price_id -> Uuid,
        publication_id -> Uuid,
        currency_code -> Currency_code,
        unit_price -> Float8,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::publication::model::Publication_type;

    publication (publication_id) {
        publication_id -> Uuid,
        publication_type -> Publication_type,
        work_id -> Uuid,
        isbn -> Nullable<Text>,
        publication_url -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;

    publisher (publisher_id) {
        publisher_id -> Uuid,
        publisher_name -> Text,
        publisher_shortname -> Nullable<Text>,
        publisher_url -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;

    publisher_account (account_id, publisher_id) {
        account_id -> Uuid,
        publisher_id -> Uuid,
        is_admin -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::series::model::Series_type;

    series (series_id) {
        series_id -> Uuid,
        series_type -> Series_type,
        series_name -> Text,
        issn_print -> Text,
        issn_digital -> Text,
        series_url -> Nullable<Text>,
        imprint_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::subject::model::Subject_type;

    subject (subject_id) {
        subject_id -> Uuid,
        work_id -> Uuid,
        subject_type -> Subject_type,
        subject_code -> Text,
        subject_ordinal -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::work::model::Work_type;
    use crate::work::model::Work_status;

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
        width -> Nullable<Int4>,
        height -> Nullable<Int4>,
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
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

joinable!(contribution -> contributor (contributor_id));
joinable!(contribution -> work (work_id));
joinable!(funding -> funder (funder_id));
joinable!(funding -> work (work_id));
joinable!(imprint -> publisher (publisher_id));
joinable!(issue -> series (series_id));
joinable!(issue -> work (work_id));
joinable!(language -> work (work_id));
joinable!(price -> publication (publication_id));
joinable!(publication -> work (work_id));
joinable!(publisher_account -> account (account_id));
joinable!(publisher_account -> publisher (publisher_id));
joinable!(series -> imprint (imprint_id));
joinable!(subject -> work (work_id));
joinable!(work -> imprint (imprint_id));

allow_tables_to_appear_in_same_query!(
    account,
    contribution,
    contributor,
    funder,
    funding,
    imprint,
    issue,
    language,
    price,
    publication,
    publisher_account,
    publisher,
    series,
    subject,
    work,
);
