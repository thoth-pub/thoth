use super::*;
use crate::model::Crud;
use uuid::Uuid;

fn make_funding(
    pool: &crate::db::PgPool,
    work_id: Uuid,
    institution_id: Uuid,
    program: Option<String>,
) -> Funding {
    let new_funding = NewFunding {
        work_id,
        institution_id,
        program,
        project_name: Some("Project Name".to_string()),
        project_shortname: Some("PRJ".to_string()),
        grant_number: Some("GRANT-1".to_string()),
    };

    Funding::create(pool, &new_funding).expect("Failed to create funding")
}

mod helpers {
    use super::*;
    use crate::model::{Crud, HistoryEntry};

    #[test]
    fn pk_returns_id() {
        let funding: Funding = Default::default();
        assert_eq!(funding.pk(), funding.funding_id);
    }

    #[test]
    fn history_entry_serializes_model() {
        let funding: Funding = Default::default();
        let user_id = "123456".to_string();
        let new_funding_history = funding.new_history_entry(&user_id);
        assert_eq!(new_funding_history.funding_id, funding.funding_id);
        assert_eq!(new_funding_history.user_id, user_id);
        assert_eq!(
            new_funding_history.data,
            serde_json::Value::String(serde_json::to_string(&funding).unwrap())
        );
    }
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;

    use crate::model::funding::policy::FundingPolicy;
    use crate::model::tests::db::{
        create_imprint, create_institution, create_publisher, create_work, setup_test_db,
        test_context_with_user, test_user_with_role,
    };
    use crate::model::Crud;
    use crate::policy::{CreatePolicy, DeletePolicy, Role, UpdatePolicy};

    #[test]
    fn crud_policy_allows_publisher_user_for_write() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("funding-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let institution = create_institution(pool.as_ref());
        let new_funding = NewFunding {
            work_id: work.work_id,
            institution_id: institution.institution_id,
            program: Some("Program".to_string()),
            project_name: Some("Project Name".to_string()),
            project_shortname: Some("PRJ".to_string()),
            grant_number: Some("GRANT-1".to_string()),
        };

        let funding = Funding::create(pool.as_ref(), &new_funding).expect("Failed to create");
        let patch = PatchFunding {
            funding_id: funding.funding_id,
            work_id: funding.work_id,
            institution_id: funding.institution_id,
            program: Some("Updated Program".to_string()),
            project_name: funding.project_name.clone(),
            project_shortname: funding.project_shortname.clone(),
            grant_number: funding.grant_number.clone(),
        };

        assert!(FundingPolicy::can_create(&ctx, &new_funding, ()).is_ok());
        assert!(FundingPolicy::can_update(&ctx, &funding, &patch, ()).is_ok());
        assert!(FundingPolicy::can_delete(&ctx, &funding).is_ok());
    }

    #[test]
    fn crud_policy_rejects_user_without_publisher_role() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let institution = create_institution(pool.as_ref());
        let funding = make_funding(
            pool.as_ref(),
            work.work_id,
            institution.institution_id,
            Some("Program".to_string()),
        );
        let patch = PatchFunding {
            funding_id: funding.funding_id,
            work_id: funding.work_id,
            institution_id: funding.institution_id,
            program: Some("Updated Program".to_string()),
            project_name: funding.project_name.clone(),
            project_shortname: funding.project_shortname.clone(),
            grant_number: funding.grant_number.clone(),
        };

        let user = test_user_with_role("funding-user", Role::PublisherUser, "org-other");
        let ctx = test_context_with_user(pool.clone(), user);

        let new_funding = NewFunding {
            work_id: work.work_id,
            institution_id: institution.institution_id,
            program: Some("Program".to_string()),
            project_name: Some("Project Name".to_string()),
            project_shortname: Some("PRJ".to_string()),
            grant_number: Some("GRANT-1".to_string()),
        };

        assert!(FundingPolicy::can_create(&ctx, &new_funding, ()).is_err());
        assert!(FundingPolicy::can_update(&ctx, &funding, &patch, ()).is_err());
        assert!(FundingPolicy::can_delete(&ctx, &funding).is_err());
    }
}

