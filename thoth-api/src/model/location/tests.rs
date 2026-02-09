use super::*;

mod defaults {
    use super::*;

    #[test]
    fn locationplatform_default_is_other() {
        let locationplatform: LocationPlatform = Default::default();
        assert_eq!(locationplatform, LocationPlatform::Other);
    }
}

mod display_and_parse {
    use super::*;

    #[test]
    fn locationplatform_display_formats_expected_strings() {
        assert_eq!(format!("{}", LocationPlatform::ProjectMuse), "Project MUSE");
        assert_eq!(format!("{}", LocationPlatform::Oapen), "OAPEN");
        assert_eq!(format!("{}", LocationPlatform::Doab), "DOAB");
        assert_eq!(format!("{}", LocationPlatform::Jstor), "JSTOR");
        assert_eq!(format!("{}", LocationPlatform::EbscoHost), "EBSCO Host");
        assert_eq!(format!("{}", LocationPlatform::OclcKb), "OCLC KB");
        assert_eq!(format!("{}", LocationPlatform::ProquestKb), "ProQuest KB");
        assert_eq!(
            format!("{}", LocationPlatform::ProquestExlibris),
            "ProQuest ExLibris"
        );
        assert_eq!(format!("{}", LocationPlatform::EbscoKb), "EBSCO KB");
        assert_eq!(format!("{}", LocationPlatform::JiscKb), "JISC KB");
        assert_eq!(format!("{}", LocationPlatform::GoogleBooks), "Google Books");
        assert_eq!(
            format!("{}", LocationPlatform::InternetArchive),
            "Internet Archive"
        );
        assert_eq!(format!("{}", LocationPlatform::ScienceOpen), "ScienceOpen");
        assert_eq!(format!("{}", LocationPlatform::ScieloBooks), "SciELO Books");
        assert_eq!(format!("{}", LocationPlatform::Zenodo), "Zenodo");
        assert_eq!(
            format!("{}", LocationPlatform::PublisherWebsite),
            "Publisher Website"
        );
        assert_eq!(format!("{}", LocationPlatform::Thoth), "Thoth");
        assert_eq!(format!("{}", LocationPlatform::Other), "Other");
    }

    #[test]
    fn locationplatform_fromstr_parses_expected_values() {
        use std::str::FromStr;
        assert_eq!(
            LocationPlatform::from_str("Project MUSE").unwrap(),
            LocationPlatform::ProjectMuse
        );
        assert_eq!(
            LocationPlatform::from_str("OAPEN").unwrap(),
            LocationPlatform::Oapen
        );
        assert_eq!(
            LocationPlatform::from_str("DOAB").unwrap(),
            LocationPlatform::Doab
        );
        assert_eq!(
            LocationPlatform::from_str("JSTOR").unwrap(),
            LocationPlatform::Jstor
        );
        assert_eq!(
            LocationPlatform::from_str("EBSCO Host").unwrap(),
            LocationPlatform::EbscoHost
        );
        assert_eq!(
            LocationPlatform::from_str("OCLC KB").unwrap(),
            LocationPlatform::OclcKb
        );
        assert_eq!(
            LocationPlatform::from_str("ProQuest KB").unwrap(),
            LocationPlatform::ProquestKb
        );
        assert_eq!(
            LocationPlatform::from_str("ProQuest ExLibris").unwrap(),
            LocationPlatform::ProquestExlibris
        );
        assert_eq!(
            LocationPlatform::from_str("EBSCO KB").unwrap(),
            LocationPlatform::EbscoKb
        );
        assert_eq!(
            LocationPlatform::from_str("JISC KB").unwrap(),
            LocationPlatform::JiscKb
        );
        assert_eq!(
            LocationPlatform::from_str("Google Books").unwrap(),
            LocationPlatform::GoogleBooks
        );
        assert_eq!(
            LocationPlatform::from_str("Internet Archive").unwrap(),
            LocationPlatform::InternetArchive
        );
        assert_eq!(
            LocationPlatform::from_str("ScienceOpen").unwrap(),
            LocationPlatform::ScienceOpen
        );
        assert_eq!(
            LocationPlatform::from_str("SciELO Books").unwrap(),
            LocationPlatform::ScieloBooks
        );
        assert_eq!(
            LocationPlatform::from_str("Zenodo").unwrap(),
            LocationPlatform::Zenodo
        );
        assert_eq!(
            LocationPlatform::from_str("Publisher Website").unwrap(),
            LocationPlatform::PublisherWebsite
        );
        assert_eq!(
            LocationPlatform::from_str("Thoth").unwrap(),
            LocationPlatform::Thoth
        );
        assert_eq!(
            LocationPlatform::from_str("Other").unwrap(),
            LocationPlatform::Other
        );
        assert!(LocationPlatform::from_str("Amazon").is_err());
        assert!(LocationPlatform::from_str("Twitter").is_err());
    }
}

