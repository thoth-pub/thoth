use super::*;
use crate::model::Crud;
use uuid::Uuid;

fn make_new_affiliation(
    contribution_id: Uuid,
    institution_id: Uuid,
    affiliation_ordinal: i32,
    position: Option<String>,
) -> NewAffiliation {
    NewAffiliation {
        contribution_id,
        institution_id,
        affiliation_ordinal,
        position,
    }
}

fn make_patch_affiliation(
    affiliation: &Affiliation,
    affiliation_ordinal: i32,
    position: Option<String>,
) -> PatchAffiliation {
    PatchAffiliation {
        affiliation_id: affiliation.affiliation_id,
        contribution_id: affiliation.contribution_id,
        institution_id: affiliation.institution_id,
        affiliation_ordinal,
        position,
    }
}

fn make_affiliation(
    pool: &crate::db::PgPool,
    contribution_id: Uuid,
    institution_id: Uuid,
    affiliation_ordinal: i32,
) -> Affiliation {
    let new_affiliation = make_new_affiliation(
        contribution_id,
        institution_id,
        affiliation_ordinal,
        Some(format!("Position {}", Uuid::new_v4())),
    );

    Affiliation::create(pool, &new_affiliation).expect("Failed to create affiliation")
}

mod helpers {
    use super::*;
    use crate::model::{Crud, HistoryEntry};

    #[test]
    fn pk_returns_id() {
        let affiliation: Affiliation = Default::default();
        assert_eq!(affiliation.pk(), affiliation.affiliation_id);
    }

    #[test]
    fn history_entry_serializes_model() {
        let affiliation: Affiliation = Default::default();
        let user_id = "123456".to_string();
        let new_affiliation_history = affiliation.new_history_entry(&user_id);
        assert_eq!(
            new_affiliation_history.affiliation_id,
            affiliation.affiliation_id
        );
        assert_eq!(new_affiliation_history.user_id, user_id);
        assert_eq!(
            new_affiliation_history.data,
            serde_json::Value::String(serde_json::to_string(&affiliation).unwrap())
        );
    }
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;

    use crate::model::affiliation::policy::AffiliationPolicy;
    use crate::model::tests::db::{
        create_contribution, create_contributor, create_imprint, create_institution,
        create_publisher, create_work, setup_test_db, test_context_with_user, test_user_with_role,
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
        let user = test_user_with_role("affiliation-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);
        let institution = create_institution(pool.as_ref());
        let new_affiliation = make_new_affiliation(
            contribution.contribution_id,
            institution.institution_id,
            1,
            Some("Position".to_string()),
        );

        let affiliation =
            Affiliation::create(pool.as_ref(), &new_affiliation).expect("Failed to create");
        let patch = make_patch_affiliation(&affiliation, 2, Some("Updated Position".to_string()));

        assert!(AffiliationPolicy::can_create(&ctx, &new_affiliation, ()).is_ok());
        assert!(AffiliationPolicy::can_update(&ctx, &affiliation, &patch, ()).is_ok());
        assert!(AffiliationPolicy::can_delete(&ctx, &affiliation).is_ok());
        assert!(AffiliationPolicy::can_move(&ctx, &affiliation).is_ok());
    }

    #[test]
    fn crud_policy_rejects_user_without_publisher_role() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);
        let institution = create_institution(pool.as_ref());
        let affiliation = make_affiliation(
            pool.as_ref(),
            contribution.contribution_id,
            institution.institution_id,
            1,
        );
        let patch = make_patch_affiliation(&affiliation, 2, Some("Updated Position".to_string()));

        let user = test_user_with_role("affiliation-user", Role::PublisherUser, "org-other");
        let ctx = test_context_with_user(pool.clone(), user);

        let new_affiliation = make_new_affiliation(
            contribution.contribution_id,
            institution.institution_id,
            1,
            Some("Position".to_string()),
        );

        assert!(AffiliationPolicy::can_create(&ctx, &new_affiliation, ()).is_err());
        assert!(AffiliationPolicy::can_update(&ctx, &affiliation, &patch, ()).is_err());
        assert!(AffiliationPolicy::can_delete(&ctx, &affiliation).is_err());
        assert!(AffiliationPolicy::can_move(&ctx, &affiliation).is_err());
    }
}

#[cfg(feature = "backend")]
mod crud {
    use super::*;

    use crate::model::tests::db::{
        create_contribution, create_contributor, create_imprint, create_institution,
        create_publisher, create_work, setup_test_db, test_context,
    };
    use crate::model::{Crud, Reorder};

