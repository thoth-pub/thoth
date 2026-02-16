use super::*;
use crate::model::Crud;
use std::str::FromStr;
use uuid::Uuid;

fn make_book_review(pool: &crate::db::PgPool, work_id: Uuid, review_ordinal: i32) -> BookReview {
    let data = NewBookReview {
        work_id,
        title: Some("Review title".to_string()),
        author_name: Some("Reviewer".to_string()),
        url: Some("https://example.com/review".to_string()),
        doi: Some(crate::model::Doi::from_str("https://doi.org/10.1234/REVIEW.1").unwrap()),
        review_date: chrono::NaiveDate::from_ymd_opt(2025, 1, 1),
        journal_name: Some("Journal".to_string()),
        journal_volume: Some("12".to_string()),
        journal_number: Some("3".to_string()),
        journal_issn: Some("1234-5678".to_string()),
        text: Some("Review text".to_string()),
        review_ordinal,
    };

    BookReview::create(pool, &data).expect("Failed to create book review")
}

mod defaults {
    use super::*;

    #[test]
    fn bookreviewfield_default_is_review_ordinal() {
        let field: BookReviewField = Default::default();
        assert_eq!(field, BookReviewField::ReviewOrdinal);
    }
}

mod helpers {
    use super::*;
    use crate::model::{Crud, HistoryEntry};

    #[test]
    fn pk_returns_id() {
        let review: BookReview = Default::default();
        assert_eq!(review.pk(), review.book_review_id);
    }

    #[test]
    fn history_entry_serializes_model() {
        let review: BookReview = Default::default();
        let user_id = "123456".to_string();
        let history = review.new_history_entry(&user_id);
        assert_eq!(history.book_review_id, review.book_review_id);
        assert_eq!(history.user_id, user_id);
        assert_eq!(
            history.data,
            serde_json::Value::String(serde_json::to_string(&review).unwrap())
        );
    }
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;

    use crate::model::book_review::policy::BookReviewPolicy;
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
        let user = test_user_with_role("review-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let data = NewBookReview {
            work_id: work.work_id,
            title: Some("Review title".to_string()),
            author_name: Some("Reviewer".to_string()),
            url: Some("https://example.com/review".to_string()),
            doi: None,
            review_date: chrono::NaiveDate::from_ymd_opt(2025, 1, 1),
            journal_name: Some("Journal".to_string()),
            journal_volume: Some("12".to_string()),
            journal_number: Some("3".to_string()),
            journal_issn: Some("1234-5678".to_string()),
            text: Some("Review text".to_string()),
            review_ordinal: 1,
        };

        let review = BookReview::create(pool.as_ref(), &data).expect("Failed to create");
        let patch = PatchBookReview {
            book_review_id: review.book_review_id,
            work_id: review.work_id,
            title: review.title.clone(),
            author_name: review.author_name.clone(),
            url: review.url.clone(),
            doi: review.doi.clone(),
            review_date: review.review_date,
            journal_name: review.journal_name.clone(),
            journal_volume: review.journal_volume.clone(),
            journal_number: review.journal_number.clone(),
            journal_issn: review.journal_issn.clone(),
            text: Some("Updated review text".to_string()),
            review_ordinal: 1,
        };

