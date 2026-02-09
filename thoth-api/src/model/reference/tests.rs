use super::*;
use crate::model::Crud;
use uuid::Uuid;

fn make_reference(
    pool: &crate::db::PgPool,
    work_id: Uuid,
    reference_ordinal: i32,
    unstructured_citation: Option<String>,
) -> Reference {
    let new_reference = NewReference {
        work_id,
        reference_ordinal,
        doi: None,
        unstructured_citation,
        issn: None,
        isbn: None,
        journal_title: None,
        article_title: None,
        series_title: None,
        volume_title: None,
        edition: None,
        author: None,
        volume: None,
        issue: None,
        first_page: None,
        component_number: None,
        standard_designator: None,
        standards_body_name: None,
        standards_body_acronym: None,
        url: None,
        publication_date: None,
        retrieval_date: None,
    };

    Reference::create(pool, &new_reference).expect("Failed to create reference")
}

mod defaults {
    use super::*;

    #[test]
    fn referencefield_default_is_reference_ordinal() {
        let reffield: ReferenceField = Default::default();
        assert_eq!(reffield, ReferenceField::ReferenceOrdinal);
    }
}

mod helpers {
    use super::*;
    use crate::model::{Crud, HistoryEntry};

    #[test]
    fn pk_returns_id() {
        let reference: Reference = Default::default();
        assert_eq!(reference.pk(), reference.reference_id);
    }

    #[test]
    fn history_entry_serializes_model() {
        let reference: Reference = Default::default();
        let user_id = "123456".to_string();
        let new_reference_history = reference.new_history_entry(&user_id);
        assert_eq!(new_reference_history.reference_id, reference.reference_id);
        assert_eq!(new_reference_history.user_id, user_id);
        assert_eq!(
            new_reference_history.data,
            serde_json::Value::String(serde_json::to_string(&reference).unwrap())
        );
    }
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;

    use crate::model::reference::policy::ReferencePolicy;
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
        let user = test_user_with_role("reference-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let new_reference = NewReference {
            work_id: work.work_id,
            reference_ordinal: 1,
            doi: None,
            unstructured_citation: Some("Policy Citation".to_string()),
            issn: None,
            isbn: None,
            journal_title: None,
            article_title: None,
            series_title: None,
            volume_title: None,
            edition: None,
            author: None,
            volume: None,
            issue: None,
            first_page: None,
            component_number: None,
            standard_designator: None,
            standards_body_name: None,
            standards_body_acronym: None,
            url: None,
            publication_date: None,
            retrieval_date: None,
        };

        let reference = Reference::create(pool.as_ref(), &new_reference).expect("Failed to create");
        let patch = PatchReference {
            reference_id: reference.reference_id,
            work_id: reference.work_id,
            reference_ordinal: 2,
            doi: None,
            unstructured_citation: Some("Updated Policy".to_string()),
            issn: reference.issn.clone(),
            isbn: reference.isbn.clone(),
            journal_title: reference.journal_title.clone(),
            article_title: reference.article_title.clone(),
            series_title: reference.series_title.clone(),
            volume_title: reference.volume_title.clone(),
            edition: reference.edition,
            author: reference.author.clone(),
            volume: reference.volume.clone(),
            issue: reference.issue.clone(),
            first_page: reference.first_page.clone(),
            component_number: reference.component_number.clone(),
            standard_designator: reference.standard_designator.clone(),
            standards_body_name: reference.standards_body_name.clone(),
            standards_body_acronym: reference.standards_body_acronym.clone(),
            url: reference.url.clone(),
            publication_date: reference.publication_date,
            retrieval_date: reference.retrieval_date,
        };

