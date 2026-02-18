use super::*;
use crate::model::Crud;
use std::str::FromStr;
use uuid::Uuid;

fn make_additional_resource(
    pool: &crate::db::PgPool,
    work_id: Uuid,
    resource_ordinal: i32,
    title: &str,
) -> AdditionalResource {
    let data = NewAdditionalResource {
        work_id,
        title: title.to_string(),
        description: Some("Resource description".to_string()),
        attribution: Some("Resource attribution".to_string()),
        resource_type: ResourceType::Website,
        doi: Some(crate::model::Doi::from_str("https://doi.org/10.1234/RESOURCE.1").unwrap()),
        handle: Some("hdl:1234/5678".to_string()),
        url: Some("https://example.com/resource".to_string()),
        resource_ordinal,
    };

    AdditionalResource::create(pool, &data).expect("Failed to create additional resource")
}

mod defaults {
    use super::*;

    #[test]
    fn resourcetype_default_is_other() {
        let resource_type: ResourceType = Default::default();
        assert_eq!(resource_type, ResourceType::Other);
    }

    #[test]
    fn additionalresourcefield_default_is_resource_ordinal() {
        let field: AdditionalResourceField = Default::default();
        assert_eq!(field, AdditionalResourceField::ResourceOrdinal);
    }
}

mod helpers {
    use super::*;
    use crate::model::{Crud, HistoryEntry};

    #[test]
    fn pk_returns_id() {
        let resource: AdditionalResource = Default::default();
        assert_eq!(resource.pk(), resource.additional_resource_id);
    }

    #[test]
    fn history_entry_serializes_model() {
        let resource: AdditionalResource = Default::default();
        let user_id = "123456".to_string();
        let history = resource.new_history_entry(&user_id);
        assert_eq!(
            history.additional_resource_id,
            resource.additional_resource_id
        );
        assert_eq!(history.user_id, user_id);
        assert_eq!(
            history.data,
            serde_json::Value::String(serde_json::to_string(&resource).unwrap())
        );
    }
}

mod conversions {
    use super::*;
    #[cfg(feature = "backend")]
    use crate::model::tests::db::setup_test_db;
    #[cfg(feature = "backend")]
    use crate::model::tests::{assert_db_enum_roundtrip, assert_graphql_enum_roundtrip};

    #[cfg(feature = "backend")]
    #[test]
    fn resourcetype_graphql_roundtrip() {
        assert_graphql_enum_roundtrip(ResourceType::Other);
    }

    #[cfg(feature = "backend")]
    #[test]
    fn resourcetype_db_enum_roundtrip() {
        let (_guard, pool) = setup_test_db();

        assert_db_enum_roundtrip::<ResourceType, crate::schema::sql_types::ResourceType>(
            pool.as_ref(),
            "'OTHER'::resource_type",
            ResourceType::Other,
        );
    }
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;

    use crate::model::additional_resource::policy::AdditionalResourcePolicy;
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
        let user = test_user_with_role("resource-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let data = NewAdditionalResource {
            work_id: work.work_id,
            title: "Resource".to_string(),
            description: Some("Description".to_string()),
            attribution: Some("Attribution".to_string()),
            resource_type: ResourceType::Website,
            doi: None,
            handle: None,
            url: Some("https://example.com".to_string()),
            resource_ordinal: 1,
        };

        let resource = AdditionalResource::create(pool.as_ref(), &data).expect("Failed to create");
        let patch = PatchAdditionalResource {
            additional_resource_id: resource.additional_resource_id,
            work_id: resource.work_id,
            title: "Resource Updated".to_string(),
            description: resource.description.clone(),
            attribution: resource.attribution.clone(),
            resource_type: resource.resource_type,
            doi: resource.doi.clone(),
            handle: resource.handle.clone(),
            url: resource.url.clone(),
            resource_ordinal: 1,
        };

