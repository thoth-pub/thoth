use super::*;

mod helpers {
    use super::*;
    use crate::model::{Crud, HistoryEntry};

    #[test]
    fn pk_returns_id() {
        let issue: Issue = Default::default();
        assert_eq!(issue.pk(), issue.issue_id);
    }

    #[test]
    fn history_entry_serializes_model() {
        let issue: Issue = Default::default();
        let user_id = "123456".to_string();
        let new_issue_history = issue.new_history_entry(&user_id);
        assert_eq!(new_issue_history.issue_id, issue.issue_id);
        assert_eq!(new_issue_history.user_id, user_id);
        assert_eq!(
            new_issue_history.data,
            serde_json::Value::String(serde_json::to_string(&issue).unwrap())
        );
    }
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;

    use crate::model::issue::policy::IssuePolicy;
    use crate::model::tests::db::{
        create_imprint, create_publisher, create_series, create_work, setup_test_db,
        test_context_with_user, test_user_with_role,
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
        let user = test_user_with_role("issue-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let series = create_series(pool.as_ref(), &imprint);
        let work = create_work(pool.as_ref(), &imprint);
        let new_issue = NewIssue {
            series_id: series.series_id,
            work_id: work.work_id,
            issue_ordinal: 1,
            issue_number: Some(1),
        };

        let issue = Issue::create(pool.as_ref(), &new_issue).expect("Failed to create");
        let patch = PatchIssue {
            issue_id: issue.issue_id,
            series_id: issue.series_id,
            work_id: issue.work_id,
            issue_ordinal: 2,
            issue_number: Some(2),
        };

        assert!(IssuePolicy::can_create(&ctx, &new_issue, ()).is_ok());
        assert!(IssuePolicy::can_update(&ctx, &issue, &patch, ()).is_ok());
        assert!(IssuePolicy::can_delete(&ctx, &issue).is_ok());
        assert!(IssuePolicy::can_move(&ctx, &issue).is_ok());
    }

    #[test]
    fn crud_policy_rejects_mismatched_imprints() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let other_publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let other_imprint = create_imprint(pool.as_ref(), &other_publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let series = create_series(pool.as_ref(), &other_imprint);

        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("issue-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let new_issue = NewIssue {
            series_id: series.series_id,
            work_id: work.work_id,
            issue_ordinal: 1,
            issue_number: Some(1),
        };

        assert!(IssuePolicy::can_create(&ctx, &new_issue, ()).is_err());
    }

    #[test]
    fn crud_policy_rejects_user_without_publisher_role() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let series = create_series(pool.as_ref(), &imprint);
        let work = create_work(pool.as_ref(), &imprint);

        let new_issue = NewIssue {
            series_id: series.series_id,
            work_id: work.work_id,
            issue_ordinal: 1,
            issue_number: Some(1),
        };

        let issue = Issue::create(pool.as_ref(), &new_issue).expect("Failed to create");
        let patch = PatchIssue {
            issue_id: issue.issue_id,
            series_id: issue.series_id,
            work_id: issue.work_id,
            issue_ordinal: 2,
            issue_number: Some(2),
        };

        let user = test_user_with_role("issue-user", Role::PublisherUser, "org-other");
        let ctx = test_context_with_user(pool.clone(), user);

        assert!(IssuePolicy::can_create(&ctx, &new_issue, ()).is_err());
        assert!(IssuePolicy::can_update(&ctx, &issue, &patch, ()).is_err());
        assert!(IssuePolicy::can_delete(&ctx, &issue).is_err());
        assert!(IssuePolicy::can_move(&ctx, &issue).is_err());
    }
}

#[cfg(feature = "backend")]
mod crud {
    use super::*;
    use uuid::Uuid;

    use crate::graphql::types::inputs::{Direction, IssueOrderBy};
    use crate::model::tests::db::{
        create_imprint, create_publisher, create_series, create_work, setup_test_db, test_context,
    };
    use crate::model::{Crud, Reorder};

