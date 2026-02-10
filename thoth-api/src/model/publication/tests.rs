use super::*;

mod defaults {
    use super::*;

    #[test]
    fn publicationtype_default_is_paperback() {
        let pubtype: PublicationType = Default::default();
        assert_eq!(pubtype, PublicationType::Paperback);
    }

    #[test]
    fn publicationfield_default_is_publication_type() {
        let pubfield: PublicationField = Default::default();
        assert_eq!(pubfield, PublicationField::PublicationType);
    }
}

mod display_and_parse {
    use super::*;

    #[test]
    fn publicationtype_display_formats_expected_strings() {
        assert_eq!(format!("{}", PublicationType::Paperback), "Paperback");
        assert_eq!(format!("{}", PublicationType::Hardback), "Hardback");
        assert_eq!(format!("{}", PublicationType::Pdf), "PDF");
        assert_eq!(format!("{}", PublicationType::Html), "HTML");
        assert_eq!(format!("{}", PublicationType::Xml), "XML");
        assert_eq!(format!("{}", PublicationType::Epub), "Epub");
        assert_eq!(format!("{}", PublicationType::Mobi), "Mobi");
        assert_eq!(format!("{}", PublicationType::Azw3), "AZW3");
        assert_eq!(format!("{}", PublicationType::Docx), "DOCX");
        assert_eq!(format!("{}", PublicationType::FictionBook), "FictionBook");
        assert_eq!(format!("{}", PublicationType::Mp3), "MP3");
        assert_eq!(format!("{}", PublicationType::Wav), "WAV");
    }

    #[test]
    fn publicationfield_display_formats_expected_strings() {
        assert_eq!(format!("{}", PublicationField::PublicationId), "ID");
        assert_eq!(format!("{}", PublicationField::PublicationType), "Type");
        assert_eq!(format!("{}", PublicationField::WorkId), "WorkID");
        assert_eq!(format!("{}", PublicationField::Isbn), "ISBN");
        assert_eq!(format!("{}", PublicationField::CreatedAt), "CreatedAt");
        assert_eq!(format!("{}", PublicationField::UpdatedAt), "UpdatedAt");
        assert_eq!(format!("{}", PublicationField::WidthMm), "WidthMm");
        assert_eq!(format!("{}", PublicationField::WidthIn), "WidthIn");
        assert_eq!(format!("{}", PublicationField::HeightMm), "HeightMm");
        assert_eq!(format!("{}", PublicationField::HeightIn), "HeightIn");
        assert_eq!(format!("{}", PublicationField::DepthMm), "DepthMm");
        assert_eq!(format!("{}", PublicationField::DepthIn), "DepthIn");
        assert_eq!(format!("{}", PublicationField::WeightG), "WeightG");
        assert_eq!(format!("{}", PublicationField::WeightOz), "WeightOz");
    }

    #[test]
    fn publicationtype_fromstr_parses_expected_values() {
        use std::str::FromStr;
        for (input, expected) in [
            ("Paperback", PublicationType::Paperback),
            ("Hardback", PublicationType::Hardback),
            ("PDF", PublicationType::Pdf),
            ("HTML", PublicationType::Html),
            ("XML", PublicationType::Xml),
            ("Epub", PublicationType::Epub),
            ("Mobi", PublicationType::Mobi),
            ("AZW3", PublicationType::Azw3),
            ("DOCX", PublicationType::Docx),
            ("FictionBook", PublicationType::FictionBook),
            ("MP3", PublicationType::Mp3),
            ("WAV", PublicationType::Wav),
        ]
        .iter()
        {
            assert_eq!(PublicationType::from_str(input).unwrap(), *expected);
        }

        assert!(PublicationType::from_str("PNG").is_err());
        assert!(PublicationType::from_str("Latex").is_err());
        assert!(PublicationType::from_str("azw3").is_err());
        assert!(PublicationType::from_str("Fiction Book").is_err());
    }

