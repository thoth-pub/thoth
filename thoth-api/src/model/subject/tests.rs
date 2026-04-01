use super::*;
use crate::model::Crud;
use uuid::Uuid;

fn make_subject(
    pool: &crate::db::PgPool,
    work_id: Uuid,
    subject_type: SubjectType,
    subject_code: String,
    subject_ordinal: i32,
) -> Subject {
    let new_subject = NewSubject {
        work_id,
        subject_type,
        subject_code,
        subject_ordinal,
    };

    Subject::create(pool, &new_subject).expect("Failed to create subject")
}

mod defaults {
    use super::*;

    #[test]
    fn subjecttype_default_is_keyword() {
        let subjecttype: SubjectType = Default::default();
        assert_eq!(subjecttype, SubjectType::Keyword);
    }
}

mod display_and_parse {
    use super::*;

    #[test]
    fn subjecttype_display_formats_expected_strings() {
        assert_eq!(format!("{}", SubjectType::Bic), "BIC");
        assert_eq!(format!("{}", SubjectType::Bisac), "BISAC");
        assert_eq!(format!("{}", SubjectType::Thema), "Thema");
        assert_eq!(format!("{}", SubjectType::Lcc), "LCC");
        assert_eq!(format!("{}", SubjectType::Custom), "Custom");
        assert_eq!(format!("{}", SubjectType::Keyword), "Keyword");
    }

    #[test]
    fn subjecttype_fromstr_parses_expected_values() {
        use std::str::FromStr;
        assert_eq!(SubjectType::from_str("BIC").unwrap(), SubjectType::Bic);
        assert_eq!(SubjectType::from_str("BISAC").unwrap(), SubjectType::Bisac);
        assert_eq!(SubjectType::from_str("Thema").unwrap(), SubjectType::Thema);
        assert_eq!(SubjectType::from_str("LCC").unwrap(), SubjectType::Lcc);
        assert_eq!(
            SubjectType::from_str("Custom").unwrap(),
            SubjectType::Custom
        );
        assert_eq!(
            SubjectType::from_str("Keyword").unwrap(),
            SubjectType::Keyword
        );

        assert!(SubjectType::from_str("bic").is_err());
        assert!(SubjectType::from_str("Library of Congress Subject Code").is_err());
    }
}

#[cfg(feature = "backend")]
mod conversions {
    use super::*;
    use crate::model::tests::db::setup_test_db;
    use crate::model::tests::{assert_db_enum_roundtrip, assert_graphql_enum_roundtrip};

    #[test]
    fn subjecttype_graphql_roundtrip() {
        assert_graphql_enum_roundtrip(SubjectType::Bisac);
    }

    #[test]
    fn subjecttype_db_enum_roundtrip() {
        let (_guard, pool) = setup_test_db();

        assert_db_enum_roundtrip::<SubjectType, crate::schema::sql_types::SubjectType>(
            pool.as_ref(),
            "'bisac'::subject_type",
            SubjectType::Bisac,
        );
    }
}

mod helpers {
    use super::*;
    use crate::model::{Crud, HistoryEntry};

    #[test]
    fn pk_returns_id() {
        let subject: Subject = Default::default();
        assert_eq!(subject.pk(), subject.subject_id);
    }

    #[test]
    fn history_entry_serializes_model() {
        let subject: Subject = Default::default();
        let user_id = "1234567".to_string();
        let new_subject_history = subject.new_history_entry(&user_id);
        assert_eq!(new_subject_history.subject_id, subject.subject_id);
        assert_eq!(new_subject_history.user_id, user_id);
        assert_eq!(
            new_subject_history.data,
            serde_json::Value::String(serde_json::to_string(&subject).unwrap())
        );
    }
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;

