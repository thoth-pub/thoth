use super::*;
use crate::model::Crud;
use uuid::Uuid;

fn make_new_contribution(
    work_id: Uuid,
    contributor_id: Uuid,
    contribution_type: ContributionType,
    contribution_ordinal: i32,
) -> NewContribution {
    let suffix = Uuid::new_v4();
    make_new_contribution_with_names(
        work_id,
        contributor_id,
        contribution_type,
        contribution_ordinal,
        Some("Test".to_string()),
        "Contributor",
        format!("Test Contributor {suffix}"),
    )
}

fn make_new_contribution_with_names(
    work_id: Uuid,
    contributor_id: Uuid,
    contribution_type: ContributionType,
    contribution_ordinal: i32,
    first_name: Option<String>,
    last_name: impl Into<String>,
    full_name: impl Into<String>,
) -> NewContribution {
    NewContribution {
        work_id,
        contributor_id,
        contribution_type,
        main_contribution: contribution_ordinal == 1,
        first_name,
        last_name: last_name.into(),
        full_name: full_name.into(),
        contribution_ordinal,
    }
}

fn make_patch_contribution(
    contribution: &Contribution,
    contribution_type: ContributionType,
    full_name: impl Into<String>,
    contribution_ordinal: i32,
) -> PatchContribution {
    PatchContribution {
        contribution_id: contribution.contribution_id,
        work_id: contribution.work_id,
        contributor_id: contribution.contributor_id,
        contribution_type,
        main_contribution: contribution_ordinal == 1,
        first_name: contribution.first_name.clone(),
        last_name: contribution.last_name.clone(),
        full_name: full_name.into(),
        contribution_ordinal,
    }
}

fn make_contribution(
    pool: &crate::db::PgPool,
    work_id: Uuid,
    contributor_id: Uuid,
    contribution_type: ContributionType,
    contribution_ordinal: i32,
) -> Contribution {
    let new_contribution = make_new_contribution(
        work_id,
        contributor_id,
        contribution_type,
        contribution_ordinal,
    );

    Contribution::create(pool, &new_contribution).expect("Failed to create contribution")
}

mod defaults {
    use super::*;

    #[test]
    fn contributiontype_default_is_author() {
        let contributiontype: ContributionType = Default::default();
        assert_eq!(contributiontype, ContributionType::Author);
    }
}

mod display_and_parse {
    use super::*;

    #[test]
    fn contributiontype_display_formats_expected_strings() {
        assert_eq!(format!("{}", ContributionType::Author), "Author");
        assert_eq!(format!("{}", ContributionType::Editor), "Editor");
        assert_eq!(format!("{}", ContributionType::Translator), "Translator");
        assert_eq!(
            format!("{}", ContributionType::Photographer),
            "Photographer"
        );
        assert_eq!(format!("{}", ContributionType::Illustrator), "Illustrator");
        assert_eq!(format!("{}", ContributionType::MusicEditor), "Music Editor");
        assert_eq!(format!("{}", ContributionType::ForewordBy), "Foreword By");
        assert_eq!(
            format!("{}", ContributionType::IntroductionBy),
            "Introduction By"
        );
        assert_eq!(format!("{}", ContributionType::AfterwordBy), "Afterword By");
        assert_eq!(format!("{}", ContributionType::PrefaceBy), "Preface By");
        assert_eq!(format!("{}", ContributionType::SoftwareBy), "Software By");
        assert_eq!(format!("{}", ContributionType::ResearchBy), "Research By");
        assert_eq!(
            format!("{}", ContributionType::ContributionsBy),
            "Contributions By"
        );
        assert_eq!(format!("{}", ContributionType::Indexer), "Indexer");
    }