    #[test]
    fn publicationfield_fromstr_parses_expected_values() {
        use std::str::FromStr;
        for (input, expected) in [
            ("ID", PublicationField::PublicationId),
            ("Type", PublicationField::PublicationType),
            ("WorkID", PublicationField::WorkId),
            ("ISBN", PublicationField::Isbn),
            ("CreatedAt", PublicationField::CreatedAt),
            ("UpdatedAt", PublicationField::UpdatedAt),
            ("WidthMm", PublicationField::WidthMm),
            ("WidthIn", PublicationField::WidthIn),
            ("HeightMm", PublicationField::HeightMm),
            ("HeightIn", PublicationField::HeightIn),
            ("DepthMm", PublicationField::DepthMm),
            ("DepthIn", PublicationField::DepthIn),
            ("WeightG", PublicationField::WeightG),
            ("WeightOz", PublicationField::WeightOz),
        ]
        .iter()
        {
            assert_eq!(PublicationField::from_str(input).unwrap(), *expected);
        }

        assert!(PublicationField::from_str("PublicationID").is_err());
        assert!(PublicationField::from_str("Work Title").is_err());
        assert!(PublicationField::from_str("Work DOI").is_err());
    }
}

#[cfg(feature = "backend")]
mod conversions {
    use super::*;
    use crate::model::tests::db::setup_test_db;
    use crate::model::tests::{assert_db_enum_roundtrip, assert_graphql_enum_roundtrip};

    #[test]
    fn publicationtype_graphql_roundtrip() {
        assert_graphql_enum_roundtrip(PublicationType::Paperback);
    }

    #[test]
    fn accessibilitystandard_graphql_roundtrip() {
        assert_graphql_enum_roundtrip(AccessibilityStandard::EpubA11y11aa);
    }

    #[test]
    fn accessibilityexception_graphql_roundtrip() {
        assert_graphql_enum_roundtrip(AccessibilityException::MicroEnterprises);
    }

    #[test]
    fn publicationtype_db_enum_roundtrip() {
        let (_guard, pool) = setup_test_db();

        assert_db_enum_roundtrip::<PublicationType, crate::schema::sql_types::PublicationType>(
            pool.as_ref(),
            "'Paperback'::publication_type",
            PublicationType::Paperback,
        );
    }

    #[test]
    fn accessibilitystandard_db_enum_roundtrip() {
        let (_guard, pool) = setup_test_db();

        assert_db_enum_roundtrip::<
            AccessibilityStandard,
            crate::schema::sql_types::AccessibilityStandard,
        >(
            pool.as_ref(),
            "'epub-a11y-11-aa'::accessibility_standard",
            AccessibilityStandard::EpubA11y11aa,
        );
    }

    #[test]
    fn accessibilityexception_db_enum_roundtrip() {
        let (_guard, pool) = setup_test_db();

        assert_db_enum_roundtrip::<
            AccessibilityException,
            crate::schema::sql_types::AccessibilityException,
        >(
            pool.as_ref(),
            "'micro-enterprises'::accessibility_exception",
            AccessibilityException::MicroEnterprises,
        );
    }
}

mod helpers {
    use super::*;
    use crate::model::{Crud, HistoryEntry};

    #[test]
    fn is_physical_returns_true_for_print_types() {
        let mut publication: Publication = Default::default();
        for pub_type in [PublicationType::Paperback, PublicationType::Hardback] {
            publication.publication_type = pub_type;
            assert!(publication.is_physical());
            assert!(!publication.is_digital());
        }
        for pub_type in [
            PublicationType::Azw3,
            PublicationType::Docx,
            PublicationType::Epub,
            PublicationType::FictionBook,
            PublicationType::Html,
            PublicationType::Mobi,
            PublicationType::Mp3,
            PublicationType::Pdf,
            PublicationType::Xml,
            PublicationType::Wav,
        ] {
            publication.publication_type = pub_type;
            assert!(!publication.is_physical());
            assert!(publication.is_digital());
        }
    }

    #[test]
    fn pk_returns_id() {
        let publication: Publication = Default::default();
        assert_eq!(publication.pk(), publication.publication_id);
    }

    #[test]
    fn history_entry_serializes_model() {
        let publication: Publication = Default::default();
        let user_id = "123456".to_string();
        let new_publication_history = publication.new_history_entry(&user_id);
        assert_eq!(
            new_publication_history.publication_id,
            publication.publication_id
        );
        assert_eq!(new_publication_history.user_id, user_id);
        assert_eq!(
            new_publication_history.data,
            serde_json::Value::String(serde_json::to_string(&publication).unwrap())
        );
    }
}

mod validation {
    use super::*;