        assert!(BookReviewPolicy::can_create(&ctx, &data, ()).is_ok());
        assert!(BookReviewPolicy::can_update(&ctx, &review, &patch, ()).is_ok());
        assert!(BookReviewPolicy::can_delete(&ctx, &review).is_ok());
        assert!(BookReviewPolicy::can_move(&ctx, &review).is_ok());
    }

    #[test]
    fn crud_policy_rejects_user_without_publisher_role() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let review = make_book_review(pool.as_ref(), work.work_id, 1);

        let patch = PatchBookReview {
            book_review_id: review.book_review_id,
            work_id: review.work_id,
            title: review.title.clone(),
            author_name: review.author_name.clone(),
            url: review.url.clone(),
            doi: review.doi.clone(),
            review_date: review.review_date,
            journal_name: review.journal_name.clone(),
            journal_volume: review.journal_volume.clone(),
            journal_number: review.journal_number.clone(),
            journal_issn: review.journal_issn.clone(),
            text: Some("Updated review text".to_string()),
            review_ordinal: 2,
        };

        let user = test_user_with_role("review-user", Role::PublisherUser, "org-other");
        let ctx = test_context_with_user(pool.clone(), user);

        let data = NewBookReview {
            work_id: work.work_id,
            title: Some("Review title".to_string()),
            author_name: Some("Reviewer".to_string()),
            url: Some("https://example.com/review".to_string()),
            doi: None,
            review_date: chrono::NaiveDate::from_ymd_opt(2025, 1, 1),
            journal_name: Some("Journal".to_string()),
            journal_volume: Some("12".to_string()),
            journal_number: Some("3".to_string()),
            journal_issn: Some("1234-5678".to_string()),
            text: Some("Review text".to_string()),
            review_ordinal: 1,
        };

        assert!(BookReviewPolicy::can_create(&ctx, &data, ()).is_err());
        assert!(BookReviewPolicy::can_update(&ctx, &review, &patch, ()).is_err());
        assert!(BookReviewPolicy::can_delete(&ctx, &review).is_err());
        assert!(BookReviewPolicy::can_move(&ctx, &review).is_err());
    }

    #[test]
    fn crud_policy_rejects_chapter_work() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("review-user", Role::PublisherUser, &org_id);
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

        let data = NewBookReview {
            work_id: chapter.work_id,
            title: Some("Review title".to_string()),
            author_name: Some("Reviewer".to_string()),
            url: Some("https://example.com/review".to_string()),
            doi: None,
            review_date: chrono::NaiveDate::from_ymd_opt(2025, 1, 1),
            journal_name: Some("Journal".to_string()),
            journal_volume: Some("12".to_string()),
            journal_number: Some("3".to_string()),
            journal_issn: Some("1234-5678".to_string()),
            text: Some("Review text".to_string()),
            review_ordinal: 1,
        };

        assert!(matches!(
            BookReviewPolicy::can_create(&ctx, &data, ()),
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

        let review = make_book_review(pool.as_ref(), work.work_id, 1);
        let fetched = BookReview::from_id(pool.as_ref(), &review.book_review_id)
            .expect("Failed to fetch");
        assert_eq!(review.book_review_id, fetched.book_review_id);

        let patch = PatchBookReview {
            book_review_id: review.book_review_id,
            work_id: review.work_id,
            title: review.title.clone(),
            author_name: review.author_name.clone(),
            url: review.url.clone(),
            doi: review.doi.clone(),
            review_date: review.review_date,
            journal_name: review.journal_name.clone(),
            journal_volume: review.journal_volume.clone(),
            journal_number: review.journal_number.clone(),
            journal_issn: review.journal_issn.clone(),
            text: Some("Updated review text".to_string()),
            review_ordinal: 1,
        };

        let ctx = test_context(pool.clone(), "test-user");
        let updated = review.update(&ctx, &patch).expect("Failed to update");
        assert_eq!(updated.text, patch.text);

        let deleted = updated.delete(pool.as_ref()).expect("Failed to delete");
        assert!(BookReview::from_id(pool.as_ref(), &deleted.book_review_id).is_err());
    }

    #[test]
    fn crud_change_ordinal_reorders_within_work() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let first = make_book_review(pool.as_ref(), work.work_id, 1);
        let second = make_book_review(pool.as_ref(), work.work_id, 2);
        let ctx = test_context(pool.clone(), "test-user");

        let moved = second
            .change_ordinal(&ctx, second.review_ordinal, 1)
            .expect("Failed to reorder book review");
        let shifted = BookReview::from_id(pool.as_ref(), &first.book_review_id)
            .expect("Failed to fetch shifted book review");

        assert_eq!(moved.review_ordinal, 1);
        assert_eq!(shifted.review_ordinal, 2);
    }
}
