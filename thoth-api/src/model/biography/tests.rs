use super::*;
use crate::model::Crud;
use uuid::Uuid;

fn make_new_biography(
    contribution_id: Uuid,
    content: impl Into<String>,
    canonical: bool,
    locale_code: LocaleCode,
) -> NewBiography {
    NewBiography {
        contribution_id,
        content: content.into(),
        canonical,
        locale_code,
    }
}

fn make_patch_biography(
    biography: &Biography,
    content: impl Into<String>,
    canonical: bool,
) -> PatchBiography {
    PatchBiography {
        biography_id: biography.biography_id,
        contribution_id: biography.contribution_id,
        content: content.into(),
        canonical,
        locale_code: biography.locale_code,
    }
}

fn make_biography(
    pool: &crate::db::PgPool,
    contribution_id: Uuid,
    content: String,
    locale_code: LocaleCode,
) -> Biography {
    let new_biography = make_new_biography(contribution_id, content, false, locale_code);

    Biography::create(pool, &new_biography).expect("Failed to create biography")
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;

    use crate::markup::MarkupFormat;
    use crate::model::biography::policy::BiographyPolicy;
    use crate::model::tests::db::{
        create_contribution, create_contributor, create_imprint, create_publisher, create_work,
        setup_test_db, test_context_with_user, test_user_with_role,
    };
    use crate::model::Crud;
    use crate::policy::{CreatePolicy, DeletePolicy, Role, UpdatePolicy};
    use thoth_errors::ThothError;

    #[test]
    fn crud_policy_allows_publisher_user_with_markup() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("biography-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);
        let new_biography = make_new_biography(
            contribution.contribution_id,
            "Policy Biography",
            false,
            LocaleCode::En,
        );

        let biography = Biography::create(pool.as_ref(), &new_biography).expect("Failed to create");
        let patch =
            make_patch_biography(&biography, "Updated Policy Biography", biography.canonical);

        assert!(
            BiographyPolicy::can_create(&ctx, &new_biography, Some(MarkupFormat::Html)).is_ok()
        );
        assert!(
            BiographyPolicy::can_update(&ctx, &biography, &patch, Some(MarkupFormat::Html)).is_ok()
        );
        assert!(BiographyPolicy::can_delete(&ctx, &biography).is_ok());
    }

    #[test]
    fn crud_policy_requires_markup_format() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("biography-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);
        let new_biography = make_new_biography(
            contribution.contribution_id,
            "Policy Biography",
            false,
            LocaleCode::En,
        );

        let biography = Biography::create(pool.as_ref(), &new_biography).expect("Failed to create");
        let patch =
            make_patch_biography(&biography, "Updated Policy Biography", biography.canonical);

        assert!(BiographyPolicy::can_create(&ctx, &new_biography, None).is_err());
        assert!(BiographyPolicy::can_update(&ctx, &biography, &patch, None).is_err());
    }

    #[test]
    fn crud_policy_rejects_user_without_publisher_role() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);
        let biography_item = make_biography(
            pool.as_ref(),
            contribution.contribution_id,
            "Policy Biography".to_string(),
            LocaleCode::En,
        );
        let patch = make_patch_biography(
            &biography_item,
            "Updated Policy Biography",
            biography_item.canonical,
        );

        let user = test_user_with_role("biography-user", Role::PublisherUser, "org-other");
        let ctx = test_context_with_user(pool.clone(), user);

        let new_biography = make_new_biography(
            contribution.contribution_id,
            "Policy Biography",
            false,
            LocaleCode::En,
        );

        assert!(
            BiographyPolicy::can_create(&ctx, &new_biography, Some(MarkupFormat::Html)).is_err()
        );
        assert!(BiographyPolicy::can_update(
            &ctx,
            &biography_item,
            &patch,
            Some(MarkupFormat::Html)
        )
        .is_err());
        assert!(BiographyPolicy::can_delete(&ctx, &biography_item).is_err());
    }

    #[test]
    fn crud_policy_rejects_duplicate_canonical_biography() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("biography-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);

        let canonical = make_new_biography(
            contribution.contribution_id,
            "Canonical Biography",
            true,
            LocaleCode::En,
        );
        Biography::create(pool.as_ref(), &canonical).expect("Failed to create canonical biography");

        let new_biography = make_new_biography(
            contribution.contribution_id,
            "Second Canonical",
            true,
            LocaleCode::En,
        );

        let result = BiographyPolicy::can_create(&ctx, &new_biography, Some(MarkupFormat::Html));

        assert!(matches!(
            result,
            Err(ThothError::CanonicalBiographyExistsError)
        ));
    }
}