    use crate::model::subject::policy::SubjectPolicy;
    use crate::model::tests::db::{
        create_imprint, create_publisher, create_work, setup_test_db, test_context_with_user,
        test_user_with_role,
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
        let user = test_user_with_role("subject-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let new_subject = NewSubject {
            work_id: work.work_id,
            subject_type: SubjectType::Thema,
            subject_code: "ATXZ1".to_string(),
            subject_ordinal: 1,
        };

        let subject = Subject::create(pool.as_ref(), &new_subject).expect("Failed to create");
        let patch = PatchSubject {
            subject_id: subject.subject_id,
            work_id: subject.work_id,
            subject_type: subject.subject_type,
            subject_code: subject.subject_code.clone(),
            subject_ordinal: 2,
        };

        assert!(SubjectPolicy::can_create(&ctx, &new_subject, ()).is_ok());
        assert!(SubjectPolicy::can_update(&ctx, &subject, &patch, ()).is_ok());
        assert!(SubjectPolicy::can_delete(&ctx, &subject).is_ok());
        assert!(SubjectPolicy::can_move(&ctx, &subject).is_ok());
    }

    #[test]
    fn crud_policy_rejects_invalid_thema_code() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("subject-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let new_subject = NewSubject {
            work_id: work.work_id,
            subject_type: SubjectType::Thema,
            subject_code: "INVALID".to_string(),
            subject_ordinal: 1,
        };

        assert!(SubjectPolicy::can_create(&ctx, &new_subject, ()).is_err());
    }

    #[test]
    fn crud_policy_rejects_user_without_publisher_role() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let subject = make_subject(
            pool.as_ref(),
            work.work_id,
            SubjectType::Thema,
            "ATXZ1".to_string(),
            1,
        );
        let patch = PatchSubject {
            subject_id: subject.subject_id,
            work_id: subject.work_id,
            subject_type: subject.subject_type,
            subject_code: subject.subject_code.clone(),
            subject_ordinal: 2,
        };

        let user = test_user_with_role("subject-user", Role::PublisherUser, "org-other");
        let ctx = test_context_with_user(pool.clone(), user);

        let new_subject = NewSubject {
            work_id: work.work_id,
            subject_type: SubjectType::Thema,
            subject_code: "ATXZ1".to_string(),
            subject_ordinal: 1,
        };

        assert!(SubjectPolicy::can_create(&ctx, &new_subject, ()).is_err());
        assert!(SubjectPolicy::can_update(&ctx, &subject, &patch, ()).is_err());
        assert!(SubjectPolicy::can_delete(&ctx, &subject).is_err());
        assert!(SubjectPolicy::can_move(&ctx, &subject).is_err());
    }
}

#[cfg(feature = "backend")]
mod crud {
    use super::*;
    use crate::graphql::types::inputs::{Direction, SubjectOrderBy};
    use crate::model::tests::db::{
        create_imprint, create_publisher, create_work, setup_test_db, test_context,
    };
    use crate::model::{Crud, Reorder};

    #[test]
    fn crud_roundtrip_create_fetch_update_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let new_subject = NewSubject {
            work_id: work.work_id,
            subject_type: SubjectType::Keyword,
            subject_code: "Test Subject".to_string(),
            subject_ordinal: 1,
        };

        let subject = Subject::create(pool.as_ref(), &new_subject).expect("Failed to create");
        let fetched =
            Subject::from_id(pool.as_ref(), &subject.subject_id).expect("Failed to fetch");
        assert_eq!(subject.subject_id, fetched.subject_id);

        let patch = PatchSubject {
            subject_id: subject.subject_id,
            work_id: subject.work_id,
            subject_type: SubjectType::Custom,
            subject_code: "Updated Subject".to_string(),
            subject_ordinal: 2,
        };

        let ctx = test_context(pool.clone(), "test-user");
        let updated = subject.update(&ctx, &patch).expect("Failed to update");
        assert_eq!(updated.subject_code, patch.subject_code);