    #[test]
    fn validate_dimensions_enforces_width_constraints() {
        let mut publication: Publication = Publication {
            publication_type: PublicationType::Pdf,
            width_mm: Some(100.0),
            ..Default::default()
        };
        assert_eq!(
            publication.validate_dimensions_constraints(),
            Err(ThothError::DimensionDigitalError)
        );
        publication.width_mm = None;
        assert!(publication.validate_dimensions_constraints().is_ok());
        publication.width_in = Some(39.4);
        assert_eq!(
            publication.validate_dimensions_constraints(),
            Err(ThothError::DimensionDigitalError)
        );
        publication.publication_type = PublicationType::Paperback;
        assert_eq!(
            publication.validate_dimensions_constraints(),
            Err(ThothError::WidthEmptyError)
        );
        publication.width_in = None;
        assert!(publication.validate_dimensions_constraints().is_ok());
        publication.width_mm = Some(100.0);
        assert_eq!(
            publication.validate_dimensions_constraints(),
            Err(ThothError::WidthEmptyError)
        );
        publication.width_in = Some(39.4);
        assert!(publication.validate_dimensions_constraints().is_ok());
    }

    #[test]
    fn validate_dimensions_enforces_height_constraints() {
        let mut publication: Publication = Publication {
            publication_type: PublicationType::Pdf,
            height_mm: Some(100.0),
            ..Default::default()
        };
        assert_eq!(
            publication.validate_dimensions_constraints(),
            Err(ThothError::DimensionDigitalError)
        );
        publication.height_mm = None;
        assert!(publication.validate_dimensions_constraints().is_ok());
        publication.height_in = Some(39.4);
        assert_eq!(
            publication.validate_dimensions_constraints(),
            Err(ThothError::DimensionDigitalError)
        );
        publication.publication_type = PublicationType::Paperback;
        assert_eq!(
            publication.validate_dimensions_constraints(),
            Err(ThothError::HeightEmptyError)
        );
        publication.height_in = None;
        assert!(publication.validate_dimensions_constraints().is_ok());
        publication.height_mm = Some(100.0);
        assert_eq!(
            publication.validate_dimensions_constraints(),
            Err(ThothError::HeightEmptyError)
        );
        publication.height_in = Some(39.4);
        assert!(publication.validate_dimensions_constraints().is_ok());
    }

    #[test]
    fn validate_dimensions_enforces_depth_constraints() {
        let mut publication: Publication = Publication {
            publication_type: PublicationType::Pdf,
            depth_mm: Some(10.0),
            ..Default::default()
        };
        assert_eq!(
            publication.validate_dimensions_constraints(),
            Err(ThothError::DimensionDigitalError)
        );
        publication.depth_mm = None;
        assert!(publication.validate_dimensions_constraints().is_ok());
        publication.depth_in = Some(3.94);
        assert_eq!(
            publication.validate_dimensions_constraints(),
            Err(ThothError::DimensionDigitalError)
        );
        publication.publication_type = PublicationType::Paperback;
        assert_eq!(
            publication.validate_dimensions_constraints(),
            Err(ThothError::DepthEmptyError)
        );
        publication.depth_in = None;
        assert!(publication.validate_dimensions_constraints().is_ok());
        publication.depth_mm = Some(10.0);
        assert_eq!(
            publication.validate_dimensions_constraints(),
            Err(ThothError::DepthEmptyError)
        );
        publication.depth_in = Some(3.94);
        assert!(publication.validate_dimensions_constraints().is_ok());
    }

    #[test]
    fn validate_dimensions_enforces_weight_constraints() {
        let mut publication: Publication = Publication {
            publication_type: PublicationType::Pdf,
            weight_g: Some(100.0),
            ..Default::default()
        };
        assert_eq!(
            publication.validate_dimensions_constraints(),
            Err(ThothError::DimensionDigitalError)
        );
        publication.weight_g = None;
        assert!(publication.validate_dimensions_constraints().is_ok());
        publication.weight_oz = Some(3.5);
        assert_eq!(
            publication.validate_dimensions_constraints(),
            Err(ThothError::DimensionDigitalError)
        );
        publication.publication_type = PublicationType::Paperback;
        assert_eq!(
            publication.validate_dimensions_constraints(),
            Err(ThothError::WeightEmptyError)
        );
        publication.weight_oz = None;
        assert!(publication.validate_dimensions_constraints().is_ok());
        publication.weight_g = Some(100.0);
        assert_eq!(
            publication.validate_dimensions_constraints(),
            Err(ThothError::WeightEmptyError)
        );
        publication.weight_oz = Some(3.5);
        assert!(publication.validate_dimensions_constraints().is_ok());
    }
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;

