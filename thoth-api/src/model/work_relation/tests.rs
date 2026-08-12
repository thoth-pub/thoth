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

/// THOTH-CHAPTER-01 / #803 Phase A — read-only existing-data audit + enforcement
/// design validation.
///
/// The approved #803 invariant is scoped to **chapter Works**: a Work with
/// `work_type = 'book-chapter'` may have at most one parent Work. These tests
/// validate the exact read-only SQL the operator will run, and (as design
/// validation only) prove that the recommended Phase B enforcement — a
/// BookChapter-scoped trigger that serialises on the chapter's `work` row — is
/// representable, closes the concurrent work_type-transition race, and behaves
/// as designed.
///
/// Qualifying semantics established from `master`:
/// - A chapter Work is `work.work_type = 'book-chapter'` (mutable; stored in the
///   `work` table, a different table from `work_relation`).
/// - A chapter's membership of a parent book is a `work_relation` row with
///   `relation_type = 'is-child-of'`, where `relator_work_id` is the chapter and
///   `related_work_id` is the parent book. The relation is stored bidirectionally:
///   the inverse row has `relation_type = 'has-child'` (relator = parent book).
/// - "More than one parent" means a **book-chapter** Work is the `relator_work_id`
///   of `is-child-of` rows pointing at more than one DISTINCT `related_work_id`.
///   Because `work_relation_relator_related_uniq (relator_work_id, related_work_id)`
///   already forbids two rows for the same (relator, related) pair, an exact
///   duplicate parent is impossible, so ">1 rows" and ">1 DISTINCT parents"
///   coincide for `is-child-of`.
///
/// A non-BookChapter Work carrying `is-child-of` relations is captured by a
/// SEPARATE semantic-consistency diagnostic and is NOT #803 blocking evidence.
///
/// Nothing here changes production write behaviour, adds a migration, or ships a
/// constraint. Trigger design-validation either runs inside a rolled-back
/// transaction or installs the triggers, exercises them, and drops them again
/// (guarded so cleanup runs even on panic).
#[cfg(feature = "backend")]
mod audit {
    use super::*;
    use std::collections::HashSet;
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    use diesel::connection::SimpleConnection;
    use diesel::result::{DatabaseErrorKind, Error as DieselError};
    use diesel::sql_types::{BigInt, Integer, Text, Uuid as SqlUuid};
    use diesel::{insert_into, sql_query, Connection, QueryableByName, RunQueryDsl};
    use thoth_errors::ThothError;

    use crate::db::PgPool;
    use crate::model::tests::db::{create_imprint, create_publisher, create_work, setup_test_db};
    use crate::model::work::{NewWork, Work, WorkStatus, WorkType};
    use crate::model::Crud;
    use crate::schema::work_relation;

    // ---------------------------------------------------------------------
    // #803 AUTHORITATIVE audit SQL (chapter Works = work_type 'book-chapter')
    // ---------------------------------------------------------------------

    /// (C) BookChapter distribution: how many book-chapter Works have 0 / 1 / >1
    /// DISTINCT parent books.
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

    /// (A) BLOCKING: book-chapter Works with >1 DISTINCT parent. Must be empty
    /// before Phase B activation. Scoped to `work_type = 'book-chapter'`.
    const AUDIT_BLOCKING_SQL: &str = "
SELECT
    wr.relator_work_id AS child_work_id,
    w.work_type::text  AS child_work_type,
    COUNT(DISTINCT wr.related_work_id) AS distinct_parent_count
FROM work_relation wr
JOIN work w ON w.work_id = wr.relator_work_id
WHERE wr.relation_type = 'is-child-of'
  AND w.work_type = 'book-chapter'
GROUP BY wr.relator_work_id, w.work_type
HAVING COUNT(DISTINCT wr.related_work_id) > 1
ORDER BY distinct_parent_count DESC, child_work_id";

    /// (B) DETAIL for each >1 book-chapter case (one row per (chapter, parent)).
    const AUDIT_BLOCKING_DETAIL_SQL: &str = "
SELECT
    wr.relator_work_id AS child_work_id,
    w.work_type::text  AS child_work_type,
    wr.related_work_id AS parent_work_id,
    wr.work_relation_id AS work_relation_id,
    wr.relation_ordinal AS relation_ordinal
FROM work_relation wr
JOIN work w ON w.work_id = wr.relator_work_id
WHERE wr.relation_type = 'is-child-of'
  AND w.work_type = 'book-chapter'
  AND wr.relator_work_id IN (
      SELECT wr2.relator_work_id
      FROM work_relation wr2
      JOIN work w2 ON w2.work_id = wr2.relator_work_id
      WHERE wr2.relation_type = 'is-child-of'
        AND w2.work_type = 'book-chapter'
      GROUP BY wr2.relator_work_id
      HAVING COUNT(DISTINCT wr2.related_work_id) > 1
  )
ORDER BY wr.relator_work_id, wr.relation_ordinal";

