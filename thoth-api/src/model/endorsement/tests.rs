use super::*;
use crate::model::Crud;
use uuid::Uuid;

fn make_endorsement(
    pool: &crate::db::PgPool,
    work_id: Uuid,
    endorsement_ordinal: i32,
    author_name: Option<String>,
) -> Endorsement {
    let data = NewEndorsement {
        work_id,
        author_name,
        author_role: Some("Author".to_string()),
        url: Some("https://example.com/endorsement".to_string()),
        text: Some("Endorsement text".to_string()),
        endorsement_ordinal,
    };

    Endorsement::create(pool, &data).expect("Failed to create endorsement")
}

mod defaults {
    use super::*;

    #[test]
    fn endorsementfield_default_is_endorsement_ordinal() {
        let field: EndorsementField = Default::default();
        assert_eq!(field, EndorsementField::EndorsementOrdinal);
    }
}

mod helpers {
    use super::*;
    use crate::model::{Crud, HistoryEntry};

    #[test]
    fn pk_returns_id() {
        let endorsement: Endorsement = Default::default();
        assert_eq!(endorsement.pk(), endorsement.endorsement_id);
    }

    #[test]
    fn history_entry_serializes_model() {
        let endorsement: Endorsement = Default::default();
        let user_id = "123456".to_string();
        let history = endorsement.new_history_entry(&user_id);
        assert_eq!(history.endorsement_id, endorsement.endorsement_id);
        assert_eq!(history.user_id, user_id);
        assert_eq!(
            history.data,
            serde_json::Value::String(serde_json::to_string(&endorsement).unwrap())
        );
    }
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;

    use crate::model::endorsement::policy::EndorsementPolicy;
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
        let user = test_user_with_role("endorsement-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let data = NewEndorsement {
            work_id: work.work_id,
            author_name: Some("Author".to_string()),
            author_role: Some("Role".to_string()),
            url: Some("https://example.com/endorsement".to_string()),
            text: Some("Endorsement text".to_string()),
            endorsement_ordinal: 1,
        };

        let endorsement = Endorsement::create(pool.as_ref(), &data).expect("Failed to create");
        let patch = PatchEndorsement {
            endorsement_id: endorsement.endorsement_id,
            work_id: endorsement.work_id,
            author_name: endorsement.author_name.clone(),
            author_role: endorsement.author_role.clone(),
            url: endorsement.url.clone(),
            text: Some("Updated endorsement text".to_string()),
            endorsement_ordinal: 1,
        };

        assert!(EndorsementPolicy::can_create(&ctx, &data, ()).is_ok());
        assert!(EndorsementPolicy::can_update(&ctx, &endorsement, &patch, ()).is_ok());
        assert!(EndorsementPolicy::can_delete(&ctx, &endorsement).is_ok());
        assert!(EndorsementPolicy::can_move(&ctx, &endorsement).is_ok());
    }

    #[test]
    fn crud_policy_rejects_user_without_publisher_role() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let endorsement =
            make_endorsement(pool.as_ref(), work.work_id, 1, Some("Author".to_string()));

        let patch = PatchEndorsement {
            endorsement_id: endorsement.endorsement_id,
            work_id: endorsement.work_id,
            author_name: endorsement.author_name.clone(),
            author_role: endorsement.author_role.clone(),
            url: endorsement.url.clone(),
            text: Some("Updated endorsement text".to_string()),
            endorsement_ordinal: 2,
        };

        let user = test_user_with_role("endorsement-user", Role::PublisherUser, "org-other");
        let ctx = test_context_with_user(pool.clone(), user);

        let data = NewEndorsement {
            work_id: work.work_id,
            author_name: Some("Author".to_string()),
            author_role: Some("Role".to_string()),
            url: Some("https://example.com/endorsement".to_string()),
            text: Some("Endorsement text".to_string()),
            endorsement_ordinal: 1,
        };

        assert!(EndorsementPolicy::can_create(&ctx, &data, ()).is_err());
        assert!(EndorsementPolicy::can_update(&ctx, &endorsement, &patch, ()).is_err());
        assert!(EndorsementPolicy::can_delete(&ctx, &endorsement).is_err());
        assert!(EndorsementPolicy::can_move(&ctx, &endorsement).is_err());
    }

    #[test]
    fn crud_policy_rejects_chapter_work() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("endorsement-user", Role::PublisherUser, &org_id);
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

        let data = NewEndorsement {
            work_id: chapter.work_id,
            author_name: Some("Author".to_string()),
            author_role: Some("Role".to_string()),
            url: Some("https://example.com/endorsement".to_string()),
            text: Some("Endorsement text".to_string()),
            endorsement_ordinal: 1,
        };

        assert!(matches!(
            EndorsementPolicy::can_create(&ctx, &data, ()),
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

        let data = NewEndorsement {
            work_id: work.work_id,
            author_name: Some("Author".to_string()),
            author_role: Some("Role".to_string()),
            url: Some("https://example.com/endorsement".to_string()),
            text: Some("Endorsement text".to_string()),
            endorsement_ordinal: 1,
        };

        let endorsement = Endorsement::create(pool.as_ref(), &data).expect("Failed to create");
        let fetched = Endorsement::from_id(pool.as_ref(), &endorsement.endorsement_id)
            .expect("Failed to fetch");
        assert_eq!(endorsement.endorsement_id, fetched.endorsement_id);

        let patch = PatchEndorsement {
            endorsement_id: endorsement.endorsement_id,
            work_id: endorsement.work_id,
            author_name: endorsement.author_name.clone(),
            author_role: endorsement.author_role.clone(),
            url: endorsement.url.clone(),
            text: Some("Updated endorsement text".to_string()),
            endorsement_ordinal: 1,
        };

        let ctx = test_context(pool.clone(), "test-user");
        let updated = endorsement.update(&ctx, &patch).expect("Failed to update");
        assert_eq!(updated.text, patch.text);

        let deleted = updated.delete(pool.as_ref()).expect("Failed to delete");
        assert!(Endorsement::from_id(pool.as_ref(), &deleted.endorsement_id).is_err());
    }

    #[test]
    fn crud_change_ordinal_reorders_within_work() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let first = make_endorsement(pool.as_ref(), work.work_id, 1, Some("Author 1".to_string()));
        let second = make_endorsement(pool.as_ref(), work.work_id, 2, Some("Author 2".to_string()));
        let ctx = test_context(pool.clone(), "test-user");

        let moved = second
            .change_ordinal(&ctx, second.endorsement_ordinal, 1)
            .expect("Failed to reorder endorsement");
        let shifted = Endorsement::from_id(pool.as_ref(), &first.endorsement_id)
            .expect("Failed to fetch shifted endorsement");

        assert_eq!(moved.endorsement_ordinal, 1);
        assert_eq!(shifted.endorsement_ordinal, 2);
    }
}
