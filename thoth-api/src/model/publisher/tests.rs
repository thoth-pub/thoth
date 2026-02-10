use super::*;

mod defaults {
    use super::*;

    #[test]
    fn publisherfield_default_is_publisher_name() {
        let pubfield: PublisherField = Default::default();
        assert_eq!(pubfield, PublisherField::PublisherName);
    }
}

mod display_and_parse {
    use super::*;

    #[test]
    fn publisherfield_display_formats_expected_strings() {
        assert_eq!(format!("{}", PublisherField::PublisherId), "ID");
        assert_eq!(format!("{}", PublisherField::PublisherName), "Name");
        assert_eq!(
            format!("{}", PublisherField::PublisherShortname),
            "ShortName"
        );
        assert_eq!(format!("{}", PublisherField::PublisherUrl), "URL");
        assert_eq!(format!("{}", PublisherField::ZitadelId), "ZitadelId");
        assert_eq!(format!("{}", PublisherField::CreatedAt), "CreatedAt");
        assert_eq!(format!("{}", PublisherField::UpdatedAt), "UpdatedAt");
    }

    #[test]
    fn publisherfield_fromstr_parses_expected_values() {
        use std::str::FromStr;
        assert_eq!(
            PublisherField::from_str("ID").unwrap(),
            PublisherField::PublisherId
        );
        assert_eq!(
            PublisherField::from_str("Name").unwrap(),
            PublisherField::PublisherName
        );
        assert_eq!(
            PublisherField::from_str("ShortName").unwrap(),
            PublisherField::PublisherShortname
        );
        assert_eq!(
            PublisherField::from_str("URL").unwrap(),
            PublisherField::PublisherUrl
        );
        assert_eq!(
            PublisherField::from_str("ZitadelId").unwrap(),
            PublisherField::ZitadelId
        );
        assert_eq!(
            PublisherField::from_str("CreatedAt").unwrap(),
            PublisherField::CreatedAt
        );
        assert_eq!(
            PublisherField::from_str("UpdatedAt").unwrap(),
            PublisherField::UpdatedAt
        );
        assert!(PublisherField::from_str("PublisherID").is_err());
        assert!(PublisherField::from_str("Website").is_err());
        assert!(PublisherField::from_str("Imprint").is_err());
    }

    #[test]
    fn publisher_display_formats_name() {
        let publisher = Publisher {
            publisher_name: "Test Publisher".to_string(),
            ..Default::default()
        };
        assert_eq!(format!("{publisher}"), "Test Publisher");
    }
}

mod helpers {
    use super::*;
    use crate::model::{Crud, HistoryEntry};

    #[test]
    fn pk_returns_id() {
        let publisher: Publisher = Default::default();
        assert_eq!(publisher.pk(), publisher.publisher_id);
    }

    #[test]
    fn history_entry_serializes_model() {
        let publisher: Publisher = Default::default();
        let user_id = "123456".to_string();
        let new_publisher_history = publisher.new_history_entry(&user_id);
        assert_eq!(new_publisher_history.publisher_id, publisher.publisher_id);
        assert_eq!(new_publisher_history.user_id, user_id);
        assert_eq!(
            new_publisher_history.data,
            serde_json::Value::String(serde_json::to_string(&publisher).unwrap())
        );
    }
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;

    use crate::model::publisher::policy::PublisherPolicy;
    use crate::model::tests::db::{
        create_publisher, setup_test_db, test_context_with_user, test_superuser,
        test_user_with_role,
    };
    use crate::policy::{CreatePolicy, DeletePolicy, Role, UpdatePolicy};

    #[test]
    fn crud_policy_requires_superuser_for_create_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let new_publisher = NewPublisher {
            publisher_name: "Policy Publisher".to_string(),
            publisher_shortname: None,
            publisher_url: None,
            zitadel_id: Some(format!("org-{}", Uuid::new_v4())),
            accessibility_statement: None,
            accessibility_report_url: None,
        };

        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("publisher-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        assert!(PublisherPolicy::can_create(&ctx, &new_publisher, ()).is_err());
        assert!(PublisherPolicy::can_delete(&ctx, &publisher).is_err());

        let super_ctx = test_context_with_user(pool.clone(), test_superuser("publisher-super"));
        assert!(PublisherPolicy::can_create(&super_ctx, &new_publisher, ()).is_ok());
        assert!(PublisherPolicy::can_delete(&super_ctx, &publisher).is_ok());
    }