    #[test]
    fn contributiontype_fromstr_parses_expected_values() {
        use std::str::FromStr;
        assert_eq!(
            ContributionType::from_str("Author").unwrap(),
            ContributionType::Author
        );
        assert_eq!(
            ContributionType::from_str("Editor").unwrap(),
            ContributionType::Editor
        );
        assert_eq!(
            ContributionType::from_str("Translator").unwrap(),
            ContributionType::Translator
        );
        assert_eq!(
            ContributionType::from_str("Photographer").unwrap(),
            ContributionType::Photographer
        );
        assert_eq!(
            ContributionType::from_str("Illustrator").unwrap(),
            ContributionType::Illustrator
        );
        assert_eq!(
            ContributionType::from_str("Music Editor").unwrap(),
            ContributionType::MusicEditor
        );
        assert_eq!(
            ContributionType::from_str("Foreword By").unwrap(),
            ContributionType::ForewordBy
        );
        assert_eq!(
            ContributionType::from_str("Introduction By").unwrap(),
            ContributionType::IntroductionBy
        );
        assert_eq!(
            ContributionType::from_str("Afterword By").unwrap(),
            ContributionType::AfterwordBy
        );
        assert_eq!(
            ContributionType::from_str("Preface By").unwrap(),
            ContributionType::PrefaceBy
        );
        assert_eq!(
            ContributionType::from_str("Software By").unwrap(),
            ContributionType::SoftwareBy
        );
        assert_eq!(
            ContributionType::from_str("Research By").unwrap(),
            ContributionType::ResearchBy
        );
        assert_eq!(
            ContributionType::from_str("Contributions By").unwrap(),
            ContributionType::ContributionsBy
        );
        assert_eq!(
            ContributionType::from_str("Indexer").unwrap(),
            ContributionType::Indexer
        );

        assert!(ContributionType::from_str("Juggler").is_err());
        assert!(ContributionType::from_str("Supervisor").is_err());
    }
}

#[cfg(feature = "backend")]
mod conversions {
    use super::*;
    use crate::model::tests::db::setup_test_db;
    use crate::model::tests::{assert_db_enum_roundtrip, assert_graphql_enum_roundtrip};

    #[test]
    fn contributiontype_graphql_roundtrip() {
        assert_graphql_enum_roundtrip(ContributionType::Author);
    }

    #[test]
    fn contributiontype_db_enum_roundtrip() {
        let (_guard, pool) = setup_test_db();

        assert_db_enum_roundtrip::<ContributionType, crate::schema::sql_types::ContributionType>(
            pool.as_ref(),
            "'author'::contribution_type",
            ContributionType::Author,
        );
    }
}

mod helpers {
    use super::*;
    use crate::model::{Crud, HistoryEntry};

    #[test]
    fn pk_returns_id() {
        let contribution: Contribution = Default::default();
        assert_eq!(contribution.pk(), contribution.contribution_id);
    }

    #[test]
    fn history_entry_serializes_model() {
        let contribution: Contribution = Default::default();
        let user_id = "123456".to_string();
        let new_contribution_history = contribution.new_history_entry(&user_id);
        assert_eq!(
            new_contribution_history.contribution_id,
            contribution.contribution_id
        );
        assert_eq!(new_contribution_history.user_id, user_id);
        assert_eq!(
            new_contribution_history.data,
            serde_json::Value::String(serde_json::to_string(&contribution).unwrap())
        );
    }
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;

    use crate::model::contribution::policy::ContributionPolicy;
    use crate::model::tests::db::{
        create_contributor, create_imprint, create_publisher, create_work, setup_test_db,
        test_context_with_user, test_user_with_role,
    };
    use crate::model::Crud;
    use crate::policy::{CreatePolicy, DeletePolicy, MovePolicy, Role, UpdatePolicy};

    #[test]
    fn crud_policy_allows_publisher_user_for_write() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("contribution-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let new_contribution = make_new_contribution_with_names(
            work.work_id,
            contributor.contributor_id,
            ContributionType::Author,
            1,
            contributor.first_name.clone(),
            contributor.last_name.clone(),
            contributor.full_name.clone(),
        );

        let contribution =
            Contribution::create(pool.as_ref(), &new_contribution).expect("Failed to create");
        let patch = make_patch_contribution(
            &contribution,
            ContributionType::Editor,
            format!("Updated {}", Uuid::new_v4()),
            2,
        );

