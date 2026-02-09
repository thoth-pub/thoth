use super::*;
use crate::model::Crud;
use uuid::Uuid;

fn make_abstract(
    pool: &crate::db::PgPool,
    work_id: Uuid,
    content: String,
    abstract_type: AbstractType,
    locale_code: LocaleCode,
) -> Abstract {
    let new_abstract = NewAbstract {
        work_id,
        content,
        locale_code,
        abstract_type,
        canonical: false,
    };

    Abstract::create(pool, &new_abstract).expect("Failed to create abstract")
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;

    use crate::markup::MarkupFormat;
    use crate::model::r#abstract::policy::AbstractPolicy;
    use crate::model::tests::db::{
        create_imprint, create_publisher, create_work, setup_test_db, test_context_with_user,
        test_user_with_role,
    };
    use crate::model::Crud;
    use crate::policy::{CreatePolicy, DeletePolicy, Role, UpdatePolicy};

    #[test]
    fn crud_policy_allows_publisher_user_with_markup() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("abstract-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let new_abstract = NewAbstract {
            work_id: work.work_id,
            content: "Policy Abstract".to_string(),
            locale_code: LocaleCode::En,
            abstract_type: AbstractType::Long,
            canonical: false,
        };

        let abstract_item =
            Abstract::create(pool.as_ref(), &new_abstract).expect("Failed to create");
        let patch = PatchAbstract {
            abstract_id: abstract_item.abstract_id,
            work_id: abstract_item.work_id,
            content: "Updated Policy Abstract".to_string(),
            locale_code: abstract_item.locale_code,
            abstract_type: abstract_item.abstract_type,
            canonical: abstract_item.canonical,
        };

        assert!(AbstractPolicy::can_create(&ctx, &new_abstract, Some(MarkupFormat::Html)).is_ok());
        assert!(
            AbstractPolicy::can_update(&ctx, &abstract_item, &patch, Some(MarkupFormat::Html))
                .is_ok()
        );
        assert!(AbstractPolicy::can_delete(&ctx, &abstract_item).is_ok());
    }

    #[test]
    fn crud_policy_requires_markup_format() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("abstract-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let new_abstract = NewAbstract {
            work_id: work.work_id,
            content: "Policy Abstract".to_string(),
            locale_code: LocaleCode::En,
            abstract_type: AbstractType::Long,
            canonical: false,
        };

        let abstract_item =
            Abstract::create(pool.as_ref(), &new_abstract).expect("Failed to create");
        let patch = PatchAbstract {
            abstract_id: abstract_item.abstract_id,
            work_id: abstract_item.work_id,
            content: "Updated Policy Abstract".to_string(),
            locale_code: abstract_item.locale_code,
            abstract_type: abstract_item.abstract_type,
            canonical: abstract_item.canonical,
        };

        assert!(AbstractPolicy::can_create(&ctx, &new_abstract, None).is_err());
        assert!(AbstractPolicy::can_update(&ctx, &abstract_item, &patch, None).is_err());
    }

    #[test]
    fn crud_policy_rejects_user_without_publisher_role() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let abstract_item = make_abstract(
            pool.as_ref(),
            work.work_id,
            "Policy Abstract".to_string(),
            AbstractType::Long,
            LocaleCode::En,
        );
        let patch = PatchAbstract {
            abstract_id: abstract_item.abstract_id,
            work_id: abstract_item.work_id,
            content: "Updated Policy Abstract".to_string(),
            locale_code: abstract_item.locale_code,
            abstract_type: abstract_item.abstract_type,
            canonical: abstract_item.canonical,
        };

        let user = test_user_with_role("abstract-user", Role::PublisherUser, "org-other");
        let ctx = test_context_with_user(pool.clone(), user);

        let new_abstract = NewAbstract {
            work_id: work.work_id,
            content: "Policy Abstract".to_string(),
            locale_code: LocaleCode::En,
            abstract_type: AbstractType::Long,
            canonical: false,
        };

        assert!(AbstractPolicy::can_create(&ctx, &new_abstract, Some(MarkupFormat::Html)).is_err());
        assert!(
            AbstractPolicy::can_update(&ctx, &abstract_item, &patch, Some(MarkupFormat::Html))
                .is_err()
        );
        assert!(AbstractPolicy::can_delete(&ctx, &abstract_item).is_err());
    }
}

#[cfg(feature = "backend")]
mod crud {
    use super::*;

    use crate::model::tests::db::{
        create_imprint, create_publisher, create_work, setup_test_db, test_context,
    };
    use crate::model::Crud;

    #[test]
    fn crud_roundtrip_create_fetch_update_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let new_abstract = NewAbstract {
            work_id: work.work_id,
            content: format!("Abstract {}", Uuid::new_v4()),
            locale_code: LocaleCode::En,
            abstract_type: AbstractType::Short,
            canonical: false,
        };