    #[test]
    fn crud_policy_requires_superuser_for_zitadel_change() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("publisher-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let patch_same = PatchPublisher {
            publisher_id: publisher.publisher_id,
            publisher_name: publisher.publisher_name.clone(),
            publisher_shortname: publisher.publisher_shortname.clone(),
            publisher_url: publisher.publisher_url.clone(),
            zitadel_id: publisher.zitadel_id.clone(),
            accessibility_statement: publisher.accessibility_statement.clone(),
            accessibility_report_url: publisher.accessibility_report_url.clone(),
        };

        let patch_changed = PatchPublisher {
            publisher_id: publisher.publisher_id,
            publisher_name: publisher.publisher_name.clone(),
            publisher_shortname: publisher.publisher_shortname.clone(),
            publisher_url: publisher.publisher_url.clone(),
            zitadel_id: Some(format!("org-{}", Uuid::new_v4())),
            accessibility_statement: publisher.accessibility_statement.clone(),
            accessibility_report_url: publisher.accessibility_report_url.clone(),
        };

        assert!(PublisherPolicy::can_update(&ctx, &publisher, &patch_same, ()).is_ok());
        assert!(PublisherPolicy::can_update(&ctx, &publisher, &patch_changed, ()).is_err());

        let super_ctx = test_context_with_user(pool.clone(), test_superuser("publisher-super"));
        assert!(PublisherPolicy::can_update(&super_ctx, &publisher, &patch_changed, ()).is_ok());
    }
}

#[cfg(feature = "backend")]
mod crud {
    use super::*;
    use uuid::Uuid;

    use crate::model::tests::db::{create_publisher, setup_test_db, test_context};
    use crate::model::Crud;

    fn make_publisher(pool: &crate::db::PgPool, name: String) -> Publisher {
        let new_publisher = NewPublisher {
            publisher_name: name,
            publisher_shortname: None,
            publisher_url: None,
            zitadel_id: Some(format!("org-{}", Uuid::new_v4())),
            accessibility_statement: None,
            accessibility_report_url: None,
        };

        Publisher::create(pool, &new_publisher).expect("Failed to create publisher")
    }

    #[test]
    fn crud_roundtrip_create_fetch_update_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let fetched = Publisher::from_id(pool.as_ref(), &publisher.publisher_id)
            .expect("Failed to fetch publisher");
        assert_eq!(publisher.publisher_id, fetched.publisher_id);

        let patch = PatchPublisher {
            publisher_id: publisher.publisher_id,
            publisher_name: format!("Updated {}", Uuid::new_v4()),
            publisher_shortname: Some("UPD".to_string()),
            publisher_url: Some("https://example.com".to_string()),
            zitadel_id: publisher.zitadel_id.clone(),
            accessibility_statement: publisher.accessibility_statement.clone(),
            accessibility_report_url: publisher.accessibility_report_url.clone(),
        };

        let ctx = test_context(pool.clone(), "test-user");
        let updated = publisher
            .update(&ctx, &patch)
            .expect("Failed to update publisher");
        assert_eq!(updated.publisher_name, patch.publisher_name);

