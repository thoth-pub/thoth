use super::*;
use crate::model::Doi;

mod defaults {
    use super::*;

    #[test]
    fn imprintfield_default_is_imprint_name() {
        let impfield: ImprintField = Default::default();
        assert_eq!(impfield, ImprintField::ImprintName);
    }
}

mod display_and_parse {
    use super::*;

    #[test]
    fn imprintfield_display_formats_expected_strings() {
        assert_eq!(format!("{}", ImprintField::ImprintId), "ID");
        assert_eq!(format!("{}", ImprintField::ImprintName), "Imprint");
        assert_eq!(format!("{}", ImprintField::ImprintUrl), "ImprintURL");
        assert_eq!(format!("{}", ImprintField::CrossmarkDoi), "CrossmarkDOI");
        assert_eq!(
            format!("{}", ImprintField::DefaultCurrency),
            "DefaultCurrency"
        );
        assert_eq!(format!("{}", ImprintField::DefaultPlace), "DefaultPlace");
        assert_eq!(format!("{}", ImprintField::DefaultLocale), "DefaultLocale");
        assert_eq!(format!("{}", ImprintField::CreatedAt), "CreatedAt");
        assert_eq!(format!("{}", ImprintField::UpdatedAt), "UpdatedAt");
    }

    #[test]
    fn imprintfield_fromstr_parses_expected_values() {
        use std::str::FromStr;
        assert_eq!(
            ImprintField::from_str("ID").unwrap(),
            ImprintField::ImprintId
        );
        assert_eq!(
            ImprintField::from_str("Imprint").unwrap(),
            ImprintField::ImprintName
        );
        assert_eq!(
            ImprintField::from_str("ImprintURL").unwrap(),
            ImprintField::ImprintUrl
        );
        assert_eq!(
            ImprintField::from_str("CrossmarkDOI").unwrap(),
            ImprintField::CrossmarkDoi
        );
        assert_eq!(
            ImprintField::from_str("DefaultCurrency").unwrap(),
            ImprintField::DefaultCurrency
        );
        assert_eq!(
            ImprintField::from_str("DefaultPlace").unwrap(),
            ImprintField::DefaultPlace
        );
        assert_eq!(
            ImprintField::from_str("DefaultLocale").unwrap(),
            ImprintField::DefaultLocale
        );
        assert_eq!(
            ImprintField::from_str("CreatedAt").unwrap(),
            ImprintField::CreatedAt
        );
        assert_eq!(
            ImprintField::from_str("UpdatedAt").unwrap(),
            ImprintField::UpdatedAt
        );
        assert!(ImprintField::from_str("ImprintID").is_err());
        assert!(ImprintField::from_str("Publisher").is_err());
        assert!(ImprintField::from_str("Website").is_err());
    }
}

mod helpers {
    use super::*;
    use crate::model::{Crud, HistoryEntry};

    #[test]
    fn pk_returns_id() {
        let imprint: Imprint = Default::default();
        assert_eq!(imprint.pk(), imprint.imprint_id);
    }

    #[test]
    fn history_entry_serializes_model() {
        let imprint: Imprint = Default::default();
        let user_id = "123456".to_string();
        let new_imprint_history = imprint.new_history_entry(&user_id);
        assert_eq!(new_imprint_history.imprint_id, imprint.imprint_id);
        assert_eq!(new_imprint_history.user_id, user_id);
        assert_eq!(
            new_imprint_history.data,
            serde_json::Value::String(serde_json::to_string(&imprint).unwrap())
        );
    }
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;

    use crate::model::imprint::policy::ImprintPolicy;
    use crate::model::tests::db::{
        create_imprint, create_publisher, setup_test_db, test_context_with_user,
        test_user_with_role,
    };
    use crate::model::Crud;
    use crate::policy::{CreatePolicy, DeletePolicy, Role, UpdatePolicy};

    #[test]
    fn crud_policy_allows_publisher_user_for_create_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("imprint-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let new_imprint = NewImprint {
            publisher_id: publisher.publisher_id,
            imprint_name: "Policy Imprint".to_string(),
            imprint_url: None,
            crossmark_doi: None,
            s3_bucket: None,
            cdn_domain: None,
            cloudfront_dist_id: None,
            default_currency: None,
            default_place: None,
            default_locale: None,
        };

        let imprint = Imprint::create(pool.as_ref(), &new_imprint).expect("Failed to create");

