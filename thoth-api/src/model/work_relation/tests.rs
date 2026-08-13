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

/// THOTH-CHAPTER-01 / #803 Phase B — production enforcement of the single-parent
/// rule for BookChapter works, plus the read-only audit it depends on.
///
/// The migration `20260813_v1.6.3` installs two cooperating triggers that share
/// one per-Work serialization point (a `FOR NO KEY UPDATE` locking read of the
/// relator/target `work` row, taken BEFORE reading the mutable `work_type`). The
/// relation trigger `work_relation_single_book_chapter_parent` (BEFORE INSERT OR
/// UPDATE on work_relation) rejects a second DISTINCT `is-child-of` parent for a
/// book-chapter relator; the transition trigger
/// `work_single_book_chapter_parent_on_type` (BEFORE UPDATE OF work_type on work,
/// only for a transition INTO book-chapter) refuses the transition when the work
/// already has more than one DISTINCT `is-child-of` parent. Both raise a
/// `unique_violation` with constraint name `work_relation_single_book_chapter_parent`,
/// mapped to a deterministic client message by `DATABASE_CONSTRAINT_ERRORS`.
///
/// Because the enforcement migration is embedded, these triggers are active in
/// the shared test database. Tests that need to observe corrupt (>1-parent)
/// state — the audit-detection and migration-invalid tests — construct it either
/// with `session_replication_role = replica` inside a rolled-back transaction, or
/// on a throwaway database while enforcement is reverted.
#[cfg(feature = "backend")]
mod audit {
    use super::*;
    use std::collections::HashSet;
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    use diesel::connection::SimpleConnection;
    use diesel::result::{DatabaseErrorKind, Error as DieselError};
    use diesel::sql_types::{BigInt, Integer, Text, Uuid as SqlUuid};
    use diesel::{insert_into, sql_query, Connection, PgConnection, QueryableByName, RunQueryDsl};
    use diesel_migrations::MigrationHarness;
    use thoth_errors::{ThothError, ThothResult};

    use crate::db::{PgPool, MIGRATIONS};
    use crate::model::tests::db::{
        create_imprint, create_publisher, create_work, setup_test_db, test_db_url,
    };
    use crate::model::work::{NewWork, Work, WorkStatus, WorkType};
    use crate::model::Crud;
    use crate::schema::work_relation;

    const ENFORCEMENT_CONSTRAINT_NAME: &str = "work_relation_single_book_chapter_parent";

    /// The exact `up.sql` of the enforcement migration, used by the activation-race
    /// test to run the guard + install inside a transaction we control.
    const MIGRATION_UP_SQL: &str = include_str!("../../../migrations/20260813_v1.6.3/up.sql");

    // -- read-only audit SQL (unchanged from the approved Phase A design) -------

    const AUDIT_SUMMARY_SQL: &str = "
WITH chapter_parent_counts AS (
    SELECT w.work_id, COUNT(DISTINCT wr.related_work_id) AS distinct_parents
    FROM work w
    LEFT JOIN work_relation wr
        ON wr.relator_work_id = w.work_id AND wr.relation_type = 'is-child-of'
    WHERE w.work_type = 'book-chapter'
    GROUP BY w.work_id
)
SELECT
    COUNT(*) FILTER (WHERE distinct_parents = 0) AS chapters_with_zero_parents,
    COUNT(*) FILTER (WHERE distinct_parents = 1) AS chapters_with_one_parent,
    COUNT(*) FILTER (WHERE distinct_parents > 1) AS chapters_with_multiple_parents
FROM chapter_parent_counts";

    const AUDIT_BLOCKING_SQL: &str = "
SELECT wr.relator_work_id AS child_work_id, COUNT(DISTINCT wr.related_work_id) AS distinct_parent_count
FROM work_relation wr
JOIN work w ON w.work_id = wr.relator_work_id
WHERE wr.relation_type = 'is-child-of' AND w.work_type = 'book-chapter'
GROUP BY wr.relator_work_id
HAVING COUNT(DISTINCT wr.related_work_id) > 1
ORDER BY distinct_parent_count DESC, child_work_id";