    /// (D) SEMANTIC-CONSISTENCY DIAGNOSTIC — NOT #803 blocking evidence.
    /// Non-BookChapter Works that carry `is-child-of` relations (those with >1
    /// first). Informational unless a later CTO decision broadens the invariant.
    const DIAGNOSTIC_NON_CHAPTER_IS_CHILD_OF_SQL: &str = "
SELECT
    wr.relator_work_id AS work_id,
    w.work_type::text  AS work_type,
    COUNT(DISTINCT wr.related_work_id) AS distinct_parent_count
FROM work_relation wr
JOIN work w ON w.work_id = wr.relator_work_id
WHERE wr.relation_type = 'is-child-of'
  AND w.work_type <> 'book-chapter'
GROUP BY wr.relator_work_id, w.work_type
ORDER BY distinct_parent_count DESC, work_id";

    // ---------------------------------------------------------------------
    // Candidate Phase B enforcement (Option A: BookChapter-scoped triggers).
    // Design validation only — ships NO trigger and NO migration.
    //
    // CORRECTNESS: both the relation trigger and the work_type-transition trigger
    // acquire the SAME per-Work serialization lock (`FOR NO KEY UPDATE` on the
    // chapter's `work` row) BEFORE reading the mutable `work_type` / counting
    // parents. This closes the race where a relation mutation reads `work_type`
    // (Monograph) and skips locking while a concurrent transition flips the Work
    // to book-chapter. A non-BookChapter is-child-of mutation therefore takes the
    // lock too (an operational synchronization effect, not a semantic expansion).
    // `RAISE ... USING CONSTRAINT` sets the error's constraint name so the
    // existing `DATABASE_CONSTRAINT_ERRORS` mapping can surface a deterministic
    // client message in Phase B.
    // ---------------------------------------------------------------------
    const CANDIDATE_TRIGGER_DDL: &str = "
CREATE OR REPLACE FUNCTION thoth_test_enforce_single_chapter_parent() RETURNS trigger
LANGUAGE plpgsql AS $fn$
DECLARE
    relator_type text;
BEGIN
    IF NEW.relation_type = 'is-child-of' THEN
        -- Lock the relator's work row AND read its type in one locking read,
        -- BEFORE deciding whether the invariant applies.
        SELECT work_type::text INTO relator_type
        FROM work WHERE work_id = NEW.relator_work_id
        FOR NO KEY UPDATE;

        IF relator_type = 'book-chapter' THEN
            IF EXISTS (
                SELECT 1 FROM work_relation
                WHERE relator_work_id = NEW.relator_work_id
                  AND relation_type = 'is-child-of'
                  AND related_work_id <> NEW.related_work_id
                  AND work_relation_id <> NEW.work_relation_id
            ) THEN
                RAISE EXCEPTION 'A book chapter may belong to only one parent work'
                    USING ERRCODE = 'unique_violation',
                          CONSTRAINT = 'work_relation_single_book_chapter_parent';
            END IF;
        END IF;
    END IF;
    RETURN NEW;
END;
$fn$;

CREATE TRIGGER thoth_test_single_chapter_parent
    BEFORE INSERT OR UPDATE ON work_relation
    FOR EACH ROW EXECUTE FUNCTION thoth_test_enforce_single_chapter_parent();

CREATE OR REPLACE FUNCTION thoth_test_enforce_chapter_parent_on_type() RETURNS trigger
LANGUAGE plpgsql AS $fn$
BEGIN
    -- Same serialization point: lock the work row before counting parents, so a
    -- concurrent in-flight parent insertion is either seen (committed) or blocks.
    PERFORM 1 FROM work WHERE work_id = NEW.work_id FOR NO KEY UPDATE;
    IF (SELECT COUNT(DISTINCT related_work_id) FROM work_relation
        WHERE relator_work_id = NEW.work_id
          AND relation_type = 'is-child-of') > 1 THEN
        RAISE EXCEPTION 'Work has multiple parents and cannot become a book chapter'
            USING ERRCODE = 'unique_violation',
                  CONSTRAINT = 'work_relation_single_book_chapter_parent';
    END IF;
    RETURN NEW;
END;
$fn$;

CREATE TRIGGER thoth_test_single_chapter_parent_on_type
    BEFORE UPDATE OF work_type ON work
    FOR EACH ROW
    WHEN (OLD.work_type IS DISTINCT FROM NEW.work_type AND NEW.work_type = 'book-chapter')
    EXECUTE FUNCTION thoth_test_enforce_chapter_parent_on_type();";