    use crate::model::publication::policy::PublicationPolicy;
    use crate::model::tests::db::{
        create_imprint, create_publication, create_publisher, create_work, setup_test_db,
        test_context_with_user, test_user_with_role,
    };
    use crate::model::Crud;
    use crate::policy::{CreatePolicy, DeletePolicy, Role, UpdatePolicy};

    #[test]
    fn crud_policy_allows_publisher_user_for_write() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("publication-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let new_publication = NewPublication {
            publication_type: PublicationType::Paperback,
            work_id: work.work_id,
            isbn: None,
            width_mm: None,
            width_in: None,
            height_mm: None,
            height_in: None,
            depth_mm: None,
            depth_in: None,
            weight_g: None,
            weight_oz: None,
            accessibility_standard: None,
            accessibility_additional_standard: None,
            accessibility_exception: None,
            accessibility_report_url: None,
        };

        let publication =
            Publication::create(pool.as_ref(), &new_publication).expect("Failed to create");
        let patch = PatchPublication {
            publication_id: publication.publication_id,
            publication_type: publication.publication_type,
            work_id: publication.work_id,
            isbn: publication.isbn.clone(),
            width_mm: publication.width_mm,
            width_in: publication.width_in,
            height_mm: publication.height_mm,
            height_in: publication.height_in,
            depth_mm: publication.depth_mm,
            depth_in: publication.depth_in,
            weight_g: publication.weight_g,
            weight_oz: publication.weight_oz,
            accessibility_standard: publication.accessibility_standard,
            accessibility_additional_standard: publication.accessibility_additional_standard,
            accessibility_exception: publication.accessibility_exception,
            accessibility_report_url: publication.accessibility_report_url.clone(),
        };

        assert!(PublicationPolicy::can_create(&ctx, &new_publication, ()).is_ok());
        assert!(PublicationPolicy::can_update(&ctx, &publication, &patch, ()).is_ok());
        assert!(PublicationPolicy::can_delete(&ctx, &publication).is_ok());
    }

    #[test]
    fn crud_policy_rejects_user_without_publisher_role() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let publication = create_publication(pool.as_ref(), &work);
        let patch = PatchPublication {
            publication_id: publication.publication_id,
            publication_type: publication.publication_type,
            work_id: publication.work_id,
            isbn: publication.isbn.clone(),
            width_mm: publication.width_mm,
            width_in: publication.width_in,
            height_mm: publication.height_mm,
            height_in: publication.height_in,
            depth_mm: publication.depth_mm,
            depth_in: publication.depth_in,
            weight_g: publication.weight_g,
            weight_oz: publication.weight_oz,
            accessibility_standard: publication.accessibility_standard,
            accessibility_additional_standard: publication.accessibility_additional_standard,
            accessibility_exception: publication.accessibility_exception,
            accessibility_report_url: publication.accessibility_report_url.clone(),
        };

        let user = test_user_with_role("publication-user", Role::PublisherUser, "org-other");
        let ctx = test_context_with_user(pool.clone(), user);

        let new_publication = NewPublication {
            publication_type: PublicationType::Paperback,
            work_id: work.work_id,
            isbn: None,
            width_mm: None,
            width_in: None,
            height_mm: None,
            height_in: None,
            depth_mm: None,
            depth_in: None,
            weight_g: None,
            weight_oz: None,
            accessibility_standard: None,
            accessibility_additional_standard: None,
            accessibility_exception: None,
            accessibility_report_url: None,
        };

        assert!(PublicationPolicy::can_create(&ctx, &new_publication, ()).is_err());
        assert!(PublicationPolicy::can_update(&ctx, &publication, &patch, ()).is_err());
        assert!(PublicationPolicy::can_delete(&ctx, &publication).is_err());
    }
}

#[cfg(feature = "backend")]
mod crud {
    use super::*;
    use std::str::FromStr;

    use crate::model::tests::db::{
        create_imprint, create_publication, create_publisher, create_work, setup_test_db,
        test_context,
    };
    use crate::model::work::{NewWork, Work, WorkStatus, WorkType};
    use crate::model::Crud;

