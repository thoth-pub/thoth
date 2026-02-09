use super::*;

mod defaults {
    use super::*;

    #[test]
    fn relationtype_default_is_has_child() {
        let reltype: RelationType = Default::default();
        assert_eq!(reltype, RelationType::HasChild);
    }

    #[test]
    fn workrelationfield_default_is_relation_type() {
        let workrelfield: WorkRelationField = Default::default();
        assert_eq!(workrelfield, WorkRelationField::RelationType);
    }
}

mod display_and_parse {
    use super::*;

    #[test]
    fn relationtype_display_formats_expected_strings() {
        assert_eq!(format!("{}", RelationType::Replaces), "Replaces");
        assert_eq!(
            format!("{}", RelationType::HasTranslation),
            "Has Translation"
        );
        assert_eq!(format!("{}", RelationType::HasPart), "Has Part");
        assert_eq!(format!("{}", RelationType::HasChild), "Has Child");
        assert_eq!(format!("{}", RelationType::IsReplacedBy), "Is Replaced By");
        assert_eq!(
            format!("{}", RelationType::IsTranslationOf),
            "Is Translation Of"
        );
        assert_eq!(format!("{}", RelationType::IsPartOf), "Is Part Of");
        assert_eq!(format!("{}", RelationType::IsChildOf), "Is Child Of");
    }

    #[test]
    fn relationtype_fromstr_parses_expected_values() {
        use std::str::FromStr;
        assert_eq!(
            RelationType::from_str("Replaces").unwrap(),
            RelationType::Replaces
        );
        assert_eq!(
            RelationType::from_str("Has Translation").unwrap(),
            RelationType::HasTranslation
        );
        assert_eq!(
            RelationType::from_str("Has Part").unwrap(),
            RelationType::HasPart
        );
        assert_eq!(
            RelationType::from_str("Has Child").unwrap(),
            RelationType::HasChild
        );
        assert_eq!(
            RelationType::from_str("Is Replaced By").unwrap(),
            RelationType::IsReplacedBy
        );
        assert_eq!(
            RelationType::from_str("Is Translation Of").unwrap(),
            RelationType::IsTranslationOf
        );
        assert_eq!(
            RelationType::from_str("Is Part Of").unwrap(),
            RelationType::IsPartOf
        );
        assert_eq!(
            RelationType::from_str("Is Child Of").unwrap(),
            RelationType::IsChildOf
        );

        assert!(RelationType::from_str("Has Parent").is_err());
        assert!(RelationType::from_str("Subsumes").is_err());
    }
}

mod helpers {
    use super::*;
    use crate::model::{Crud, HistoryEntry};

    #[test]
    fn pk_returns_id() {
        let work_relation: WorkRelation = Default::default();
        assert_eq!(work_relation.pk(), work_relation.work_relation_id);
    }

    #[test]
    fn history_entry_serializes_model() {
        let work_relation: WorkRelation = Default::default();
        let user_id = "123456".to_string();
        let new_work_relation_history = work_relation.new_history_entry(&user_id);
        assert_eq!(
            new_work_relation_history.work_relation_id,
            work_relation.work_relation_id
        );
        assert_eq!(new_work_relation_history.user_id, user_id);
        assert_eq!(
            new_work_relation_history.data,
            serde_json::Value::String(serde_json::to_string(&work_relation).unwrap())
        );
    }
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;
    use std::collections::HashMap;

    use zitadel::actix::introspection::IntrospectedUser;

    use crate::model::tests::db::{
        create_imprint, create_publisher, create_work, setup_test_db, test_context_with_user,
        test_user_with_role,
    };
    use crate::model::work_relation::policy::WorkRelationPolicy;
    use crate::policy::{CreatePolicy, Role};