    #[test]
    fn crud_roundtrip_create_fetch_update_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);
        let institution = create_institution(pool.as_ref());

        let new_affiliation = make_new_affiliation(
            contribution.contribution_id,
            institution.institution_id,
            1,
            Some(format!("Position {}", Uuid::new_v4())),
        );

        let affiliation = Affiliation::create(pool.as_ref(), &new_affiliation)
            .expect("Failed to create affiliation");
        let fetched = Affiliation::from_id(pool.as_ref(), &affiliation.affiliation_id)
            .expect("Failed to fetch");
        assert_eq!(affiliation.affiliation_id, fetched.affiliation_id);

        let patch = make_patch_affiliation(&affiliation, 2, Some("Updated Position".to_string()));

        let ctx = test_context(pool.clone(), "test-user");
        let updated = affiliation.update(&ctx, &patch).expect("Failed to update");
        assert_eq!(updated.position, patch.position);

        let deleted = updated.delete(pool.as_ref()).expect("Failed to delete");
        assert!(Affiliation::from_id(pool.as_ref(), &deleted.affiliation_id).is_err());
    }

    #[test]
    fn crud_all_respects_limit_and_offset() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);
        let institution_one = create_institution(pool.as_ref());
        let institution_two = create_institution(pool.as_ref());

        make_affiliation(
            pool.as_ref(),
            contribution.contribution_id,
            institution_one.institution_id,
            1,
        );
        make_affiliation(
            pool.as_ref(),
            contribution.contribution_id,
            institution_two.institution_id,
            2,
        );

        let order = AffiliationOrderBy {
            field: AffiliationField::AffiliationId,
            direction: Direction::Asc,
        };

        let first = Affiliation::all(
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
        .expect("Failed to fetch affiliations");
        let second = Affiliation::all(
            pool.as_ref(),
            1,
            1,
            None,
            AffiliationOrderBy {
                field: AffiliationField::AffiliationId,
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
        .expect("Failed to fetch affiliations");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_ne!(first[0].affiliation_id, second[0].affiliation_id);
    }

    #[test]
    fn crud_count_returns_total() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);
        let institution_one = create_institution(pool.as_ref());
        let institution_two = create_institution(pool.as_ref());

        make_affiliation(
            pool.as_ref(),
            contribution.contribution_id,
            institution_one.institution_id,
            1,
        );
        make_affiliation(
            pool.as_ref(),
            contribution.contribution_id,
            institution_two.institution_id,
            2,
        );

        let count = Affiliation::count(pool.as_ref(), None, vec![], vec![], vec![], None, None)
            .expect("Failed to count affiliations");
        assert_eq!(count, 2);
    }

    #[test]
    fn crud_filter_parent_institution_id_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);
        let institution_one = create_institution(pool.as_ref());
        let institution_two = create_institution(pool.as_ref());

        let matches = make_affiliation(
            pool.as_ref(),
            contribution.contribution_id,
            institution_one.institution_id,
            1,
        );
        make_affiliation(
            pool.as_ref(),
            contribution.contribution_id,
            institution_two.institution_id,
            2,
        );

        let filtered = Affiliation::all(
            pool.as_ref(),
            10,
            0,
            None,
            AffiliationOrderBy {
                field: AffiliationField::AffiliationId,
                direction: Direction::Asc,
            },
            vec![],
            Some(institution_one.institution_id),
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter affiliations by institution");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].affiliation_id, matches.affiliation_id);
    }

    #[test]
    fn crud_ordering_by_id_respects_direction() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);
        let institution_one = create_institution(pool.as_ref());
        let institution_two = create_institution(pool.as_ref());

        let first = make_affiliation(
            pool.as_ref(),
            contribution.contribution_id,
            institution_one.institution_id,
            1,
        );
        let second = make_affiliation(
            pool.as_ref(),
            contribution.contribution_id,
            institution_two.institution_id,
            2,
        );
        let mut ids = [first.affiliation_id, second.affiliation_id];
        ids.sort();

        let asc = Affiliation::all(
            pool.as_ref(),
            2,
            0,
            None,
            AffiliationOrderBy {
                field: AffiliationField::AffiliationId,
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
        .expect("Failed to order affiliations (asc)");

        let desc = Affiliation::all(
            pool.as_ref(),
            2,
            0,
            None,
            AffiliationOrderBy {
                field: AffiliationField::AffiliationId,
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
        .expect("Failed to order affiliations (desc)");

        assert_eq!(asc[0].affiliation_id, ids[0]);
        assert_eq!(desc[0].affiliation_id, ids[1]);
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
        let institution = create_institution(pool.as_ref());

        let matches = make_affiliation(
            pool.as_ref(),
            contribution.contribution_id,
            institution.institution_id,
            1,
        );
        make_affiliation(
            pool.as_ref(),
            other_contribution.contribution_id,
            institution.institution_id,
            1,
        );

        let filtered = Affiliation::all(
            pool.as_ref(),
            10,
            0,
            None,
            AffiliationOrderBy {
                field: AffiliationField::AffiliationId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            Some(contribution.contribution_id),
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter affiliations by contribution");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].affiliation_id, matches.affiliation_id);
    }

    #[test]
    fn crud_filter_publishers_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);
        let institution = create_institution(pool.as_ref());
        let matches = make_affiliation(
            pool.as_ref(),
            contribution.contribution_id,
            institution.institution_id,
            1,
        );

        let other_publisher = create_publisher(pool.as_ref());
        let other_imprint = create_imprint(pool.as_ref(), &other_publisher);
        let other_work = create_work(pool.as_ref(), &other_imprint);
        let other_contributor = create_contributor(pool.as_ref());
        let other_contribution =
            create_contribution(pool.as_ref(), &other_work, &other_contributor);
        let other_institution = create_institution(pool.as_ref());
        make_affiliation(
            pool.as_ref(),
            other_contribution.contribution_id,
            other_institution.institution_id,
            1,
        );

        let filtered = Affiliation::all(
            pool.as_ref(),
            10,
            0,
            None,
            AffiliationOrderBy {
                field: AffiliationField::AffiliationId,
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
        .expect("Failed to filter affiliations by publisher");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].affiliation_id, matches.affiliation_id);
    }

    #[test]
    fn crud_ordering_by_fields_is_supported() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);
        let institution_one = create_institution(pool.as_ref());
        let institution_two = create_institution(pool.as_ref());

        make_affiliation(
            pool.as_ref(),
            contribution.contribution_id,
            institution_one.institution_id,
            1,
        );
        make_affiliation(
            pool.as_ref(),
            contribution.contribution_id,
            institution_two.institution_id,
            2,
        );

        let fields: Vec<fn() -> AffiliationField> = vec![
            || AffiliationField::AffiliationId,
            || AffiliationField::ContributionId,
            || AffiliationField::InstitutionId,
            || AffiliationField::AffiliationOrdinal,
            || AffiliationField::Position,
            || AffiliationField::CreatedAt,
            || AffiliationField::UpdatedAt,
        ];

        for field in fields {
            for direction in [Direction::Asc, Direction::Desc] {
                let results = Affiliation::all(
                    pool.as_ref(),
                    10,
                    0,
                    None,
                    AffiliationOrderBy {
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
                .expect("Failed to order affiliations");

                assert_eq!(results.len(), 2);
            }
        }
    }

    #[test]
    fn crud_change_ordinal_reorders_affiliations() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);
        let institution_one = create_institution(pool.as_ref());
        let institution_two = create_institution(pool.as_ref());

        let first = make_affiliation(
            pool.as_ref(),
            contribution.contribution_id,
            institution_one.institution_id,
            1,
        );
        let second = make_affiliation(
            pool.as_ref(),
            contribution.contribution_id,
            institution_two.institution_id,
            2,
        );
        let third = make_affiliation(
            pool.as_ref(),
            contribution.contribution_id,
            create_institution(pool.as_ref()).institution_id,
            3,
        );

        let ctx = test_context(pool.clone(), "test-user");
        let updated = first
            .change_ordinal(&ctx, first.affiliation_ordinal, 2)
            .expect("Failed to change affiliation ordinal");

        let refreshed_first =
            Affiliation::from_id(pool.as_ref(), &updated.affiliation_id).expect("Failed to fetch");
        let refreshed_second =
            Affiliation::from_id(pool.as_ref(), &second.affiliation_id).expect("Failed to fetch");
        let refreshed_third =
            Affiliation::from_id(pool.as_ref(), &third.affiliation_id).expect("Failed to fetch");

        assert_eq!(refreshed_first.affiliation_ordinal, 2);
        assert_eq!(refreshed_second.affiliation_ordinal, 1);
        assert_eq!(refreshed_third.affiliation_ordinal, 3);
    }

    #[test]
    fn crud_change_ordinal_move_up_reorders_affiliations() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);
        let institution_one = create_institution(pool.as_ref());
        let institution_two = create_institution(pool.as_ref());

        let first = make_affiliation(
            pool.as_ref(),
            contribution.contribution_id,
            institution_one.institution_id,
            1,
        );
        let second = make_affiliation(
            pool.as_ref(),
            contribution.contribution_id,
            institution_two.institution_id,
            2,
        );
        let third = make_affiliation(
            pool.as_ref(),
            contribution.contribution_id,
            create_institution(pool.as_ref()).institution_id,
            3,
        );

        let ctx = test_context(pool.clone(), "test-user");
        let updated = second
            .change_ordinal(&ctx, second.affiliation_ordinal, 1)
            .expect("Failed to move affiliation ordinal up");

        let refreshed_first =
            Affiliation::from_id(pool.as_ref(), &first.affiliation_id).expect("Failed to fetch");
        let refreshed_second =
            Affiliation::from_id(pool.as_ref(), &updated.affiliation_id).expect("Failed to fetch");
        let refreshed_third =
            Affiliation::from_id(pool.as_ref(), &third.affiliation_id).expect("Failed to fetch");

        assert_eq!(refreshed_second.affiliation_ordinal, 1);
        assert_eq!(refreshed_first.affiliation_ordinal, 2);
        assert_eq!(refreshed_third.affiliation_ordinal, 3);
    }
}