mod conversions {
    use super::*;

    #[test]
    fn location_into_patch_location_copies_fields() {
        let location = Location {
            location_id: Uuid::parse_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
            publication_id: Uuid::parse_str("00000000-0000-0000-AAAA-000000000002").unwrap(),
            landing_page: Some("https://www.book.com/pb_landing".to_string()),
            full_text_url: Some("https://example.com/full_text.pdf".to_string()),
            location_platform: LocationPlatform::PublisherWebsite,
            created_at: Default::default(),
            updated_at: Default::default(),
            canonical: true,
        };

        let patch_location = PatchLocation::from(location.clone());

        assert_eq!(patch_location.location_id, location.location_id);
        assert_eq!(patch_location.publication_id, location.publication_id);
        assert_eq!(patch_location.landing_page, location.landing_page);
        assert_eq!(patch_location.full_text_url, location.full_text_url);
        assert_eq!(patch_location.location_platform, location.location_platform);
        assert_eq!(patch_location.canonical, location.canonical);
    }
}

mod helpers {
    use super::*;
    use crate::model::{Crud, HistoryEntry};

    #[test]
    fn pk_returns_id() {
        let location: Location = Default::default();
        assert_eq!(location.pk(), location.location_id);
    }

    #[test]
    fn history_entry_serializes_model() {
        let location: Location = Default::default();
        let user_id = "123456".to_string();
        let new_location_history = location.new_history_entry(&user_id);
        assert_eq!(new_location_history.location_id, location.location_id);
        assert_eq!(new_location_history.user_id, user_id);
        assert_eq!(
            new_location_history.data,
            serde_json::Value::String(serde_json::to_string(&location).unwrap())
        );
    }
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;

    use crate::model::location::policy::LocationPolicy;
    use crate::model::tests::db::{
        create_imprint, create_publication, create_publisher, create_work, setup_test_db,
        test_context_with_user, test_superuser, test_user_with_role,
    };
    use crate::model::Crud;
    use crate::policy::{CreatePolicy, DeletePolicy, Role, UpdatePolicy};
    use thoth_errors::ThothError;

    #[test]
    fn crud_policy_allows_publisher_user_for_write() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("location-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let publication = create_publication(pool.as_ref(), &work);
        let new_location = NewLocation {
            publication_id: publication.publication_id,
            landing_page: Some("https://example.com/landing".to_string()),
            full_text_url: None,
            location_platform: LocationPlatform::PublisherWebsite,
            canonical: true,
        };

        let location = Location::create(pool.as_ref(), &new_location).expect("Failed to create");
        let patch = PatchLocation {
            location_id: location.location_id,
            publication_id: location.publication_id,
            landing_page: Some("https://example.com/updated".to_string()),
            full_text_url: None,
            location_platform: location.location_platform,
            canonical: location.canonical,
        };

        assert!(LocationPolicy::can_create(&ctx, &new_location, ()).is_ok());
        assert!(LocationPolicy::can_update(&ctx, &location, &patch, ()).is_ok());
        assert!(LocationPolicy::can_delete(&ctx, &location).is_ok());
    }

    #[test]
    fn crud_policy_rejects_non_superuser_for_thoth_platform() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("location-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let publication = create_publication(pool.as_ref(), &work);
        let new_location = NewLocation {
            publication_id: publication.publication_id,
            landing_page: Some("https://example.com/landing".to_string()),
            full_text_url: Some("https://example.com/full".to_string()),
            location_platform: LocationPlatform::Thoth,
            canonical: true,
        };

        assert!(LocationPolicy::can_create(&ctx, &new_location, ()).is_err());

        let superuser = test_superuser("location-superuser");
        let super_ctx = test_context_with_user(pool.clone(), superuser);
        assert!(LocationPolicy::can_create(&super_ctx, &new_location, ()).is_ok());
    }