        assert!(AdditionalResourcePolicy::can_create(&ctx, &data, ()).is_ok());
        assert!(AdditionalResourcePolicy::can_update(&ctx, &resource, &patch, ()).is_ok());
        assert!(AdditionalResourcePolicy::can_delete(&ctx, &resource).is_ok());
        assert!(AdditionalResourcePolicy::can_move(&ctx, &resource).is_ok());
    }

    #[test]
    fn crud_policy_rejects_user_without_publisher_role() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let resource = make_additional_resource(pool.as_ref(), work.work_id, 1, "Resource");

        let patch = PatchAdditionalResource {
            additional_resource_id: resource.additional_resource_id,
            work_id: resource.work_id,
            title: "Resource Updated".to_string(),
            description: resource.description.clone(),
            attribution: resource.attribution.clone(),
            resource_type: resource.resource_type,
            doi: resource.doi.clone(),
            handle: resource.handle.clone(),
            url: resource.url.clone(),
            resource_ordinal: 2,
        };

        let user = test_user_with_role("resource-user", Role::PublisherUser, "org-other");
        let ctx = test_context_with_user(pool.clone(), user);

        let data = NewAdditionalResource {
            work_id: work.work_id,
            title: "Resource".to_string(),
            description: Some("Description".to_string()),
            attribution: Some("Attribution".to_string()),
            resource_type: ResourceType::Website,
            doi: None,
            handle: None,
            url: Some("https://example.com".to_string()),
            resource_ordinal: 1,
        };

        assert!(AdditionalResourcePolicy::can_create(&ctx, &data, ()).is_err());
        assert!(AdditionalResourcePolicy::can_update(&ctx, &resource, &patch, ()).is_err());
        assert!(AdditionalResourcePolicy::can_delete(&ctx, &resource).is_err());
        assert!(AdditionalResourcePolicy::can_move(&ctx, &resource).is_err());
    }

    #[test]
    fn crud_policy_rejects_chapter_work() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("resource-user", Role::PublisherUser, &org_id);
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
                page_interval: Some("1–10".to_string()),
            },
        )
        .expect("Failed to create chapter work");

        let data = NewAdditionalResource {
            work_id: chapter.work_id,
            title: "Resource".to_string(),
            description: Some("Description".to_string()),
            attribution: None,
            resource_type: ResourceType::Website,
            doi: None,
            handle: None,
            url: Some("https://example.com".to_string()),
            resource_ordinal: 1,
        };

        assert!(matches!(
            AdditionalResourcePolicy::can_create(&ctx, &data, ()),
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

        let data = NewAdditionalResource {
            work_id: work.work_id,
            title: "Resource".to_string(),
            description: Some("Description".to_string()),
            attribution: Some("Attribution".to_string()),
            resource_type: ResourceType::Website,
            doi: None,
            handle: None,
            url: Some("https://example.com".to_string()),
            resource_ordinal: 1,
        };

        let resource = AdditionalResource::create(pool.as_ref(), &data).expect("Failed to create");
        let fetched = AdditionalResource::from_id(pool.as_ref(), &resource.additional_resource_id)
            .expect("Failed to fetch");
        assert_eq!(
            resource.additional_resource_id,
            fetched.additional_resource_id
        );

        let patch = PatchAdditionalResource {
            additional_resource_id: resource.additional_resource_id,
            work_id: resource.work_id,
            title: "Resource Updated".to_string(),
            description: Some("Description Updated".to_string()),
            attribution: resource.attribution.clone(),
            resource_type: ResourceType::Document,
            doi: resource.doi.clone(),
            handle: resource.handle.clone(),
            url: resource.url.clone(),
            resource_ordinal: 1,
        };

        let ctx = test_context(pool.clone(), "test-user");
        let updated = resource.update(&ctx, &patch).expect("Failed to update");
        assert_eq!(updated.title, patch.title);

        let deleted = updated.delete(pool.as_ref()).expect("Failed to delete");
        assert!(
            AdditionalResource::from_id(pool.as_ref(), &deleted.additional_resource_id).is_err()
        );
    }

    #[test]
    fn crud_all_respects_limit_and_offset() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        make_additional_resource(pool.as_ref(), work.work_id, 1, "Resource 1");
        make_additional_resource(pool.as_ref(), work.work_id, 2, "Resource 2");

        let first = AdditionalResource::all(
            pool.as_ref(),
            1,
            0,
            None,
            AdditionalResourceOrderBy {
                field: AdditionalResourceField::ResourceOrdinal,
                direction: crate::graphql::types::inputs::Direction::Asc,
            },
            vec![],
            Some(work.work_id),
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to fetch additional resources");

        let second = AdditionalResource::all(
            pool.as_ref(),
            1,
            1,
            None,
            AdditionalResourceOrderBy {
                field: AdditionalResourceField::ResourceOrdinal,
                direction: crate::graphql::types::inputs::Direction::Asc,
            },
            vec![],
            Some(work.work_id),
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to fetch additional resources");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_ne!(
            first[0].additional_resource_id,
            second[0].additional_resource_id
        );
    }

    #[test]
    fn crud_change_ordinal_reorders_within_work() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let first = make_additional_resource(pool.as_ref(), work.work_id, 1, "Resource 1");
        let second = make_additional_resource(pool.as_ref(), work.work_id, 2, "Resource 2");
        let ctx = test_context(pool.clone(), "test-user");

        let moved = second
            .change_ordinal(&ctx, second.resource_ordinal, 1)
            .expect("Failed to reorder additional resource");
        let shifted = AdditionalResource::from_id(pool.as_ref(), &first.additional_resource_id)
            .expect("Failed to fetch shifted additional resource");

        assert_eq!(moved.resource_ordinal, 1);
        assert_eq!(shifted.resource_ordinal, 2);
    }
}