    fn multi_org_user(user_id: &str, role: Role, org_ids: &[String]) -> IntrospectedUser {
        let mut scoped = HashMap::new();
        for org_id in org_ids {
            scoped.insert(org_id.clone(), "role".to_string());
        }
        let mut project_roles = HashMap::new();
        project_roles.insert(role.as_ref().to_string(), scoped);

        IntrospectedUser {
            user_id: user_id.to_string(),
            username: None,
            name: None,
            given_name: None,
            family_name: None,
            preferred_username: None,
            email: None,
            email_verified: None,
            locale: None,
            project_roles: Some(project_roles),
            metadata: None,
        }
    }

    #[test]
    fn crud_policy_rejects_missing_publisher_role_for_related_work() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let other_publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let other_imprint = create_imprint(pool.as_ref(), &other_publisher);
        let relator = create_work(pool.as_ref(), &imprint);
        let related = create_work(pool.as_ref(), &other_imprint);

        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("work-relation-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let new_relation = NewWorkRelation {
            relator_work_id: relator.work_id,
            related_work_id: related.work_id,
            relation_type: RelationType::HasPart,
            relation_ordinal: 1,
        };

        assert!(WorkRelationPolicy::can_create(&ctx, &new_relation, ()).is_err());
    }

    #[test]
    fn crud_policy_allows_user_with_roles_for_both_publishers() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let other_publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let other_imprint = create_imprint(pool.as_ref(), &other_publisher);
        let relator = create_work(pool.as_ref(), &imprint);
        let related = create_work(pool.as_ref(), &other_imprint);

        let org_ids = vec![
            publisher
                .zitadel_id
                .clone()
                .expect("publisher missing zitadel id"),
            other_publisher
                .zitadel_id
                .clone()
                .expect("publisher missing zitadel id"),
        ];
        let user = multi_org_user("work-relation-user", Role::PublisherUser, &org_ids);
        let ctx = test_context_with_user(pool.clone(), user);

        let new_relation = NewWorkRelation {
            relator_work_id: relator.work_id,
            related_work_id: related.work_id,
            relation_type: RelationType::HasPart,
            relation_ordinal: 1,
        };

        assert!(WorkRelationPolicy::can_create(&ctx, &new_relation, ()).is_ok());
    }
}

#[cfg(feature = "backend")]
mod crud {
    use super::*;
    use crate::model::tests::db::{
        create_imprint, create_publisher, create_work, setup_test_db, test_context,
    };
    use crate::model::{Crud, Reorder};

    fn make_work_relation(
        pool: &crate::db::PgPool,
        relator_work_id: Uuid,
        related_work_id: Uuid,
        relation_type: RelationType,
        relation_ordinal: i32,
    ) -> WorkRelation {
        let new_relation = NewWorkRelation {
            relator_work_id,
            related_work_id,
            relation_type,
            relation_ordinal,
        };

        WorkRelation::create(pool, &new_relation).expect("Failed to create work relation")
    }

    #[test]
    fn crud_roundtrip_create_fetch_update_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let related_work = create_work(pool.as_ref(), &imprint);

        let new_relation = NewWorkRelation {
            relator_work_id: work.work_id,
            related_work_id: related_work.work_id,
            relation_type: RelationType::HasPart,
            relation_ordinal: 1,
        };

        let relation = WorkRelation::create(pool.as_ref(), &new_relation)
            .expect("Failed to create work relation");
        let fetched = WorkRelation::from_id(pool.as_ref(), &relation.work_relation_id)
            .expect("Failed to fetch");
        assert_eq!(relation.work_relation_id, fetched.work_relation_id);

        let patch = PatchWorkRelation {
            work_relation_id: relation.work_relation_id,
            relator_work_id: relation.relator_work_id,
            related_work_id: relation.related_work_id,
            relation_type: RelationType::Replaces,
            relation_ordinal: 2,
        };

        let ctx = test_context(pool.clone(), "test-user");
        let updated = relation.update(&ctx, &patch).expect("Failed to update");
        assert_eq!(updated.relation_type, patch.relation_type);

        let inverse = updated
            .get_inverse(pool.as_ref())
            .expect("Failed to fetch inverse relation");

