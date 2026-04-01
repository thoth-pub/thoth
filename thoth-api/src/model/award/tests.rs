use super::*;
use crate::model::{CountryCode, Crud};
use uuid::Uuid;

fn make_award(pool: &crate::db::PgPool, work_id: Uuid, award_ordinal: i32, title: &str) -> Award {
    let data = NewAward {
        work_id,
        title: title.to_string(),
        url: Some("https://example.com/award".to_string()),
        category: Some("Prize".to_string()),
        year: Some("2025".to_string()),
        jury: Some("Main Jury".to_string()),
        country: Some(CountryCode::Gbr),
        prize_statement: Some("Award note".to_string()),
        role: Some(AwardRole::Winner),
        award_ordinal,
    };

    Award::create(pool, &data).expect("Failed to create award")
}

mod conversions {
    use super::*;
    #[cfg(feature = "backend")]
    use crate::model::tests::db::setup_test_db;
    #[cfg(feature = "backend")]
    use crate::model::tests::{assert_db_enum_roundtrip, assert_graphql_enum_roundtrip};

    #[cfg(feature = "backend")]
    #[test]
    fn awardrole_graphql_roundtrip() {
        assert_graphql_enum_roundtrip(AwardRole::JointWinner);
    }

    #[cfg(feature = "backend")]
    #[test]
    fn awardrole_db_enum_roundtrip() {
        let (_guard, pool) = setup_test_db();

        assert_db_enum_roundtrip::<AwardRole, crate::schema::sql_types::AwardRole>(
            pool.as_ref(),
            "'JOINT_WINNER'::award_role",
            AwardRole::JointWinner,
        );
    }
}

mod defaults {
    use super::*;

    #[test]
    fn awardfield_default_is_award_ordinal() {
        let field: AwardField = Default::default();
        assert_eq!(field, AwardField::AwardOrdinal);
    }
}

mod helpers {
    use super::*;
    use crate::model::{Crud, HistoryEntry};

    #[test]
    fn pk_returns_id() {
        let award: Award = Default::default();
        assert_eq!(award.pk(), award.award_id);
    }

    #[test]
    fn history_entry_serializes_model() {
        let award: Award = Default::default();
        let user_id = "123456".to_string();
        let history = award.new_history_entry(&user_id);
        assert_eq!(history.award_id, award.award_id);
        assert_eq!(history.user_id, user_id);
        assert_eq!(
            history.data,
            serde_json::Value::String(serde_json::to_string(&award).unwrap())
        );
    }
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;

    use crate::model::award::policy::AwardPolicy;
    use crate::model::tests::db::{
        create_imprint, create_publisher, create_work, setup_test_db, test_context_with_user,
        test_user_with_role,
    };
    use crate::model::work::{NewWork, Work, WorkStatus, WorkType};
    use crate::policy::{CreatePolicy, DeletePolicy, MovePolicy, Role, UpdatePolicy};

    #[test]
    fn crud_policy_allows_publisher_user_for_write() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("award-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let data = NewAward {
            work_id: work.work_id,
            title: "Award".to_string(),
            url: Some("https://example.com/award".to_string()),
            category: Some("Prize".to_string()),
            year: Some("2025".to_string()),
            jury: Some("Main Jury".to_string()),
            country: Some(CountryCode::Gbr),
            prize_statement: Some("Award note".to_string()),
            role: Some(AwardRole::Winner),
            award_ordinal: 1,
        };

        let award = Award::create(pool.as_ref(), &data).expect("Failed to create");
        let patch = PatchAward {
            award_id: award.award_id,
            work_id: award.work_id,
            title: "Award Updated".to_string(),
            url: award.url.clone(),
            category: award.category.clone(),
            year: award.year.clone(),
            jury: award.jury.clone(),
            country: award.country,
            prize_statement: award.prize_statement.clone(),
            role: award.role,
            award_ordinal: 1,
        };

