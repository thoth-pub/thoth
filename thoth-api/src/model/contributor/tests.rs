use super::*;
use crate::model::{Crud, Orcid};
use uuid::Uuid;

fn make_contributor(pool: &crate::db::PgPool, full_name: String, last_name: String) -> Contributor {
    let new_contributor = NewContributor {
        first_name: Some("Test".to_string()),
        last_name,
        full_name,
        orcid: None,
        website: None,
    };

    Contributor::create(pool, &new_contributor).expect("Failed to create contributor")
}

mod defaults {
    use super::*;

    #[test]
    fn contributorfield_default_is_full_name() {
        let contfield: ContributorField = Default::default();
        assert_eq!(contfield, ContributorField::FullName);
    }
}

mod display_and_parse {
    use super::*;

    #[test]
    fn contributorfield_display_formats_expected_strings() {
        assert_eq!(format!("{}", ContributorField::ContributorId), "ID");
        assert_eq!(format!("{}", ContributorField::FirstName), "FirstName");
        assert_eq!(format!("{}", ContributorField::LastName), "LastName");
        assert_eq!(format!("{}", ContributorField::FullName), "FullName");
        assert_eq!(format!("{}", ContributorField::Orcid), "ORCID");
        assert_eq!(format!("{}", ContributorField::Website), "Website");
        assert_eq!(format!("{}", ContributorField::CreatedAt), "CreatedAt");
        assert_eq!(format!("{}", ContributorField::UpdatedAt), "UpdatedAt");
    }

    #[test]
    fn contributorfield_fromstr_parses_expected_values() {
        use std::str::FromStr;
        assert_eq!(
            ContributorField::from_str("ID").unwrap(),
            ContributorField::ContributorId
        );
        assert_eq!(
            ContributorField::from_str("FirstName").unwrap(),
            ContributorField::FirstName
        );
        assert_eq!(
            ContributorField::from_str("LastName").unwrap(),
            ContributorField::LastName
        );
        assert_eq!(
            ContributorField::from_str("FullName").unwrap(),
            ContributorField::FullName
        );
        assert_eq!(
            ContributorField::from_str("ORCID").unwrap(),
            ContributorField::Orcid
        );
        assert_eq!(
            ContributorField::from_str("UpdatedAt").unwrap(),
            ContributorField::UpdatedAt
        );
        assert!(ContributorField::from_str("ContributorID").is_err());
        assert!(ContributorField::from_str("Biography").is_err());
        assert!(ContributorField::from_str("Institution").is_err());
    }

    #[test]
    fn contributor_display_includes_orcid_when_present() {
        let contributor = Contributor {
            full_name: "Jane Doe".to_string(),
            orcid: Some(Orcid("https://orcid.org/0000-0002-1234-5678".to_string())),
            ..Default::default()
        };
        assert_eq!(format!("{contributor}"), "Jane Doe - 0000-0002-1234-5678");
    }

    #[test]
    fn contributor_display_omits_orcid_when_absent() {
        let contributor = Contributor {
            full_name: "Jane Doe".to_string(),
            orcid: None,
            ..Default::default()
        };
        assert_eq!(format!("{contributor}"), "Jane Doe");
    }
}

mod helpers {
    use super::*;
    use crate::model::{Crud, HistoryEntry};

    #[test]
    fn pk_returns_id() {
        let contributor: Contributor = Default::default();
        assert_eq!(contributor.pk(), contributor.contributor_id);
    }

    #[test]
    fn history_entry_serializes_model() {
        let contributor: Contributor = Default::default();
        let user_id = "123456".to_string();
        let new_contributor_history = contributor.new_history_entry(&user_id);
        assert_eq!(
            new_contributor_history.contributor_id,
            contributor.contributor_id
        );
        assert_eq!(new_contributor_history.user_id, user_id);
        assert_eq!(
            new_contributor_history.data,
            serde_json::Value::String(serde_json::to_string(&contributor).unwrap())
        );
    }
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;

    use crate::graphql::Context;
    use crate::model::contributor::policy::ContributorPolicy;
    use crate::model::tests::db::{
        create_contribution, create_imprint, create_publisher, create_work, setup_test_db,
        test_context, test_context_with_user, test_user_with_role,
    };
    use crate::model::Crud;
    use crate::policy::{CreatePolicy, DeletePolicy, Role, UpdatePolicy};