    fn make_issue(
        pool: &crate::db::PgPool,
        series_id: Uuid,
        work_id: Uuid,
        issue_ordinal: i32,
        issue_number: Option<i32>,
    ) -> Issue {
        let new_issue = NewIssue {
            series_id,
            work_id,
            issue_ordinal,
            issue_number,
        };

        Issue::create(pool, &new_issue).expect("Failed to create issue")
    }

    #[test]
    fn crud_roundtrip_create_fetch_update_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let series = create_series(pool.as_ref(), &imprint);
        let work = create_work(pool.as_ref(), &imprint);

        let new_issue = NewIssue {
            series_id: series.series_id,
            work_id: work.work_id,
            issue_ordinal: 1,
            issue_number: Some(1),
        };

        let issue = Issue::create(pool.as_ref(), &new_issue).expect("Failed to create");
        let fetched = Issue::from_id(pool.as_ref(), &issue.issue_id).expect("Failed to fetch");
        assert_eq!(issue.issue_id, fetched.issue_id);

        let patch = PatchIssue {
            issue_id: issue.issue_id,
            series_id: issue.series_id,
            work_id: issue.work_id,
            issue_ordinal: 2,
            issue_number: Some(2),
        };

        let ctx = test_context(pool.clone(), "test-user");
        let updated = issue.update(&ctx, &patch).expect("Failed to update");
        assert_eq!(updated.issue_ordinal, patch.issue_ordinal);

