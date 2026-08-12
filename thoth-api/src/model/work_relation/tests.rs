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

#[cfg(feature = "backend")]
mod conversions {
    use super::*;
    use crate::model::tests::db::setup_test_db;
    use crate::model::tests::{assert_db_enum_roundtrip, assert_graphql_enum_roundtrip};

    #[test]
    fn relationtype_graphql_roundtrip() {
        assert_graphql_enum_roundtrip(RelationType::HasPart);
    }

    #[test]
    fn workrelationfield_graphql_roundtrip() {
        assert_graphql_enum_roundtrip(WorkRelationField::RelationType);
    }

    #[test]
    fn relationtype_db_enum_roundtrip() {
        let (_guard, pool) = setup_test_db();

        assert_db_enum_roundtrip::<RelationType, crate::schema::sql_types::RelationType>(
            pool.as_ref(),
            "'has-part'::relation_type",
            RelationType::HasPart,
        );
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

    #[test]
    fn relationtype_convert_to_inverse_pairs() {
        assert_eq!(
            RelationType::HasTranslation.convert_to_inverse(),
            RelationType::IsTranslationOf
        );
        assert_eq!(
            RelationType::IsTranslationOf.convert_to_inverse(),
            RelationType::HasTranslation
        );
        assert_eq!(
            RelationType::IsReplacedBy.convert_to_inverse(),
            RelationType::Replaces
        );
        assert_eq!(
            RelationType::Replaces.convert_to_inverse(),
            RelationType::IsReplacedBy
        );
        assert_eq!(
            RelationType::IsPartOf.convert_to_inverse(),
            RelationType::HasPart
        );
        assert_eq!(
            RelationType::HasPart.convert_to_inverse(),
            RelationType::IsPartOf
        );
        assert_eq!(
            RelationType::IsChildOf.convert_to_inverse(),
            RelationType::HasChild
        );
        assert_eq!(
            RelationType::HasChild.convert_to_inverse(),
            RelationType::IsChildOf
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
    use crate::model::Crud;
    use crate::policy::{CreatePolicy, DeletePolicy, MovePolicy, Role, UpdatePolicy};

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

        let relation = WorkRelation::create(pool.as_ref(), &new_relation)
            .expect("Failed to create work relation");
        let patch = PatchWorkRelation {
            work_relation_id: relation.work_relation_id,
            relator_work_id: relation.relator_work_id,
            related_work_id: relation.related_work_id,
            relation_type: RelationType::Replaces,
            relation_ordinal: 2,
        };

        assert!(WorkRelationPolicy::can_update(&ctx, &relation, &patch, ()).is_err());
        assert!(WorkRelationPolicy::can_delete(&ctx, &relation).is_err());
        assert!(WorkRelationPolicy::can_move(&ctx, &relation).is_err());
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

        let relation = WorkRelation::create(pool.as_ref(), &new_relation)
            .expect("Failed to create work relation");
        let patch = PatchWorkRelation {
            work_relation_id: relation.work_relation_id,
            relator_work_id: relation.relator_work_id,
            related_work_id: relation.related_work_id,
            relation_type: RelationType::Replaces,
            relation_ordinal: 2,
        };

        assert!(WorkRelationPolicy::can_update(&ctx, &relation, &patch, ()).is_ok());
        assert!(WorkRelationPolicy::can_delete(&ctx, &relation).is_ok());
        assert!(WorkRelationPolicy::can_move(&ctx, &relation).is_ok());
    }
}

#[cfg(feature = "backend")]
mod crud {
    use super::*;
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
    use thoth_errors::ThothError;

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

    #[test]
    fn crud_change_ordinal_noop_keeps_relation() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let relator = create_work(pool.as_ref(), &imprint);
        let related = create_work(pool.as_ref(), &imprint);

        let relation = make_work_relation(
            pool.as_ref(),
            relator.work_id,
            related.work_id,
            RelationType::HasPart,
            1,
        );

        let ctx = test_context(pool.clone(), "test-user");
        let updated = relation
            .change_ordinal(&ctx, relation.relation_ordinal, relation.relation_ordinal)
            .expect("Failed to no-op change ordinal");

        assert_eq!(updated.relation_ordinal, relation.relation_ordinal);
    }

    #[test]
    fn crud_change_ordinal_move_up_reorders_work_relations() {
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
        let updated = second
            .change_ordinal(&ctx, second.relation_ordinal, 1)
            .expect("Failed to move relation ordinal up");

        let refreshed_first =
            WorkRelation::from_id(pool.as_ref(), &first.work_relation_id).expect("Failed to fetch");
        let refreshed_second = WorkRelation::from_id(pool.as_ref(), &updated.work_relation_id)
            .expect("Failed to fetch");

        assert_eq!(refreshed_second.relation_ordinal, 1);
        assert_eq!(refreshed_first.relation_ordinal, 2);
    }

    #[test]
    fn crud_get_inverse_rejects_mismatched_relation_types() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let relator = create_work(pool.as_ref(), &imprint);
        let related = create_work(pool.as_ref(), &imprint);

        let relation = make_work_relation(
            pool.as_ref(),
            relator.work_id,
            related.work_id,
            RelationType::HasPart,
            1,
        );
        let inverse = relation
            .get_inverse(pool.as_ref())
            .expect("Failed to fetch inverse relation");

        let mut connection = pool.get().expect("Failed to get DB connection");
        diesel::update(
            crate::schema::work_relation::dsl::work_relation.find(inverse.work_relation_id),
        )
        .set(crate::schema::work_relation::dsl::relation_type.eq(RelationType::Replaces))
        .execute(&mut connection)
        .expect("Failed to update inverse relation type");

        let result = relation.get_inverse(pool.as_ref());
        assert!(matches!(
            result,
            Err(ThothError::InternalError(msg))
                if msg.contains("Found mismatched relation types")
        ));
    }
}