        assert!(ImprintPolicy::can_create(&ctx, &new_imprint, ()).is_ok());
        assert!(ImprintPolicy::can_delete(&ctx, &imprint).is_ok());
    }

    #[test]
    fn crud_policy_requires_publisher_admin_for_update() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let patch = PatchImprint {
            imprint_id: imprint.imprint_id,
            publisher_id: imprint.publisher_id,
            imprint_name: "Updated Imprint".to_string(),
            imprint_url: imprint.imprint_url.clone(),
            crossmark_doi: imprint.crossmark_doi.clone(),
            s3_bucket: imprint.s3_bucket.clone(),
            cdn_domain: imprint.cdn_domain.clone(),
            cloudfront_dist_id: imprint.cloudfront_dist_id.clone(),
            default_currency: imprint.default_currency.clone(),
            default_place: imprint.default_place.clone(),
            default_locale: imprint.default_locale.clone(),
        };

        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("imprint-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);
        assert!(ImprintPolicy::can_update(&ctx, &imprint, &patch, ()).is_err());

        let admin = test_user_with_role("imprint-admin", Role::PublisherAdmin, &org_id);
        let admin_ctx = test_context_with_user(pool.clone(), admin);
        assert!(ImprintPolicy::can_update(&admin_ctx, &imprint, &patch, ()).is_ok());
    }
}

#[cfg(feature = "backend")]
mod crud {
    use super::*;
    use uuid::Uuid;

    use crate::model::tests::db::{create_imprint, create_publisher, setup_test_db, test_context};
    use crate::model::Crud;

    fn make_imprint(pool: &crate::db::PgPool, publisher_id: Uuid, name: String) -> Imprint {
        let new_imprint = NewImprint {
            publisher_id,
            imprint_name: name,
            imprint_url: None,
            crossmark_doi: None,
            s3_bucket: None,
            cdn_domain: None,
            cloudfront_dist_id: None,
            default_currency: None,
            default_place: None,
            default_locale: None,
        };

        Imprint::create(pool, &new_imprint).expect("Failed to create imprint")
    }

    #[test]
    fn crud_roundtrip_create_fetch_update_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let fetched_imprint =
            Imprint::from_id(pool.as_ref(), &imprint.imprint_id).expect("Failed to fetch imprint");
        assert_eq!(imprint.imprint_id, fetched_imprint.imprint_id);

        let patch = PatchImprint {
            imprint_id: imprint.imprint_id,
            publisher_id: imprint.publisher_id,
            imprint_name: format!("Updated {}", Uuid::new_v4()),
            imprint_url: Some("https://example.com".to_string()),
            crossmark_doi: imprint.crossmark_doi.clone(),
            s3_bucket: imprint.s3_bucket.clone(),
            cdn_domain: imprint.cdn_domain.clone(),
            cloudfront_dist_id: imprint.cloudfront_dist_id.clone(),
            default_currency: imprint.default_currency.clone(),
            default_place: imprint.default_place.clone(),
            default_locale: imprint.default_locale.clone(),
        };

        let ctx = test_context(pool.clone(), "test-user");
        let updated = imprint
            .update(&ctx, &patch)
            .expect("Failed to update imprint");
        assert_eq!(updated.imprint_name, patch.imprint_name);