    const DROP_TRIGGER_DDL: &str = "
DROP TRIGGER IF EXISTS thoth_test_single_chapter_parent ON work_relation;
DROP TRIGGER IF EXISTS thoth_test_single_chapter_parent_on_type ON work;
DROP FUNCTION IF EXISTS thoth_test_enforce_single_chapter_parent();
DROP FUNCTION IF EXISTS thoth_test_enforce_chapter_parent_on_type();";

    const ENFORCEMENT_CONSTRAINT_NAME: &str = "work_relation_single_book_chapter_parent";

    // --- QueryableByName rows -------------------------------------------------

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
    struct BlockingChapter {
        #[diesel(sql_type = SqlUuid)]
        child_work_id: Uuid,
        #[diesel(sql_type = Text)]
        child_work_type: String,
        #[diesel(sql_type = BigInt)]
        distinct_parent_count: i64,
    }

    #[derive(QueryableByName)]
    struct BlockingDetail {
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

    #[derive(QueryableByName)]
    struct NonChapterIsChildOf {
        #[diesel(sql_type = SqlUuid)]
        work_id: Uuid,
        #[diesel(sql_type = Text)]
        work_type: String,
        #[diesel(sql_type = BigInt)]
        distinct_parent_count: i64,
    }

    #[derive(QueryableByName)]
    struct Pid {
        #[diesel(sql_type = Integer)]
        pid: i32,
    }

    #[derive(QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = BigInt)]
        n: i64,
    }

    #[derive(QueryableByName)]
    struct TextRow {
        #[diesel(sql_type = Text)]
        val: String,
    }

    // --- helpers --------------------------------------------------------------

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

    /// Insert a single raw `work_relation` row (no inverse).
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

    /// Insert BOTH sides of a chapter->parent membership so the deferred pairing
    /// FK (`work_relation_active_passive_pair`) is satisfied at COMMIT. Returns the
    /// result of the `is-child-of` insert (the row the enforcement trigger guards).
    fn insert_child_pair(
        conn: &mut diesel::PgConnection,
        chapter: Uuid,
        parent: Uuid,
        chapter_ordinal: i32,
        parent_ordinal: i32,
    ) -> Result<usize, DieselError> {
        let child = insert_direct(
            conn,
            chapter,
            parent,
            RelationType::IsChildOf,
            chapter_ordinal,
        );
        if child.is_ok() {
            let _ = insert_direct(
                conn,
                parent,
                chapter,
                RelationType::HasChild,
                parent_ordinal,
            );
        }
        child
    }

    fn install_triggers_committed(pool: &PgPool) {
        let mut conn = pool.get().expect("Failed to get DB connection");
        // Defensive: remove any triggers a previous crashed run may have leaked.
        conn.batch_execute(DROP_TRIGGER_DDL)
            .expect("Failed to drop pre-existing candidate triggers");
        conn.batch_execute(CANDIDATE_TRIGGER_DDL)
            .expect("Failed to install candidate triggers");
    }

    /// Drops the candidate triggers on Drop so they never leak into other tests,
    /// even if the test panics.
    struct TriggerGuard(Arc<PgPool>);
    impl Drop for TriggerGuard {
        fn drop(&mut self) {
            if let Ok(mut conn) = self.0.get() {
                let _ = conn.batch_execute(DROP_TRIGGER_DDL);
            }
        }
    }

    fn backend_pid(conn: &mut diesel::PgConnection) -> i32 {
        sql_query("SELECT pg_backend_pid() AS pid")
            .get_result::<Pid>(conn)
            .expect("Failed to read backend pid")
            .pid
    }