    #[test]
    fn crud_policy_requires_authentication_for_create_update() {
        let (_guard, pool) = setup_test_db();

        let ctx = Context::new(pool.clone(), None);

        let new_contributor = NewContributor {
            first_name: Some("Test".to_string()),
            last_name: "Contributor".to_string(),
            full_name: "Test Contributor".to_string(),
            orcid: None,
            website: None,
        };

        let contributor =
            Contributor::create(pool.as_ref(), &new_contributor).expect("Failed to create");
        let patch = PatchContributor {
            contributor_id: contributor.contributor_id,
            first_name: contributor.first_name.clone(),
            last_name: contributor.last_name.clone(),
            full_name: "Updated Contributor".to_string(),
            orcid: contributor.orcid.clone(),
            website: contributor.website.clone(),
        };

        assert!(ContributorPolicy::can_create(&ctx, &new_contributor, ()).is_err());
        assert!(ContributorPolicy::can_update(&ctx, &contributor, &patch, ()).is_err());
    }

    #[test]
    fn crud_policy_allows_authenticated_user_for_create_update() {
        let (_guard, pool) = setup_test_db();

        let ctx = test_context(pool.clone(), "contributor-user");

        let new_contributor = NewContributor {
            first_name: Some("Test".to_string()),
            last_name: "Contributor".to_string(),
            full_name: "Test Contributor".to_string(),
            orcid: None,
            website: None,
        };

        let contributor =
            Contributor::create(pool.as_ref(), &new_contributor).expect("Failed to create");
        let patch = PatchContributor {
            contributor_id: contributor.contributor_id,
            first_name: contributor.first_name.clone(),
            last_name: contributor.last_name.clone(),
            full_name: "Updated Contributor".to_string(),
            orcid: contributor.orcid.clone(),
            website: contributor.website.clone(),
        };

        assert!(ContributorPolicy::can_create(&ctx, &new_contributor, ()).is_ok());
        assert!(ContributorPolicy::can_update(&ctx, &contributor, &patch, ()).is_ok());
    }

    #[test]
    fn crud_policy_delete_requires_publisher_membership() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = make_contributor(
            pool.as_ref(),
            format!("Contributor {}", Uuid::new_v4()),
            "Contributor".to_string(),
        );
        create_contribution(pool.as_ref(), &work, &contributor);

        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("contributor-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);
        assert!(ContributorPolicy::can_delete(&ctx, &contributor).is_ok());

        let other_user = test_user_with_role("contributor-user", Role::PublisherUser, "org-other");
        let other_ctx = test_context_with_user(pool.clone(), other_user);
        assert!(ContributorPolicy::can_delete(&other_ctx, &contributor).is_err());
    }
}

#[cfg(feature = "backend")]
mod crud {
    use super::*;

    use crate::graphql::types::inputs::Direction;
    use crate::model::contributor::ContributorOrderBy;
    use crate::model::tests::db::{setup_test_db, test_context};
    use crate::model::Crud;

    #[test]
    fn crud_roundtrip_create_fetch_update_delete() {
        let (_guard, pool) = setup_test_db();

        let suffix = Uuid::new_v4();
        let new_contributor = NewContributor {
            first_name: Some("Test".to_string()),
            last_name: format!("Contributor {suffix}"),
            full_name: format!("Test Contributor {suffix}"),
            orcid: None,
            website: None,
        };

        let contributor =
            Contributor::create(pool.as_ref(), &new_contributor).expect("Failed to create");
        let fetched = Contributor::from_id(pool.as_ref(), &contributor.contributor_id)
            .expect("Failed to fetch");
        assert_eq!(contributor.contributor_id, fetched.contributor_id);

        let patch = PatchContributor {
            contributor_id: contributor.contributor_id,
            first_name: contributor.first_name.clone(),
            last_name: contributor.last_name.clone(),
            full_name: format!("Updated {suffix}"),
            orcid: contributor.orcid.clone(),
            website: Some("https://example.com".to_string()),
        };

        let ctx = test_context(pool.clone(), "test-user");
        let updated = contributor.update(&ctx, &patch).expect("Failed to update");
        assert_eq!(updated.full_name, patch.full_name);

        let deleted = updated.delete(pool.as_ref()).expect("Failed to delete");
        assert!(Contributor::from_id(pool.as_ref(), &deleted.contributor_id).is_err());
    }