        assert!(ContributionPolicy::can_create(&ctx, &new_contribution, ()).is_ok());
        assert!(ContributionPolicy::can_update(&ctx, &contribution, &patch, ()).is_ok());
        assert!(ContributionPolicy::can_delete(&ctx, &contribution).is_ok());
        assert!(ContributionPolicy::can_move(&ctx, &contribution).is_ok());
    }

    #[test]
    fn crud_policy_rejects_user_without_publisher_role() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = make_contribution(
            pool.as_ref(),
            work.work_id,
            contributor.contributor_id,
            ContributionType::Author,
            1,
        );
        let patch = make_patch_contribution(
            &contribution,
            ContributionType::Editor,
            format!("Updated {}", Uuid::new_v4()),
            2,
        );

        let user = test_user_with_role("contribution-user", Role::PublisherUser, "org-other");
        let ctx = test_context_with_user(pool.clone(), user);

        let new_contribution = make_new_contribution_with_names(
            work.work_id,
            contributor.contributor_id,
            ContributionType::Author,
            1,
            contributor.first_name.clone(),
            contributor.last_name.clone(),
            contributor.full_name.clone(),
        );

        assert!(ContributionPolicy::can_create(&ctx, &new_contribution, ()).is_err());
        assert!(ContributionPolicy::can_update(&ctx, &contribution, &patch, ()).is_err());
        assert!(ContributionPolicy::can_delete(&ctx, &contribution).is_err());
        assert!(ContributionPolicy::can_move(&ctx, &contribution).is_err());
    }
}

#[cfg(feature = "backend")]
mod crud {
    use super::*;

    use crate::graphql::types::inputs::{ContributionOrderBy, Direction};
    use crate::model::biography::{Biography, NewBiography};
    use crate::model::locale::LocaleCode;
    use crate::model::tests::db::{
        create_contribution, create_contributor, create_imprint, create_publisher, create_work,
        setup_test_db, test_context,
    };
    use crate::model::{Crud, Reorder};

    #[allow(clippy::too_many_arguments)]
    fn make_contribution_with_names(
        pool: &crate::db::PgPool,
        work_id: Uuid,
        contributor_id: Uuid,
        contribution_type: ContributionType,
        contribution_ordinal: i32,
        first_name: &str,
        last_name: &str,
        full_name: &str,
    ) -> Contribution {
        let new_contribution = make_new_contribution_with_names(
            work_id,
            contributor_id,
            contribution_type,
            contribution_ordinal,
            Some(first_name.to_string()),
            last_name,
            full_name,
        );

        Contribution::create(pool, &new_contribution).expect("Failed to create contribution")
    }