/// THOTH-CHAPTER-01 / #803 Phase A — read-only existing-data audit.
///
/// These tests validate the exact read-only SQL the operator will run to audit
/// production for chapter Works with more than one parent Work, and (as design
/// validation only) prove that the candidate Phase B enforcement mechanism — a
/// partial unique index — is representable and behaves as designed.
///
/// Qualifying semantics established from `master`:
/// - A chapter's membership of a parent book is a `work_relation` row with
///   `relation_type = 'is-child-of'`, where `relator_work_id` is the chapter and
///   `related_work_id` is the parent book. The relation is stored bidirectionally:
///   the inverse row has `relation_type = 'has-child'` (relator = parent book).
/// - "More than one parent" therefore means: a work appears as the
///   `relator_work_id` of `is-child-of` rows pointing at more than one DISTINCT
///   `related_work_id`. Because `work_relation_relator_related_uniq
///   (relator_work_id, related_work_id)` already forbids two rows for the same
///   (relator, related) pair, an exact duplicate parent is impossible, so ">1 rows"
///   and ">1 DISTINCT parents" coincide for `is-child-of`.
///
/// Nothing here changes production write behaviour, adds a migration, or ships a
/// constraint. The index-build tests run inside a rolled-back transaction against
/// the disposable test database.
#[cfg(feature = "backend")]
mod audit {
    use super::*;
    use std::collections::HashSet;

    use diesel::result::{DatabaseErrorKind, Error as DieselError};
    use diesel::sql_types::{BigInt, Integer, Text, Uuid as SqlUuid};
    use diesel::{insert_into, sql_query, Connection, QueryableByName, RunQueryDsl};
    use thoth_errors::ThothError;

    use crate::db::PgPool;
    use crate::model::tests::db::{create_imprint, create_publisher, create_work, setup_test_db};
    use crate::model::work::{NewWork, Work, WorkStatus, WorkType};
    use crate::model::Crud;
    use crate::schema::work_relation;

    /// Authoritative summary over Works whose stored type is `book-chapter`:
    /// how many have 0 / 1 / >1 DISTINCT parent books.
    const AUDIT_SUMMARY_SQL: &str = "
WITH chapter_parent_counts AS (
    SELECT
        w.work_id,
        COUNT(DISTINCT wr.related_work_id) AS distinct_parents
    FROM work w
    LEFT JOIN work_relation wr
        ON wr.relator_work_id = w.work_id
       AND wr.relation_type = 'is-child-of'
    WHERE w.work_type = 'book-chapter'
    GROUP BY w.work_id
)
SELECT
    COUNT(*) FILTER (WHERE distinct_parents = 0) AS chapters_with_zero_parents,
    COUNT(*) FILTER (WHERE distinct_parents = 1) AS chapters_with_one_parent,
    COUNT(*) FILTER (WHERE distinct_parents > 1) AS chapters_with_multiple_parents
FROM chapter_parent_counts";