#[cfg(feature = "backend")]
mod crud {
    use super::*;

    use crate::graphql::types::inputs::{Direction, FundingOrderBy};
    use crate::model::tests::db::{
        create_imprint, create_institution, create_publisher, create_work, setup_test_db,
        test_context,
    };
    use crate::model::Crud;

    #[test]
    fn crud_roundtrip_create_fetch_update_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let institution = create_institution(pool.as_ref());

        let new_funding = NewFunding {
            work_id: work.work_id,
            institution_id: institution.institution_id,
            program: Some(format!("Program {}", Uuid::new_v4())),
            project_name: Some("Project Name".to_string()),
            project_shortname: Some("PRJ".to_string()),
            grant_number: Some("GRANT-1".to_string()),
        };

        let funding = Funding::create(pool.as_ref(), &new_funding).expect("Failed to create");
        let fetched =
            Funding::from_id(pool.as_ref(), &funding.funding_id).expect("Failed to fetch");
        assert_eq!(funding.funding_id, fetched.funding_id);

        let patch = PatchFunding {
            funding_id: funding.funding_id,
            work_id: funding.work_id,
            institution_id: funding.institution_id,
            program: Some("Updated Program".to_string()),
            project_name: funding.project_name.clone(),
            project_shortname: funding.project_shortname.clone(),
            grant_number: Some("GRANT-2".to_string()),
        };

        let ctx = test_context(pool.clone(), "test-user");
        let updated = funding.update(&ctx, &patch).expect("Failed to update");
        assert_eq!(updated.program, patch.program);

