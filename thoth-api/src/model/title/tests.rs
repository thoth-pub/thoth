use super::*;
use crate::model::Crud;
use uuid::Uuid;

fn make_title(
    pool: &crate::db::PgPool,
    work_id: Uuid,
    full_title: String,
    locale_code: LocaleCode,
) -> Title {
    let new_title = NewTitle {
        work_id,
        locale_code,
        full_title,
        title: "Test Title".to_string(),
        subtitle: None,
        canonical: false,
    };

    Title::create(pool, &new_title).expect("Failed to create title")
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;

    use crate::markup::MarkupFormat;
    use crate::model::tests::db::{
        create_imprint, create_publisher, create_work, setup_test_db, test_context_with_user,
        test_user_with_role,
    };
    use crate::model::title::policy::TitlePolicy;
    use crate::model::Crud;
    use crate::policy::{CreatePolicy, DeletePolicy, Role, UpdatePolicy};

    #[test]
    fn crud_policy_allows_publisher_user_with_markup() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("title-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let new_title = NewTitle {
            work_id: work.work_id,
            locale_code: LocaleCode::En,
            full_title: "Policy Title".to_string(),
            title: "Policy".to_string(),
            subtitle: None,
            canonical: false,
        };

        let title = Title::create(pool.as_ref(), &new_title).expect("Failed to create");
        let patch = PatchTitle {
            title_id: title.title_id,
            work_id: title.work_id,
            locale_code: title.locale_code,
            full_title: "Updated Policy Title".to_string(),
            title: "Updated".to_string(),
            subtitle: None,
            canonical: false,
        };

        assert!(TitlePolicy::can_create(&ctx, &new_title, Some(MarkupFormat::Html)).is_ok());
        assert!(TitlePolicy::can_update(&ctx, &title, &patch, Some(MarkupFormat::Html)).is_ok());
        assert!(TitlePolicy::can_delete(&ctx, &title).is_ok());
    }

    #[test]
    fn crud_policy_requires_markup_format() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("title-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let new_title = NewTitle {
            work_id: work.work_id,
            locale_code: LocaleCode::En,
            full_title: "Policy Title".to_string(),
            title: "Policy".to_string(),
            subtitle: None,
            canonical: false,
        };

        let title = Title::create(pool.as_ref(), &new_title).expect("Failed to create");
        let patch = PatchTitle {
            title_id: title.title_id,
            work_id: title.work_id,
            locale_code: title.locale_code,
            full_title: "Updated Policy Title".to_string(),
            title: "Updated".to_string(),
            subtitle: None,
            canonical: false,
        };

        assert!(TitlePolicy::can_create(&ctx, &new_title, None).is_err());
        assert!(TitlePolicy::can_update(&ctx, &title, &patch, None).is_err());
    }

    #[test]
    fn crud_policy_rejects_user_without_publisher_role() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let title = make_title(
            pool.as_ref(),
            work.work_id,
            "Policy Title".to_string(),
            LocaleCode::En,
        );
        let patch = PatchTitle {
            title_id: title.title_id,
            work_id: title.work_id,
            locale_code: title.locale_code,
            full_title: "Updated Policy Title".to_string(),
            title: "Updated".to_string(),
            subtitle: None,
            canonical: false,
        };

        let user = test_user_with_role("title-user", Role::PublisherUser, "org-other");
        let ctx = test_context_with_user(pool.clone(), user);

        let new_title = NewTitle {
            work_id: work.work_id,
            locale_code: LocaleCode::En,
            full_title: "Policy Title".to_string(),
            title: "Policy".to_string(),
            subtitle: None,
            canonical: false,
        };

        assert!(TitlePolicy::can_create(&ctx, &new_title, Some(MarkupFormat::Html)).is_err());
        assert!(TitlePolicy::can_update(&ctx, &title, &patch, Some(MarkupFormat::Html)).is_err());
        assert!(TitlePolicy::can_delete(&ctx, &title).is_err());
    }
}

#[cfg(feature = "backend")]
mod crud {
    use super::*;

    use crate::model::tests::db::{
        create_imprint, create_publisher, create_work, setup_test_db, test_context,
    };
    use crate::model::Crud;

    #[test]
    fn crud_roundtrip_create_fetch_update_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let new_title = NewTitle {
            work_id: work.work_id,
            locale_code: LocaleCode::En,
            full_title: format!("Full Title {}", Uuid::new_v4()),
            title: "Test Title".to_string(),
            subtitle: None,
            canonical: false,
        };