    /// Authoritative BLOCKING check: every Work that is `is-child-of` more than one
    /// DISTINCT parent. This is exactly the set the candidate partial unique index
    /// would forbid, so it is computed over ALL such relators regardless of
    /// `work_type`. Any row returned must be resolved before Phase B activation.
    const AUDIT_MULTI_PARENT_SQL: &str = "
SELECT
    wr.relator_work_id AS child_work_id,
    COUNT(DISTINCT wr.related_work_id) AS distinct_parent_count
FROM work_relation wr
WHERE wr.relation_type = 'is-child-of'
GROUP BY wr.relator_work_id
HAVING COUNT(DISTINCT wr.related_work_id) > 1
ORDER BY distinct_parent_count DESC, child_work_id";

    /// Per-parent detail for every >1 case, sufficient for a data owner to resolve
    /// each affected chapter explicitly (one row per (chapter, parent)).
    const AUDIT_MULTI_PARENT_DETAIL_SQL: &str = "
SELECT
    wr.relator_work_id AS child_work_id,
    w.work_type::text AS child_work_type,
    wr.related_work_id AS parent_work_id,
    wr.work_relation_id AS work_relation_id,
    wr.relation_ordinal AS relation_ordinal
FROM work_relation wr
JOIN work w ON w.work_id = wr.relator_work_id
WHERE wr.relation_type = 'is-child-of'
  AND wr.relator_work_id IN (
      SELECT relator_work_id
      FROM work_relation
      WHERE relation_type = 'is-child-of'
      GROUP BY relator_work_id
      HAVING COUNT(DISTINCT related_work_id) > 1
  )
ORDER BY wr.relator_work_id, wr.relation_ordinal";

    /// Candidate Phase B enforcement mechanism (design validation only — NOT shipped
    /// by this Phase A change and NOT wrapped in a migration).
    const CANDIDATE_INDEX_DDL: &str =
        "CREATE UNIQUE INDEX work_relation_single_is_child_of_parent_uniq \
ON work_relation (relator_work_id) WHERE relation_type = 'is-child-of'";

    #[derive(QueryableByName)]
    struct ChapterParentSummary {
        #[diesel(sql_type = BigInt)]
        chapters_with_zero_parents: i64,
        #[diesel(sql_type = BigInt)]
        chapters_with_one_parent: i64,
        #[diesel(sql_type = BigInt)]
        chapters_with_multiple_parents: i64,
    }

    #[derive(QueryableByName)]
    struct MultiParentChild {
        #[diesel(sql_type = SqlUuid)]
        child_work_id: Uuid,
        #[diesel(sql_type = BigInt)]
        distinct_parent_count: i64,
    }

    #[derive(QueryableByName)]
    struct MultiParentDetail {
        #[diesel(sql_type = SqlUuid)]
        child_work_id: Uuid,
        #[diesel(sql_type = Text)]
        child_work_type: String,
        #[diesel(sql_type = SqlUuid)]
        parent_work_id: Uuid,
        #[diesel(sql_type = SqlUuid)]
        work_relation_id: Uuid,
        #[diesel(sql_type = Integer)]
        relation_ordinal: i32,
    }

    fn make_chapter(pool: &PgPool, imprint_id: Uuid) -> Work {
        let new_work = NewWork {
            work_type: WorkType::BookChapter,
            work_status: WorkStatus::Forthcoming,
            reference: None,
            // Chapters must not have an edition (DB check `work_chapter_no_edition`).
            edition: None,
            imprint_id,
            doi: None,
            publication_date: None,
            withdrawn_date: None,
            place: None,
            page_count: None,
            page_breakdown: None,
            image_count: None,
            table_count: None,
            audio_count: None,
            video_count: None,
            license: None,
            copyright_holder: None,
            landing_page: None,
            lccn: None,
            oclc: None,
            general_note: None,
            bibliography_note: None,
            toc: None,
            resources_description: None,
            cover_url: None,
            cover_caption: None,
            first_page: None,
            last_page: None,
            page_interval: None,
        };
        Work::create(pool, &new_work).expect("Failed to create chapter work in DB")
    }

    /// Create a relation through the real production write path (`WorkRelation::create`),
    /// which also creates the paired inverse row in the same transaction.
    fn relate(
        pool: &PgPool,
        relator: Uuid,
        related: Uuid,
        relation_type: RelationType,
        ordinal: i32,
    ) -> WorkRelation {
        WorkRelation::create(
            pool,
            &NewWorkRelation {
                relator_work_id: relator,
                related_work_id: related,
                relation_type,
                relation_ordinal: ordinal,
            },
        )
        .expect("Failed to create work relation in DB")
    }