        let deleted = updated
            .delete(pool.as_ref())
            .expect("Failed to delete publisher");
        assert!(Publisher::from_id(pool.as_ref(), &deleted.publisher_id).is_err());
    }

    #[test]
    fn crud_all_respects_limit_and_offset() {
        let (_guard, pool) = setup_test_db();

        create_publisher(pool.as_ref());
        create_publisher(pool.as_ref());

        let order = PublisherOrderBy {
            field: PublisherField::PublisherId,
            direction: Direction::Asc,
        };

        let first = Publisher::all(
            pool.as_ref(),
            1,
            0,
            None,
            order.clone(),
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to fetch publishers");
        let second = Publisher::all(
            pool.as_ref(),
            1,
            1,
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
        .expect("Failed to fetch publishers");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_ne!(first[0].publisher_id, second[0].publisher_id);
    }

    #[test]
    fn crud_count_returns_total() {
        let (_guard, pool) = setup_test_db();

        create_publisher(pool.as_ref());
        create_publisher(pool.as_ref());

        let count = Publisher::count(pool.as_ref(), None, vec![], vec![], vec![], None, None)
            .expect("Failed to count publishers");
        assert_eq!(count, 2);
    }

    #[test]
    fn crud_count_filters_by_publisher_ids() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        create_publisher(pool.as_ref());

        let count = Publisher::count(
            pool.as_ref(),
            None,
            vec![publisher.publisher_id],
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to count publishers by id");
        assert_eq!(count, 1);
    }

    #[test]
    fn crud_count_filters_by_name() {
        let (_guard, pool) = setup_test_db();

        let marker = format!("Filter {}", Uuid::new_v4());
        make_publisher(pool.as_ref(), format!("Press {marker}"));
        make_publisher(pool.as_ref(), "Other Press".to_string());

        let count = Publisher::count(
            pool.as_ref(),
            Some(marker),
            vec![],
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to count publishers by name filter");

        assert_eq!(count, 1);
    }

    #[test]
    fn crud_filter_matches_name() {
        let (_guard, pool) = setup_test_db();

        let marker = format!("Filter {}", Uuid::new_v4());
        let matches = make_publisher(pool.as_ref(), format!("Press {marker}"));
        make_publisher(pool.as_ref(), "Other Press".to_string());

        let order = PublisherOrderBy {
            field: PublisherField::PublisherId,
            direction: Direction::Asc,
        };

        let filtered = Publisher::all(
            pool.as_ref(),
            10,
            0,
            Some(marker),
            order,
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter publishers");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].publisher_id, matches.publisher_id);
    }

    #[test]
    fn crud_filter_publisher_ids_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let other = create_publisher(pool.as_ref());

        let filtered = Publisher::all(
            pool.as_ref(),
            10,
            0,
            None,
            PublisherOrderBy {
                field: PublisherField::PublisherId,
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
        .expect("Failed to filter publishers by ids");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].publisher_id, publisher.publisher_id);
        assert_ne!(filtered[0].publisher_id, other.publisher_id);
    }

    #[test]
    fn crud_by_zitadel_ids_returns_matches() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");

        let results = Publisher::by_zitadel_ids(pool.as_ref(), vec![org_id])
            .expect("Failed to fetch publishers by zitadel id");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].publisher_id, publisher.publisher_id);
    }

    #[test]
    fn crud_by_zitadel_ids_returns_empty_for_empty_input() {
        let (_guard, pool) = setup_test_db();

        let results = Publisher::by_zitadel_ids(pool.as_ref(), vec![])
            .expect("Failed to fetch publishers by zitadel id");

        assert!(results.is_empty());
    }

    #[test]
    fn crud_ordering_by_id_respects_direction() {
        let (_guard, pool) = setup_test_db();

        let first = create_publisher(pool.as_ref());
        let second = create_publisher(pool.as_ref());
        let mut ids = [first.publisher_id, second.publisher_id];
        ids.sort();

        let asc = Publisher::all(
            pool.as_ref(),
            2,
            0,
            None,
            PublisherOrderBy {
                field: PublisherField::PublisherId,
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
        .expect("Failed to order publishers (asc)");

        let desc = Publisher::all(
            pool.as_ref(),
            2,
            0,
            None,
            PublisherOrderBy {
                field: PublisherField::PublisherId,
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
        .expect("Failed to order publishers (desc)");

        assert_eq!(asc[0].publisher_id, ids[0]);
        assert_eq!(desc[0].publisher_id, ids[1]);
    }

    #[test]
    fn crud_ordering_by_fields_is_supported() {
        let (_guard, pool) = setup_test_db();

        create_publisher(pool.as_ref());
        create_publisher(pool.as_ref());

        let fields: Vec<fn() -> PublisherField> = vec![
            || PublisherField::PublisherId,
            || PublisherField::PublisherName,
            || PublisherField::PublisherShortname,
            || PublisherField::PublisherUrl,
            || PublisherField::ZitadelId,
            || PublisherField::AccessibilityStatement,
            || PublisherField::AccessibilityReportUrl,
            || PublisherField::CreatedAt,
            || PublisherField::UpdatedAt,
        ];

        for field in fields {
            for direction in [Direction::Asc, Direction::Desc] {
                let results = Publisher::all(
                    pool.as_ref(),
                    10,
                    0,
                    None,
                    PublisherOrderBy {
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
                .expect("Failed to order publishers");

                assert_eq!(results.len(), 2);
            }
        }
    }
}