        let deleted = updated.delete(pool.as_ref()).expect("Failed to delete");
        assert!(Funding::from_id(pool.as_ref(), &deleted.funding_id).is_err());
    }

    #[test]
    fn crud_all_respects_limit_and_offset() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let institution = create_institution(pool.as_ref());

        make_funding(
            pool.as_ref(),
            work.work_id,
            institution.institution_id,
            Some(format!("Program {}", Uuid::new_v4())),
        );
        make_funding(
            pool.as_ref(),
            work.work_id,
            institution.institution_id,
            Some(format!("Program {}", Uuid::new_v4())),
        );

        let order = FundingOrderBy {
            field: FundingField::FundingId,
            direction: Direction::Asc,
        };

        let first = Funding::all(
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
        .expect("Failed to fetch fundings");
        let second = Funding::all(
            pool.as_ref(),
            1,
            1,
            None,
            FundingOrderBy {
                field: FundingField::FundingId,
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
        .expect("Failed to fetch fundings");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_ne!(first[0].funding_id, second[0].funding_id);
    }

    #[test]
    fn crud_count_returns_total() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let institution = create_institution(pool.as_ref());

        make_funding(
            pool.as_ref(),
            work.work_id,
            institution.institution_id,
            Some(format!("Program {}", Uuid::new_v4())),
        );
        make_funding(
            pool.as_ref(),
            work.work_id,
            institution.institution_id,
            Some(format!("Program {}", Uuid::new_v4())),
        );

        let count = Funding::count(pool.as_ref(), None, vec![], vec![], vec![], None, None)
            .expect("Failed to count fundings");
        assert_eq!(count, 2);
    }

    #[test]
    fn crud_filter_parent_work_id_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let other_work = create_work(pool.as_ref(), &imprint);
        let institution = create_institution(pool.as_ref());

        let matches = make_funding(
            pool.as_ref(),
            work.work_id,
            institution.institution_id,
            Some("Program Match".to_string()),
        );
        make_funding(
            pool.as_ref(),
            other_work.work_id,
            institution.institution_id,
            Some("Program Other".to_string()),
        );

        let filtered = Funding::all(
            pool.as_ref(),
            10,
            0,
            None,
            FundingOrderBy {
                field: FundingField::FundingId,
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
        .expect("Failed to filter fundings by work");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].funding_id, matches.funding_id);
    }

    #[test]
    fn crud_ordering_by_id_respects_direction() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let institution = create_institution(pool.as_ref());

        let first = make_funding(
            pool.as_ref(),
            work.work_id,
            institution.institution_id,
            Some(format!("Program {}", Uuid::new_v4())),
        );
        let second = make_funding(
            pool.as_ref(),
            work.work_id,
            institution.institution_id,
            Some(format!("Program {}", Uuid::new_v4())),
        );
        let mut ids = [first.funding_id, second.funding_id];
        ids.sort();

        let asc = Funding::all(
            pool.as_ref(),
            2,
            0,
            None,
            FundingOrderBy {
                field: FundingField::FundingId,
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
        .expect("Failed to order fundings (asc)");

        let desc = Funding::all(
            pool.as_ref(),
            2,
            0,
            None,
            FundingOrderBy {
                field: FundingField::FundingId,
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
        .expect("Failed to order fundings (desc)");

        assert_eq!(asc[0].funding_id, ids[0]);
        assert_eq!(desc[0].funding_id, ids[1]);
    }

    #[test]
    fn crud_filter_parent_institution_id_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let institution = create_institution(pool.as_ref());
        let other_institution = create_institution(pool.as_ref());

        let matches = make_funding(
            pool.as_ref(),
            work.work_id,
            institution.institution_id,
            Some("Program Match".to_string()),
        );
        make_funding(
            pool.as_ref(),
            work.work_id,
            other_institution.institution_id,
            Some("Program Other".to_string()),
        );

        let filtered = Funding::all(
            pool.as_ref(),
            10,
            0,
            None,
            FundingOrderBy {
                field: FundingField::FundingId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            Some(institution.institution_id),
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter fundings by institution");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].funding_id, matches.funding_id);
    }

    #[test]
    fn crud_filter_publishers_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let institution = create_institution(pool.as_ref());
        let matches = make_funding(
            pool.as_ref(),
            work.work_id,
            institution.institution_id,
            Some("Program Match".to_string()),
        );

        let other_publisher = create_publisher(pool.as_ref());
        let other_imprint = create_imprint(pool.as_ref(), &other_publisher);
        let other_work = create_work(pool.as_ref(), &other_imprint);
        let other_institution = create_institution(pool.as_ref());
        make_funding(
            pool.as_ref(),
            other_work.work_id,
            other_institution.institution_id,
            Some("Program Other".to_string()),
        );

        let filtered = Funding::all(
            pool.as_ref(),
            10,
            0,
            None,
            FundingOrderBy {
                field: FundingField::FundingId,
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
        .expect("Failed to filter fundings by publisher");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].funding_id, matches.funding_id);
    }

    #[test]
    fn crud_ordering_by_fields_is_supported() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let institution = create_institution(pool.as_ref());

        Funding::create(
            pool.as_ref(),
            &NewFunding {
                work_id: work.work_id,
                institution_id: institution.institution_id,
                program: Some("Program A".to_string()),
                project_name: Some("Project A".to_string()),
                project_shortname: Some("PA".to_string()),
                grant_number: Some("GRANT-A".to_string()),
            },
        )
        .expect("Failed to create funding");
        Funding::create(
            pool.as_ref(),
            &NewFunding {
                work_id: work.work_id,
                institution_id: institution.institution_id,
                program: Some("Program B".to_string()),
                project_name: Some("Project B".to_string()),
                project_shortname: Some("PB".to_string()),
                grant_number: Some("GRANT-B".to_string()),
            },
        )
        .expect("Failed to create funding");

        let fields: Vec<fn() -> FundingField> = vec![
            || FundingField::FundingId,
            || FundingField::WorkId,
            || FundingField::InstitutionId,
            || FundingField::Program,
            || FundingField::ProjectName,
            || FundingField::ProjectShortname,
            || FundingField::GrantNumber,
            || FundingField::CreatedAt,
            || FundingField::UpdatedAt,
        ];

        for field in fields {
            for direction in [Direction::Asc, Direction::Desc] {
                let results = Funding::all(
                    pool.as_ref(),
                    10,
                    0,
                    None,
                    FundingOrderBy {
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
                .expect("Failed to order fundings");

                assert_eq!(results.len(), 2);
            }
        }
    }
}