        assert!(ReferencePolicy::can_create(&ctx, &new_reference, ()).is_ok());
        assert!(ReferencePolicy::can_update(&ctx, &reference, &patch, ()).is_ok());
        assert!(ReferencePolicy::can_delete(&ctx, &reference).is_ok());
        assert!(ReferencePolicy::can_move(&ctx, &reference).is_ok());
    }

    #[test]
    fn crud_policy_rejects_user_without_publisher_role() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let reference = make_reference(
            pool.as_ref(),
            work.work_id,
            1,
            Some("Policy Citation".to_string()),
        );
        let patch = PatchReference {
            reference_id: reference.reference_id,
            work_id: reference.work_id,
            reference_ordinal: 2,
            doi: None,
            unstructured_citation: Some("Updated Policy".to_string()),
            issn: reference.issn.clone(),
            isbn: reference.isbn.clone(),
            journal_title: reference.journal_title.clone(),
            article_title: reference.article_title.clone(),
            series_title: reference.series_title.clone(),
            volume_title: reference.volume_title.clone(),
            edition: reference.edition,
            author: reference.author.clone(),
            volume: reference.volume.clone(),
            issue: reference.issue.clone(),
            first_page: reference.first_page.clone(),
            component_number: reference.component_number.clone(),
            standard_designator: reference.standard_designator.clone(),
            standards_body_name: reference.standards_body_name.clone(),
            standards_body_acronym: reference.standards_body_acronym.clone(),
            url: reference.url.clone(),
            publication_date: reference.publication_date,
            retrieval_date: reference.retrieval_date,
        };

        let user = test_user_with_role("reference-user", Role::PublisherUser, "org-other");
        let ctx = test_context_with_user(pool.clone(), user);

        let new_reference = NewReference {
            work_id: work.work_id,
            reference_ordinal: 1,
            doi: None,
            unstructured_citation: Some("Policy Citation".to_string()),
            issn: None,
            isbn: None,
            journal_title: None,
            article_title: None,
            series_title: None,
            volume_title: None,
            edition: None,
            author: None,
            volume: None,
            issue: None,
            first_page: None,
            component_number: None,
            standard_designator: None,
            standards_body_name: None,
            standards_body_acronym: None,
            url: None,
            publication_date: None,
            retrieval_date: None,
        };

        assert!(ReferencePolicy::can_create(&ctx, &new_reference, ()).is_err());
        assert!(ReferencePolicy::can_update(&ctx, &reference, &patch, ()).is_err());
        assert!(ReferencePolicy::can_delete(&ctx, &reference).is_err());
        assert!(ReferencePolicy::can_move(&ctx, &reference).is_err());
    }
}

#[cfg(feature = "backend")]
mod crud {
    use super::*;
    use std::str::FromStr;

    use chrono::NaiveDate;

    use crate::model::tests::db::{
        create_imprint, create_publisher, create_work, setup_test_db, test_context,
    };
    use crate::model::{Crud, Doi, Isbn, Reorder};

    #[test]
    fn crud_roundtrip_create_fetch_update_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let new_reference = NewReference {
            work_id: work.work_id,
            reference_ordinal: 1,
            doi: None,
            unstructured_citation: Some("Initial citation".to_string()),
            issn: None,
            isbn: None,
            journal_title: None,
            article_title: None,
            series_title: None,
            volume_title: None,
            edition: None,
            author: None,
            volume: None,
            issue: None,
            first_page: None,
            component_number: None,
            standard_designator: None,
            standards_body_name: None,
            standards_body_acronym: None,
            url: None,
            publication_date: None,
            retrieval_date: None,
        };

        let reference = Reference::create(pool.as_ref(), &new_reference).expect("Failed to create");
        let fetched =
            Reference::from_id(pool.as_ref(), &reference.reference_id).expect("Failed to fetch");
        assert_eq!(reference.reference_id, fetched.reference_id);