    #[test]
    fn crud_all_respects_limit_and_offset() {
        let (_guard, pool) = setup_test_db();

        make_contributor(
            pool.as_ref(),
            format!("Contributor {}", Uuid::new_v4()),
            "Alpha".to_string(),
        );
        make_contributor(
            pool.as_ref(),
            format!("Contributor {}", Uuid::new_v4()),
            "Beta".to_string(),
        );

        let order = ContributorOrderBy {
            field: ContributorField::ContributorId,
            direction: Direction::Asc,
        };

        let first = Contributor::all(
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
        .expect("Failed to fetch contributors");
        let second = Contributor::all(
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
        .expect("Failed to fetch contributors");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_ne!(first[0].contributor_id, second[0].contributor_id);
    }

    #[test]
    fn crud_count_returns_total() {
        let (_guard, pool) = setup_test_db();

        make_contributor(
            pool.as_ref(),
            format!("Contributor {}", Uuid::new_v4()),
            "Alpha".to_string(),
        );
        make_contributor(
            pool.as_ref(),
            format!("Contributor {}", Uuid::new_v4()),
            "Beta".to_string(),
        );

        let count = Contributor::count(pool.as_ref(), None, vec![], vec![], vec![], None, None)
            .expect("Failed to count contributors");
        assert_eq!(count, 2);
    }

    #[test]
    fn crud_filter_matches_full_name() {
        let (_guard, pool) = setup_test_db();

        let marker = format!("Filter {}", Uuid::new_v4());
        let matches = make_contributor(
            pool.as_ref(),
            format!("Contributor {marker}"),
            "Alpha".to_string(),
        );
        make_contributor(
            pool.as_ref(),
            "Other Contributor".to_string(),
            "Beta".to_string(),
        );

        let filtered = Contributor::all(
            pool.as_ref(),
            10,
            0,
            Some(marker),
            ContributorOrderBy {
                field: ContributorField::ContributorId,
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
        .expect("Failed to filter contributors");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].contributor_id, matches.contributor_id);
    }

    #[test]
    fn crud_ordering_by_id_respects_direction() {
        let (_guard, pool) = setup_test_db();

        let first = make_contributor(
            pool.as_ref(),
            format!("Contributor {}", Uuid::new_v4()),
            "Alpha".to_string(),
        );
        let second = make_contributor(
            pool.as_ref(),
            format!("Contributor {}", Uuid::new_v4()),
            "Beta".to_string(),
        );
        let mut ids = [first.contributor_id, second.contributor_id];
        ids.sort();

        let asc = Contributor::all(
            pool.as_ref(),
            2,
            0,
            None,
            ContributorOrderBy {
                field: ContributorField::ContributorId,
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
        .expect("Failed to order contributors (asc)");

        let desc = Contributor::all(
            pool.as_ref(),
            2,
            0,
            None,
            ContributorOrderBy {
                field: ContributorField::ContributorId,
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
        .expect("Failed to order contributors (desc)");

        assert_eq!(asc[0].contributor_id, ids[0]);
        assert_eq!(desc[0].contributor_id, ids[1]);
    }

    #[test]
    fn crud_count_with_filter_matches_orcid() {
        let (_guard, pool) = setup_test_db();

        let marker = "0000-0002-1825-0097";
        Contributor::create(
            pool.as_ref(),
            &NewContributor {
                first_name: Some("Filter".to_string()),
                last_name: "Match".to_string(),
                full_name: "Filter Match".to_string(),
                orcid: Some(Orcid(format!("https://orcid.org/{marker}"))),
                website: None,
            },
        )
        .expect("Failed to create contributor");
        Contributor::create(
            pool.as_ref(),
            &NewContributor {
                first_name: Some("Other".to_string()),
                last_name: "Person".to_string(),
                full_name: "Other Person".to_string(),
                orcid: None,
                website: None,
            },
        )
        .expect("Failed to create contributor");

        let count = Contributor::count(
            pool.as_ref(),
            Some(marker.to_string()),
            vec![],
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to count contributors by filter");

        assert_eq!(count, 1);
    }

    #[test]
    fn crud_ordering_by_fields_is_supported() {
        let (_guard, pool) = setup_test_db();

        Contributor::create(
            pool.as_ref(),
            &NewContributor {
                first_name: Some("Alice".to_string()),
                last_name: "Alpha".to_string(),
                full_name: "Alice Alpha".to_string(),
                orcid: Some(Orcid("https://orcid.org/0000-0001-2345-6789".to_string())),
                website: Some("https://example.com/a".to_string()),
            },
        )
        .expect("Failed to create contributor");
        Contributor::create(
            pool.as_ref(),
            &NewContributor {
                first_name: Some("Bob".to_string()),
                last_name: "Beta".to_string(),
                full_name: "Bob Beta".to_string(),
                orcid: Some(Orcid("https://orcid.org/0000-0002-3456-7890".to_string())),
                website: Some("https://example.com/b".to_string()),
            },
        )
        .expect("Failed to create contributor");

        let fields: Vec<fn() -> ContributorField> = vec![
            || ContributorField::ContributorId,
            || ContributorField::FirstName,
            || ContributorField::LastName,
            || ContributorField::FullName,
            || ContributorField::Orcid,
            || ContributorField::Website,
            || ContributorField::CreatedAt,
            || ContributorField::UpdatedAt,
        ];

        for field in fields {
            for direction in [Direction::Asc, Direction::Desc] {
                let results = Contributor::all(
                    pool.as_ref(),
                    10,
                    0,
                    None,
                    ContributorOrderBy {
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
                .expect("Failed to order contributors");

                assert_eq!(results.len(), 2);
            }
        }
    }
}