        let deleted = updated.delete(pool.as_ref()).expect("Failed to delete");
        assert!(Subject::from_id(pool.as_ref(), &deleted.subject_id).is_err());
    }

    #[test]
    fn crud_all_respects_limit_and_offset() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        make_subject(
            pool.as_ref(),
            work.work_id,
            SubjectType::Keyword,
            "Subject A".to_string(),
            1,
        );
        make_subject(
            pool.as_ref(),
            work.work_id,
            SubjectType::Keyword,
            "Subject B".to_string(),
            2,
        );

        let order = SubjectOrderBy {
            field: SubjectField::SubjectId,
            direction: Direction::Asc,
        };

        let first = Subject::all(
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
        .expect("Failed to fetch subjects");
        let second = Subject::all(
            pool.as_ref(),
            1,
            1,
            None,
            SubjectOrderBy {
                field: SubjectField::SubjectId,
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
        .expect("Failed to fetch subjects");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_ne!(first[0].subject_id, second[0].subject_id);
    }

    #[test]
    fn crud_count_returns_total() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        make_subject(
            pool.as_ref(),
            work.work_id,
            SubjectType::Keyword,
            "Subject A".to_string(),
            1,
        );
        make_subject(
            pool.as_ref(),
            work.work_id,
            SubjectType::Keyword,
            "Subject B".to_string(),
            2,
        );

        let count = Subject::count(pool.as_ref(), None, vec![], vec![], vec![], None, None)
            .expect("Failed to count subjects");
        assert_eq!(count, 2);
    }

    #[test]
    fn crud_count_filters_by_subject_type() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        make_subject(
            pool.as_ref(),
            work.work_id,
            SubjectType::Keyword,
            "Subject A".to_string(),
            1,
        );
        make_subject(
            pool.as_ref(),
            work.work_id,
            SubjectType::Custom,
            "Subject B".to_string(),
            2,
        );

        let count = Subject::count(
            pool.as_ref(),
            None,
            vec![],
            vec![SubjectType::Keyword],
            vec![],
            None,
            None,
        )
        .expect("Failed to count subjects by type");
        assert_eq!(count, 1);
    }

    #[test]
    fn crud_count_filters_by_subject_code() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        make_subject(
            pool.as_ref(),
            work.work_id,
            SubjectType::Keyword,
            "ABC123".to_string(),
            1,
        );
        make_subject(
            pool.as_ref(),
            work.work_id,
            SubjectType::Keyword,
            "XYZ999".to_string(),
            2,
        );

        let count = Subject::count(
            pool.as_ref(),
            Some("ABC".to_string()),
            vec![],
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to count subjects by code");

        assert_eq!(count, 1);
    }

    #[test]
    fn crud_filter_matches_subject_code() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let marker = "Keyword-123";
        let matches = make_subject(
            pool.as_ref(),
            work.work_id,
            SubjectType::Keyword,
            marker.to_string(),
            1,
        );
        make_subject(
            pool.as_ref(),
            work.work_id,
            SubjectType::Keyword,
            "Other Subject".to_string(),
            2,
        );

        let filtered = Subject::all(
            pool.as_ref(),
            10,
            0,
            Some("Keyword-123".to_string()),
            SubjectOrderBy {
                field: SubjectField::SubjectId,
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
        .expect("Failed to filter subjects");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].subject_id, matches.subject_id);
    }

    #[test]
    fn crud_filter_parent_work_id_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let other_work = create_work(pool.as_ref(), &imprint);

        let matches = make_subject(
            pool.as_ref(),
            work.work_id,
            SubjectType::Keyword,
            "Subject A".to_string(),
            1,
        );
        make_subject(
            pool.as_ref(),
            other_work.work_id,
            SubjectType::Keyword,
            "Subject B".to_string(),
            2,
        );

        let filtered = Subject::all(
            pool.as_ref(),
            10,
            0,
            None,
            SubjectOrderBy {
                field: SubjectField::SubjectId,
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
        .expect("Failed to filter subjects by work");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].subject_id, matches.subject_id);
    }

    #[test]
    fn crud_filter_param_limits_subject_types() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let matches = make_subject(
            pool.as_ref(),
            work.work_id,
            SubjectType::Keyword,
            "Subject A".to_string(),
            1,
        );
        make_subject(
            pool.as_ref(),
            work.work_id,
            SubjectType::Custom,
            "Subject B".to_string(),
            2,
        );

        let filtered = Subject::all(
            pool.as_ref(),
            10,
            0,
            None,
            SubjectOrderBy {
                field: SubjectField::SubjectId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![SubjectType::Keyword],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter subjects by type");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].subject_id, matches.subject_id);
    }

    #[test]
    fn crud_filter_publishers_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let matches = make_subject(
            pool.as_ref(),
            work.work_id,
            SubjectType::Keyword,
            "Subject A".to_string(),
            1,
        );

        let other_publisher = create_publisher(pool.as_ref());
        let other_imprint = create_imprint(pool.as_ref(), &other_publisher);
        let other_work = create_work(pool.as_ref(), &other_imprint);
        make_subject(
            pool.as_ref(),
            other_work.work_id,
            SubjectType::Keyword,
            "Subject B".to_string(),
            1,
        );

        let filtered = Subject::all(
            pool.as_ref(),
            10,
            0,
            None,
            SubjectOrderBy {
                field: SubjectField::SubjectId,
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
        .expect("Failed to filter subjects by publisher");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].subject_id, matches.subject_id);
    }

    #[test]
    fn crud_ordering_by_id_respects_direction() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let first = make_subject(
            pool.as_ref(),
            work.work_id,
            SubjectType::Keyword,
            "Subject A".to_string(),
            1,
        );
        let second = make_subject(
            pool.as_ref(),
            work.work_id,
            SubjectType::Keyword,
            "Subject B".to_string(),
            2,
        );
        let mut ids = [first.subject_id, second.subject_id];
        ids.sort();

        let asc = Subject::all(
            pool.as_ref(),
            2,
            0,
            None,
            SubjectOrderBy {
                field: SubjectField::SubjectId,
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
        .expect("Failed to order subjects (asc)");

        let desc = Subject::all(
            pool.as_ref(),
            2,
            0,
            None,
            SubjectOrderBy {
                field: SubjectField::SubjectId,
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
        .expect("Failed to order subjects (desc)");

        assert_eq!(asc[0].subject_id, ids[0]);
        assert_eq!(desc[0].subject_id, ids[1]);
    }

    #[test]
    fn crud_ordering_by_fields_is_supported() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        make_subject(
            pool.as_ref(),
            work.work_id,
            SubjectType::Keyword,
            "Subject A".to_string(),
            1,
        );
        make_subject(
            pool.as_ref(),
            work.work_id,
            SubjectType::Custom,
            "Subject B".to_string(),
            2,
        );

        let fields: Vec<fn() -> SubjectField> = vec![
            || SubjectField::SubjectId,
            || SubjectField::WorkId,
            || SubjectField::SubjectType,
            || SubjectField::SubjectCode,
            || SubjectField::SubjectOrdinal,
            || SubjectField::CreatedAt,
            || SubjectField::UpdatedAt,
        ];

        for field in fields {
            for direction in [Direction::Asc, Direction::Desc] {
                let results = Subject::all(
                    pool.as_ref(),
                    10,
                    0,
                    None,
                    SubjectOrderBy {
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
                .expect("Failed to order subjects");

                assert_eq!(results.len(), 2);
            }
        }
    }

    #[test]
    fn crud_change_ordinal_reorders_subjects() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let first = make_subject(
            pool.as_ref(),
            work.work_id,
            SubjectType::Keyword,
            "Subject A".to_string(),
            1,
        );
        let second = make_subject(
            pool.as_ref(),
            work.work_id,
            SubjectType::Keyword,
            "Subject B".to_string(),
            2,
        );

        let ctx = test_context(pool.clone(), "test-user");
        let updated = first
            .change_ordinal(&ctx, first.subject_ordinal, 2)
            .expect("Failed to change subject ordinal");

        let refreshed_first =
            Subject::from_id(pool.as_ref(), &updated.subject_id).expect("Failed to fetch");
        let refreshed_second =
            Subject::from_id(pool.as_ref(), &second.subject_id).expect("Failed to fetch");

        assert_eq!(refreshed_first.subject_ordinal, 2);
        assert_eq!(refreshed_second.subject_ordinal, 1);
    }
}