        let patch = PatchReference {
            reference_id: reference.reference_id,
            work_id: reference.work_id,
            reference_ordinal: 2,
            doi: None,
            unstructured_citation: Some("Updated citation".to_string()),
            issn: reference.issn.clone(),
            isbn: reference.isbn.clone(),
            journal_title: reference.journal_title.clone(),
            article_title: reference.article_title.clone(),
            series_title: reference.series_title.clone(),
            volume_title: reference.volume_title.clone(),
            edition: reference.edition,
            author: reference.author.clone(),
            volume: reference.volume.clone(),
            issue: reference.issue.clone(),
            first_page: reference.first_page.clone(),
            component_number: reference.component_number.clone(),
            standard_designator: reference.standard_designator.clone(),
            standards_body_name: reference.standards_body_name.clone(),
            standards_body_acronym: reference.standards_body_acronym.clone(),
            url: reference.url.clone(),
            publication_date: reference.publication_date,
            retrieval_date: reference.retrieval_date,
        };

        let ctx = test_context(pool.clone(), "test-user");
        let updated = reference.update(&ctx, &patch).expect("Failed to update");
        assert_eq!(updated.reference_ordinal, patch.reference_ordinal);

        let deleted = updated.delete(pool.as_ref()).expect("Failed to delete");
        assert!(Reference::from_id(pool.as_ref(), &deleted.reference_id).is_err());
    }

    #[test]
    fn crud_all_respects_limit_and_offset() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        make_reference(
            pool.as_ref(),
            work.work_id,
            1,
            Some(format!("Citation {}", Uuid::new_v4())),
        );
        make_reference(
            pool.as_ref(),
            work.work_id,
            2,
            Some(format!("Citation {}", Uuid::new_v4())),
        );

        let order = ReferenceOrderBy {
            field: ReferenceField::ReferenceId,
            direction: Direction::Asc,
        };

        let first = Reference::all(
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
        .expect("Failed to fetch references");
        let second = Reference::all(
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
        .expect("Failed to fetch references");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_ne!(first[0].reference_id, second[0].reference_id);
    }

    #[test]
    fn crud_count_returns_total() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        make_reference(
            pool.as_ref(),
            work.work_id,
            1,
            Some(format!("Citation {}", Uuid::new_v4())),
        );
        make_reference(
            pool.as_ref(),
            work.work_id,
            2,
            Some(format!("Citation {}", Uuid::new_v4())),
        );

        let count = Reference::count(pool.as_ref(), None, vec![], vec![], vec![], None, None)
            .expect("Failed to count references");
        assert_eq!(count, 2);
    }

    #[test]
    fn crud_filter_matches_unstructured_citation() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let marker = format!("Marker {}", Uuid::new_v4());
        let matches = make_reference(
            pool.as_ref(),
            work.work_id,
            1,
            Some(format!("Citation {marker}")),
        );
        make_reference(
            pool.as_ref(),
            work.work_id,
            2,
            Some("Other Citation".to_string()),
        );

        let filtered = Reference::all(
            pool.as_ref(),
            10,
            0,
            Some(marker),
            ReferenceOrderBy {
                field: ReferenceField::ReferenceId,
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
        .expect("Failed to filter references");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].reference_id, matches.reference_id);
    }

    #[test]
    fn crud_ordering_by_id_respects_direction() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let first = make_reference(
            pool.as_ref(),
            work.work_id,
            1,
            Some(format!("Citation {}", Uuid::new_v4())),
        );
        let second = make_reference(
            pool.as_ref(),
            work.work_id,
            2,
            Some(format!("Citation {}", Uuid::new_v4())),
        );
        let mut ids = [first.reference_id, second.reference_id];
        ids.sort();

        let asc = Reference::all(
            pool.as_ref(),
            2,
            0,
            None,
            ReferenceOrderBy {
                field: ReferenceField::ReferenceId,
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
        .expect("Failed to order references (asc)");

        let desc = Reference::all(
            pool.as_ref(),
            2,
            0,
            None,
            ReferenceOrderBy {
                field: ReferenceField::ReferenceId,
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
        .expect("Failed to order references (desc)");

        assert_eq!(asc[0].reference_id, ids[0]);
        assert_eq!(desc[0].reference_id, ids[1]);
    }

    #[test]
    fn crud_filter_ignores_empty_filter() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        make_reference(
            pool.as_ref(),
            work.work_id,
            1,
            Some(format!("Citation {}", Uuid::new_v4())),
        );
        make_reference(
            pool.as_ref(),
            work.work_id,
            2,
            Some(format!("Citation {}", Uuid::new_v4())),
        );

        let filtered = Reference::all(
            pool.as_ref(),
            10,
            0,
            Some(String::new()),
            ReferenceOrderBy {
                field: ReferenceField::ReferenceId,
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
        .expect("Failed to fetch references");

        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn crud_filter_parent_work_id_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let other_work = create_work(pool.as_ref(), &imprint);

        let matches = make_reference(
            pool.as_ref(),
            work.work_id,
            1,
            Some(format!("Citation {}", Uuid::new_v4())),
        );
        make_reference(
            pool.as_ref(),
            other_work.work_id,
            1,
            Some(format!("Citation {}", Uuid::new_v4())),
        );

        let filtered = Reference::all(
            pool.as_ref(),
            10,
            0,
            None,
            ReferenceOrderBy {
                field: ReferenceField::ReferenceId,
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
        .expect("Failed to filter references by work");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].reference_id, matches.reference_id);
    }

    #[test]
    fn crud_filter_publishers_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let matches = make_reference(
            pool.as_ref(),
            work.work_id,
            1,
            Some("Publisher Citation".to_string()),
        );

        let other_publisher = create_publisher(pool.as_ref());
        let other_imprint = create_imprint(pool.as_ref(), &other_publisher);
        let other_work = create_work(pool.as_ref(), &other_imprint);
        make_reference(
            pool.as_ref(),
            other_work.work_id,
            1,
            Some("Other Citation".to_string()),
        );

        let filtered = Reference::all(
            pool.as_ref(),
            10,
            0,
            None,
            ReferenceOrderBy {
                field: ReferenceField::ReferenceId,
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
        .expect("Failed to filter references by publisher");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].reference_id, matches.reference_id);
    }

    #[test]
    fn crud_ordering_by_fields_is_supported() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let first_reference = NewReference {
            work_id: work.work_id,
            reference_ordinal: 1,
            doi: Some(Doi::from_str("https://doi.org/10.1234/REF.ONE").unwrap()),
            unstructured_citation: Some("First Citation".to_string()),
            issn: Some("1234-5678".to_string()),
            isbn: Some(Isbn::from_str("9780131103627").unwrap()),
            journal_title: Some("Journal A".to_string()),
            article_title: Some("Article A".to_string()),
            series_title: Some("Series A".to_string()),
            volume_title: Some("Volume A".to_string()),
            edition: Some(1),
            author: Some("Author A".to_string()),
            volume: Some("10".to_string()),
            issue: Some("1".to_string()),
            first_page: Some("1".to_string()),
            component_number: Some("A".to_string()),
            standard_designator: Some("STD-A".to_string()),
            standards_body_name: Some("Standards Org".to_string()),
            standards_body_acronym: Some("SO".to_string()),
            url: Some("https://example.com/a".to_string()),
            publication_date: Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
            retrieval_date: Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap()),
        };

        let second_reference = NewReference {
            work_id: work.work_id,
            reference_ordinal: 2,
            doi: Some(Doi::from_str("https://doi.org/10.1234/REF.TWO").unwrap()),
            unstructured_citation: Some("Second Citation".to_string()),
            issn: Some("8765-4321".to_string()),
            isbn: Some(Isbn::from_str("9780262033848").unwrap()),
            journal_title: Some("Journal B".to_string()),
            article_title: Some("Article B".to_string()),
            series_title: Some("Series B".to_string()),
            volume_title: Some("Volume B".to_string()),
            edition: Some(2),
            author: Some("Author B".to_string()),
            volume: Some("20".to_string()),
            issue: Some("2".to_string()),
            first_page: Some("10".to_string()),
            component_number: Some("B".to_string()),
            standard_designator: Some("STD-B".to_string()),
            standards_body_name: Some("Standards Org B".to_string()),
            standards_body_acronym: Some("SOB".to_string()),
            url: Some("https://example.com/b".to_string()),
            publication_date: Some(NaiveDate::from_ymd_opt(2019, 1, 1).unwrap()),
            retrieval_date: Some(NaiveDate::from_ymd_opt(2022, 1, 1).unwrap()),
        };

        Reference::create(pool.as_ref(), &first_reference).expect("Failed to create reference");
        Reference::create(pool.as_ref(), &second_reference).expect("Failed to create reference");

        let fields: Vec<fn() -> ReferenceField> = vec![
            || ReferenceField::ReferenceId,
            || ReferenceField::WorkId,
            || ReferenceField::ReferenceOrdinal,
            || ReferenceField::Doi,
            || ReferenceField::UnstructuredCitation,
            || ReferenceField::Issn,
            || ReferenceField::Isbn,
            || ReferenceField::JournalTitle,
            || ReferenceField::ArticleTitle,
            || ReferenceField::SeriesTitle,
            || ReferenceField::VolumeTitle,
            || ReferenceField::Edition,
            || ReferenceField::Author,
            || ReferenceField::Volume,
            || ReferenceField::Issue,
            || ReferenceField::FirstPage,
            || ReferenceField::ComponentNumber,
            || ReferenceField::StandardDesignator,
            || ReferenceField::StandardsBodyName,
            || ReferenceField::StandardsBodyAcronym,
            || ReferenceField::Url,
            || ReferenceField::PublicationDate,
            || ReferenceField::RetrievalDate,
            || ReferenceField::CreatedAt,
            || ReferenceField::UpdatedAt,
        ];

        for field in fields {
            for direction in [Direction::Asc, Direction::Desc] {
                let results = Reference::all(
                    pool.as_ref(),
                    10,
                    0,
                    None,
                    ReferenceOrderBy {
                        field: field(),
                        direction,
                    },
                    vec![],
                    Some(work.work_id),
                    None,
                    vec![],
                    vec![],
                    None,
                    None,
                )
                .expect("Failed to order references");

                assert_eq!(results.len(), 2);
            }
        }
    }

    #[test]
    fn crud_count_with_filter_matches_reference() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let marker = format!("Citation {}", Uuid::new_v4());
        make_reference(pool.as_ref(), work.work_id, 1, Some(marker.clone()));
        make_reference(
            pool.as_ref(),
            work.work_id,
            2,
            Some("Other Citation".to_string()),
        );

        let count = Reference::count(
            pool.as_ref(),
            Some(marker),
            vec![],
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to count filtered references");

        assert_eq!(count, 1);
    }

    #[test]
    fn crud_change_ordinal_reorders_references() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let first = make_reference(
            pool.as_ref(),
            work.work_id,
            1,
            Some("Citation One".to_string()),
        );
        let second = make_reference(
            pool.as_ref(),
            work.work_id,
            2,
            Some("Citation Two".to_string()),
        );

        let ctx = test_context(pool.clone(), "test-user");
        let updated = first
            .change_ordinal(&ctx, first.reference_ordinal, 2)
            .expect("Failed to change reference ordinal");

        let refreshed_first =
            Reference::from_id(pool.as_ref(), &updated.reference_id).expect("Failed to fetch");
        let refreshed_second =
            Reference::from_id(pool.as_ref(), &second.reference_id).expect("Failed to fetch");

        assert_eq!(refreshed_first.reference_ordinal, 2);
        assert_eq!(refreshed_second.reference_ordinal, 1);
    }
}