    #[test]
    fn crud_policy_rejects_non_superuser_for_thoth_update_and_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("location-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let publication = create_publication(pool.as_ref(), &work);
        let location = Location::create(
            pool.as_ref(),
            &NewLocation {
                publication_id: publication.publication_id,
                landing_page: Some("https://example.com/landing".to_string()),
                full_text_url: Some("https://example.com/full".to_string()),
                location_platform: LocationPlatform::Thoth,
                canonical: true,
            },
        )
        .expect("Failed to create location");

        let patch = PatchLocation {
            location_id: location.location_id,
            publication_id: location.publication_id,
            landing_page: Some("https://example.com/updated".to_string()),
            full_text_url: Some("https://example.com/full.pdf".to_string()),
            location_platform: location.location_platform,
            canonical: location.canonical,
        };

        let update_result = LocationPolicy::can_update(&ctx, &location, &patch, ());
        assert!(matches!(update_result, Err(ThothError::ThothLocationError)));

        let delete_result = LocationPolicy::can_delete(&ctx, &location);
        assert!(matches!(delete_result, Err(ThothError::ThothLocationError)));
    }

    #[test]
    fn crud_policy_rejects_non_superuser_thoth_canonical_update() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("location-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let publication = create_publication(pool.as_ref(), &work);

        Location::create(
            pool.as_ref(),
            &NewLocation {
                publication_id: publication.publication_id,
                landing_page: Some("https://example.com/landing".to_string()),
                full_text_url: Some("https://example.com/full".to_string()),
                location_platform: LocationPlatform::Thoth,
                canonical: true,
            },
        )
        .expect("Failed to create canonical thoth location");

        let location = Location::create(
            pool.as_ref(),
            &NewLocation {
                publication_id: publication.publication_id,
                landing_page: Some("https://example.com/other".to_string()),
                full_text_url: None,
                location_platform: LocationPlatform::PublisherWebsite,
                canonical: false,
            },
        )
        .expect("Failed to create location");

        let patch = PatchLocation {
            location_id: location.location_id,
            publication_id: location.publication_id,
            landing_page: location.landing_page.clone(),
            full_text_url: location.full_text_url.clone(),
            location_platform: location.location_platform,
            canonical: true,
        };

        let result = LocationPolicy::can_update(&ctx, &location, &patch, ());
        assert!(matches!(result, Err(ThothError::ThothUpdateCanonicalError)));
    }
}

#[cfg(feature = "backend")]
mod crud {
    use super::*;
    use uuid::Uuid;

    use crate::model::tests::db::{
        create_imprint, create_publication, create_publisher, create_work, setup_test_db,
        test_context,
    };
    use crate::model::Crud;
    use thoth_errors::ThothError;

    fn make_location(
        pool: &crate::db::PgPool,
        publication_id: Uuid,
        location_platform: LocationPlatform,
        canonical: bool,
        landing_page: Option<String>,
    ) -> Location {
        let new_location = NewLocation {
            publication_id,
            landing_page,
            full_text_url: None,
            location_platform,
            canonical,
        };

        Location::create(pool, &new_location).expect("Failed to create location")
    }

    #[test]
    fn crud_roundtrip_create_fetch_update_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let publication = create_publication(pool.as_ref(), &work);

        let new_location = NewLocation {
            publication_id: publication.publication_id,
            landing_page: Some("https://example.com/landing".to_string()),
            full_text_url: None,
            location_platform: LocationPlatform::PublisherWebsite,
            canonical: true,
        };

        let location = Location::create(pool.as_ref(), &new_location).expect("Failed to create");
        let fetched =
            Location::from_id(pool.as_ref(), &location.location_id).expect("Failed to fetch");
        assert_eq!(location.location_id, fetched.location_id);

        let patch = PatchLocation {
            location_id: location.location_id,
            publication_id: location.publication_id,
            landing_page: Some("https://example.com/updated".to_string()),
            full_text_url: Some("https://example.com/full.pdf".to_string()),
            location_platform: LocationPlatform::Other,
            canonical: true,
        };