        let title = Title::create(pool.as_ref(), &new_title).expect("Failed to create title");
        let fetched = Title::from_id(pool.as_ref(), &title.title_id).expect("Failed to fetch");
        assert_eq!(title.title_id, fetched.title_id);

        let patch = PatchTitle {
            title_id: title.title_id,
            work_id: title.work_id,
            locale_code: title.locale_code,
            full_title: format!("Updated Full {}", Uuid::new_v4()),
            title: "Updated Title".to_string(),
            subtitle: Some("Updated Subtitle".to_string()),
            canonical: true,
        };

        let ctx = test_context(pool.clone(), "test-user");
        let updated = title.update(&ctx, &patch).expect("Failed to update title");
        assert_eq!(updated.full_title, patch.full_title);

        let deleted = updated
            .delete(pool.as_ref())
            .expect("Failed to delete title");
        assert!(Title::from_id(pool.as_ref(), &deleted.title_id).is_err());
    }

    #[test]
    fn crud_all_respects_limit_and_offset() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        make_title(
            pool.as_ref(),
            work.work_id,
            format!("Full Title {}", Uuid::new_v4()),
            LocaleCode::En,
        );
        make_title(
            pool.as_ref(),
            work.work_id,
            format!("Full Title {}", Uuid::new_v4()),
            LocaleCode::Fr,
        );

        let order = TitleOrderBy {
            field: TitleField::TitleId,
            direction: Direction::Asc,
        };

        let first = Title::all(
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
        .expect("Failed to fetch titles");
        let second = Title::all(
            pool.as_ref(),
            1,
            1,
            None,
            TitleOrderBy {
                field: TitleField::TitleId,
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
        .expect("Failed to fetch titles");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_ne!(first[0].title_id, second[0].title_id);
    }

    #[test]
    fn crud_count_returns_total() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        make_title(
            pool.as_ref(),
            work.work_id,
            format!("Full Title {}", Uuid::new_v4()),
            LocaleCode::En,
        );
        make_title(
            pool.as_ref(),
            work.work_id,
            format!("Full Title {}", Uuid::new_v4()),
            LocaleCode::Fr,
        );

        let count = Title::count(pool.as_ref(), None, vec![], vec![], vec![], None, None)
            .expect("Failed to count titles");
        assert_eq!(count, 2);
    }

    #[test]
    fn crud_filter_matches_full_title() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let marker = format!("Filter {}", Uuid::new_v4());
        let matches = make_title(
            pool.as_ref(),
            work.work_id,
            format!("Full Title {marker}"),
            LocaleCode::En,
        );
        make_title(
            pool.as_ref(),
            work.work_id,
            "Other Title".to_string(),
            LocaleCode::Fr,
        );

        let filtered = Title::all(
            pool.as_ref(),
            10,
            0,
            Some(marker),
            TitleOrderBy {
                field: TitleField::TitleId,
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
        .expect("Failed to filter titles");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].title_id, matches.title_id);
    }

    #[test]
    fn crud_filter_param_limits_locale_codes() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let matches = make_title(
            pool.as_ref(),
            work.work_id,
            format!("Full Title {}", Uuid::new_v4()),
            LocaleCode::En,
        );
        make_title(
            pool.as_ref(),
            work.work_id,
            format!("Full Title {}", Uuid::new_v4()),
            LocaleCode::Fr,
        );

        let filtered = Title::all(
            pool.as_ref(),
            10,
            0,
            None,
            TitleOrderBy {
                field: TitleField::TitleId,
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
        .expect("Failed to filter titles by locale");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].title_id, matches.title_id);
    }

    #[test]
    fn crud_ordering_by_id_respects_direction() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let first = make_title(
            pool.as_ref(),
            work.work_id,
            format!("Full Title {}", Uuid::new_v4()),
            LocaleCode::En,
        );
        let second = make_title(
            pool.as_ref(),
            work.work_id,
            format!("Full Title {}", Uuid::new_v4()),
            LocaleCode::Fr,
        );
        let mut ids = [first.title_id, second.title_id];
        ids.sort();

        let asc = Title::all(
            pool.as_ref(),
            2,
            0,
            None,
            TitleOrderBy {
                field: TitleField::TitleId,
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
        .expect("Failed to order titles (asc)");

        let desc = Title::all(
            pool.as_ref(),
            2,
            0,
            None,
            TitleOrderBy {
                field: TitleField::TitleId,
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
        .expect("Failed to order titles (desc)");

        assert_eq!(asc[0].title_id, ids[0]);
        assert_eq!(desc[0].title_id, ids[1]);
    }

    #[test]
    fn crud_canonical_from_work_id_returns_title() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let title = Title::create(
            pool.as_ref(),
            &NewTitle {
                work_id: work.work_id,
                locale_code: LocaleCode::En,
                full_title: "Canonical Title".to_string(),
                title: "Canonical".to_string(),
                subtitle: Some("Subtitle".to_string()),
                canonical: true,
            },
        )
        .expect("Failed to create title");

        let fetched = Title::canonical_from_work_id(pool.as_ref(), &work.work_id)
            .expect("Failed to fetch canonical title");

        assert_eq!(fetched.title_id, title.title_id);
    }

    #[test]
    fn crud_filter_parent_work_id_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let other_work = create_work(pool.as_ref(), &imprint);

        let matches = make_title(
            pool.as_ref(),
            work.work_id,
            format!("Full Title {}", Uuid::new_v4()),
            LocaleCode::En,
        );
        make_title(
            pool.as_ref(),
            other_work.work_id,
            format!("Full Title {}", Uuid::new_v4()),
            LocaleCode::Fr,
        );

        let filtered = Title::all(
            pool.as_ref(),
            10,
            0,
            None,
            TitleOrderBy {
                field: TitleField::TitleId,
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
        .expect("Failed to filter titles by work");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].title_id, matches.title_id);
    }

    #[test]
    fn crud_filter_matches_subtitle() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let marker = format!("Subtitle {}", Uuid::new_v4());

        let matches = Title::create(
            pool.as_ref(),
            &NewTitle {
                work_id: work.work_id,
                locale_code: LocaleCode::En,
                full_title: "Full Title".to_string(),
                title: "Title".to_string(),
                subtitle: Some(marker.clone()),
                canonical: false,
            },
        )
        .expect("Failed to create title");
        make_title(
            pool.as_ref(),
            work.work_id,
            "Other Title".to_string(),
            LocaleCode::Fr,
        );

        let filtered = Title::all(
            pool.as_ref(),
            10,
            0,
            Some(marker),
            TitleOrderBy {
                field: TitleField::TitleId,
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
        .expect("Failed to filter titles by subtitle");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].title_id, matches.title_id);
    }

    #[test]
    fn crud_count_with_filter_matches_title() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let marker = format!("Count {}", Uuid::new_v4());

        make_title(
            pool.as_ref(),
            work.work_id,
            format!("Full Title {marker}"),
            LocaleCode::En,
        );
        make_title(
            pool.as_ref(),
            work.work_id,
            "Other Title".to_string(),
            LocaleCode::Fr,
        );

        let count = Title::count(
            pool.as_ref(),
            Some(marker),
            vec![],
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to count filtered titles");

        assert_eq!(count, 1);
    }

    #[test]
    fn crud_ordering_by_fields_is_supported() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let other_work = create_work(pool.as_ref(), &imprint);

        Title::create(
            pool.as_ref(),
            &NewTitle {
                work_id: work.work_id,
                locale_code: LocaleCode::En,
                full_title: "Full A".to_string(),
                title: "Title A".to_string(),
                subtitle: Some("Subtitle A".to_string()),
                canonical: true,
            },
        )
        .expect("Failed to create title");
        Title::create(
            pool.as_ref(),
            &NewTitle {
                work_id: other_work.work_id,
                locale_code: LocaleCode::Fr,
                full_title: "Full B".to_string(),
                title: "Title B".to_string(),
                subtitle: Some("Subtitle B".to_string()),
                canonical: false,
            },
        )
        .expect("Failed to create title");

        let fields: Vec<fn() -> TitleField> = vec![
            || TitleField::TitleId,
            || TitleField::WorkId,
            || TitleField::LocaleCode,
            || TitleField::FullTitle,
            || TitleField::Title,
            || TitleField::Subtitle,
            || TitleField::Canonical,
        ];

        for field in fields {
            for direction in [Direction::Asc, Direction::Desc] {
                let results = Title::all(
                    pool.as_ref(),
                    10,
                    0,
                    None,
                    TitleOrderBy {
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
                .expect("Failed to order titles");

                assert_eq!(results.len(), 2);
            }
        }
    }
}