#[cfg(feature = "backend")]
mod crud {
    use super::*;

    use crate::model::tests::db::{
        create_contribution, create_contributor, create_imprint, create_publisher, create_work,
        setup_test_db, test_context,
    };
    use crate::model::Crud;

    #[test]
    fn crud_roundtrip_create_fetch_update_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);

        let new_biography = make_new_biography(
            contribution.contribution_id,
            format!("Biography {}", Uuid::new_v4()),
            false,
            LocaleCode::En,
        );

        let biography =
            Biography::create(pool.as_ref(), &new_biography).expect("Failed to create biography");
        let fetched =
            Biography::from_id(pool.as_ref(), &biography.biography_id).expect("Failed to fetch");
        assert_eq!(biography.biography_id, fetched.biography_id);

        let patch = make_patch_biography(&biography, format!("Updated {}", Uuid::new_v4()), true);

        let ctx = test_context(pool.clone(), "test-user");
        let updated = biography
            .update(&ctx, &patch)
            .expect("Failed to update biography");
        assert_eq!(updated.content, patch.content);

        let deleted = updated
            .delete(pool.as_ref())
            .expect("Failed to delete biography");
        assert!(Biography::from_id(pool.as_ref(), &deleted.biography_id).is_err());
    }

    #[test]
    fn crud_all_respects_limit_and_offset() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);

        make_biography(
            pool.as_ref(),
            contribution.contribution_id,
            format!("Biography {}", Uuid::new_v4()),
            LocaleCode::En,
        );
        make_biography(
            pool.as_ref(),
            contribution.contribution_id,
            format!("Biography {}", Uuid::new_v4()),
            LocaleCode::Fr,
        );

        let order = BiographyOrderBy {
            field: BiographyField::BiographyId,
            direction: Direction::Asc,
        };

        let first = Biography::all(
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
        .expect("Failed to fetch biographies");
        let second = Biography::all(
            pool.as_ref(),
            1,
            1,
            None,
            BiographyOrderBy {
                field: BiographyField::BiographyId,
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
        .expect("Failed to fetch biographies");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_ne!(first[0].biography_id, second[0].biography_id);
    }

    #[test]
    fn crud_count_returns_total() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);

        make_biography(
            pool.as_ref(),
            contribution.contribution_id,
            format!("Biography {}", Uuid::new_v4()),
            LocaleCode::En,
        );
        make_biography(
            pool.as_ref(),
            contribution.contribution_id,
            format!("Biography {}", Uuid::new_v4()),
            LocaleCode::Fr,
        );

        let count = Biography::count(pool.as_ref(), None, vec![], vec![], vec![], None, None)
            .expect("Failed to count biographies");
        assert_eq!(count, 2);
    }

    #[test]
    fn crud_filter_matches_content() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);

        let marker = format!("Filter {}", Uuid::new_v4());
        let matches = make_biography(
            pool.as_ref(),
            contribution.contribution_id,
            format!("Biography {marker}"),
            LocaleCode::En,
        );
        make_biography(
            pool.as_ref(),
            contribution.contribution_id,
            "Other biography".to_string(),
            LocaleCode::Fr,
        );

        let filtered = Biography::all(
            pool.as_ref(),
            10,
            0,
            Some(marker),
            BiographyOrderBy {
                field: BiographyField::BiographyId,
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
        .expect("Failed to filter biographies");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].biography_id, matches.biography_id);
    }

    #[test]
    fn crud_ordering_by_id_respects_direction() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);

        let first = make_biography(
            pool.as_ref(),
            contribution.contribution_id,
            format!("Biography {}", Uuid::new_v4()),
            LocaleCode::En,
        );
        let second = make_biography(
            pool.as_ref(),
            contribution.contribution_id,
            format!("Biography {}", Uuid::new_v4()),
            LocaleCode::Fr,
        );
        let mut ids = [first.biography_id, second.biography_id];
        ids.sort();

        let asc = Biography::all(
            pool.as_ref(),
            2,
            0,
            None,
            BiographyOrderBy {
                field: BiographyField::BiographyId,
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
        .expect("Failed to order biographies (asc)");

        let desc = Biography::all(
            pool.as_ref(),
            2,
            0,
            None,
            BiographyOrderBy {
                field: BiographyField::BiographyId,
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
        .expect("Failed to order biographies (desc)");

        assert_eq!(asc[0].biography_id, ids[0]);
        assert_eq!(desc[0].biography_id, ids[1]);
    }

    #[test]
    fn crud_canonical_from_contribution_id_returns_biography() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);

        let biography = Biography::create(
            pool.as_ref(),
            &make_new_biography(
                contribution.contribution_id,
                "Canonical biography",
                true,
                LocaleCode::En,
            ),
        )
        .expect("Failed to create biography");

        let fetched =
            Biography::canonical_from_contribution_id(pool.as_ref(), &contribution.contribution_id)
                .expect("Failed to fetch canonical biography");

        assert_eq!(fetched.biography_id, biography.biography_id);
    }

    #[test]
    fn crud_filter_parent_contribution_id_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let other_work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let other_contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);
        let other_contribution =
            create_contribution(pool.as_ref(), &other_work, &other_contributor);

        let matches = make_biography(
            pool.as_ref(),
            contribution.contribution_id,
            "Biography A".to_string(),
            LocaleCode::En,
        );
        make_biography(
            pool.as_ref(),
            other_contribution.contribution_id,
            "Biography B".to_string(),
            LocaleCode::Fr,
        );

        let filtered = Biography::all(
            pool.as_ref(),
            10,
            0,
            None,
            BiographyOrderBy {
                field: BiographyField::BiographyId,
                direction: Direction::Asc,
            },
            vec![],
            Some(contribution.contribution_id),
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter biographies by contribution");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].biography_id, matches.biography_id);
    }

    #[test]
    fn crud_filter_param_limits_locale_codes() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);

        let matches = make_biography(
            pool.as_ref(),
            contribution.contribution_id,
            "Biography EN".to_string(),
            LocaleCode::En,
        );
        make_biography(
            pool.as_ref(),
            contribution.contribution_id,
            "Biography FR".to_string(),
            LocaleCode::Fr,
        );

        let filtered = Biography::all(
            pool.as_ref(),
            10,
            0,
            None,
            BiographyOrderBy {
                field: BiographyField::BiographyId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![LocaleCode::En],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter biographies by locale");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].biography_id, matches.biography_id);
    }

    #[test]
    fn crud_ordering_by_fields_is_supported() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);

        Biography::create(
            pool.as_ref(),
            &make_new_biography(
                contribution.contribution_id,
                "Biography A",
                true,
                LocaleCode::En,
            ),
        )
        .expect("Failed to create biography");
        Biography::create(
            pool.as_ref(),
            &make_new_biography(
                contribution.contribution_id,
                "Biography B",
                false,
                LocaleCode::Fr,
            ),
        )
        .expect("Failed to create biography");

        let fields: Vec<fn() -> BiographyField> = vec![
            || BiographyField::BiographyId,
            || BiographyField::ContributionId,
            || BiographyField::Content,
            || BiographyField::Canonical,
            || BiographyField::LocaleCode,
        ];

        for field in fields {
            for direction in [Direction::Asc, Direction::Desc] {
                let results = Biography::all(
                    pool.as_ref(),
                    10,
                    0,
                    None,
                    BiographyOrderBy {
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
                .expect("Failed to order biographies");

                assert_eq!(results.len(), 2);
            }
        }
    }

    #[test]
    fn crud_count_with_filter_matches_content() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);

        let marker = format!("Marker {}", Uuid::new_v4());
        make_biography(
            pool.as_ref(),
            contribution.contribution_id,
            format!("Biography {marker}"),
            LocaleCode::En,
        );
        make_biography(
            pool.as_ref(),
            contribution.contribution_id,
            "Other biography".to_string(),
            LocaleCode::Fr,
        );

        let count = Biography::count(
            pool.as_ref(),
            Some(marker),
            vec![],
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to count filtered biographies");

        assert_eq!(count, 1);
    }
}