        let deleted = updated.delete(pool.as_ref()).expect("Failed to delete");
        assert!(WorkRelation::from_id(pool.as_ref(), &deleted.work_relation_id).is_err());
        assert!(WorkRelation::from_id(pool.as_ref(), &inverse.work_relation_id).is_err());
    }

    #[test]
    fn crud_all_respects_limit_and_offset() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let relator = create_work(pool.as_ref(), &imprint);
        let related_one = create_work(pool.as_ref(), &imprint);
        let related_two = create_work(pool.as_ref(), &imprint);

        make_work_relation(
            pool.as_ref(),
            relator.work_id,
            related_one.work_id,
            RelationType::HasPart,
            1,
        );
        make_work_relation(
            pool.as_ref(),
            relator.work_id,
            related_two.work_id,
            RelationType::HasPart,
            2,
        );

        let order = WorkRelationOrderBy {
            field: WorkRelationField::WorkRelationId,
            direction: Direction::Asc,
        };

        let first = WorkRelation::all(
            pool.as_ref(),
            1,
            0,
            None,
            order.clone(),
            vec![],
            None,
            None,
            vec![RelationType::HasPart],
            vec![],
            None,
            None,
        )
        .expect("Failed to fetch work relations");
        let second = WorkRelation::all(
            pool.as_ref(),
            1,
            1,
            None,
            order,
            vec![],
            None,
            None,
            vec![RelationType::HasPart],
            vec![],
            None,
            None,
        )
        .expect("Failed to fetch work relations");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_ne!(first[0].work_relation_id, second[0].work_relation_id);
    }

    #[test]
    fn crud_count_returns_total() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let relator = create_work(pool.as_ref(), &imprint);
        let related_one = create_work(pool.as_ref(), &imprint);
        let related_two = create_work(pool.as_ref(), &imprint);

        make_work_relation(
            pool.as_ref(),
            relator.work_id,
            related_one.work_id,
            RelationType::HasPart,
            1,
        );
        make_work_relation(
            pool.as_ref(),
            relator.work_id,
            related_two.work_id,
            RelationType::HasPart,
            2,
        );

        let count = WorkRelation::count(
            pool.as_ref(),
            None,
            vec![],
            vec![RelationType::HasPart],
            vec![],
            None,
            None,
        )
        .expect("Failed to count work relations");
        assert_eq!(count, 2);
    }

    #[test]
    fn crud_filter_param_limits_relation_types() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let relator = create_work(pool.as_ref(), &imprint);
        let related_one = create_work(pool.as_ref(), &imprint);
        let related_two = create_work(pool.as_ref(), &imprint);

        let matches = make_work_relation(
            pool.as_ref(),
            relator.work_id,
            related_one.work_id,
            RelationType::HasPart,
            1,
        );
        make_work_relation(
            pool.as_ref(),
            relator.work_id,
            related_two.work_id,
            RelationType::Replaces,
            2,
        );

        let filtered = WorkRelation::all(
            pool.as_ref(),
            10,
            0,
            None,
            WorkRelationOrderBy {
                field: WorkRelationField::WorkRelationId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![RelationType::HasPart],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter work relations by type");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].work_relation_id, matches.work_relation_id);
    }

    #[test]
    fn crud_filter_parent_work_id_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let relator = create_work(pool.as_ref(), &imprint);
        let other_relator = create_work(pool.as_ref(), &imprint);
        let related = create_work(pool.as_ref(), &imprint);

        let matches = make_work_relation(
            pool.as_ref(),
            relator.work_id,
            related.work_id,
            RelationType::HasPart,
            1,
        );
        make_work_relation(
            pool.as_ref(),
            other_relator.work_id,
            related.work_id,
            RelationType::HasPart,
            2,
        );

        let filtered = WorkRelation::all(
            pool.as_ref(),
            10,
            0,
            None,
            WorkRelationOrderBy {
                field: WorkRelationField::WorkRelationId,
                direction: Direction::Asc,
            },
            vec![],
            Some(relator.work_id),
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter work relations by relator");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].work_relation_id, matches.work_relation_id);
    }

    #[test]
    fn crud_ordering_by_id_respects_direction() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let relator = create_work(pool.as_ref(), &imprint);
        let related_one = create_work(pool.as_ref(), &imprint);
        let related_two = create_work(pool.as_ref(), &imprint);

        let first = make_work_relation(
            pool.as_ref(),
            relator.work_id,
            related_one.work_id,
            RelationType::HasPart,
            1,
        );
        let second = make_work_relation(
            pool.as_ref(),
            relator.work_id,
            related_two.work_id,
            RelationType::HasPart,
            2,
        );
        let mut ids = [first.work_relation_id, second.work_relation_id];
        ids.sort();

        let asc = WorkRelation::all(
            pool.as_ref(),
            2,
            0,
            None,
            WorkRelationOrderBy {
                field: WorkRelationField::WorkRelationId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![RelationType::HasPart],
            vec![],
            None,
            None,
        )
        .expect("Failed to order work relations (asc)");

        let desc = WorkRelation::all(
            pool.as_ref(),
            2,
            0,
            None,
            WorkRelationOrderBy {
                field: WorkRelationField::WorkRelationId,
                direction: Direction::Desc,
            },
            vec![],
            None,
            None,
            vec![RelationType::HasPart],
            vec![],
            None,
            None,
        )
        .expect("Failed to order work relations (desc)");

        assert_eq!(asc[0].work_relation_id, ids[0]);
        assert_eq!(desc[0].work_relation_id, ids[1]);
    }

    #[test]
    fn crud_ordering_by_fields_is_supported() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let relator = create_work(pool.as_ref(), &imprint);
        let related_one = create_work(pool.as_ref(), &imprint);
        let related_two = create_work(pool.as_ref(), &imprint);

        make_work_relation(
            pool.as_ref(),
            relator.work_id,
            related_one.work_id,
            RelationType::HasPart,
            1,
        );
        make_work_relation(
            pool.as_ref(),
            relator.work_id,
            related_two.work_id,
            RelationType::HasPart,
            2,
        );

        let fields: Vec<fn() -> WorkRelationField> = vec![
            || WorkRelationField::WorkRelationId,
            || WorkRelationField::RelatorWorkId,
            || WorkRelationField::RelatedWorkId,
            || WorkRelationField::RelationType,
            || WorkRelationField::RelationOrdinal,
            || WorkRelationField::CreatedAt,
            || WorkRelationField::UpdatedAt,
        ];

        for field in fields {
            for direction in [Direction::Asc, Direction::Desc] {
                let results = WorkRelation::all(
                    pool.as_ref(),
                    10,
                    0,
                    None,
                    WorkRelationOrderBy {
                        field: field(),
                        direction,
                    },
                    vec![],
                    None,
                    None,
                    vec![RelationType::HasPart],
                    vec![],
                    None,
                    None,
                )
                .expect("Failed to order work relations");

                assert_eq!(results.len(), 2);
            }
        }
    }

    #[test]
    fn crud_change_ordinal_reorders_work_relations() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let relator = create_work(pool.as_ref(), &imprint);
        let related_one = create_work(pool.as_ref(), &imprint);
        let related_two = create_work(pool.as_ref(), &imprint);

        let first = make_work_relation(
            pool.as_ref(),
            relator.work_id,
            related_one.work_id,
            RelationType::HasPart,
            1,
        );
        let second = make_work_relation(
            pool.as_ref(),
            relator.work_id,
            related_two.work_id,
            RelationType::HasPart,
            2,
        );

        let ctx = test_context(pool.clone(), "test-user");
        let updated = first
            .change_ordinal(&ctx, first.relation_ordinal, 2)
            .expect("Failed to change relation ordinal");

        let refreshed_first = WorkRelation::from_id(pool.as_ref(), &updated.work_relation_id)
            .expect("Failed to fetch");
        let refreshed_second = WorkRelation::from_id(pool.as_ref(), &second.work_relation_id)
            .expect("Failed to fetch");

        assert_eq!(refreshed_first.relation_ordinal, 2);
        assert_eq!(refreshed_second.relation_ordinal, 1);
    }
}