        let deleted = updated
            .delete(pool.as_ref())
            .expect("Failed to delete imprint");
        assert!(Imprint::from_id(pool.as_ref(), &deleted.imprint_id).is_err());
    }

    #[test]
    fn crud_all_respects_limit_and_offset() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        create_imprint(pool.as_ref(), &publisher);
        create_imprint(pool.as_ref(), &publisher);

        let order = ImprintOrderBy {
            field: ImprintField::ImprintId,
            direction: Direction::Asc,
        };

        let first = Imprint::all(
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
        .expect("Failed to fetch imprints");
        let second = Imprint::all(
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
        .expect("Failed to fetch imprints");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_ne!(first[0].imprint_id, second[0].imprint_id);
    }

    #[test]
    fn crud_count_returns_total() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        create_imprint(pool.as_ref(), &publisher);
        create_imprint(pool.as_ref(), &publisher);

        let count = Imprint::count(pool.as_ref(), None, vec![], vec![], vec![], None, None)
            .expect("Failed to count imprints");
        assert_eq!(count, 2);
    }

    #[test]
    fn crud_count_filters_by_publishers() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let other_publisher = create_publisher(pool.as_ref());
        make_imprint(
            pool.as_ref(),
            publisher.publisher_id,
            "Match Imprint".to_string(),
        );
        make_imprint(
            pool.as_ref(),
            other_publisher.publisher_id,
            "Other Imprint".to_string(),
        );

        let count = Imprint::count(
            pool.as_ref(),
            None,
            vec![publisher.publisher_id],
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to count imprints by publisher");

        assert_eq!(count, 1);
    }

    #[test]
    fn crud_filter_matches_imprint_name() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let marker = format!("Filter {}", Uuid::new_v4());
        let matches = make_imprint(
            pool.as_ref(),
            publisher.publisher_id,
            format!("Imprint {marker}"),
        );
        make_imprint(
            pool.as_ref(),
            publisher.publisher_id,
            "Other Imprint".to_string(),
        );

        let order = ImprintOrderBy {
            field: ImprintField::ImprintId,
            direction: Direction::Asc,
        };

        let filtered = Imprint::all(
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
        .expect("Failed to filter imprints");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].imprint_id, matches.imprint_id);
    }

    #[test]
    fn crud_ordering_by_id_respects_direction() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let first = create_imprint(pool.as_ref(), &publisher);
        let second = create_imprint(pool.as_ref(), &publisher);
        let mut ids = [first.imprint_id, second.imprint_id];
        ids.sort();

        let asc = Imprint::all(
            pool.as_ref(),
            2,
            0,
            None,
            ImprintOrderBy {
                field: ImprintField::ImprintId,
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
        .expect("Failed to order imprints (asc)");

        let desc = Imprint::all(
            pool.as_ref(),
            2,
            0,
            None,
            ImprintOrderBy {
                field: ImprintField::ImprintId,
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
        .expect("Failed to order imprints (desc)");

        assert_eq!(asc[0].imprint_id, ids[0]);
        assert_eq!(desc[0].imprint_id, ids[1]);
    }

    #[test]
    fn crud_filter_parent_publisher_id_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let other_publisher = create_publisher(pool.as_ref());
        let matches = make_imprint(
            pool.as_ref(),
            publisher.publisher_id,
            "Match Imprint".to_string(),
        );
        make_imprint(
            pool.as_ref(),
            other_publisher.publisher_id,
            "Other Imprint".to_string(),
        );

        let filtered = Imprint::all(
            pool.as_ref(),
            10,
            0,
            None,
            ImprintOrderBy {
                field: ImprintField::ImprintId,
                direction: Direction::Asc,
            },
            vec![],
            Some(publisher.publisher_id),
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter imprints by publisher");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].imprint_id, matches.imprint_id);
    }

    #[test]
    fn crud_filter_publishers_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let other_publisher = create_publisher(pool.as_ref());
        let matches = make_imprint(
            pool.as_ref(),
            publisher.publisher_id,
            "Match Imprint".to_string(),
        );
        make_imprint(
            pool.as_ref(),
            other_publisher.publisher_id,
            "Other Imprint".to_string(),
        );

        let filtered = Imprint::all(
            pool.as_ref(),
            10,
            0,
            None,
            ImprintOrderBy {
                field: ImprintField::ImprintId,
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
        .expect("Failed to filter imprints by publishers");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].imprint_id, matches.imprint_id);
    }

    #[test]
    fn crud_count_with_filter_matches_imprint_url() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        Imprint::create(
            pool.as_ref(),
            &NewImprint {
                publisher_id: publisher.publisher_id,
                imprint_name: "Imprint A".to_string(),
                imprint_url: Some("https://example.com/imprint-a".to_string()),
                crossmark_doi: None,
                s3_bucket: None,
                cdn_domain: None,
                cloudfront_dist_id: None,
                default_currency: None,
                default_place: None,
                default_locale: None,
            },
        )
        .expect("Failed to create imprint");
        Imprint::create(
            pool.as_ref(),
            &NewImprint {
                publisher_id: publisher.publisher_id,
                imprint_name: "Imprint B".to_string(),
                imprint_url: Some("https://example.com/imprint-b".to_string()),
                crossmark_doi: None,
                s3_bucket: None,
                cdn_domain: None,
                cloudfront_dist_id: None,
                default_currency: None,
                default_place: None,
                default_locale: None,
            },
        )
        .expect("Failed to create imprint");

        let count = Imprint::count(
            pool.as_ref(),
            Some("imprint-a".to_string()),
            vec![],
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to count filtered imprints");

        assert_eq!(count, 1);
    }

    #[test]
    fn crud_ordering_by_fields_is_supported() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        Imprint::create(
            pool.as_ref(),
            &NewImprint {
                publisher_id: publisher.publisher_id,
                imprint_name: "Imprint A".to_string(),
                imprint_url: Some("https://example.com/a".to_string()),
                crossmark_doi: Some(Doi("https://doi.org/10.1234/A".to_string())),
                s3_bucket: None,
                cdn_domain: None,
                cloudfront_dist_id: None,
                default_currency: None,
                default_place: None,
                default_locale: None,
            },
        )
        .expect("Failed to create imprint");
        Imprint::create(
            pool.as_ref(),
            &NewImprint {
                publisher_id: publisher.publisher_id,
                imprint_name: "Imprint B".to_string(),
                imprint_url: Some("https://example.com/b".to_string()),
                crossmark_doi: Some(Doi("https://doi.org/10.1234/B".to_string())),
                s3_bucket: None,
                cdn_domain: None,
                cloudfront_dist_id: None,
                default_currency: None,
                default_place: None,
                default_locale: None,
            },
        )
        .expect("Failed to create imprint");

        let fields: Vec<fn() -> ImprintField> = vec![
            || ImprintField::ImprintId,
            || ImprintField::ImprintName,
            || ImprintField::ImprintUrl,
            || ImprintField::CrossmarkDoi,
            || ImprintField::CreatedAt,
            || ImprintField::UpdatedAt,
        ];

        for field in fields {
            for direction in [Direction::Asc, Direction::Desc] {
                let results = Imprint::all(
                    pool.as_ref(),
                    10,
                    0,
                    None,
                    ImprintOrderBy {
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
                .expect("Failed to order imprints");

                assert_eq!(results.len(), 2);
            }
        }
    }
}