    fn make_publication(
        pool: &crate::db::PgPool,
        work_id: Uuid,
        publication_type: PublicationType,
        isbn: Option<Isbn>,
    ) -> Publication {
        let new_publication = NewPublication {
            publication_type,
            work_id,
            isbn,
            width_mm: None,
            width_in: None,
            height_mm: None,
            height_in: None,
            depth_mm: None,
            depth_in: None,
            weight_g: None,
            weight_oz: None,
            accessibility_standard: None,
            accessibility_additional_standard: None,
            accessibility_exception: None,
            accessibility_report_url: None,
        };

        Publication::create(pool, &new_publication).expect("Failed to create publication")
    }

    fn make_work_with_type(
        pool: &crate::db::PgPool,
        imprint_id: Uuid,
        work_type: WorkType,
    ) -> Work {
        let new_work = NewWork {
            work_type,
            work_status: WorkStatus::Forthcoming,
            reference: None,
            edition: if work_type == WorkType::BookChapter {
                None
            } else {
                Some(1)
            },
            imprint_id,
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
            cover_url: None,
            cover_caption: None,
            first_page: None,
            last_page: None,
            page_interval: None,
        };

        Work::create(pool, &new_work).expect("Failed to create work")
    }

    #[test]
    fn crud_roundtrip_create_fetch_update_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let publication = create_publication(pool.as_ref(), &work);
        let fetched_publication = Publication::from_id(pool.as_ref(), &publication.publication_id)
            .expect("Failed to fetch publication");
        assert_eq!(
            publication.publication_id,
            fetched_publication.publication_id
        );

        let patch = PatchPublication {
            publication_id: publication.publication_id,
            publication_type: publication.publication_type,
            work_id: publication.work_id,
            isbn: publication.isbn.clone(),
            width_mm: Some(123.0),
            width_in: Some(4.84),
            height_mm: publication.height_mm,
            height_in: publication.height_in,
            depth_mm: publication.depth_mm,
            depth_in: publication.depth_in,
            weight_g: publication.weight_g,
            weight_oz: publication.weight_oz,
            accessibility_standard: publication.accessibility_standard,
            accessibility_additional_standard: publication.accessibility_additional_standard,
            accessibility_exception: publication.accessibility_exception,
            accessibility_report_url: publication.accessibility_report_url.clone(),
        };

        let ctx = test_context(pool.clone(), "test-user");
        let updated = publication
            .update(&ctx, &patch)
            .expect("Failed to update publication");
        assert_eq!(updated.width_mm, patch.width_mm);