    /// Poll until the given backend is waiting on a heavyweight lock, or a bounded
    /// timeout elapses. Deterministic gate (not a fixed correctness sleep).
    fn wait_until_blocked(conn: &mut diesel::PgConnection, pid: i32) -> bool {
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            let blocked = sql_query(
                "SELECT COUNT(*) AS n FROM pg_stat_activity \
                 WHERE pid = $1 AND wait_event_type = 'Lock'",
            )
            .bind::<Integer, _>(pid)
            .get_result::<CountRow>(conn)
            .expect("Failed to poll pg_stat_activity")
            .n;
            if blocked > 0 {
                return true;
            }
            thread::sleep(Duration::from_millis(20));
        }
        false
    }

    fn work_type_of(conn: &mut diesel::PgConnection, work_id: Uuid) -> String {
        sql_query("SELECT work_type::text AS val FROM work WHERE work_id = $1")
            .bind::<SqlUuid, _>(work_id)
            .get_result::<TextRow>(conn)
            .expect("Failed to read work_type")
            .val
    }

    fn is_child_of_parent_count(conn: &mut diesel::PgConnection, work_id: Uuid) -> i64 {
        sql_query(
            "SELECT COUNT(DISTINCT related_work_id) AS n FROM work_relation \
             WHERE relator_work_id = $1 AND relation_type = 'is-child-of'",
        )
        .bind::<SqlUuid, _>(work_id)
        .get_result::<CountRow>(conn)
        .expect("Failed to count parents")
        .n
    }

    // --- audit tests (accepted; scope unchanged) ------------------------------

    #[test]
    fn audit_blocking_is_scoped_to_book_chapter_works() {
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

        // Non-chapter Works acting analogously (must NOT be #803 blocking).
        let mono_multi = create_work(pool.as_ref(), &imprint); // Monograph is-child-of 2 parents
        let part_shared = create_work(pool.as_ref(), &imprint); // Monograph is-part-of 2 sets

        relate(
            pool.as_ref(),
            b_one.work_id,
            ch_one.work_id,
            RelationType::HasChild,
            1,
        );
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
        relate(
            pool.as_ref(),
            ch_unrelated.work_id,
            b_g.work_id,
            RelationType::Replaces,
            1,
        );
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

        let summary: ChapterParentSummary = sql_query(AUDIT_SUMMARY_SQL)
            .get_result(&mut conn)
            .expect("audit summary query failed");
        assert_eq!(summary.chapters_with_zero_parents, 2);
        assert_eq!(summary.chapters_with_one_parent, 1);
        assert_eq!(summary.chapters_with_multiple_parents, 2);

        let flagged: Vec<BlockingChapter> = sql_query(AUDIT_BLOCKING_SQL)
            .load(&mut conn)
            .expect("audit blocking query failed");
        let flagged_ids: HashSet<Uuid> = flagged.iter().map(|r| r.child_work_id).collect();
        assert_eq!(
            flagged.len(),
            2,
            "only the two >1-parent book-chapters must block"
        );
        for row in &flagged {
            assert_eq!(row.distinct_parent_count, 2);
            assert_eq!(row.child_work_type, "book-chapter");
        }
        assert!(flagged_ids.contains(&ch_multi.work_id));
        assert!(flagged_ids.contains(&ch_multi2.work_id));
        assert!(!flagged_ids.contains(&mono_multi.work_id));
        assert!(!flagged_ids.contains(&ch_one.work_id));
        assert!(!flagged_ids.contains(&ch_zero.work_id));
        assert!(!flagged_ids.contains(&ch_unrelated.work_id));
        assert!(!flagged_ids.contains(&part_shared.work_id));
        assert!(!flagged_ids.contains(&b_a.work_id));

        let detail: Vec<BlockingDetail> = sql_query(AUDIT_BLOCKING_DETAIL_SQL)
            .load(&mut conn)
            .expect("audit detail query failed");
        assert!(detail.iter().all(|d| d.child_work_type == "book-chapter"));
        assert!(!detail.iter().any(|d| d.child_work_id == mono_multi.work_id));
        let ch_multi_parents: HashSet<Uuid> = detail
            .iter()
            .filter(|d| d.child_work_id == ch_multi.work_id)
            .map(|d| d.parent_work_id)
            .collect();
        assert_eq!(ch_multi_parents, HashSet::from([b_a.work_id, b_b.work_id]));
        for d in detail
            .iter()
            .filter(|d| d.child_work_id == ch_multi.work_id)
        {
            assert!(d.relation_ordinal >= 1);
            assert_ne!(d.work_relation_id, Uuid::nil());
        }

        let diagnostic: Vec<NonChapterIsChildOf> =
            sql_query(DIAGNOSTIC_NON_CHAPTER_IS_CHILD_OF_SQL)
                .load(&mut conn)
                .expect("diagnostic query failed");
        let diag_ids: HashSet<Uuid> = diagnostic.iter().map(|r| r.work_id).collect();
        assert!(diag_ids.contains(&mono_multi.work_id));
        let mono_row = diagnostic
            .iter()
            .find(|r| r.work_id == mono_multi.work_id)
            .expect("mono_multi must be in the diagnostic");
        assert_eq!(mono_row.distinct_parent_count, 2);
        assert_eq!(mono_row.work_type, "monograph");
        assert!(!diag_ids.contains(&ch_multi.work_id));
        assert!(!diag_ids.contains(&ch_multi2.work_id));
        assert!(!diag_ids.contains(&part_shared.work_id));
    }

    #[test]
    fn exact_duplicate_parent_is_already_rejected() {
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

    // --- trigger design validation (single-connection, rolled back) -----------

    #[test]
    fn enforcement_trigger_enforces_single_parent_for_book_chapter_only() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);

        let chapter = make_chapter(pool.as_ref(), imprint.imprint_id);
        let other_chapter = make_chapter(pool.as_ref(), imprint.imprint_id);
        let parent_a = create_work(pool.as_ref(), &imprint);
        let parent_b = create_work(pool.as_ref(), &imprint);
        let mono = create_work(pool.as_ref(), &imprint);
        let parent_c = create_work(pool.as_ref(), &imprint);
        let parent_d = create_work(pool.as_ref(), &imprint);
        let part = create_work(pool.as_ref(), &imprint);
        let set_one = create_work(pool.as_ref(), &imprint);
        let set_two = create_work(pool.as_ref(), &imprint);
        let big_book = create_work(pool.as_ref(), &imprint);
        let child_one = make_chapter(pool.as_ref(), imprint.imprint_id);
        let child_two = make_chapter(pool.as_ref(), imprint.imprint_id);

        let mut triggers_installed = false;
        let mut first_parent_ok = false;
        let mut second_parent_rejected = false;
        let mut other_chapter_ok = false;
        let mut non_chapter_multi_ok = false;
        let mut is_part_of_multi_ok = false;
        let mut has_child_multi_ok = false;

        let mut conn = pool.get().expect("Failed to get DB connection");
        let _ = conn.transaction::<(), DieselError, _>(|c| {
            triggers_installed = c.batch_execute(CANDIDATE_TRIGGER_DDL).is_ok();
            first_parent_ok = insert_direct(
                c,
                chapter.work_id,
                parent_a.work_id,
                RelationType::IsChildOf,
                1,
            )
            .is_ok();
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
                &second,
                Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, info))
                    if info.constraint_name() == Some(ENFORCEMENT_CONSTRAINT_NAME)
            );
            other_chapter_ok = insert_direct(
                c,
                other_chapter.work_id,
                parent_a.work_id,
                RelationType::IsChildOf,
                1,
            )
            .is_ok();
            let _ = insert_direct(
                c,
                mono.work_id,
                parent_c.work_id,
                RelationType::IsChildOf,
                1,
            );
            non_chapter_multi_ok = insert_direct(
                c,
                mono.work_id,
                parent_d.work_id,
                RelationType::IsChildOf,
                2,
            )
            .is_ok();
            let _ = insert_direct(c, part.work_id, set_one.work_id, RelationType::IsPartOf, 1);
            is_part_of_multi_ok =
                insert_direct(c, part.work_id, set_two.work_id, RelationType::IsPartOf, 2).is_ok();
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
            Err(DieselError::RollbackTransaction)
        });

        assert!(triggers_installed);
        assert!(first_parent_ok, "assigning a first parent must succeed");
        assert!(
            second_parent_rejected,
            "a second DISTINCT book-chapter parent must be rejected"
        );
        assert!(
            other_chapter_ok,
            "a different chapter may take its own single parent"
        );
        assert!(
            non_chapter_multi_ok,
            "a non-BookChapter Work must not be constrained"
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
    fn enforcement_trigger_allows_replacing_the_single_parent() {
        // Replacing Parent A with Parent B on the SAME is-child-of row (chapter
        // still has one relation) must be allowed: the trigger excludes the row
        // being updated.
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let chapter = make_chapter(pool.as_ref(), imprint.imprint_id);
        let parent_a = create_work(pool.as_ref(), &imprint);
        let parent_b = create_work(pool.as_ref(), &imprint);

        let mut replaced_ok = false;
        let mut conn = pool.get().expect("Failed to get DB connection");
        let _ = conn.transaction::<(), DieselError, _>(|c| {
            c.batch_execute(CANDIDATE_TRIGGER_DDL).unwrap();
            insert_direct(
                c,
                chapter.work_id,
                parent_a.work_id,
                RelationType::IsChildOf,
                1,
            )
            .unwrap();
            // Re-point the chapter's single parent from A to B.
            replaced_ok = sql_query(
                "UPDATE work_relation SET related_work_id = $1 \
                 WHERE relator_work_id = $2 AND relation_type = 'is-child-of'",
            )
            .bind::<SqlUuid, _>(parent_b.work_id)
            .bind::<SqlUuid, _>(chapter.work_id)
            .execute(c)
            .is_ok();
            Err(DieselError::RollbackTransaction)
        });
        assert!(replaced_ok, "replacing the single parent must be allowed");
    }

    #[test]
    fn enforcement_trigger_protects_work_type_transition_to_book_chapter() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);

        let mono = create_work(pool.as_ref(), &imprint);
        let parent_c = create_work(pool.as_ref(), &imprint);
        let parent_d = create_work(pool.as_ref(), &imprint);
        let mono_single = create_work(pool.as_ref(), &imprint);
        let parent_e = create_work(pool.as_ref(), &imprint);

        let mut triggers_installed = false;
        let mut two_parents_ok = false;
        let mut multi_transition_rejected = false;
        let mut single_transition_ok = false;

        let mut conn = pool.get().expect("Failed to get DB connection");
        let _ = conn.transaction::<(), DieselError, _>(|c| {
            triggers_installed = c.batch_execute(CANDIDATE_TRIGGER_DDL).is_ok();
            let a = insert_direct(
                c,
                mono.work_id,
                parent_c.work_id,
                RelationType::IsChildOf,
                1,
            );
            let b = insert_direct(
                c,
                mono.work_id,
                parent_d.work_id,
                RelationType::IsChildOf,
                2,
            );
            two_parents_ok = a.is_ok() && b.is_ok();
            let flip_multi = c.transaction::<(), DieselError, _>(|c2| {
                sql_query(
                    "UPDATE work SET work_type = 'book-chapter', edition = NULL WHERE work_id = $1",
                )
                .bind::<SqlUuid, _>(mono.work_id)
                .execute(c2)
                .map(|_| ())
            });
            multi_transition_rejected = matches!(
                &flip_multi,
                Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, info))
                    if info.constraint_name() == Some(ENFORCEMENT_CONSTRAINT_NAME)
            );
            let _ = insert_direct(
                c,
                mono_single.work_id,
                parent_e.work_id,
                RelationType::IsChildOf,
                1,
            );
            let flip_single = sql_query(
                "UPDATE work SET work_type = 'book-chapter', edition = NULL WHERE work_id = $1",
            )
            .bind::<SqlUuid, _>(mono_single.work_id)
            .execute(c);
            single_transition_ok = flip_single.is_ok();
            Err(DieselError::RollbackTransaction)
        });

        assert!(triggers_installed);
        assert!(
            two_parents_ok,
            "a Monograph may hold two is-child-of parents pre-enforcement"
        );
        assert!(
            multi_transition_rejected,
            "flipping a multi-parent Work to book-chapter must reject"
        );
        assert!(
            single_transition_ok,
            "a single-parent Work may become a book-chapter"
        );
    }

    // --- production write-path validation (committed triggers) ----------------

    #[test]
    fn enforcement_trigger_rejects_second_parent_via_production_create_path() {
        // Exercises the REAL WorkRelation::create path (submitted row + paired
        // inverse, transactional, ordinal advisory lock). A rejected second parent
        // must roll back BOTH rows.
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let chapter = make_chapter(pool.as_ref(), imprint.imprint_id);
        let parent_a = create_work(pool.as_ref(), &imprint);
        let parent_b = create_work(pool.as_ref(), &imprint);

        // First parent via production path (before triggers exist): succeeds.
        relate(
            pool.as_ref(),
            parent_a.work_id,
            chapter.work_id,
            RelationType::HasChild,
            1,
        );

        install_triggers_committed(pool.as_ref());
        let _cleanup = TriggerGuard(pool.clone());

        // Second DISTINCT parent via the production path must be rejected.
        let second = WorkRelation::create(
            pool.as_ref(),
            &NewWorkRelation {
                relator_work_id: parent_b.work_id,
                related_work_id: chapter.work_id,
                relation_type: RelationType::HasChild,
                relation_ordinal: 1,
            },
        );
        assert!(
            second.is_err(),
            "second distinct parent via create() must be rejected"
        );
        if let Err(err) = &second {
            let msg = format!("{err}");
            assert!(
                msg.to_lowercase().contains("parent"),
                "error should describe the single-parent rule, got: {msg}"
            );
        }

        // The paired inverse (parent_b -> has-child -> chapter) must have rolled back.
        let mut conn = pool.get().expect("conn");
        assert_eq!(
            is_child_of_parent_count(&mut conn, chapter.work_id),
            1,
            "the chapter must still have exactly one parent"
        );
        let orphan_has_child = sql_query(
            "SELECT COUNT(*) AS n FROM work_relation \
             WHERE relator_work_id = $1 AND related_work_id = $2 AND relation_type = 'has-child'",
        )
        .bind::<SqlUuid, _>(parent_b.work_id)
        .bind::<SqlUuid, _>(chapter.work_id)
        .get_result::<CountRow>(&mut conn)
        .expect("count")
        .n;
        assert_eq!(
            orphan_has_child, 0,
            "the paired inverse row must not persist"
        );
    }

    // --- deterministic two-connection concurrency evidence --------------------

    #[test]
    fn concurrent_relation_holds_lock_then_transition_is_rejected() {
        // Ordering B: the relation mutation acquires the serialization lock first
        // (W is a Monograph, so no rejection); a concurrent work_type transition to
        // book-chapter then blocks and, after the relation commits a second parent,
        // is rejected. Permitted outcome: W stays Monograph with two parents.
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let w = create_work(pool.as_ref(), &imprint); // Monograph
        let parent_a = create_work(pool.as_ref(), &imprint);
        let parent_b = create_work(pool.as_ref(), &imprint);
        relate(
            pool.as_ref(),
            parent_a.work_id,
            w.work_id,
            RelationType::HasChild,
            1,
        );

        install_triggers_committed(pool.as_ref());
        let _cleanup = TriggerGuard(pool.clone());

        let (relheld_tx, relheld_rx) = mpsc::channel::<()>();
        let (relgo_tx, relgo_rx) = mpsc::channel::<()>();
        let (typepid_tx, typepid_rx) = mpsc::channel::<i32>();
        let (typego_tx, typego_rx) = mpsc::channel::<()>();
        let (typeissue_tx, typeissue_rx) = mpsc::channel::<()>();

        let rel_pool = pool.clone();
        let (w_id, pb_id) = (w.work_id, parent_b.work_id);
        let rel = thread::spawn(move || {
            let mut c = rel_pool.get().expect("rel conn");
            c.batch_execute("BEGIN").unwrap();
            let ins = insert_child_pair(&mut c, w_id, pb_id, 2, 1); // acquires the work-row lock
            relheld_tx.send(()).unwrap();
            relgo_rx.recv().unwrap();
            let committed = ins.is_ok() && c.batch_execute("COMMIT").is_ok();
            if !committed {
                let _ = c.batch_execute("ROLLBACK");
            }
            committed
        });

        let type_pool = pool.clone();
        let w_id2 = w.work_id;
        let typ = thread::spawn(move || {
            let mut c = type_pool.get().expect("type conn");
            typepid_tx.send(backend_pid(&mut c)).unwrap();
            typego_rx.recv().unwrap();
            c.batch_execute("BEGIN").unwrap();
            typeissue_tx.send(()).unwrap();
            let res = sql_query(
                "UPDATE work SET work_type = 'book-chapter', edition = NULL WHERE work_id = $1",
            )
            .bind::<SqlUuid, _>(w_id2)
            .execute(&mut c);
            let _ = c.batch_execute("ROLLBACK");
            res.is_err()
        });

        let mut poll = pool.get().expect("poll conn");
        let type_pid = typepid_rx.recv().unwrap();
        relheld_rx.recv().unwrap();
        typego_tx.send(()).unwrap();
        typeissue_rx.recv().unwrap();
        let blocked = wait_until_blocked(&mut poll, type_pid);
        relgo_tx.send(()).unwrap();
        let rel_committed = rel.join().unwrap();
        let type_rejected = typ.join().unwrap();

        assert!(
            blocked,
            "the transition must actually block on the relation's lock"
        );
        assert!(
            rel_committed,
            "the Monograph second-parent transaction must commit"
        );
        assert!(
            type_rejected,
            "the book-chapter transition must be rejected"
        );
        let wt = work_type_of(&mut poll, w.work_id);
        let pc = is_child_of_parent_count(&mut poll, w.work_id);
        assert!(
            !(wt == "book-chapter" && pc > 1),
            "FORBIDDEN: book-chapter with >1 parent"
        );
        assert_eq!(wt, "monograph");
        assert_eq!(pc, 2);
    }

    #[test]
    fn concurrent_transition_holds_lock_then_second_parent_is_rejected() {
        // Ordering A: the work_type transition acquires the serialization lock
        // first (W has one parent, so it is allowed); a concurrent second-parent
        // insertion then blocks and, after the transition commits, is rejected.
        // Permitted outcome: W becomes book-chapter with exactly one parent.
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let w = create_work(pool.as_ref(), &imprint); // Monograph
        let parent_a = create_work(pool.as_ref(), &imprint);
        let parent_b = create_work(pool.as_ref(), &imprint);
        relate(
            pool.as_ref(),
            parent_a.work_id,
            w.work_id,
            RelationType::HasChild,
            1,
        );

        install_triggers_committed(pool.as_ref());
        let _cleanup = TriggerGuard(pool.clone());

        let (typeheld_tx, typeheld_rx) = mpsc::channel::<()>();
        let (typego_tx, typego_rx) = mpsc::channel::<()>();
        let (relpid_tx, relpid_rx) = mpsc::channel::<i32>();
        let (relgo_tx, relgo_rx) = mpsc::channel::<()>();
        let (relissue_tx, relissue_rx) = mpsc::channel::<()>();

        let type_pool = pool.clone();
        let w_id = w.work_id;
        let typ = thread::spawn(move || {
            let mut c = type_pool.get().expect("type conn");
            c.batch_execute("BEGIN").unwrap();
            // Transition to book-chapter: allowed (one parent); holds the lock.
            let res = sql_query(
                "UPDATE work SET work_type = 'book-chapter', edition = NULL WHERE work_id = $1",
            )
            .bind::<SqlUuid, _>(w_id)
            .execute(&mut c);
            typeheld_tx.send(()).unwrap();
            typego_rx.recv().unwrap();
            let committed = res.is_ok() && c.batch_execute("COMMIT").is_ok();
            if !committed {
                let _ = c.batch_execute("ROLLBACK");
            }
            committed
        });

        let rel_pool = pool.clone();
        let (w_id2, pb_id) = (w.work_id, parent_b.work_id);
        let rel = thread::spawn(move || {
            let mut c = rel_pool.get().expect("rel conn");
            relpid_tx.send(backend_pid(&mut c)).unwrap();
            relgo_rx.recv().unwrap();
            c.batch_execute("BEGIN").unwrap();
            relissue_tx.send(()).unwrap();
            let ins = insert_child_pair(&mut c, w_id2, pb_id, 2, 1); // blocks, then rejected
            let _ = c.batch_execute("ROLLBACK");
            ins.is_err()
        });

        let mut poll = pool.get().expect("poll conn");
        let rel_pid = relpid_rx.recv().unwrap();
        typeheld_rx.recv().unwrap();
        relgo_tx.send(()).unwrap();
        relissue_rx.recv().unwrap();
        let blocked = wait_until_blocked(&mut poll, rel_pid);
        typego_tx.send(()).unwrap();
        let type_committed = typ.join().unwrap();
        let rel_rejected = rel.join().unwrap();

        assert!(
            blocked,
            "the second-parent insertion must actually block on the transition's lock"
        );
        assert!(
            type_committed,
            "the single-parent book-chapter transition must commit"
        );
        assert!(rel_rejected, "the second distinct parent must be rejected");
        let wt = work_type_of(&mut poll, w.work_id);
        let pc = is_child_of_parent_count(&mut poll, w.work_id);
        assert!(
            !(wt == "book-chapter" && pc > 1),
            "FORBIDDEN: book-chapter with >1 parent"
        );
        assert_eq!(wt, "book-chapter");
        assert_eq!(pc, 1);
    }
}
