use super::*;
use crate::model::Crud;
use uuid::Uuid;

fn make_work_featured_video(
    pool: &crate::db::PgPool,
    work_id: Uuid,
    url: Option<String>,
) -> WorkFeaturedVideo {
    let data = NewWorkFeaturedVideo {
        work_id,
        title: Some("Featured video".to_string()),
        url,
        width: 560,
        height: 315,
    };

    WorkFeaturedVideo::create(pool, &data).expect("Failed to create featured video")
}

mod defaults {
    use super::*;

    #[test]
    fn workfeaturedvideofield_default_is_updated_at() {
        let field: WorkFeaturedVideoField = Default::default();
        assert_eq!(field, WorkFeaturedVideoField::UpdatedAt);
    }
}

mod helpers {
    use super::*;
    use crate::model::{Crud, HistoryEntry};

    #[test]
    fn pk_returns_id() {
        let video: WorkFeaturedVideo = Default::default();
        assert_eq!(video.pk(), video.work_featured_video_id);
    }

    #[test]
    fn history_entry_serializes_model() {
        let video: WorkFeaturedVideo = Default::default();
        let user_id = "123456".to_string();
        let history = video.new_history_entry(&user_id);
        assert_eq!(history.work_featured_video_id, video.work_featured_video_id);
        assert_eq!(history.user_id, user_id);
        assert_eq!(
            history.data,
            serde_json::Value::String(serde_json::to_string(&video).unwrap())
        );
    }
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;

    use crate::model::tests::db::{
        create_imprint, create_publisher, create_work, setup_test_db, test_context_with_user,
        test_user_with_role,
    };
    use crate::model::work::{NewWork, Work, WorkStatus, WorkType};
    use crate::model::work_featured_video::policy::WorkFeaturedVideoPolicy;
    use crate::policy::{CreatePolicy, DeletePolicy, Role, UpdatePolicy};

    #[test]
    fn crud_policy_allows_publisher_user_for_write() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("featured-video-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let data = NewWorkFeaturedVideo {
            work_id: work.work_id,
            title: Some("Featured video".to_string()),
            url: Some("https://cdn.example.org/video.mp4".to_string()),
            width: 560,
            height: 315,
        };

        let video = WorkFeaturedVideo::create(pool.as_ref(), &data).expect("Failed to create");
        let patch = PatchWorkFeaturedVideo {
            work_featured_video_id: video.work_featured_video_id,
            work_id: video.work_id,
            title: video.title.clone(),
            url: Some("https://cdn.example.org/video-v2.mp4".to_string()),
            width: video.width,
            height: video.height,
        };

        assert!(WorkFeaturedVideoPolicy::can_create(&ctx, &data, ()).is_ok());
        assert!(WorkFeaturedVideoPolicy::can_update(&ctx, &video, &patch, ()).is_ok());
        assert!(WorkFeaturedVideoPolicy::can_delete(&ctx, &video).is_ok());
    }

    #[test]
    fn crud_policy_rejects_user_without_publisher_role() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let video = make_work_featured_video(
            pool.as_ref(),
            work.work_id,
            Some("https://cdn.example.org/video.mp4".to_string()),
        );

        let patch = PatchWorkFeaturedVideo {
            work_featured_video_id: video.work_featured_video_id,
            work_id: video.work_id,
            title: video.title.clone(),
            url: Some("https://cdn.example.org/video-v2.mp4".to_string()),
            width: video.width,
            height: video.height,
        };

        let user = test_user_with_role("featured-video-user", Role::PublisherUser, "org-other");
        let ctx = test_context_with_user(pool.clone(), user);

        let data = NewWorkFeaturedVideo {
            work_id: work.work_id,
            title: Some("Featured video".to_string()),
            url: Some("https://cdn.example.org/video.mp4".to_string()),
            width: 560,
            height: 315,
        };

        assert!(WorkFeaturedVideoPolicy::can_create(&ctx, &data, ()).is_err());
        assert!(WorkFeaturedVideoPolicy::can_update(&ctx, &video, &patch, ()).is_err());
        assert!(WorkFeaturedVideoPolicy::can_delete(&ctx, &video).is_err());
    }

    #[test]
    fn crud_policy_rejects_chapter_work() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("featured-video-user", Role::PublisherUser, &org_id);
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

        let data = NewWorkFeaturedVideo {
            work_id: chapter.work_id,
            title: Some("Featured video".to_string()),
            url: Some("https://cdn.example.org/video.mp4".to_string()),
            width: 560,
            height: 315,
        };

        assert!(matches!(
            WorkFeaturedVideoPolicy::can_create(&ctx, &data, ()),
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

    #[test]
    fn crud_roundtrip_create_fetch_update_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let video = make_work_featured_video(
            pool.as_ref(),
            work.work_id,
            Some("https://cdn.example.org/video.mp4".to_string()),
        );
        let fetched = WorkFeaturedVideo::from_id(pool.as_ref(), &video.work_featured_video_id)
            .expect("Failed to fetch");
        assert_eq!(video.work_featured_video_id, fetched.work_featured_video_id);

        let patch = PatchWorkFeaturedVideo {
            work_featured_video_id: video.work_featured_video_id,
            work_id: video.work_id,
            title: Some("Updated featured video".to_string()),
            url: Some("https://cdn.example.org/video-v2.mp4".to_string()),
            width: 640,
            height: 360,
        };

        let ctx = test_context(pool.clone(), "test-user");
        let updated = video.update(&ctx, &patch).expect("Failed to update");
        assert_eq!(updated.url, patch.url);

        let deleted = updated.delete(pool.as_ref()).expect("Failed to delete");
        assert!(
            WorkFeaturedVideo::from_id(pool.as_ref(), &deleted.work_featured_video_id).is_err()
        );
    }

    #[test]
    fn crud_from_work_id_returns_record() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let video = make_work_featured_video(
            pool.as_ref(),
            work.work_id,
            Some("https://cdn.example.org/video.mp4".to_string()),
        );

        let fetched = WorkFeaturedVideo::from_work_id(pool.as_ref(), &work.work_id)
            .expect("Failed to fetch by work id")
            .expect("Expected featured video");

        assert_eq!(fetched.work_featured_video_id, video.work_featured_video_id);
    }
}
