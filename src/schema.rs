table! {
    use diesel::sql_types::*;
    use crate::sql_types::*;

    contribution (work_id, contributor_id, contribution_type) {
        work_id -> Uuid,
        contributor_id -> Uuid,
        contribution_type -> Contribution_type,
        main_contribution -> Bool,
        biography -> Nullable<Text>,
        institution -> Nullable<Text>,
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
    }
}

table! {
    use diesel::sql_types::*;
    use crate::sql_types::*;

    publication (publication_id) {
        publication_id -> Uuid,
        publication_type -> Publication_type,
        work_id -> Uuid,
        isbn -> Nullable<Text>,
        publication_url -> Nullable<Text>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::sql_types::*;

    work (work_id) {
        work_id -> Uuid,
        work_type -> Work_type,
        title -> Text,
        subtitle -> Nullable<Text>,
        publisher -> Nullable<Text>,
        doi -> Nullable<Text>,
        publication_date -> Nullable<Date>,
    }
}

joinable!(contribution -> contributor (contributor_id));
joinable!(contribution -> work (work_id));
joinable!(publication -> work (work_id));

allow_tables_to_appear_in_same_query!(
    contribution,
    contributor,
    publication,
    work,
);