        let deleted = updated.delete(pool.as_ref()).expect("Failed to delete");
        assert!(Issue::from_id(pool.as_ref(), &deleted.issue_id).is_err());
    }

    #[test]
    fn crud_all_respects_limit_and_offset() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let series = create_series(pool.as_ref(), &imprint);
        let other_series = create_series(pool.as_ref(), &imprint);
        let work = create_work(pool.as_ref(), &imprint);

        make_issue(pool.as_ref(), series.series_id, work.work_id, 1, None);
        make_issue(pool.as_ref(), other_series.series_id, work.work_id, 1, None);

        let order = IssueOrderBy {
            field: IssueField::IssueId,
            direction: Direction::Asc,
        };

        let first = Issue::all(
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
        .expect("Failed to fetch issues");
        let second = Issue::all(
            pool.as_ref(),
            1,
            1,
            None,
            IssueOrderBy {
                field: IssueField::IssueId,
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
        .expect("Failed to fetch issues");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_ne!(first[0].issue_id, second[0].issue_id);
    }

    #[test]
    fn crud_count_returns_total() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let series = create_series(pool.as_ref(), &imprint);
        let other_series = create_series(pool.as_ref(), &imprint);
        let work = create_work(pool.as_ref(), &imprint);

        make_issue(pool.as_ref(), series.series_id, work.work_id, 1, None);
        make_issue(pool.as_ref(), other_series.series_id, work.work_id, 1, None);

        let count = Issue::count(pool.as_ref(), None, vec![], vec![], vec![], None, None)
            .expect("Failed to count issues");
        assert_eq!(count, 2);
    }

    #[test]
    fn crud_ordering_by_id_respects_direction() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let series = create_series(pool.as_ref(), &imprint);
        let other_series = create_series(pool.as_ref(), &imprint);
        let work = create_work(pool.as_ref(), &imprint);

        let first = make_issue(pool.as_ref(), series.series_id, work.work_id, 1, None);
        let second = make_issue(pool.as_ref(), other_series.series_id, work.work_id, 1, None);
        let mut ids = [first.issue_id, second.issue_id];
        ids.sort();

        let asc = Issue::all(
            pool.as_ref(),
            2,
            0,
            None,
            IssueOrderBy {
                field: IssueField::IssueId,
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
        .expect("Failed to order issues (asc)");

        let desc = Issue::all(
            pool.as_ref(),
            2,
            0,
            None,
            IssueOrderBy {
                field: IssueField::IssueId,
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
        .expect("Failed to order issues (desc)");

        assert_eq!(asc[0].issue_id, ids[0]);
        assert_eq!(desc[0].issue_id, ids[1]);
    }

    #[test]
    fn crud_filter_parent_work_id_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let series = create_series(pool.as_ref(), &imprint);
        let work = create_work(pool.as_ref(), &imprint);
        let other_work = create_work(pool.as_ref(), &imprint);

        let matches = make_issue(pool.as_ref(), series.series_id, work.work_id, 1, None);
        make_issue(pool.as_ref(), series.series_id, other_work.work_id, 2, None);

        let filtered = Issue::all(
            pool.as_ref(),
            10,
            0,
            None,
            IssueOrderBy {
                field: IssueField::IssueId,
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
        .expect("Failed to filter issues by work");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].issue_id, matches.issue_id);
    }

    #[test]
    fn crud_filter_parent_series_id_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let series = create_series(pool.as_ref(), &imprint);
        let other_series = create_series(pool.as_ref(), &imprint);
        let work = create_work(pool.as_ref(), &imprint);

        let matches = make_issue(pool.as_ref(), series.series_id, work.work_id, 1, None);
        make_issue(pool.as_ref(), other_series.series_id, work.work_id, 2, None);

        let filtered = Issue::all(
            pool.as_ref(),
            10,
            0,
            None,
            IssueOrderBy {
                field: IssueField::IssueId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            Some(series.series_id),
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter issues by series");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].issue_id, matches.issue_id);
    }

    #[test]
    fn crud_filter_publishers_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let series = create_series(pool.as_ref(), &imprint);
        let work = create_work(pool.as_ref(), &imprint);
        let matches = make_issue(pool.as_ref(), series.series_id, work.work_id, 1, None);

        let other_publisher = create_publisher(pool.as_ref());
        let other_imprint = create_imprint(pool.as_ref(), &other_publisher);
        let other_series = create_series(pool.as_ref(), &other_imprint);
        let other_work = create_work(pool.as_ref(), &other_imprint);
        make_issue(
            pool.as_ref(),
            other_series.series_id,
            other_work.work_id,
            1,
            None,
        );

        let filtered = Issue::all(
            pool.as_ref(),
            10,
            0,
            None,
            IssueOrderBy {
                field: IssueField::IssueId,
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
        .expect("Failed to filter issues by publisher");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].issue_id, matches.issue_id);
    }

    #[test]
    fn crud_ordering_by_fields_is_supported() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let series = create_series(pool.as_ref(), &imprint);
        let work = create_work(pool.as_ref(), &imprint);
        let other_work = create_work(pool.as_ref(), &imprint);

        make_issue(pool.as_ref(), series.series_id, work.work_id, 1, Some(1));
        make_issue(
            pool.as_ref(),
            series.series_id,
            other_work.work_id,
            2,
            Some(2),
        );

        let fields: Vec<fn() -> IssueField> = vec![
            || IssueField::IssueId,
            || IssueField::SeriesId,
            || IssueField::WorkId,
            || IssueField::IssueOrdinal,
            || IssueField::IssueNumber,
            || IssueField::CreatedAt,
            || IssueField::UpdatedAt,
        ];

        for field in fields {
            for direction in [Direction::Asc, Direction::Desc] {
                let results = Issue::all(
                    pool.as_ref(),
                    10,
                    0,
                    None,
                    IssueOrderBy {
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
                .expect("Failed to order issues");

                assert_eq!(results.len(), 2);
            }
        }
    }

    #[test]
    fn crud_change_ordinal_reorders_issues() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let series = create_series(pool.as_ref(), &imprint);
        let work = create_work(pool.as_ref(), &imprint);
        let other_work = create_work(pool.as_ref(), &imprint);

        let first = make_issue(pool.as_ref(), series.series_id, work.work_id, 1, None);
        let second = make_issue(pool.as_ref(), series.series_id, other_work.work_id, 2, None);

        let ctx = test_context(pool.clone(), "test-user");
        let updated = first
            .change_ordinal(&ctx, first.issue_ordinal, 2)
            .expect("Failed to change issue ordinal");

        let refreshed_first =
            Issue::from_id(pool.as_ref(), &updated.issue_id).expect("Failed to fetch");
        let refreshed_second =
            Issue::from_id(pool.as_ref(), &second.issue_id).expect("Failed to fetch");

        assert_eq!(refreshed_first.issue_ordinal, 2);
        assert_eq!(refreshed_second.issue_ordinal, 1);
    }
}