        let abstract_ =
            Abstract::create(pool.as_ref(), &new_abstract).expect("Failed to create abstract");
        let fetched =
            Abstract::from_id(pool.as_ref(), &abstract_.abstract_id).expect("Failed to fetch");
        assert_eq!(abstract_.abstract_id, fetched.abstract_id);

        let patch = PatchAbstract {
            abstract_id: abstract_.abstract_id,
            work_id: abstract_.work_id,
            content: format!("Updated {}", Uuid::new_v4()),
            locale_code: abstract_.locale_code,
            abstract_type: AbstractType::Long,
            canonical: true,
        };

        let ctx = test_context(pool.clone(), "test-user");
        let updated = abstract_
            .update(&ctx, &patch)
            .expect("Failed to update abstract");
        assert_eq!(updated.content, patch.content);

        let deleted = updated
            .delete(pool.as_ref())
            .expect("Failed to delete abstract");
        assert!(Abstract::from_id(pool.as_ref(), &deleted.abstract_id).is_err());
    }

    #[test]
    fn crud_all_respects_limit_and_offset() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        make_abstract(
            pool.as_ref(),
            work.work_id,
            format!("Abstract {}", Uuid::new_v4()),
            AbstractType::Short,
            LocaleCode::En,
        );
        make_abstract(
            pool.as_ref(),
            work.work_id,
            format!("Abstract {}", Uuid::new_v4()),
            AbstractType::Long,
            LocaleCode::En,
        );

        let order = AbstractOrderBy {
            field: AbstractField::AbstractId,
            direction: Direction::Asc,
        };

        let first = Abstract::all(
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
        .expect("Failed to fetch abstracts");
        let second = Abstract::all(
            pool.as_ref(),
            1,
            1,
            None,
            AbstractOrderBy {
                field: AbstractField::AbstractId,
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
        .expect("Failed to fetch abstracts");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_ne!(first[0].abstract_id, second[0].abstract_id);
    }

    #[test]
    fn crud_count_returns_total() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        make_abstract(
            pool.as_ref(),
            work.work_id,
            format!("Abstract {}", Uuid::new_v4()),
            AbstractType::Short,
            LocaleCode::En,
        );
        make_abstract(
            pool.as_ref(),
            work.work_id,
            format!("Abstract {}", Uuid::new_v4()),
            AbstractType::Long,
            LocaleCode::En,
        );

        let count = Abstract::count(pool.as_ref(), None, vec![], vec![], vec![], None, None)
            .expect("Failed to count abstracts");
        assert_eq!(count, 2);
    }

    #[test]
    fn crud_filter_matches_content() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let marker = format!("Filter {}", Uuid::new_v4());
        let matches = make_abstract(
            pool.as_ref(),
            work.work_id,
            format!("Abstract {marker}"),
            AbstractType::Short,
            LocaleCode::En,
        );
        make_abstract(
            pool.as_ref(),
            work.work_id,
            "Other abstract".to_string(),
            AbstractType::Long,
            LocaleCode::En,
        );

        let filtered = Abstract::all(
            pool.as_ref(),
            10,
            0,
            Some(marker),
            AbstractOrderBy {
                field: AbstractField::AbstractId,
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
        .expect("Failed to filter abstracts");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].abstract_id, matches.abstract_id);
    }

    #[test]
    fn crud_filter_param_limits_abstract_type() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let matches = make_abstract(
            pool.as_ref(),
            work.work_id,
            format!("Abstract {}", Uuid::new_v4()),
            AbstractType::Short,
            LocaleCode::En,
        );
        make_abstract(
            pool.as_ref(),
            work.work_id,
            format!("Abstract {}", Uuid::new_v4()),
            AbstractType::Long,
            LocaleCode::En,
        );

        let filtered = Abstract::all(
            pool.as_ref(),
            10,
            0,
            None,
            AbstractOrderBy {
                field: AbstractField::AbstractId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![],
            vec![],
            Some(AbstractType::Short),
            None,
        )
        .expect("Failed to filter abstracts by type");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].abstract_id, matches.abstract_id);
    }

    #[test]
    fn crud_ordering_by_id_respects_direction() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let first = make_abstract(
            pool.as_ref(),
            work.work_id,
            format!("Abstract {}", Uuid::new_v4()),
            AbstractType::Short,
            LocaleCode::En,
        );
        let second = make_abstract(
            pool.as_ref(),
            work.work_id,
            format!("Abstract {}", Uuid::new_v4()),
            AbstractType::Long,
            LocaleCode::En,
        );
        let mut ids = [first.abstract_id, second.abstract_id];
        ids.sort();

        let asc = Abstract::all(
            pool.as_ref(),
            2,
            0,
            None,
            AbstractOrderBy {
                field: AbstractField::AbstractId,
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
        .expect("Failed to order abstracts (asc)");

        let desc = Abstract::all(
            pool.as_ref(),
            2,
            0,
            None,
            AbstractOrderBy {
                field: AbstractField::AbstractId,
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
        .expect("Failed to order abstracts (desc)");

        assert_eq!(asc[0].abstract_id, ids[0]);
        assert_eq!(desc[0].abstract_id, ids[1]);
    }

    #[test]
    fn crud_canonical_from_work_id_returns_short_and_long() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let short = Abstract::create(
            pool.as_ref(),
            &NewAbstract {
                work_id: work.work_id,
                content: "Short canonical".to_string(),
                locale_code: LocaleCode::En,
                abstract_type: AbstractType::Short,
                canonical: true,
            },
        )
        .expect("Failed to create short canonical abstract");
        let long = Abstract::create(
            pool.as_ref(),
            &NewAbstract {
                work_id: work.work_id,
                content: "Long canonical".to_string(),
                locale_code: LocaleCode::En,
                abstract_type: AbstractType::Long,
                canonical: true,
            },
        )
        .expect("Failed to create long canonical abstract");

        let fetched_short = Abstract::short_canonical_from_work_id(pool.as_ref(), &work.work_id)
            .expect("Failed to fetch short canonical abstract");
        let fetched_long = Abstract::long_canonical_from_work_id(pool.as_ref(), &work.work_id)
            .expect("Failed to fetch long canonical abstract");

        assert_eq!(fetched_short.abstract_id, short.abstract_id);
        assert_eq!(fetched_long.abstract_id, long.abstract_id);
    }

    #[test]
    fn crud_filter_parent_work_id_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let other_work = create_work(pool.as_ref(), &imprint);

        let matches = make_abstract(
            pool.as_ref(),
            work.work_id,
            format!("Abstract {}", Uuid::new_v4()),
            AbstractType::Short,
            LocaleCode::En,
        );
        make_abstract(
            pool.as_ref(),
            other_work.work_id,
            format!("Abstract {}", Uuid::new_v4()),
            AbstractType::Long,
            LocaleCode::En,
        );

        let filtered = Abstract::all(
            pool.as_ref(),
            10,
            0,
            None,
            AbstractOrderBy {
                field: AbstractField::AbstractId,
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
        .expect("Failed to filter abstracts by work");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].abstract_id, matches.abstract_id);
    }

    #[test]
    fn crud_filter_param_limits_locale_codes() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let matches = make_abstract(
            pool.as_ref(),
            work.work_id,
            format!("Abstract {}", Uuid::new_v4()),
            AbstractType::Short,
            LocaleCode::En,
        );
        make_abstract(
            pool.as_ref(),
            work.work_id,
            format!("Abstract {}", Uuid::new_v4()),
            AbstractType::Long,
            LocaleCode::Fr,
        );

        let filtered = Abstract::all(
            pool.as_ref(),
            10,
            0,
            None,
            AbstractOrderBy {
                field: AbstractField::AbstractId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![LocaleCode::En],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter abstracts by locale");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].abstract_id, matches.abstract_id);
    }

    #[test]
    fn crud_ordering_by_fields_is_supported() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        Abstract::create(
            pool.as_ref(),
            &NewAbstract {
                work_id: work.work_id,
                content: "Abstract A".to_string(),
                locale_code: LocaleCode::En,
                abstract_type: AbstractType::Short,
                canonical: true,
            },
        )
        .expect("Failed to create abstract");
        Abstract::create(
            pool.as_ref(),
            &NewAbstract {
                work_id: work.work_id,
                content: "Abstract B".to_string(),
                locale_code: LocaleCode::Fr,
                abstract_type: AbstractType::Long,
                canonical: true,
            },
        )
        .expect("Failed to create abstract");

        let fields: Vec<fn() -> AbstractField> = vec![
            || AbstractField::AbstractId,
            || AbstractField::WorkId,
            || AbstractField::LocaleCode,
            || AbstractField::AbstractType,
            || AbstractField::Content,
            || AbstractField::Canonical,
        ];

        for field in fields {
            for direction in [Direction::Asc, Direction::Desc] {
                let results = Abstract::all(
                    pool.as_ref(),
                    10,
                    0,
                    None,
                    AbstractOrderBy {
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
                .expect("Failed to order abstracts");

                assert_eq!(results.len(), 2);
            }
        }
    }

    #[test]
    fn crud_count_with_filter_matches_content() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let marker = format!("Filter {}", Uuid::new_v4());

        make_abstract(
            pool.as_ref(),
            work.work_id,
            format!("Abstract {marker}"),
            AbstractType::Short,
            LocaleCode::En,
        );
        make_abstract(
            pool.as_ref(),
            work.work_id,
            "Other abstract".to_string(),
            AbstractType::Long,
            LocaleCode::En,
        );

        let count = Abstract::count(
            pool.as_ref(),
            Some(marker),
            vec![],
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to count filtered abstracts");

        assert_eq!(count, 1);
    }
}