        assert!(AwardPolicy::can_create(&ctx, &data, ()).is_ok());
        assert!(AwardPolicy::can_update(&ctx, &award, &patch, ()).is_ok());
        assert!(AwardPolicy::can_delete(&ctx, &award).is_ok());
        assert!(AwardPolicy::can_move(&ctx, &award).is_ok());
    }

    #[test]
    fn crud_policy_rejects_user_without_publisher_role() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let award = make_award(pool.as_ref(), work.work_id, 1, "Award");

        let patch = PatchAward {
            award_id: award.award_id,
            work_id: award.work_id,
            title: "Award Updated".to_string(),
            url: award.url.clone(),
            category: award.category.clone(),
            year: award.year.clone(),
            jury: award.jury.clone(),
            country: award.country,
            prize_statement: award.prize_statement.clone(),
            role: award.role,
            award_ordinal: 2,
        };

        let user = test_user_with_role("award-user", Role::PublisherUser, "org-other");
        let ctx = test_context_with_user(pool.clone(), user);

        let data = NewAward {
            work_id: work.work_id,
            title: "Award".to_string(),
            url: Some("https://example.com/award".to_string()),
            category: Some("Prize".to_string()),
            year: Some("2025".to_string()),
            jury: Some("Main Jury".to_string()),
            country: Some(CountryCode::Gbr),
            prize_statement: Some("Award note".to_string()),
            role: Some(AwardRole::Winner),
            award_ordinal: 1,
        };

        assert!(AwardPolicy::can_create(&ctx, &data, ()).is_err());
        assert!(AwardPolicy::can_update(&ctx, &award, &patch, ()).is_err());
        assert!(AwardPolicy::can_delete(&ctx, &award).is_err());
        assert!(AwardPolicy::can_move(&ctx, &award).is_err());
    }

    #[test]
    fn crud_policy_rejects_chapter_work() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("award-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let chapter = Work::create(
            pool.as_ref(),
            &NewWork {
                work_type: WorkType::BookChapter,
                work_status: WorkStatus::Forthcoming,
                reference: None,
                edition: None,
                imprint_id: imprint.imprint_id,
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
                first_page: Some("1".to_string()),
                last_page: Some("10".to_string()),
                page_interval: Some("1-10".to_string()),
            },
        )
        .expect("Failed to create chapter work");

        let data = NewAward {
            work_id: chapter.work_id,
            title: "Award".to_string(),
            url: Some("https://example.com/award".to_string()),
            category: Some("Prize".to_string()),
            year: Some("2025".to_string()),
            jury: Some("Main Jury".to_string()),
            country: Some(CountryCode::Gbr),
            prize_statement: Some("Award note".to_string()),
            role: Some(AwardRole::Winner),
            award_ordinal: 1,
        };

        assert!(matches!(
            AwardPolicy::can_create(&ctx, &data, ()),
            Err(thoth_errors::ThothError::ChapterBookMetadataError)
        ));
    }
}

#[cfg(feature = "backend")]
mod crud {
    use super::*;

    use crate::model::tests::db::{
        create_imprint, create_publisher, create_work, setup_test_db, test_context,
    };
    use crate::model::Reorder;

    #[test]
    fn crud_roundtrip_create_fetch_update_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let data = NewAward {
            work_id: work.work_id,
            title: "Award".to_string(),
            url: Some("https://example.com/award".to_string()),
            category: Some("Prize".to_string()),
            year: Some("2025-2026".to_string()),
            jury: Some("Main Jury".to_string()),
            country: Some(CountryCode::Gbr),
            prize_statement: Some("Award note".to_string()),
            role: Some(AwardRole::Winner),
            award_ordinal: 1,
        };

        let award = Award::create(pool.as_ref(), &data).expect("Failed to create");
        let fetched = Award::from_id(pool.as_ref(), &award.award_id).expect("Failed to fetch");
        assert_eq!(award.award_id, fetched.award_id);
        assert_eq!(fetched.year.as_deref(), Some("2025-2026"));
        assert_eq!(fetched.jury.as_deref(), Some("Main Jury"));
        assert_eq!(fetched.country, Some(CountryCode::Gbr));

        let patch = PatchAward {
            award_id: award.award_id,
            work_id: award.work_id,
            title: "Award Updated".to_string(),
            url: award.url.clone(),
            category: award.category.clone(),
            year: Some("2026".to_string()),
            jury: Some("Updated Jury".to_string()),
            country: Some(CountryCode::Fra),
            prize_statement: Some("Updated award note".to_string()),
            role: Some(AwardRole::JointWinner),
            award_ordinal: 1,
        };

        let ctx = test_context(pool.clone(), "test-user");
        let updated = award.update(&ctx, &patch).expect("Failed to update");
        assert_eq!(updated.title, patch.title);
        assert_eq!(updated.year, patch.year);
        assert_eq!(updated.jury, patch.jury);
        assert_eq!(updated.country, patch.country);
        assert_eq!(updated.role, patch.role);

        let deleted = updated.delete(pool.as_ref()).expect("Failed to delete");
        assert!(Award::from_id(pool.as_ref(), &deleted.award_id).is_err());
    }

    #[test]
    fn crud_change_ordinal_reorders_within_work() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let first = make_award(pool.as_ref(), work.work_id, 1, "Award 1");
        let second = make_award(pool.as_ref(), work.work_id, 2, "Award 2");
        let ctx = test_context(pool.clone(), "test-user");

        let moved = second
            .change_ordinal(&ctx, second.award_ordinal, 1)
            .expect("Failed to reorder award");
        let shifted =
            Award::from_id(pool.as_ref(), &first.award_id).expect("Failed to fetch shifted award");

        assert_eq!(moved.award_ordinal, 1);
        assert_eq!(shifted.award_ordinal, 2);
    }
}