    /// Insert a single raw `work_relation` row (no inverse) — used only inside the
    /// rolled-back index-design-validation transactions.
    fn insert_direct(
        conn: &mut diesel::PgConnection,
        relator: Uuid,
        related: Uuid,
        relation_type: RelationType,
        ordinal: i32,
    ) -> Result<usize, DieselError> {
        insert_into(work_relation::table)
            .values(&NewWorkRelation {
                relator_work_id: relator,
                related_work_id: related,
                relation_type,
                relation_ordinal: ordinal,
            })
            .execute(conn)
    }

    #[test]
    fn audit_classifies_zero_one_and_multiple_parents() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);

        // Parent books / sets (Monographs — parent-side type is irrelevant here).
        let b_one = create_work(pool.as_ref(), &imprint);
        let b_a = create_work(pool.as_ref(), &imprint);
        let b_b = create_work(pool.as_ref(), &imprint);
        let b_c = create_work(pool.as_ref(), &imprint);
        let b_d = create_work(pool.as_ref(), &imprint);
        let b_e = create_work(pool.as_ref(), &imprint);
        let b_f = create_work(pool.as_ref(), &imprint);
        let b_g = create_work(pool.as_ref(), &imprint);
        let set_p = create_work(pool.as_ref(), &imprint);
        let set_q = create_work(pool.as_ref(), &imprint);

        // Chapter Works (book-chapter).
        let ch_zero = make_chapter(pool.as_ref(), imprint.imprint_id); // 0 parents
        let ch_one = make_chapter(pool.as_ref(), imprint.imprint_id); // 1 parent
        let ch_multi = make_chapter(pool.as_ref(), imprint.imprint_id); // 2 parents (book side)
        let ch_multi2 = make_chapter(pool.as_ref(), imprint.imprint_id); // 2 parents (chapter side)
        let ch_unrelated = make_chapter(pool.as_ref(), imprint.imprint_id); // only an unrelated relation

        // Non-chapter Works acting analogously.
        let mono_multi = create_work(pool.as_ref(), &imprint); // Monograph is-child-of 2 parents
        let part_shared = create_work(pool.as_ref(), &imprint); // Monograph is-part-of 2 sets

        // 1 parent: create from the parent (book) side -> inverse is-child-of on ch_one.
        relate(
            pool.as_ref(),
            b_one.work_id,
            ch_one.work_id,
            RelationType::HasChild,
            1,
        );

        // 2 DISTINCT parents from the book side.
        relate(
            pool.as_ref(),
            b_a.work_id,
            ch_multi.work_id,
            RelationType::HasChild,
            1,
        );
        relate(
            pool.as_ref(),
            b_b.work_id,
            ch_multi.work_id,
            RelationType::HasChild,
            1,
        );

        // 2 DISTINCT parents created from the chapter side (orientation independence).
        relate(
            pool.as_ref(),
            ch_multi2.work_id,
            b_c.work_id,
            RelationType::IsChildOf,
            1,
        );
        relate(
            pool.as_ref(),
            ch_multi2.work_id,
            b_d.work_id,
            RelationType::IsChildOf,
            2,
        );

        // Non-chapter Work with 2 is-child-of parents (still caught by the blocking check).
        relate(
            pool.as_ref(),
            b_e.work_id,
            mono_multi.work_id,
            RelationType::HasChild,
            1,
        );
        relate(
            pool.as_ref(),
            b_f.work_id,
            mono_multi.work_id,
            RelationType::HasChild,
            1,
        );

        // Unrelated relation type on a chapter must NOT count as a parent.
        relate(
            pool.as_ref(),
            ch_unrelated.work_id,
            b_g.work_id,
            RelationType::Replaces,
            1,
        );

        // is-part-of to 2 sets must NOT count as chapter parents.
        relate(
            pool.as_ref(),
            set_p.work_id,
            part_shared.work_id,
            RelationType::HasPart,
            1,
        );
        relate(
            pool.as_ref(),
            set_q.work_id,
            part_shared.work_id,
            RelationType::HasPart,
            1,
        );

        let mut conn = pool.get().expect("Failed to get DB connection");

        // --- Summary over book-chapter Works ---
        let summary: ChapterParentSummary = sql_query(AUDIT_SUMMARY_SQL)
            .get_result(&mut conn)
            .expect("audit summary query failed");
        assert_eq!(summary.chapters_with_zero_parents, 2); // ch_zero, ch_unrelated
        assert_eq!(summary.chapters_with_one_parent, 1); // ch_one
        assert_eq!(summary.chapters_with_multiple_parents, 2); // ch_multi, ch_multi2

        // --- Blocking check: works with >1 DISTINCT is-child-of parent ---
        let flagged: Vec<MultiParentChild> = sql_query(AUDIT_MULTI_PARENT_SQL)
            .load(&mut conn)
            .expect("audit multi-parent query failed");
        let flagged_ids: HashSet<Uuid> = flagged.iter().map(|r| r.child_work_id).collect();
        assert_eq!(
            flagged.len(),
            3,
            "exactly the three >1-parent works must be flagged"
        );
        for row in &flagged {
            assert_eq!(row.distinct_parent_count, 2);
        }
        assert!(flagged_ids.contains(&ch_multi.work_id));
        assert!(flagged_ids.contains(&ch_multi2.work_id));
        assert!(flagged_ids.contains(&mono_multi.work_id));
        // Must NOT be flagged:
        assert!(!flagged_ids.contains(&ch_zero.work_id));
        assert!(!flagged_ids.contains(&ch_one.work_id)); // single parent
        assert!(!flagged_ids.contains(&ch_unrelated.work_id)); // unrelated relation type
        assert!(!flagged_ids.contains(&part_shared.work_id)); // is-part-of, not is-child-of
        assert!(!flagged_ids.contains(&b_a.work_id)); // parent book (opposite orientation)
        assert!(!flagged_ids.contains(&set_p.work_id)); // set (opposite orientation)

        // --- Detail rows expose the distinct parents for resolution ---
        let detail: Vec<MultiParentDetail> = sql_query(AUDIT_MULTI_PARENT_DETAIL_SQL)
            .load(&mut conn)
            .expect("audit detail query failed");
        let ch_multi_parents: HashSet<Uuid> = detail
            .iter()
            .filter(|d| d.child_work_id == ch_multi.work_id)
            .map(|d| d.parent_work_id)
            .collect();
        assert_eq!(
            ch_multi_parents,
            HashSet::from([b_a.work_id, b_b.work_id]),
            "detail must list both DISTINCT parents"
        );
        for d in detail
            .iter()
            .filter(|d| d.child_work_id == ch_multi.work_id)
        {
            assert_eq!(d.child_work_type, "book-chapter");
            assert!(d.relation_ordinal >= 1);
            assert_ne!(d.work_relation_id, Uuid::nil());
        }
    }

    #[test]
    fn exact_duplicate_parent_is_already_rejected() {
        // Idempotency semantics on master: a second, identical parent relation is
        // rejected by the existing `work_relation_relator_related_uniq` constraint.
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let book = create_work(pool.as_ref(), &imprint);
        let chapter = make_chapter(pool.as_ref(), imprint.imprint_id);

        relate(
            pool.as_ref(),
            book.work_id,
            chapter.work_id,
            RelationType::HasChild,
            1,
        );

        let duplicate = WorkRelation::create(
            pool.as_ref(),
            &NewWorkRelation {
                relator_work_id: book.work_id,
                related_work_id: chapter.work_id,
                relation_type: RelationType::HasChild,
                relation_ordinal: 2,
            },
        );
        assert!(matches!(
            &duplicate,
            Err(ThothError::DatabaseConstraintError(msg))
                if msg.contains("A relation between these two works already exists")
        ));
    }

    #[test]
    fn candidate_partial_unique_index_enforces_single_parent_over_valid_data() {
        // DESIGN VALIDATION for Phase B, executed inside a rolled-back transaction on
        // the disposable test DB. Ships no migration and no production constraint.
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);

        let chapter = make_chapter(pool.as_ref(), imprint.imprint_id);
        let other_chapter = make_chapter(pool.as_ref(), imprint.imprint_id);
        let parent_a = create_work(pool.as_ref(), &imprint);
        let parent_b = create_work(pool.as_ref(), &imprint);
        let big_book = create_work(pool.as_ref(), &imprint);
        let child_one = make_chapter(pool.as_ref(), imprint.imprint_id);
        let child_two = make_chapter(pool.as_ref(), imprint.imprint_id);
        let part = create_work(pool.as_ref(), &imprint);
        let set_one = create_work(pool.as_ref(), &imprint);
        let set_two = create_work(pool.as_ref(), &imprint);

        let mut index_built = false;
        let mut first_parent_ok = false;
        let mut second_parent_rejected = false;
        let mut other_chapter_ok = false;
        let mut is_part_of_multi_ok = false;
        let mut has_child_multi_ok = false;

        let mut conn = pool.get().expect("Failed to get DB connection");
        let _ = conn.transaction::<(), DieselError, _>(|c| {
            index_built = sql_query(CANDIDATE_INDEX_DDL).execute(c).is_ok();
            // First parent: succeeds.
            first_parent_ok = insert_direct(
                c,
                chapter.work_id,
                parent_a.work_id,
                RelationType::IsChildOf,
                1,
            )
            .is_ok();
            // Second DISTINCT parent (distinct ordinal, distinct parent): only the
            // candidate single-parent index can reject it. Use a savepoint so the
            // outer transaction survives the violation.
            let second = c.transaction::<(), DieselError, _>(|c2| {
                insert_direct(
                    c2,
                    chapter.work_id,
                    parent_b.work_id,
                    RelationType::IsChildOf,
                    2,
                )
                .map(|_| ())
            });
            second_parent_rejected = matches!(
                second,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::UniqueViolation,
                    _
                ))
            );
            // A different chapter may still take its own single parent.
            other_chapter_ok = insert_direct(
                c,
                other_chapter.work_id,
                parent_a.work_id,
                RelationType::IsChildOf,
                1,
            )
            .is_ok();
            // is-part-of is NOT covered by the index: a work may be part of many sets.
            let _ = insert_direct(c, part.work_id, set_one.work_id, RelationType::IsPartOf, 1);
            is_part_of_multi_ok =
                insert_direct(c, part.work_id, set_two.work_id, RelationType::IsPartOf, 2).is_ok();
            // has-child is NOT covered: a book may still have many children.
            let _ = insert_direct(
                c,
                big_book.work_id,
                child_one.work_id,
                RelationType::HasChild,
                1,
            );
            has_child_multi_ok = insert_direct(
                c,
                big_book.work_id,
                child_two.work_id,
                RelationType::HasChild,
                2,
            )
            .is_ok();
            // Discard the index and all rows.
            Err(DieselError::RollbackTransaction)
        });

        assert!(
            index_built,
            "candidate index must build over valid (<=1 parent) data"
        );
        assert!(first_parent_ok, "assigning a first parent must succeed");
        assert!(
            second_parent_rejected,
            "a second DISTINCT parent must be rejected by the candidate index"
        );
        assert!(
            other_chapter_ok,
            "a different chapter may take its own single parent"
        );
        assert!(
            is_part_of_multi_ok,
            "IS_PART_OF relations must remain unaffected"
        );
        assert!(
            has_child_multi_ok,
            "a book may still have multiple children"
        );
    }

    #[test]
    fn candidate_partial_unique_index_build_fails_over_corrupt_data() {
        // DESIGN VALIDATION: the enforcing index BUILD must fail (never silently
        // succeed or reparent) when a chapter already has >1 distinct parent.
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let chapter = make_chapter(pool.as_ref(), imprint.imprint_id);
        let parent_a = create_work(pool.as_ref(), &imprint);
        let parent_b = create_work(pool.as_ref(), &imprint);

        let mut both_parents_inserted = false;
        let mut index_build_failed = false;

        let mut conn = pool.get().expect("Failed to get DB connection");
        let _ = conn.transaction::<(), DieselError, _>(|c| {
            let a = insert_direct(
                c,
                chapter.work_id,
                parent_a.work_id,
                RelationType::IsChildOf,
                1,
            );
            let b = insert_direct(
                c,
                chapter.work_id,
                parent_b.work_id,
                RelationType::IsChildOf,
                2,
            );
            both_parents_inserted = a.is_ok() && b.is_ok();
            index_build_failed = sql_query(CANDIDATE_INDEX_DDL).execute(c).is_err();
            Err(DieselError::RollbackTransaction)
        });

        assert!(
            both_parents_inserted,
            "the corrupt two-parent state must be representable pre-enforcement"
        );
        assert!(
            index_build_failed,
            "index build must FAIL over corrupt >1-parent data (no silent activation)"
        );
    }
}