    const DIAGNOSTIC_NON_CHAPTER_IS_CHILD_OF_SQL: &str = "
SELECT wr.relator_work_id AS work_id, w.work_type::text AS work_type, COUNT(DISTINCT wr.related_work_id) AS distinct_parent_count
FROM work_relation wr
JOIN work w ON w.work_id = wr.relator_work_id
WHERE wr.relation_type = 'is-child-of' AND w.work_type <> 'book-chapter'
GROUP BY wr.relator_work_id, w.work_type
ORDER BY distinct_parent_count DESC, work_id";

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
        #[diesel(sql_type = BigInt)]
        distinct_parent_count: i64,
    }

    #[derive(QueryableByName)]
    struct NonChapterRow {
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

    // -- helpers ---------------------------------------------------------------

    fn make_work_of_type(pool: &PgPool, imprint_id: Uuid, work_type: WorkType) -> Work {
        let edition = if work_type == WorkType::BookChapter {
            None
        } else {
            Some(1)
        };
        let new_work = NewWork {
            work_type,
            work_status: WorkStatus::Forthcoming,
            reference: None,
            edition,
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
        Work::create(pool, &new_work).expect("Failed to create work in DB")
    }

    fn make_chapter(pool: &PgPool, imprint_id: Uuid) -> Work {
        make_work_of_type(pool, imprint_id, WorkType::BookChapter)
    }

    fn new_relation(
        relator: Uuid,
        related: Uuid,
        relation_type: RelationType,
        ordinal: i32,
    ) -> NewWorkRelation {
        NewWorkRelation {
            relator_work_id: relator,
            related_work_id: related,
            relation_type,
            relation_ordinal: ordinal,
        }
    }

    /// Create a relation via the real production path, expecting success.
    fn relate(
        pool: &PgPool,
        relator: Uuid,
        related: Uuid,
        relation_type: RelationType,
        ordinal: i32,
    ) -> WorkRelation {
        WorkRelation::create(
            pool,
            &new_relation(relator, related, relation_type, ordinal),
        )
        .expect("Failed to create work relation in DB")
    }

    /// Create a relation via the real production path, returning the result.
    fn try_relate(
        pool: &PgPool,
        relator: Uuid,
        related: Uuid,
        relation_type: RelationType,
        ordinal: i32,
    ) -> ThothResult<WorkRelation> {
        WorkRelation::create(
            pool,
            &new_relation(relator, related, relation_type, ordinal),
        )
    }

    fn insert_direct(
        conn: &mut PgConnection,
        relator: Uuid,
        related: Uuid,
        relation_type: RelationType,
        ordinal: i32,
    ) -> Result<usize, DieselError> {
        insert_into(work_relation::table)
            .values(&new_relation(relator, related, relation_type, ordinal))
            .execute(conn)
    }

    /// Insert both sides of a chapter->parent membership (is-child-of first, so the
    /// enforcement trigger fires on it), satisfying the deferred pairing FK.
    fn insert_child_pair(
        conn: &mut PgConnection,
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

    fn backend_pid(conn: &mut PgConnection) -> i32 {
        sql_query("SELECT pg_backend_pid() AS pid")
            .get_result::<Pid>(conn)
            .expect("pid")
            .pid
    }

    fn wait_until_blocked(conn: &mut PgConnection, pid: i32) -> bool {
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            let blocked = sql_query(
                "SELECT COUNT(*) AS n FROM pg_stat_activity WHERE pid = $1 AND wait_event_type = 'Lock'",
            )
            .bind::<Integer, _>(pid)
            .get_result::<CountRow>(conn)
            .expect("poll pg_stat_activity")
            .n;
            if blocked > 0 {
                return true;
            }
            thread::sleep(Duration::from_millis(20));
        }
        false
    }

    fn work_type_of(conn: &mut PgConnection, work_id: Uuid) -> String {
        sql_query("SELECT work_type::text AS val FROM work WHERE work_id = $1")
            .bind::<SqlUuid, _>(work_id)
            .get_result::<TextRow>(conn)
            .expect("work_type")
            .val
    }

    fn parent_count(conn: &mut PgConnection, work_id: Uuid) -> i64 {
        sql_query(
            "SELECT COUNT(DISTINCT related_work_id) AS n FROM work_relation \
             WHERE relator_work_id = $1 AND relation_type = 'is-child-of'",
        )
        .bind::<SqlUuid, _>(work_id)
        .get_result::<CountRow>(conn)
        .expect("parent count")
        .n
    }

    fn has_child_row_count(conn: &mut PgConnection, relator: Uuid, related: Uuid) -> i64 {
        sql_query(
            "SELECT COUNT(*) AS n FROM work_relation \
             WHERE relator_work_id = $1 AND related_work_id = $2 AND relation_type = 'has-child'",
        )
        .bind::<SqlUuid, _>(relator)
        .bind::<SqlUuid, _>(related)
        .get_result::<CountRow>(conn)
        .expect("has-child count")
        .n
    }

    fn is_enforcement_violation(err: &ThothError) -> bool {
        matches!(err, ThothError::DatabaseConstraintError(msg) if msg.contains("only one parent work"))
    }

    // ======================================================================
    // 1. READ-ONLY AUDIT (pre-activation gate + migration guard predicate)
    // ======================================================================

    #[test]
    fn audit_detects_multi_parent_book_chapter() {
        // Enforcement prevents creating a >1-parent chapter, so the corrupt state
        // is injected with triggers disabled for the session, inside a rolled-back
        // transaction. The audit SELECTs run in the same transaction.
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);

        let ch_zero = make_chapter(pool.as_ref(), imprint.imprint_id);
        let ch_one = make_chapter(pool.as_ref(), imprint.imprint_id);
        let ch_multi = make_chapter(pool.as_ref(), imprint.imprint_id);
        let mono_multi = create_work(pool.as_ref(), &imprint); // Monograph
        let p_a = create_work(pool.as_ref(), &imprint);
        let p_b = create_work(pool.as_ref(), &imprint);
        let p_c = create_work(pool.as_ref(), &imprint);
        let p_d = create_work(pool.as_ref(), &imprint);
        let p_e = create_work(pool.as_ref(), &imprint);

        let mut zero = -1i64;
        let mut one = -1i64;
        let mut many = -1i64;
        let mut blocked_ids: HashSet<Uuid> = HashSet::new();
        let mut diag: Vec<(Uuid, String, i64)> = Vec::new();

        let mut conn = pool.get().expect("conn");
        let _ = conn.transaction::<(), DieselError, _>(|c| {
            c.batch_execute("SET session_replication_role = 'replica'")?;
            // ch_one: 1 parent; ch_multi: 2 parents; mono_multi: 2 parents.
            insert_direct(c, ch_one.work_id, p_e.work_id, RelationType::IsChildOf, 1)?;
            insert_direct(c, ch_multi.work_id, p_a.work_id, RelationType::IsChildOf, 1)?;
            insert_direct(c, ch_multi.work_id, p_b.work_id, RelationType::IsChildOf, 2)?;
            insert_direct(
                c,
                mono_multi.work_id,
                p_c.work_id,
                RelationType::IsChildOf,
                1,
            )?;
            insert_direct(
                c,
                mono_multi.work_id,
                p_d.work_id,
                RelationType::IsChildOf,
                2,
            )?;
            c.batch_execute("SET session_replication_role = 'origin'")?;

            let summary: ChapterParentSummary =
                sql_query(AUDIT_SUMMARY_SQL).get_result(c).expect("summary");
            zero = summary.chapters_with_zero_parents;
            one = summary.chapters_with_one_parent;
            many = summary.chapters_with_multiple_parents;

            let blocking: Vec<BlockingChapter> =
                sql_query(AUDIT_BLOCKING_SQL).load(c).expect("blocking");
            blocked_ids = blocking.iter().map(|r| r.child_work_id).collect();
            assert!(blocking.iter().all(|r| r.distinct_parent_count == 2));

            let non_chapter: Vec<NonChapterRow> = sql_query(DIAGNOSTIC_NON_CHAPTER_IS_CHILD_OF_SQL)
                .load(c)
                .expect("diag");
            diag = non_chapter
                .iter()
                .map(|r| (r.work_id, r.work_type.clone(), r.distinct_parent_count))
                .collect();

            Err(DieselError::RollbackTransaction)
        });

        assert_eq!(
            (zero, one, many),
            (1, 1, 1),
            "book-chapter 0/1/>1 distribution"
        );
        assert_eq!(blocked_ids.len(), 1);
        assert!(
            blocked_ids.contains(&ch_multi.work_id),
            "the >1-parent chapter must block"
        );
        assert!(
            !blocked_ids.contains(&mono_multi.work_id),
            "monograph is not #803 blocking"
        );
        assert!(!blocked_ids.contains(&ch_one.work_id));
        assert!(!blocked_ids.contains(&ch_zero.work_id));
        assert!(
            diag.iter()
                .any(|(id, wt, c)| *id == mono_multi.work_id && wt == "monograph" && *c == 2),
            "the monograph with two is-child-of parents must be in the non-chapter diagnostic"
        );
        assert!(
            !diag.iter().any(|(id, _, _)| *id == ch_multi.work_id),
            "book-chapters must not appear in the non-chapter diagnostic"
        );
    }

    // ======================================================================
    // 2. ENFORCEMENT via the production WorkRelation::create path
    // ======================================================================

    #[test]
    fn first_parent_allowed_then_second_rejected_has_child_orientation() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let chapter = make_chapter(pool.as_ref(), imprint.imprint_id);
        let parent_a = create_work(pool.as_ref(), &imprint);
        let parent_b = create_work(pool.as_ref(), &imprint);

        // First parent (Parent -> HasChild -> Chapter): allowed.
        relate(
            pool.as_ref(),
            parent_a.work_id,
            chapter.work_id,
            RelationType::HasChild,
            1,
        );

        // Second distinct parent, same orientation: rejected, inverse rolled back.
        let second = try_relate(
            pool.as_ref(),
            parent_b.work_id,
            chapter.work_id,
            RelationType::HasChild,
            1,
        );
        assert!(matches!(&second, Err(e) if is_enforcement_violation(e)));

        let mut conn = pool.get().expect("conn");
        assert_eq!(
            parent_count(&mut conn, chapter.work_id),
            1,
            "chapter keeps one parent"
        );
        assert_eq!(
            has_child_row_count(&mut conn, parent_b.work_id, chapter.work_id),
            0,
            "the paired inverse row must have rolled back"
        );
    }

    #[test]
    fn second_parent_rejected_is_child_of_orientation() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let chapter = make_chapter(pool.as_ref(), imprint.imprint_id);
        let parent_a = create_work(pool.as_ref(), &imprint);
        let parent_b = create_work(pool.as_ref(), &imprint);

        // First parent submitted as Chapter -> IsChildOf -> Parent.
        relate(
            pool.as_ref(),
            chapter.work_id,
            parent_a.work_id,
            RelationType::IsChildOf,
            1,
        );

        // Second distinct parent, same orientation: rejected; the paired inverse
        // (Parent B -> HasChild -> Chapter), inserted first inside create(), rolls back.
        let second = try_relate(
            pool.as_ref(),
            chapter.work_id,
            parent_b.work_id,
            RelationType::IsChildOf,
            2,
        );
        assert!(matches!(&second, Err(e) if is_enforcement_violation(e)));

        let mut conn = pool.get().expect("conn");
        assert_eq!(parent_count(&mut conn, chapter.work_id), 1);
        assert_eq!(
            has_child_row_count(&mut conn, parent_b.work_id, chapter.work_id),
            0
        );
    }

    #[test]
    fn exact_duplicate_parent_still_rejected_by_existing_constraint() {
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
        let dup = try_relate(
            pool.as_ref(),
            book.work_id,
            chapter.work_id,
            RelationType::HasChild,
            2,
        );
        assert!(matches!(
            &dup,
            Err(ThothError::DatabaseConstraintError(msg))
                if msg.contains("A relation between these two works already exists")
        ));
    }

    // ======================================================================
    // 3. RELATION-UPDATE SEMANTICS (trigger-level, rolled back)
    // ======================================================================

    #[test]
    fn replacing_the_single_parent_is_allowed() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let chapter = make_chapter(pool.as_ref(), imprint.imprint_id);
        let parent_a = create_work(pool.as_ref(), &imprint);
        let parent_b = create_work(pool.as_ref(), &imprint);
        relate(
            pool.as_ref(),
            chapter.work_id,
            parent_a.work_id,
            RelationType::IsChildOf,
            1,
        );

        let mut allowed = false;
        let mut conn = pool.get().expect("conn");
        let _ = conn.transaction::<(), DieselError, _>(|c| {
            allowed = sql_query(
                "UPDATE work_relation SET related_work_id = $1 \
                 WHERE relator_work_id = $2 AND relation_type = 'is-child-of'",
            )
            .bind::<SqlUuid, _>(parent_b.work_id)
            .bind::<SqlUuid, _>(chapter.work_id)
            .execute(c)
            .is_ok();
            Err(DieselError::RollbackTransaction)
        });
        assert!(
            allowed,
            "re-pointing the chapter's only parent must be allowed"
        );
    }

    #[test]
    fn changing_a_relation_into_a_second_is_child_of_parent_is_rejected() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let chapter = make_chapter(pool.as_ref(), imprint.imprint_id);
        let parent_a = create_work(pool.as_ref(), &imprint);
        let other = create_work(pool.as_ref(), &imprint);
        relate(
            pool.as_ref(),
            chapter.work_id,
            parent_a.work_id,
            RelationType::IsChildOf,
            1,
        );
        // A separate relation from the chapter to `other`, not yet a parent.
        relate(
            pool.as_ref(),
            chapter.work_id,
            other.work_id,
            RelationType::IsPartOf,
            2,
        );

        let mut rejected = false;
        let mut conn = pool.get().expect("conn");
        let _ = conn.transaction::<(), DieselError, _>(|c| {
            let r = c.transaction::<(), DieselError, _>(|c2| {
                sql_query(
                    "UPDATE work_relation SET relation_type = 'is-child-of' \
                     WHERE relator_work_id = $1 AND related_work_id = $2",
                )
                .bind::<SqlUuid, _>(chapter.work_id)
                .bind::<SqlUuid, _>(other.work_id)
                .execute(c2)
                .map(|_| ())
            });
            rejected = matches!(
                &r,
                Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, info))
                    if info.constraint_name() == Some(ENFORCEMENT_CONSTRAINT_NAME)
            );
            Err(DieselError::RollbackTransaction)
        });
        assert!(
            rejected,
            "turning another relation into a second is-child-of parent must reject"
        );
    }

    #[test]
    fn moving_a_relation_onto_a_book_chapter_with_a_parent_is_rejected() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let chapter = make_chapter(pool.as_ref(), imprint.imprint_id);
        let parent_a = create_work(pool.as_ref(), &imprint);
        let mono = create_work(pool.as_ref(), &imprint);
        let z = create_work(pool.as_ref(), &imprint);
        relate(
            pool.as_ref(),
            chapter.work_id,
            parent_a.work_id,
            RelationType::IsChildOf,
            1,
        );
        // An is-child-of relation owned by a Monograph, that we try to move onto the chapter.
        relate(
            pool.as_ref(),
            mono.work_id,
            z.work_id,
            RelationType::IsChildOf,
            1,
        );

        let mut rejected = false;
        let mut conn = pool.get().expect("conn");
        let _ = conn.transaction::<(), DieselError, _>(|c| {
            let r = c.transaction::<(), DieselError, _>(|c2| {
                sql_query(
                    "UPDATE work_relation SET relator_work_id = $1 \
                     WHERE relator_work_id = $2 AND related_work_id = $3 AND relation_type = 'is-child-of'",
                )
                .bind::<SqlUuid, _>(chapter.work_id)
                .bind::<SqlUuid, _>(mono.work_id)
                .bind::<SqlUuid, _>(z.work_id)
                .execute(c2)
                .map(|_| ())
            });
            rejected = matches!(
                &r,
                Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, info))
                    if info.constraint_name() == Some(ENFORCEMENT_CONSTRAINT_NAME)
            );
            Err(DieselError::RollbackTransaction)
        });
        assert!(
            rejected,
            "moving a relation onto a chapter that already has a parent must reject"
        );
    }

    #[test]
    fn changing_a_relation_away_from_is_child_of_is_allowed() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let chapter = make_chapter(pool.as_ref(), imprint.imprint_id);
        let parent_a = create_work(pool.as_ref(), &imprint);
        relate(
            pool.as_ref(),
            chapter.work_id,
            parent_a.work_id,
            RelationType::IsChildOf,
            1,
        );

        let mut allowed = false;
        let mut conn = pool.get().expect("conn");
        let _ = conn.transaction::<(), DieselError, _>(|c| {
            allowed = sql_query(
                "UPDATE work_relation SET relation_type = 'is-part-of' \
                 WHERE relator_work_id = $1 AND relation_type = 'is-child-of'",
            )
            .bind::<SqlUuid, _>(chapter.work_id)
            .execute(c)
            .is_ok();
            Err(DieselError::RollbackTransaction)
        });
        assert!(
            allowed,
            "moving a row away from is-child-of must be allowed by #803"
        );
    }

    // ======================================================================
    // 4. WORKTYPE TRANSITION
    // ======================================================================

    #[test]
    fn transition_to_book_chapter_rejected_when_multiple_parents() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let mono = create_work(pool.as_ref(), &imprint); // Monograph
        let p_c = create_work(pool.as_ref(), &imprint);
        let p_d = create_work(pool.as_ref(), &imprint);
        // A Monograph may legitimately hold two is-child-of parents.
        relate(
            pool.as_ref(),
            p_c.work_id,
            mono.work_id,
            RelationType::HasChild,
            1,
        );
        relate(
            pool.as_ref(),
            p_d.work_id,
            mono.work_id,
            RelationType::HasChild,
            1,
        );

        let mut conn = pool.get().expect("conn");
        let res = sql_query(
            "UPDATE work SET work_type = 'book-chapter', edition = NULL WHERE work_id = $1",
        )
        .bind::<SqlUuid, _>(mono.work_id)
        .execute(&mut conn);
        assert!(matches!(
            &res,
            Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, info))
                if info.constraint_name() == Some(ENFORCEMENT_CONSTRAINT_NAME)
        ));
        assert_eq!(
            work_type_of(&mut conn, mono.work_id),
            "monograph",
            "type unchanged"
        );
    }

    #[test]
    fn transition_to_book_chapter_allowed_when_single_parent() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let mono = create_work(pool.as_ref(), &imprint);
        let p_c = create_work(pool.as_ref(), &imprint);
        relate(
            pool.as_ref(),
            p_c.work_id,
            mono.work_id,
            RelationType::HasChild,
            1,
        );

        let mut conn = pool.get().expect("conn");
        let res = sql_query(
            "UPDATE work SET work_type = 'book-chapter', edition = NULL WHERE work_id = $1",
        )
        .bind::<SqlUuid, _>(mono.work_id)
        .execute(&mut conn);
        assert!(
            res.is_ok(),
            "a single-parent work may become a book-chapter"
        );
        assert_eq!(work_type_of(&mut conn, mono.work_id), "book-chapter");
    }

    // ======================================================================
    // 5. NON-BOOKCHAPTER REGRESSIONS
    // ======================================================================

    #[test]
    fn non_book_chapter_works_are_not_constrained() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);

        // Monograph with two is-child-of parents: allowed.
        let mono = create_work(pool.as_ref(), &imprint);
        relate(
            pool.as_ref(),
            create_work(pool.as_ref(), &imprint).work_id,
            mono.work_id,
            RelationType::HasChild,
            1,
        );
        relate(
            pool.as_ref(),
            create_work(pool.as_ref(), &imprint).work_id,
            mono.work_id,
            RelationType::HasChild,
            1,
        );

        // EditedBook with two is-child-of parents: not constrained by #803.
        let edited = make_work_of_type(pool.as_ref(), imprint.imprint_id, WorkType::EditedBook);
        relate(
            pool.as_ref(),
            create_work(pool.as_ref(), &imprint).work_id,
            edited.work_id,
            RelationType::HasChild,
            1,
        );
        relate(
            pool.as_ref(),
            create_work(pool.as_ref(), &imprint).work_id,
            edited.work_id,
            RelationType::HasChild,
            1,
        );

        // A book may have many children (has-child on the parent side).
        let big_book = create_work(pool.as_ref(), &imprint);
        relate(
            pool.as_ref(),
            big_book.work_id,
            make_chapter(pool.as_ref(), imprint.imprint_id).work_id,
            RelationType::HasChild,
            1,
        );
        relate(
            pool.as_ref(),
            big_book.work_id,
            make_chapter(pool.as_ref(), imprint.imprint_id).work_id,
            RelationType::HasChild,
            2,
        );

        // is-part-of to multiple sets is unaffected.
        let part = create_work(pool.as_ref(), &imprint);
        relate(
            pool.as_ref(),
            create_work(pool.as_ref(), &imprint).work_id,
            part.work_id,
            RelationType::HasPart,
            1,
        );
        relate(
            pool.as_ref(),
            create_work(pool.as_ref(), &imprint).work_id,
            part.work_id,
            RelationType::HasPart,
            1,
        );

        let mut conn = pool.get().expect("conn");
        assert_eq!(parent_count(&mut conn, mono.work_id), 2);
        assert_eq!(parent_count(&mut conn, edited.work_id), 2);
    }

    // ======================================================================
    // 6. DETERMINISTIC TWO-CONNECTION CONCURRENCY (real triggers)
    // ======================================================================

    #[test]
    fn concurrent_two_parents_at_most_one_commits() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let chapter = make_chapter(pool.as_ref(), imprint.imprint_id);
        let parent_a = create_work(pool.as_ref(), &imprint);
        let parent_b = create_work(pool.as_ref(), &imprint);

        let (a_held_tx, a_held_rx) = mpsc::channel::<()>();
        let (a_go_tx, a_go_rx) = mpsc::channel::<()>();
        let (b_pid_tx, b_pid_rx) = mpsc::channel::<i32>();
        let (b_go_tx, b_go_rx) = mpsc::channel::<()>();
        let (b_issue_tx, b_issue_rx) = mpsc::channel::<()>();

        let a_pool = pool.clone();
        let (ch, pa) = (chapter.work_id, parent_a.work_id);
        let t_a = thread::spawn(move || {
            let mut c = a_pool.get().expect("a conn");
            c.batch_execute("BEGIN").unwrap();
            let ins = insert_child_pair(&mut c, ch, pa, 1, 1); // locks the chapter work row
            a_held_tx.send(()).unwrap();
            a_go_rx.recv().unwrap();
            let ok = ins.is_ok() && c.batch_execute("COMMIT").is_ok();
            if !ok {
                let _ = c.batch_execute("ROLLBACK");
            }
            ok
        });

        let b_pool = pool.clone();
        let (ch2, pb) = (chapter.work_id, parent_b.work_id);
        let t_b = thread::spawn(move || {
            let mut c = b_pool.get().expect("b conn");
            b_pid_tx.send(backend_pid(&mut c)).unwrap();
            b_go_rx.recv().unwrap();
            c.batch_execute("BEGIN").unwrap();
            b_issue_tx.send(()).unwrap();
            let ins = insert_child_pair(&mut c, ch2, pb, 2, 1); // blocks then rejects
            let _ = c.batch_execute("ROLLBACK");
            ins.is_err()
        });

        let mut poll = pool.get().expect("poll");
        let b_pid = b_pid_rx.recv().unwrap();
        a_held_rx.recv().unwrap();
        b_go_tx.send(()).unwrap();
        b_issue_rx.recv().unwrap();
        let blocked = wait_until_blocked(&mut poll, b_pid);
        a_go_tx.send(()).unwrap();
        let a_ok = t_a.join().unwrap();
        let b_rejected = t_b.join().unwrap();

        assert!(
            blocked,
            "the second inserter must block on the chapter work-row lock"
        );
        assert!(a_ok, "exactly one parent assignment commits");
        assert!(b_rejected, "the other is rejected");
        assert_eq!(
            parent_count(&mut poll, chapter.work_id),
            1,
            "chapter ends with one parent"
        );
    }

    #[test]
    fn concurrent_relation_first_then_transition_rejected() {
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

        let (rel_held_tx, rel_held_rx) = mpsc::channel::<()>();
        let (rel_go_tx, rel_go_rx) = mpsc::channel::<()>();
        let (type_pid_tx, type_pid_rx) = mpsc::channel::<i32>();
        let (type_go_tx, type_go_rx) = mpsc::channel::<()>();
        let (type_issue_tx, type_issue_rx) = mpsc::channel::<()>();

        let rel_pool = pool.clone();
        let (w_id, pb) = (w.work_id, parent_b.work_id);
        let t_rel = thread::spawn(move || {
            let mut c = rel_pool.get().expect("rel conn");
            c.batch_execute("BEGIN").unwrap();
            let ins = insert_child_pair(&mut c, w_id, pb, 2, 1);
            rel_held_tx.send(()).unwrap();
            rel_go_rx.recv().unwrap();
            let ok = ins.is_ok() && c.batch_execute("COMMIT").is_ok();
            if !ok {
                let _ = c.batch_execute("ROLLBACK");
            }
            ok
        });

        let type_pool = pool.clone();
        let w_id2 = w.work_id;
        let t_type = thread::spawn(move || {
            let mut c = type_pool.get().expect("type conn");
            type_pid_tx.send(backend_pid(&mut c)).unwrap();
            type_go_rx.recv().unwrap();
            c.batch_execute("BEGIN").unwrap();
            type_issue_tx.send(()).unwrap();
            let res = sql_query(
                "UPDATE work SET work_type = 'book-chapter', edition = NULL WHERE work_id = $1",
            )
            .bind::<SqlUuid, _>(w_id2)
            .execute(&mut c);
            let _ = c.batch_execute("ROLLBACK");
            res.is_err()
        });

        let mut poll = pool.get().expect("poll");
        let type_pid = type_pid_rx.recv().unwrap();
        rel_held_rx.recv().unwrap();
        type_go_tx.send(()).unwrap();
        type_issue_rx.recv().unwrap();
        let blocked = wait_until_blocked(&mut poll, type_pid);
        rel_go_tx.send(()).unwrap();
        let rel_ok = t_rel.join().unwrap();
        let type_rejected = t_type.join().unwrap();

        assert!(blocked, "the transition must block on the relation's lock");
        assert!(rel_ok, "the monograph second-parent commits");
        assert!(type_rejected, "the transition is rejected");
        assert_eq!(work_type_of(&mut poll, w.work_id), "monograph");
        assert_eq!(parent_count(&mut poll, w.work_id), 2);
    }

    #[test]
    fn concurrent_transition_first_then_relation_rejected() {
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

        let (type_held_tx, type_held_rx) = mpsc::channel::<()>();
        let (type_go_tx, type_go_rx) = mpsc::channel::<()>();
        let (rel_pid_tx, rel_pid_rx) = mpsc::channel::<i32>();
        let (rel_go_tx, rel_go_rx) = mpsc::channel::<()>();
        let (rel_issue_tx, rel_issue_rx) = mpsc::channel::<()>();

        let type_pool = pool.clone();
        let w_id = w.work_id;
        let t_type = thread::spawn(move || {
            let mut c = type_pool.get().expect("type conn");
            c.batch_execute("BEGIN").unwrap();
            let res = sql_query(
                "UPDATE work SET work_type = 'book-chapter', edition = NULL WHERE work_id = $1",
            )
            .bind::<SqlUuid, _>(w_id)
            .execute(&mut c);
            type_held_tx.send(()).unwrap();
            type_go_rx.recv().unwrap();
            let ok = res.is_ok() && c.batch_execute("COMMIT").is_ok();
            if !ok {
                let _ = c.batch_execute("ROLLBACK");
            }
            ok
        });

        let rel_pool = pool.clone();
        let (w_id2, pb) = (w.work_id, parent_b.work_id);
        let t_rel = thread::spawn(move || {
            let mut c = rel_pool.get().expect("rel conn");
            rel_pid_tx.send(backend_pid(&mut c)).unwrap();
            rel_go_rx.recv().unwrap();
            c.batch_execute("BEGIN").unwrap();
            rel_issue_tx.send(()).unwrap();
            let ins = insert_child_pair(&mut c, w_id2, pb, 2, 1);
            let _ = c.batch_execute("ROLLBACK");
            ins.is_err()
        });

        let mut poll = pool.get().expect("poll");
        let rel_pid = rel_pid_rx.recv().unwrap();
        type_held_rx.recv().unwrap();
        rel_go_tx.send(()).unwrap();
        rel_issue_rx.recv().unwrap();
        let blocked = wait_until_blocked(&mut poll, rel_pid);
        type_go_tx.send(()).unwrap();
        let type_ok = t_type.join().unwrap();
        let rel_rejected = t_rel.join().unwrap();

        assert!(
            blocked,
            "the second-parent insert must block on the transition's lock"
        );
        assert!(
            type_ok,
            "the transition to book-chapter commits with one parent"
        );
        assert!(rel_rejected, "the second parent is rejected");
        assert_eq!(work_type_of(&mut poll, w.work_id), "book-chapter");
        assert_eq!(parent_count(&mut poll, w.work_id), 1);
    }

    // ======================================================================
    // 7. MIGRATION TESTS (isolated throwaway databases)
    // ======================================================================

    struct TempMigrationDb {
        admin_url: String,
        name: String,
    }

    impl TempMigrationDb {
        fn new() -> Self {
            let admin_url = test_db_url();
            let name = format!("thoth_mig_{}", Uuid::new_v4().simple());
            let mut admin = PgConnection::establish(&admin_url).expect("admin conn");
            admin
                .batch_execute(&format!("CREATE DATABASE \"{name}\""))
                .expect("create temp db");
            TempMigrationDb { admin_url, name }
        }

        fn url(&self) -> String {
            let (prefix, _) = self.admin_url.rsplit_once('/').expect("db url has a path");
            format!("{prefix}/{}", self.name)
        }

        fn conn(&self) -> PgConnection {
            PgConnection::establish(&self.url()).expect("temp db conn")
        }

        fn pool(&self) -> PgPool {
            // Small pool so parallel migration tests do not exhaust `max_connections`.
            diesel::r2d2::Pool::builder()
                .max_size(4)
                .build(diesel::r2d2::ConnectionManager::<PgConnection>::new(
                    self.url(),
                ))
                .expect("temp pool")
        }
    }

    impl Drop for TempMigrationDb {
        fn drop(&mut self) {
            if let Ok(mut admin) = PgConnection::establish(&self.admin_url) {
                let _ = admin.batch_execute(&format!(
                    "SELECT pg_terminate_backend(pid) FROM pg_stat_activity \
                     WHERE datname = '{}' AND pid <> pg_backend_pid()",
                    self.name
                ));
                let _ = admin.batch_execute(&format!("DROP DATABASE IF EXISTS \"{}\"", self.name));
            }
        }
    }

    fn enforcement_triggers_present(conn: &mut PgConnection) -> bool {
        sql_query(
            "SELECT COUNT(*) AS n FROM pg_trigger \
             WHERE tgname IN ('work_relation_single_book_chapter_parent', 'work_single_book_chapter_parent_on_type')",
        )
        .get_result::<CountRow>(conn)
        .expect("trigger count")
        .n
            == 2
    }

    #[test]
    fn migration_up_on_empty_database_succeeds() {
        let db = TempMigrationDb::new();
        let mut conn = db.conn();
        conn.run_pending_migrations(MIGRATIONS)
            .expect("migrations should apply on an empty database");
        assert!(
            enforcement_triggers_present(&mut conn),
            "triggers installed"
        );
    }

    #[test]
    fn migration_up_on_valid_populated_database_succeeds() {
        let db = TempMigrationDb::new();
        let mut conn = db.conn();
        conn.run_pending_migrations(MIGRATIONS)
            .expect("initial migrations");
        // Revert enforcement so we can build representative valid data first.
        conn.revert_last_migration(MIGRATIONS)
            .expect("revert enforcement");

        let pool = db.pool();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        let _zero_parent = make_chapter(&pool, imprint.imprint_id);
        let one_parent = make_chapter(&pool, imprint.imprint_id);
        let parent = create_work(&pool, &imprint);
        relate(
            &pool,
            parent.work_id,
            one_parent.work_id,
            RelationType::HasChild,
            1,
        );
        // A legitimate non-chapter relation.
        let set = create_work(&pool, &imprint);
        let part = create_work(&pool, &imprint);
        relate(&pool, set.work_id, part.work_id, RelationType::HasPart, 1);

        let relations_before = sql_query("SELECT COUNT(*) AS n FROM work_relation")
            .get_result::<CountRow>(&mut conn)
            .unwrap()
            .n;

        conn.run_pending_migrations(MIGRATIONS)
            .expect("re-applying enforcement over valid data must succeed");
        assert!(enforcement_triggers_present(&mut conn));

        let relations_after = sql_query("SELECT COUNT(*) AS n FROM work_relation")
            .get_result::<CountRow>(&mut conn)
            .unwrap()
            .n;
        assert_eq!(
            relations_before, relations_after,
            "existing relations unchanged"
        );
        assert_eq!(parent_count(&mut conn, one_parent.work_id), 1);
    }

    #[test]
    fn migration_up_aborts_on_invalid_data() {
        let db = TempMigrationDb::new();
        let mut conn = db.conn();
        conn.run_pending_migrations(MIGRATIONS)
            .expect("initial migrations");
        conn.revert_last_migration(MIGRATIONS)
            .expect("revert enforcement");

        let pool = db.pool();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        let chapter = make_chapter(&pool, imprint.imprint_id);
        let p_a = create_work(&pool, &imprint);
        let p_b = create_work(&pool, &imprint);
        // Enforcement is reverted, so a corrupt two-parent chapter can be created.
        relate(
            &pool,
            p_a.work_id,
            chapter.work_id,
            RelationType::HasChild,
            1,
        );
        relate(
            &pool,
            p_b.work_id,
            chapter.work_id,
            RelationType::HasChild,
            1,
        );

        let relations_before = sql_query("SELECT COUNT(*) AS n FROM work_relation")
            .get_result::<CountRow>(&mut conn)
            .unwrap()
            .n;

        let result = conn.run_pending_migrations(MIGRATIONS);
        assert!(
            result.is_err(),
            "the guard must abort the migration over invalid data"
        );
        assert!(
            !enforcement_triggers_present(&mut conn),
            "no triggers may be partially installed"
        );

        let relations_after = sql_query("SELECT COUNT(*) AS n FROM work_relation")
            .get_result::<CountRow>(&mut conn)
            .unwrap()
            .n;
        assert_eq!(
            relations_before, relations_after,
            "no relation deleted or reparented"
        );
        assert_eq!(
            parent_count(&mut conn, chapter.work_id),
            2,
            "corrupt data left intact"
        );
    }

    #[test]
    fn migration_down_removes_enforcement_without_data_loss() {
        let db = TempMigrationDb::new();
        let mut conn = db.conn();
        conn.run_pending_migrations(MIGRATIONS).expect("migrations");

        let pool = db.pool();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        let chapter = make_chapter(&pool, imprint.imprint_id);
        let parent = create_work(&pool, &imprint);
        relate(
            &pool,
            parent.work_id,
            chapter.work_id,
            RelationType::HasChild,
            1,
        );

        let before = sql_query("SELECT COUNT(*) AS n FROM work_relation")
            .get_result::<CountRow>(&mut conn)
            .unwrap()
            .n;

        conn.revert_last_migration(MIGRATIONS)
            .expect("down migration");
        assert!(!enforcement_triggers_present(&mut conn), "triggers removed");

        let after = sql_query("SELECT COUNT(*) AS n FROM work_relation")
            .get_result::<CountRow>(&mut conn)
            .unwrap()
            .n;
        assert_eq!(before, after, "down migration must not touch data");
        assert_eq!(parent_count(&mut conn, chapter.work_id), 1);
    }

    #[test]
    fn migration_up_down_up_roundtrip_succeeds() {
        let db = TempMigrationDb::new();
        let mut conn = db.conn();
        conn.run_pending_migrations(MIGRATIONS).expect("up");
        conn.revert_last_migration(MIGRATIONS).expect("down");
        conn.run_pending_migrations(MIGRATIONS).expect("up again");
        assert!(enforcement_triggers_present(&mut conn));
    }

    #[test]
    fn migration_activation_race_blocks_concurrent_violation() {
        // On a throwaway DB with enforcement pending, one connection runs the exact
        // migration up.sql (LOCK + guard + install) inside a transaction and holds
        // it; a concurrent writer attempts a violating second parent and must block
        // on the table lock for the entire guard->install window, then be rejected
        // once enforcement is committed.
        let db = TempMigrationDb::new();
        let mut setup = db.conn();
        setup
            .run_pending_migrations(MIGRATIONS)
            .expect("migrations");
        setup
            .revert_last_migration(MIGRATIONS)
            .expect("revert enforcement");

        let pool = db.pool();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        let chapter = make_chapter(&pool, imprint.imprint_id);
        let parent_a = create_work(&pool, &imprint);
        let parent_b = create_work(&pool, &imprint);
        relate(
            &pool,
            parent_a.work_id,
            chapter.work_id,
            RelationType::HasChild,
            1,
        );

        let (mig_locked_tx, mig_locked_rx) = mpsc::channel::<()>();
        let (mig_go_tx, mig_go_rx) = mpsc::channel::<()>();
        let (w_pid_tx, w_pid_rx) = mpsc::channel::<i32>();
        let (w_issue_tx, w_issue_rx) = mpsc::channel::<()>();

        let mig_url = db.url();
        let t_mig = thread::spawn(move || {
            let mut c = PgConnection::establish(&mig_url).expect("mig conn");
            c.batch_execute("BEGIN").unwrap();
            // Runs LOCK TABLE ... SHARE ROW EXCLUSIVE; guard (passes, 1 parent);
            // CREATE FUNCTION/TRIGGER x2 — all while holding the table locks.
            let installed = c.batch_execute(MIGRATION_UP_SQL).is_ok();
            mig_locked_tx.send(()).unwrap();
            mig_go_rx.recv().unwrap();
            let committed = installed && c.batch_execute("COMMIT").is_ok();
            if !committed {
                let _ = c.batch_execute("ROLLBACK");
            }
            committed
        });

        let writer_url = db.url();
        let (ch, pb) = (chapter.work_id, parent_b.work_id);
        let t_writer = thread::spawn(move || {
            let mut c = PgConnection::establish(&writer_url).expect("writer conn");
            w_pid_tx.send(backend_pid(&mut c)).unwrap();
            c.batch_execute("BEGIN").unwrap();
            w_issue_tx.send(()).unwrap();
            // Blocks on the migration's SHARE ROW EXCLUSIVE table lock, then (after
            // the migration commits) is rejected by the now-active trigger.
            let ins = insert_child_pair(&mut c, ch, pb, 2, 1);
            let _ = c.batch_execute("ROLLBACK");
            ins.is_err()
        });

        let mut poll = db.conn();
        let w_pid = w_pid_rx.recv().unwrap();
        mig_locked_rx.recv().unwrap();
        w_issue_rx.recv().unwrap();
        let blocked = wait_until_blocked(&mut poll, w_pid);
        mig_go_tx.send(()).unwrap();
        let mig_ok = t_mig.join().unwrap();
        let writer_rejected = t_writer.join().unwrap();

        assert!(
            blocked,
            "the concurrent writer must block on the migration's table lock"
        );
        assert!(mig_ok, "the migration activation commits");
        assert!(
            writer_rejected,
            "the violating writer cannot slip through the guard->install window"
        );
        assert_eq!(
            parent_count(&mut poll, chapter.work_id),
            1,
            "no invalid second parent persisted"
        );
    }
}