        let ctx = test_context(pool.clone(), "test-user");
        let updated = location.update(&ctx, &patch).expect("Failed to update");
        assert_eq!(updated.landing_page, patch.landing_page);

        let deleted = updated.delete(pool.as_ref()).expect("Failed to delete");
        assert!(Location::from_id(pool.as_ref(), &deleted.location_id).is_err());
    }

    #[test]
    fn crud_update_rejects_changing_canonical_to_non_canonical() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let publication = create_publication(pool.as_ref(), &work);

        let location = make_location(
            pool.as_ref(),
            publication.publication_id,
            LocationPlatform::PublisherWebsite,
            true,
            Some("https://example.com/landing".to_string()),
        );
        let patch = PatchLocation {
            location_id: location.location_id,
            publication_id: location.publication_id,
            landing_page: location.landing_page.clone(),
            full_text_url: location.full_text_url.clone(),
            location_platform: location.location_platform,
            canonical: false,
        };

        let ctx = test_context(pool.clone(), "test-user");
        let result = location.update(&ctx, &patch);
        assert!(matches!(result, Err(ThothError::CanonicalLocationError)));
    }

    #[test]
    fn crud_all_respects_limit_and_offset() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let publication = create_publication(pool.as_ref(), &work);

        make_location(
            pool.as_ref(),
            publication.publication_id,
            LocationPlatform::PublisherWebsite,
            true,
            Some(format!("https://example.com/{}", Uuid::new_v4())),
        );
        make_location(
            pool.as_ref(),
            publication.publication_id,
            LocationPlatform::Other,
            false,
            Some(format!("https://example.com/{}", Uuid::new_v4())),
        );

        let order = LocationOrderBy {
            field: LocationField::LocationId,
            direction: Direction::Asc,
        };

        let first = Location::all(
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
        .expect("Failed to fetch locations");
        let second = Location::all(
            pool.as_ref(),
            1,
            1,
            None,
            LocationOrderBy {
                field: LocationField::LocationId,
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
        .expect("Failed to fetch locations");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_ne!(first[0].location_id, second[0].location_id);
    }

    #[test]
    fn crud_count_returns_total() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let publication = create_publication(pool.as_ref(), &work);

        make_location(
            pool.as_ref(),
            publication.publication_id,
            LocationPlatform::PublisherWebsite,
            true,
            Some(format!("https://example.com/{}", Uuid::new_v4())),
        );
        make_location(
            pool.as_ref(),
            publication.publication_id,
            LocationPlatform::Other,
            false,
            Some(format!("https://example.com/{}", Uuid::new_v4())),
        );

        let count = Location::count(pool.as_ref(), None, vec![], vec![], vec![], None, None)
            .expect("Failed to count locations");
        assert_eq!(count, 2);
    }

    #[test]
    fn crud_count_filters_by_platform() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let publication = create_publication(pool.as_ref(), &work);

        make_location(
            pool.as_ref(),
            publication.publication_id,
            LocationPlatform::PublisherWebsite,
            true,
            Some(format!("https://example.com/{}", Uuid::new_v4())),
        );
        make_location(
            pool.as_ref(),
            publication.publication_id,
            LocationPlatform::Other,
            false,
            Some(format!("https://example.com/{}", Uuid::new_v4())),
        );

        let count = Location::count(
            pool.as_ref(),
            None,
            vec![],
            vec![LocationPlatform::PublisherWebsite],
            vec![],
            None,
            None,
        )
        .expect("Failed to count locations by platform");
        assert_eq!(count, 1);
    }

    #[test]
    fn crud_filter_param_limits_location_platforms() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let publication = create_publication(pool.as_ref(), &work);

        let matches = make_location(
            pool.as_ref(),
            publication.publication_id,
            LocationPlatform::PublisherWebsite,
            true,
            Some(format!("https://example.com/{}", Uuid::new_v4())),
        );
        make_location(
            pool.as_ref(),
            publication.publication_id,
            LocationPlatform::Other,
            false,
            Some(format!("https://example.com/{}", Uuid::new_v4())),
        );

        let filtered = Location::all(
            pool.as_ref(),
            10,
            0,
            None,
            LocationOrderBy {
                field: LocationField::LocationId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![LocationPlatform::PublisherWebsite],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter locations by platform");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].location_id, matches.location_id);
    }

    #[test]
    fn crud_filter_parent_publication_id_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let publication = create_publication(pool.as_ref(), &work);
        let other_work = create_work(pool.as_ref(), &imprint);
        let other_publication = create_publication(pool.as_ref(), &other_work);

        let matches = make_location(
            pool.as_ref(),
            publication.publication_id,
            LocationPlatform::PublisherWebsite,
            true,
            Some(format!("https://example.com/{}", Uuid::new_v4())),
        );
        make_location(
            pool.as_ref(),
            other_publication.publication_id,
            LocationPlatform::Other,
            false,
            Some(format!("https://example.com/{}", Uuid::new_v4())),
        );

        let filtered = Location::all(
            pool.as_ref(),
            10,
            0,
            None,
            LocationOrderBy {
                field: LocationField::LocationId,
                direction: Direction::Asc,
            },
            vec![],
            Some(publication.publication_id),
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter locations by publication");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].location_id, matches.location_id);
    }

    #[test]
    fn crud_filter_publishers_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let publication = create_publication(pool.as_ref(), &work);
        let matches = make_location(
            pool.as_ref(),
            publication.publication_id,
            LocationPlatform::PublisherWebsite,
            true,
            Some(format!("https://example.com/{}", Uuid::new_v4())),
        );

        let other_publisher = create_publisher(pool.as_ref());
        let other_imprint = create_imprint(pool.as_ref(), &other_publisher);
        let other_work = create_work(pool.as_ref(), &other_imprint);
        let other_publication = create_publication(pool.as_ref(), &other_work);
        make_location(
            pool.as_ref(),
            other_publication.publication_id,
            LocationPlatform::Other,
            false,
            Some(format!("https://example.com/{}", Uuid::new_v4())),
        );

        let filtered = Location::all(
            pool.as_ref(),
            10,
            0,
            None,
            LocationOrderBy {
                field: LocationField::LocationId,
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
        .expect("Failed to filter locations by publisher");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].location_id, matches.location_id);
    }

    #[test]
    fn crud_ordering_by_id_respects_direction() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let publication = create_publication(pool.as_ref(), &work);

        let first = make_location(
            pool.as_ref(),
            publication.publication_id,
            LocationPlatform::PublisherWebsite,
            true,
            Some(format!("https://example.com/{}", Uuid::new_v4())),
        );
        let second = make_location(
            pool.as_ref(),
            publication.publication_id,
            LocationPlatform::Other,
            false,
            Some(format!("https://example.com/{}", Uuid::new_v4())),
        );
        let mut ids = [first.location_id, second.location_id];
        ids.sort();

        let asc = Location::all(
            pool.as_ref(),
            2,
            0,
            None,
            LocationOrderBy {
                field: LocationField::LocationId,
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
        .expect("Failed to order locations (asc)");

        let desc = Location::all(
            pool.as_ref(),
            2,
            0,
            None,
            LocationOrderBy {
                field: LocationField::LocationId,
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
        .expect("Failed to order locations (desc)");

        assert_eq!(asc[0].location_id, ids[0]);
        assert_eq!(desc[0].location_id, ids[1]);
    }

    #[test]
    fn crud_ordering_by_fields_is_supported() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let publication = create_publication(pool.as_ref(), &work);

        make_location(
            pool.as_ref(),
            publication.publication_id,
            LocationPlatform::PublisherWebsite,
            true,
            Some(format!("https://example.com/{}", Uuid::new_v4())),
        );
        make_location(
            pool.as_ref(),
            publication.publication_id,
            LocationPlatform::Other,
            false,
            Some(format!("https://example.com/{}", Uuid::new_v4())),
        );

        let fields: Vec<fn() -> LocationField> = vec![
            || LocationField::LocationId,
            || LocationField::PublicationId,
            || LocationField::LandingPage,
            || LocationField::FullTextUrl,
            || LocationField::LocationPlatform,
            || LocationField::Canonical,
            || LocationField::CreatedAt,
            || LocationField::UpdatedAt,
        ];

        for field in fields {
            for direction in [Direction::Asc, Direction::Desc] {
                let results = Location::all(
                    pool.as_ref(),
                    10,
                    0,
                    None,
                    LocationOrderBy {
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
                .expect("Failed to order locations");

                assert_eq!(results.len(), 2);
            }
        }
    }
}