        let deleted = updated
            .delete(pool.as_ref())
            .expect("Failed to delete publication");
        assert!(Publication::from_id(pool.as_ref(), &deleted.publication_id).is_err());
    }

    #[test]
    fn crud_validate_rejects_chapter_with_isbn() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = make_work_with_type(pool.as_ref(), imprint.imprint_id, WorkType::BookChapter);

        let publication = Publication::create(
            pool.as_ref(),
            &NewPublication {
                publication_type: PublicationType::Pdf,
                work_id: work.work_id,
                isbn: Some(Isbn::from_str("978-0-306-40615-7").unwrap()),
                width_mm: None,
                width_in: None,
                height_mm: None,
                height_in: None,
                depth_mm: None,
                depth_in: None,
                weight_g: None,
                weight_oz: None,
                accessibility_standard: None,
                accessibility_additional_standard: None,
                accessibility_exception: None,
                accessibility_report_url: None,
            },
        )
        .expect("Failed to create publication");

        let result = publication.validate(pool.as_ref());
        assert!(matches!(result, Err(ThothError::ChapterIsbnError)));
    }

    #[test]
    fn crud_validate_rejects_chapter_with_dimensions() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = make_work_with_type(pool.as_ref(), imprint.imprint_id, WorkType::BookChapter);

        let publication = Publication {
            publication_id: Uuid::new_v4(),
            publication_type: PublicationType::Pdf,
            work_id: work.work_id,
            isbn: None,
            created_at: Default::default(),
            updated_at: Default::default(),
            width_mm: Some(100.0),
            width_in: None,
            height_mm: None,
            height_in: None,
            depth_mm: None,
            depth_in: None,
            weight_g: None,
            weight_oz: None,
            accessibility_standard: None,
            accessibility_additional_standard: None,
            accessibility_exception: None,
            accessibility_report_url: None,
        };

        let result = publication.validate(pool.as_ref());
        assert!(matches!(result, Err(ThothError::ChapterDimensionError)));
    }

    #[test]
    fn crud_validate_allows_chapter_without_isbn_or_dimensions() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = make_work_with_type(pool.as_ref(), imprint.imprint_id, WorkType::BookChapter);

        let publication = Publication::create(
            pool.as_ref(),
            &NewPublication {
                publication_type: PublicationType::Pdf,
                work_id: work.work_id,
                isbn: None,
                width_mm: None,
                width_in: None,
                height_mm: None,
                height_in: None,
                depth_mm: None,
                depth_in: None,
                weight_g: None,
                weight_oz: None,
                accessibility_standard: None,
                accessibility_additional_standard: None,
                accessibility_exception: None,
                accessibility_report_url: None,
            },
        )
        .expect("Failed to create publication");

        assert!(publication.validate(pool.as_ref()).is_ok());
    }

    #[test]
    fn crud_all_respects_limit_and_offset() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        make_publication(
            pool.as_ref(),
            work.work_id,
            PublicationType::Paperback,
            None,
        );
        make_publication(pool.as_ref(), work.work_id, PublicationType::Pdf, None);

        let order = PublicationOrderBy {
            field: PublicationField::PublicationId,
            direction: Direction::Asc,
        };

        let first = Publication::all(
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
        .expect("Failed to fetch publications");
        let second = Publication::all(
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
        .expect("Failed to fetch publications");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_ne!(first[0].publication_id, second[0].publication_id);
    }

    #[test]
    fn crud_count_returns_total() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        make_publication(
            pool.as_ref(),
            work.work_id,
            PublicationType::Paperback,
            None,
        );
        make_publication(pool.as_ref(), work.work_id, PublicationType::Pdf, None);

        let count = Publication::count(pool.as_ref(), None, vec![], vec![], vec![], None, None)
            .expect("Failed to count publications");
        assert_eq!(count, 2);
    }

    #[test]
    fn crud_count_filters_by_publication_type() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        make_publication(
            pool.as_ref(),
            work.work_id,
            PublicationType::Paperback,
            None,
        );
        make_publication(pool.as_ref(), work.work_id, PublicationType::Pdf, None);

        let count = Publication::count(
            pool.as_ref(),
            None,
            vec![],
            vec![PublicationType::Paperback],
            vec![],
            None,
            None,
        )
        .expect("Failed to count publications by type");
        assert_eq!(count, 1);
    }

    #[test]
    fn crud_count_filters_by_publishers() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        make_publication(
            pool.as_ref(),
            work.work_id,
            PublicationType::Paperback,
            None,
        );

        let other_publisher = create_publisher(pool.as_ref());
        let other_imprint = create_imprint(pool.as_ref(), &other_publisher);
        let other_work = create_work(pool.as_ref(), &other_imprint);
        make_publication(
            pool.as_ref(),
            other_work.work_id,
            PublicationType::Pdf,
            None,
        );

        let count = Publication::count(
            pool.as_ref(),
            None,
            vec![publisher.publisher_id],
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to count publications by publisher");

        assert_eq!(count, 1);
    }

    #[test]
    fn crud_count_filters_by_isbn_substring() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        make_publication(
            pool.as_ref(),
            work.work_id,
            PublicationType::Paperback,
            Some(Isbn::from_str("978-0-306-40615-7").unwrap()),
        );
        make_publication(
            pool.as_ref(),
            work.work_id,
            PublicationType::Pdf,
            Some(Isbn::from_str("978-1-4028-9462-6").unwrap()),
        );

        let count = Publication::count(
            pool.as_ref(),
            Some("306-40615".to_string()),
            vec![],
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to count publications by ISBN filter");

        assert_eq!(count, 1);
    }

    #[test]
    fn crud_filter_matches_isbn() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let marker = "978-0-306-40615-7";
        let matches = make_publication(
            pool.as_ref(),
            work.work_id,
            PublicationType::Paperback,
            Some(Isbn::from_str(marker).unwrap()),
        );
        make_publication(
            pool.as_ref(),
            work.work_id,
            PublicationType::Pdf,
            Some(Isbn::from_str("978-1-4028-9462-6").unwrap()),
        );

        let filtered = Publication::all(
            pool.as_ref(),
            10,
            0,
            Some("306-40615".to_string()),
            PublicationOrderBy {
                field: PublicationField::PublicationId,
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
        .expect("Failed to filter publications");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].publication_id, matches.publication_id);
    }

    #[test]
    fn crud_filter_parent_work_id_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let other_work = create_work(pool.as_ref(), &imprint);

        let matches = make_publication(
            pool.as_ref(),
            work.work_id,
            PublicationType::Paperback,
            None,
        );
        make_publication(
            pool.as_ref(),
            other_work.work_id,
            PublicationType::Pdf,
            None,
        );

        let filtered = Publication::all(
            pool.as_ref(),
            10,
            0,
            None,
            PublicationOrderBy {
                field: PublicationField::PublicationId,
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
        .expect("Failed to filter publications by work");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].publication_id, matches.publication_id);
    }

    #[test]
    fn crud_filter_publishers_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let matches = make_publication(
            pool.as_ref(),
            work.work_id,
            PublicationType::Paperback,
            None,
        );

        let other_publisher = create_publisher(pool.as_ref());
        let other_imprint = create_imprint(pool.as_ref(), &other_publisher);
        let other_work = create_work(pool.as_ref(), &other_imprint);
        make_publication(
            pool.as_ref(),
            other_work.work_id,
            PublicationType::Pdf,
            None,
        );

        let filtered = Publication::all(
            pool.as_ref(),
            10,
            0,
            None,
            PublicationOrderBy {
                field: PublicationField::PublicationId,
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
        .expect("Failed to filter publications by publisher");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].publication_id, matches.publication_id);
    }

    #[test]
    fn crud_filter_param_limits_publication_types() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let matches = make_publication(
            pool.as_ref(),
            work.work_id,
            PublicationType::Paperback,
            None,
        );
        make_publication(pool.as_ref(), work.work_id, PublicationType::Pdf, None);

        let filtered = Publication::all(
            pool.as_ref(),
            10,
            0,
            None,
            PublicationOrderBy {
                field: PublicationField::PublicationId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![PublicationType::Paperback],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter publications by type");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].publication_id, matches.publication_id);
    }

    #[test]
    fn crud_ordering_by_id_respects_direction() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let first = make_publication(
            pool.as_ref(),
            work.work_id,
            PublicationType::Paperback,
            None,
        );
        let second = make_publication(pool.as_ref(), work.work_id, PublicationType::Pdf, None);
        let mut ids = [first.publication_id, second.publication_id];
        ids.sort();

        let asc = Publication::all(
            pool.as_ref(),
            2,
            0,
            None,
            PublicationOrderBy {
                field: PublicationField::PublicationId,
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
        .expect("Failed to order publications (asc)");

        let desc = Publication::all(
            pool.as_ref(),
            2,
            0,
            None,
            PublicationOrderBy {
                field: PublicationField::PublicationId,
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
        .expect("Failed to order publications (desc)");

        assert_eq!(asc[0].publication_id, ids[0]);
        assert_eq!(desc[0].publication_id, ids[1]);
    }

    #[test]
    fn crud_ordering_by_fields_is_supported() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        make_publication(
            pool.as_ref(),
            work.work_id,
            PublicationType::Paperback,
            None,
        );
        make_publication(pool.as_ref(), work.work_id, PublicationType::Pdf, None);

        let fields: Vec<fn() -> PublicationField> = vec![
            || PublicationField::PublicationId,
            || PublicationField::PublicationType,
            || PublicationField::WorkId,
            || PublicationField::Isbn,
            || PublicationField::CreatedAt,
            || PublicationField::UpdatedAt,
            || PublicationField::WidthMm,
            || PublicationField::WidthIn,
            || PublicationField::HeightMm,
            || PublicationField::HeightIn,
            || PublicationField::DepthMm,
            || PublicationField::DepthIn,
            || PublicationField::WeightG,
            || PublicationField::WeightOz,
            || PublicationField::AccessibilityStandard,
            || PublicationField::AccessibilityAdditionalStandard,
            || PublicationField::AccessibilityException,
            || PublicationField::AccessibilityReportUrl,
        ];

        for field in fields {
            for direction in [Direction::Asc, Direction::Desc] {
                let results = Publication::all(
                    pool.as_ref(),
                    10,
                    0,
                    None,
                    PublicationOrderBy {
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
                .expect("Failed to order publications");

                assert_eq!(results.len(), 2);
            }
        }
    }
}