    #[test]
    fn crud_roundtrip_create_fetch_update_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);

        let fetched = Contribution::from_id(pool.as_ref(), &contribution.contribution_id)
            .expect("Failed to fetch");
        assert_eq!(contribution.contribution_id, fetched.contribution_id);

        let patch = make_patch_contribution(
            &contribution,
            ContributionType::Editor,
            format!("Updated {}", Uuid::new_v4()),
            2,
        );

        let ctx = test_context(pool.clone(), "test-user");
        let updated = contribution.update(&ctx, &patch).expect("Failed to update");
        assert_eq!(updated.full_name, patch.full_name);

        let deleted = updated.delete(pool.as_ref()).expect("Failed to delete");
        assert!(Contribution::from_id(pool.as_ref(), &deleted.contribution_id).is_err());
    }

    #[test]
    fn crud_all_respects_limit_and_offset() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());

        make_contribution(
            pool.as_ref(),
            work.work_id,
            contributor.contributor_id,
            ContributionType::Author,
            1,
        );
        make_contribution(
            pool.as_ref(),
            work.work_id,
            contributor.contributor_id,
            ContributionType::Editor,
            2,
        );

        let order = ContributionOrderBy {
            field: ContributionField::ContributionId,
            direction: Direction::Asc,
        };

        let first = Contribution::all(
            pool.as_ref(),
            1,
            0,
            None,
            order,
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to fetch contributions");
        let second = Contribution::all(
            pool.as_ref(),
            1,
            1,
            None,
            ContributionOrderBy {
                field: ContributionField::ContributionId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to fetch contributions");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_ne!(first[0].contribution_id, second[0].contribution_id);
    }

    #[test]
    fn crud_count_returns_total() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());

        make_contribution(
            pool.as_ref(),
            work.work_id,
            contributor.contributor_id,
            ContributionType::Author,
            1,
        );
        make_contribution(
            pool.as_ref(),
            work.work_id,
            contributor.contributor_id,
            ContributionType::Editor,
            2,
        );

        let count = Contribution::count(pool.as_ref(), None, vec![], vec![], vec![], None, None)
            .expect("Failed to count contributions");
        assert_eq!(count, 2);
    }

    #[test]
    fn crud_filter_param_limits_contribution_types() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());

        let matches = make_contribution(
            pool.as_ref(),
            work.work_id,
            contributor.contributor_id,
            ContributionType::Author,
            1,
        );
        make_contribution(
            pool.as_ref(),
            work.work_id,
            contributor.contributor_id,
            ContributionType::Editor,
            2,
        );

        let filtered = Contribution::all(
            pool.as_ref(),
            10,
            0,
            None,
            ContributionOrderBy {
                field: ContributionField::ContributionId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![ContributionType::Author],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter contributions by type");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].contribution_id, matches.contribution_id);
    }

    #[test]
    fn crud_ordering_by_id_respects_direction() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());

        let _first = make_contribution(
            pool.as_ref(),
            work.work_id,
            contributor.contributor_id,
            ContributionType::Author,
            1,
        );
        let _second = make_contribution(
            pool.as_ref(),
            work.work_id,
            contributor.contributor_id,
            ContributionType::Editor,
            2,
        );
        let asc = Contribution::all(
            pool.as_ref(),
            2,
            0,
            None,
            ContributionOrderBy {
                field: ContributionField::ContributionId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to order contributions (asc)");

        let desc = Contribution::all(
            pool.as_ref(),
            2,
            0,
            None,
            ContributionOrderBy {
                field: ContributionField::ContributionId,
                direction: Direction::Desc,
            },
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to order contributions (desc)");

        assert_eq!(asc.len(), 2);
        assert_eq!(desc.len(), 2);
        let asc_ids = [asc[0].contribution_id, asc[1].contribution_id];
        let desc_ids = [desc[0].contribution_id, desc[1].contribution_id];
        assert_ne!(asc_ids[0], asc_ids[1]);
        assert_eq!(desc_ids, [asc_ids[1], asc_ids[0]]);
    }

    #[test]
    fn crud_filter_parent_work_id_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let other_work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());

        let matches = make_contribution(
            pool.as_ref(),
            work.work_id,
            contributor.contributor_id,
            ContributionType::Author,
            1,
        );
        make_contribution(
            pool.as_ref(),
            other_work.work_id,
            contributor.contributor_id,
            ContributionType::Editor,
            1,
        );

        let filtered = Contribution::all(
            pool.as_ref(),
            10,
            0,
            None,
            ContributionOrderBy {
                field: ContributionField::ContributionId,
                direction: Direction::Asc,
            },
            vec![],
            Some(work.work_id),
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter contributions by work");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].contribution_id, matches.contribution_id);
    }

    #[test]
    fn crud_filter_parent_contributor_id_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let other_contributor = create_contributor(pool.as_ref());

        let matches = make_contribution(
            pool.as_ref(),
            work.work_id,
            contributor.contributor_id,
            ContributionType::Author,
            1,
        );
        make_contribution(
            pool.as_ref(),
            work.work_id,
            other_contributor.contributor_id,
            ContributionType::Editor,
            2,
        );

        let filtered = Contribution::all(
            pool.as_ref(),
            10,
            0,
            None,
            ContributionOrderBy {
                field: ContributionField::ContributionId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            Some(contributor.contributor_id),
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter contributions by contributor");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].contribution_id, matches.contribution_id);
    }

    #[test]
    fn crud_filter_publishers_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let matches = make_contribution(
            pool.as_ref(),
            work.work_id,
            contributor.contributor_id,
            ContributionType::Author,
            1,
        );

        let other_publisher = create_publisher(pool.as_ref());
        let other_imprint = create_imprint(pool.as_ref(), &other_publisher);
        let other_work = create_work(pool.as_ref(), &other_imprint);
        let other_contributor = create_contributor(pool.as_ref());
        make_contribution(
            pool.as_ref(),
            other_work.work_id,
            other_contributor.contributor_id,
            ContributionType::Editor,
            1,
        );

        let filtered = Contribution::all(
            pool.as_ref(),
            10,
            0,
            None,
            ContributionOrderBy {
                field: ContributionField::ContributionId,
                direction: Direction::Asc,
            },
            vec![publisher.publisher_id],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter contributions by publisher");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].contribution_id, matches.contribution_id);
    }

    #[test]
    fn crud_ordering_by_fields_is_supported() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let other_contributor = create_contributor(pool.as_ref());

        let first = make_contribution_with_names(
            pool.as_ref(),
            work.work_id,
            contributor.contributor_id,
            ContributionType::Author,
            1,
            "Alice",
            "Alpha",
            "Alice Alpha",
        );
        let second = make_contribution_with_names(
            pool.as_ref(),
            work.work_id,
            other_contributor.contributor_id,
            ContributionType::Editor,
            2,
            "Bob",
            "Beta",
            "Bob Beta",
        );

        Biography::create(
            pool.as_ref(),
            &NewBiography {
                contribution_id: first.contribution_id,
                content: "Bio A".to_string(),
                canonical: true,
                locale_code: LocaleCode::En,
            },
        )
        .expect("Failed to create biography");
        Biography::create(
            pool.as_ref(),
            &NewBiography {
                contribution_id: second.contribution_id,
                content: "Bio B".to_string(),
                canonical: true,
                locale_code: LocaleCode::En,
            },
        )
        .expect("Failed to create biography");

        let fields: Vec<fn() -> ContributionField> = vec![
            || ContributionField::ContributionId,
            || ContributionField::WorkId,
            || ContributionField::ContributorId,
            || ContributionField::ContributionType,
            || ContributionField::MainContribution,
            || ContributionField::Biography,
            || ContributionField::CreatedAt,
            || ContributionField::UpdatedAt,
            || ContributionField::FirstName,
            || ContributionField::LastName,
            || ContributionField::FullName,
            || ContributionField::ContributionOrdinal,
        ];

        for field in fields {
            for direction in [Direction::Asc, Direction::Desc] {
                let results = Contribution::all(
                    pool.as_ref(),
                    10,
                    0,
                    None,
                    ContributionOrderBy {
                        field: field(),
                        direction,
                    },
                    vec![],
                    None,
                    None,
                    vec![],
                    vec![],
                    None,
                    None,
                )
                .expect("Failed to order contributions");

                assert_eq!(results.len(), 2);
            }
        }
    }

    #[test]
    fn crud_count_with_filter_matches_contribution_type() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());

        make_contribution(
            pool.as_ref(),
            work.work_id,
            contributor.contributor_id,
            ContributionType::Author,
            1,
        );
        make_contribution(
            pool.as_ref(),
            work.work_id,
            contributor.contributor_id,
            ContributionType::Editor,
            2,
        );

        let count = Contribution::count(
            pool.as_ref(),
            None,
            vec![],
            vec![ContributionType::Author],
            vec![],
            None,
            None,
        )
        .expect("Failed to count filtered contributions");

        assert_eq!(count, 1);
    }

    #[test]
    fn crud_change_ordinal_reorders_contributions() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());

        let first = make_contribution(
            pool.as_ref(),
            work.work_id,
            contributor.contributor_id,
            ContributionType::Author,
            1,
        );
        let second = make_contribution(
            pool.as_ref(),
            work.work_id,
            contributor.contributor_id,
            ContributionType::Editor,
            2,
        );

        let ctx = test_context(pool.clone(), "test-user");
        let updated = first
            .change_ordinal(&ctx, first.contribution_ordinal, 2)
            .expect("Failed to change contribution ordinal");

        let refreshed_first = Contribution::from_id(pool.as_ref(), &updated.contribution_id)
            .expect("Failed to fetch");
        let refreshed_second =
            Contribution::from_id(pool.as_ref(), &second.contribution_id).expect("Failed to fetch");

        assert_eq!(refreshed_first.contribution_ordinal, 2);
        assert_eq!(refreshed_second.contribution_ordinal, 1);
    }
}
