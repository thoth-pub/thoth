use super::*;
use uuid::Uuid;

const TEST_SHA256_HEX: &str = "444b138b41e3c48ca505b1740091b0c93ce9a71c7c9d24956e6cf8716f1aad7e";

#[cfg(feature = "backend")]
fn make_new_frontcover_file(work_id: Uuid, object_key: impl Into<String>) -> NewFile {
    let object_key = object_key.into();
    NewFile {
        file_type: FileType::Frontcover,
        work_id: Some(work_id),
        publication_id: None,
        object_key: object_key.clone(),
        cdn_url: format!("https://cdn.example.org/{object_key}"),
        mime_type: "image/jpeg".to_string(),
        bytes: 1024,
        sha256: TEST_SHA256_HEX.to_string(),
    }
}

#[cfg(feature = "backend")]
fn make_new_publication_file(publication_id: Uuid, object_key: impl Into<String>) -> NewFile {
    let object_key = object_key.into();
    NewFile {
        file_type: FileType::Publication,
        work_id: None,
        publication_id: Some(publication_id),
        object_key: object_key.clone(),
        cdn_url: format!("https://cdn.example.org/{object_key}"),
        mime_type: "application/pdf".to_string(),
        bytes: 2048,
        sha256: TEST_SHA256_HEX.to_string(),
    }
}

#[cfg(feature = "backend")]
fn make_new_frontcover_upload(work_id: Uuid, extension: impl Into<String>) -> NewFileUpload {
    NewFileUpload {
        file_type: FileType::Frontcover,
        work_id: Some(work_id),
        publication_id: None,
        declared_mime_type: "image/jpeg".to_string(),
        declared_extension: extension.into(),
        declared_sha256: TEST_SHA256_HEX.to_string(),
    }
}

#[cfg(feature = "backend")]
fn make_new_publication_upload(
    publication_id: Uuid,
    extension: impl Into<String>,
) -> NewFileUpload {
    NewFileUpload {
        file_type: FileType::Publication,
        work_id: None,
        publication_id: Some(publication_id),
        declared_mime_type: "application/pdf".to_string(),
        declared_extension: extension.into(),
        declared_sha256: TEST_SHA256_HEX.to_string(),
    }
}

#[cfg(feature = "backend")]
fn create_pdf_publication(
    pool: &crate::db::PgPool,
    work_id: Uuid,
) -> crate::model::publication::Publication {
    use crate::model::publication::{NewPublication, Publication, PublicationType};
    use crate::model::Crud;

    let new_publication = NewPublication {
        publication_type: PublicationType::Pdf,
        work_id,
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

    Publication::create(pool, &new_publication).expect("Failed to create PDF publication")
}

mod display_and_parse {
    use super::*;

    #[test]
    fn filetype_display_formats_expected_strings() {
        assert_eq!(format!("{}", FileType::Publication), "publication");
        assert_eq!(format!("{}", FileType::Frontcover), "frontcover");
    }

    #[test]
    fn filetype_fromstr_parses_expected_values() {
        use std::str::FromStr;

        assert_eq!(
            FileType::from_str("publication").unwrap(),
            FileType::Publication
        );
        assert_eq!(
            FileType::from_str("frontcover").unwrap(),
            FileType::Frontcover
        );
        assert!(FileType::from_str("Publication").is_err());
        assert!(FileType::from_str("cover").is_err());
    }
}

#[cfg(feature = "backend")]
mod conversions {
    use super::*;
    use crate::model::tests::db::setup_test_db;
    use crate::model::tests::{assert_db_enum_roundtrip, assert_graphql_enum_roundtrip};

    #[test]
    fn filetype_graphql_roundtrip() {
        assert_graphql_enum_roundtrip(FileType::Publication);
        assert_graphql_enum_roundtrip(FileType::Frontcover);
    }

    #[test]
    fn filetype_db_enum_roundtrip() {
        let (_guard, pool) = setup_test_db();

        assert_db_enum_roundtrip::<FileType, crate::schema::sql_types::FileType>(
            pool.as_ref(),
            "'publication'::file_type",
            FileType::Publication,
        );
        assert_db_enum_roundtrip::<FileType, crate::schema::sql_types::FileType>(
            pool.as_ref(),
            "'frontcover'::file_type",
            FileType::Frontcover,
        );
    }
}

mod helpers {
    use super::*;
    use crate::model::{Crud, Timestamp};

    #[test]
    fn pk_returns_file_id() {
        let file_id = Uuid::new_v4();
        let file = File {
            file_id,
            file_type: FileType::Frontcover,
            work_id: None,
            publication_id: None,
            object_key: "test/key".to_string(),
            cdn_url: "https://cdn.example.com/test.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            bytes: 1024,
            sha256: TEST_SHA256_HEX.to_string(),
            created_at: Timestamp::default(),
            updated_at: Timestamp::default(),
        };

        assert_eq!(file.pk(), file_id);
    }

    #[test]
    fn pk_returns_file_upload_id() {
        let file_upload_id = Uuid::new_v4();
        let upload = FileUpload {
            file_upload_id,
            file_type: FileType::Frontcover,
            work_id: None,
            publication_id: None,
            declared_mime_type: "image/jpeg".to_string(),
            declared_extension: "jpg".to_string(),
            declared_sha256: TEST_SHA256_HEX.to_string(),
            created_at: Timestamp::default(),
            updated_at: Timestamp::default(),
        };

        assert_eq!(upload.pk(), file_upload_id);
    }
}

#[cfg(feature = "backend")]
mod validation {
    use super::*;
    use crate::model::publication::PublicationType;
    use thoth_errors::ThothError;

    #[test]
    fn frontcover_allows_known_extensions() {
        for ext in ["jpg", "jpeg", "png", "webp"] {
            assert!(FilePolicy::validate_file_extension(ext, &FileType::Frontcover, None).is_ok());
        }
    }

    #[test]
    fn frontcover_extension_validation_is_case_insensitive() {
        assert!(FilePolicy::validate_file_extension("JPG", &FileType::Frontcover, None).is_ok());
        assert!(FilePolicy::validate_file_extension("WeBp", &FileType::Frontcover, None).is_ok());
    }

    #[test]
    fn frontcover_rejects_unknown_extensions() {
        assert_eq!(
            FilePolicy::validate_file_extension("tiff", &FileType::Frontcover, None).unwrap_err(),
            ThothError::InvalidFileExtension
        );
    }

    #[test]
    fn publication_pdf_allows_pdf() {
        assert!(FilePolicy::validate_file_extension(
            "pdf",
            &FileType::Publication,
            Some(PublicationType::Pdf)
        )
        .is_ok());
    }

    #[test]
    fn publication_pdf_rejects_other_extensions() {
        assert_eq!(
            FilePolicy::validate_file_extension(
                "epub",
                &FileType::Publication,
                Some(PublicationType::Pdf)
            )
            .unwrap_err(),
            ThothError::InvalidFileExtension
        );
    }

    #[test]
    fn publication_requires_publication_type_for_validation() {
        assert_eq!(
            FilePolicy::validate_file_extension("pdf", &FileType::Publication, None).unwrap_err(),
            ThothError::PublicationTypeRequiredForFileValidation
        );
    }

    #[test]
    fn publication_rejects_unsupported_publication_types() {
        assert_eq!(
            FilePolicy::validate_file_extension(
                "pdf",
                &FileType::Publication,
                Some(PublicationType::Paperback)
            )
            .unwrap_err(),
            ThothError::UnsupportedPublicationTypeForFileUpload
        );
    }

    #[test]
    fn new_file_upload_from_publication_lowercases_extension() {
        let data = NewPublicationFileUpload {
            publication_id: Uuid::new_v4(),
            declared_mime_type: "application/pdf".to_string(),
            declared_extension: "PDF".to_string(),
            declared_sha256: TEST_SHA256_HEX.to_string(),
        };

        let upload: NewFileUpload = data.into();
        assert_eq!(upload.file_type, FileType::Publication);
        assert_eq!(upload.declared_extension, "pdf");
    }

    #[test]
    fn new_file_upload_from_frontcover_lowercases_extension() {
        let data = NewFrontcoverFileUpload {
            work_id: Uuid::new_v4(),
            declared_mime_type: "image/jpeg".to_string(),
            declared_extension: "JPG".to_string(),
            declared_sha256: TEST_SHA256_HEX.to_string(),
        };

        let upload: NewFileUpload = data.into();
        assert_eq!(upload.file_type, FileType::Frontcover);
        assert_eq!(upload.declared_extension, "jpg");
    }

    #[test]
    fn upload_request_headers_contains_required_checksum_headers() {
        let headers = upload_request_headers("application/pdf", TEST_SHA256_HEX)
            .expect("Expected upload headers");

        assert_eq!(headers.len(), 3);
        assert_eq!(headers[0].name, "Content-Type");
        assert_eq!(headers[0].value, "application/pdf");
        assert_eq!(headers[1].name, "x-amz-checksum-sha256");
        assert_eq!(
            headers[1].value,
            "REsTi0HjxIylBbF0AJGwyTzppxx8nSSVbmz4cW8arX4="
        );
        assert_eq!(headers[2].name, "x-amz-sdk-checksum-algorithm");
        assert_eq!(headers[2].value, "SHA256");
    }
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;
    use crate::model::publication::PublicationType;
    use crate::model::tests::db::{
        create_imprint, create_publisher, create_work, setup_test_db, test_context_with_user,
        test_user_with_role,
    };
    use crate::model::Crud;
    use crate::policy::{CreatePolicy, DeletePolicy, Role};
    use thoth_errors::ThothError;

    #[test]
    fn crud_policy_allows_cdn_write_user_for_write() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let publication = create_pdf_publication(pool.as_ref(), work.work_id);

        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("file-user", Role::CdnWrite, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let new_file = make_new_frontcover_file(
            work.work_id,
            format!("10.1234/{}/cover.jpg", Uuid::new_v4()),
        );
        let new_upload = make_new_publication_upload(publication.publication_id, "pdf");

        let file = File::create(pool.as_ref(), &new_file).expect("Failed to create file");
        let upload =
            FileUpload::create(pool.as_ref(), &new_upload).expect("Failed to create file upload");

        assert!(FilePolicy::can_create(&ctx, &new_file, ()).is_ok());
        assert!(FilePolicy::can_create(&ctx, &new_upload, Some(PublicationType::Pdf)).is_ok());
        assert!(FilePolicy::can_delete(&ctx, &file).is_ok());
        assert!(FilePolicy::can_delete(&ctx, &upload).is_ok());
        assert!(FilePolicy::can_complete_upload(&ctx, &upload, Some(PublicationType::Pdf)).is_ok());
    }

    #[test]
    fn crud_policy_rejects_user_without_cdn_write_role() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let publication = create_pdf_publication(pool.as_ref(), work.work_id);

        let new_file = make_new_frontcover_file(
            work.work_id,
            format!("10.1234/{}/cover.jpg", Uuid::new_v4()),
        );
        let new_upload = make_new_publication_upload(publication.publication_id, "pdf");

        let file = File::create(pool.as_ref(), &new_file).expect("Failed to create file");
        let upload =
            FileUpload::create(pool.as_ref(), &new_upload).expect("Failed to create file upload");

        let user = test_user_with_role("file-user", Role::CdnWrite, "org-other");
        let ctx = test_context_with_user(pool.clone(), user);

        assert!(FilePolicy::can_create(&ctx, &new_file, ()).is_err());
        assert!(FilePolicy::can_create(&ctx, &new_upload, Some(PublicationType::Pdf)).is_err());
        assert!(FilePolicy::can_delete(&ctx, &file).is_err());
        assert!(FilePolicy::can_delete(&ctx, &upload).is_err());
        assert!(
            FilePolicy::can_complete_upload(&ctx, &upload, Some(PublicationType::Pdf)).is_err()
        );
    }

    #[test]
    fn can_complete_upload_validates_extension_and_publication_type() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let publication = create_pdf_publication(pool.as_ref(), work.work_id);

        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("file-user", Role::CdnWrite, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let valid_upload = FileUpload::create(
            pool.as_ref(),
            &make_new_publication_upload(publication.publication_id, "pdf"),
        )
        .expect("Failed to create valid upload");

        assert!(
            FilePolicy::can_complete_upload(&ctx, &valid_upload, Some(PublicationType::Pdf))
                .is_ok()
        );

        let other_work = create_work(pool.as_ref(), &imprint);
        let other_publication = create_pdf_publication(pool.as_ref(), other_work.work_id);

        let invalid_upload = FileUpload::create(
            pool.as_ref(),
            &make_new_publication_upload(other_publication.publication_id, "epub"),
        )
        .expect("Failed to create invalid upload");

        assert_eq!(
            FilePolicy::can_complete_upload(&ctx, &invalid_upload, Some(PublicationType::Pdf))
                .unwrap_err(),
            ThothError::InvalidFileExtension
        );
        assert_eq!(
            FilePolicy::can_complete_upload(&ctx, &valid_upload, None).unwrap_err(),
            ThothError::PublicationTypeRequiredForFileValidation
        );
    }
}

#[cfg(feature = "backend")]
mod crud {
    use super::*;
    use crate::model::tests::db::{
        create_imprint, create_publisher, create_work, setup_test_db, test_context,
        test_context_with_user, test_user_with_role,
    };
    use crate::model::work::Work;
    use crate::model::{Crud, Doi, PublisherId};
    use crate::policy::Role;
    use std::str::FromStr;
    use thoth_errors::ThothError;

    #[test]
    fn crud_roundtrip_file_create_fetch_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let new_file = make_new_frontcover_file(
            work.work_id,
            format!("10.1234/{}/cover.jpg", Uuid::new_v4()),
        );

        let file = File::create(pool.as_ref(), &new_file).expect("Failed to create file");
        let fetched = File::from_id(pool.as_ref(), &file.file_id).expect("Failed to fetch file");
        assert_eq!(fetched.file_id, file.file_id);

        let deleted = fetched
            .delete(pool.as_ref())
            .expect("Failed to delete file");
        assert!(File::from_id(pool.as_ref(), &deleted.file_id).is_err());
    }

    #[test]
    fn crud_roundtrip_file_upload_create_fetch_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let new_upload = make_new_frontcover_upload(work.work_id, "jpg");

        let upload =
            FileUpload::create(pool.as_ref(), &new_upload).expect("Failed to create file upload");
        let fetched = FileUpload::from_id(pool.as_ref(), &upload.file_upload_id)
            .expect("Failed to fetch file upload");
        assert_eq!(fetched.file_upload_id, upload.file_upload_id);

        let deleted = fetched
            .delete(pool.as_ref())
            .expect("Failed to delete file upload");
        assert!(FileUpload::from_id(pool.as_ref(), &deleted.file_upload_id).is_err());
    }

    #[test]
    fn crud_lookup_helpers_return_expected_records() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let publication = create_pdf_publication(pool.as_ref(), work.work_id);

        let frontcover_file = File::create(
            pool.as_ref(),
            &make_new_frontcover_file(
                work.work_id,
                format!("10.1234/{}/frontcover.jpg", Uuid::new_v4()),
            ),
        )
        .expect("Failed to create frontcover file");
        let publication_file = File::create(
            pool.as_ref(),
            &make_new_publication_file(
                publication.publication_id,
                format!("10.1234/{}/publication.pdf", Uuid::new_v4()),
            ),
        )
        .expect("Failed to create publication file");

        let from_object = File::from_object_key(pool.as_ref(), &frontcover_file.object_key)
            .expect("Failed to fetch by object key");
        assert_eq!(from_object.file_id, frontcover_file.file_id);

        let from_work = File::from_work_id(pool.as_ref(), &work.work_id)
            .expect("Failed to fetch frontcover by work id")
            .expect("Expected frontcover file");
        assert_eq!(from_work.file_id, frontcover_file.file_id);

        let from_publication =
            File::from_publication_id(pool.as_ref(), &publication.publication_id)
                .expect("Failed to fetch publication file by publication id")
                .expect("Expected publication file");
        assert_eq!(from_publication.file_id, publication_file.file_id);

        let other_work = create_work(pool.as_ref(), &imprint);
        let other_publication = create_pdf_publication(pool.as_ref(), other_work.work_id);
        assert!(File::from_work_id(pool.as_ref(), &other_work.work_id)
            .expect("Failed to query frontcover by work id")
            .is_none());
        assert!(
            File::from_publication_id(pool.as_ref(), &other_publication.publication_id)
                .expect("Failed to query publication file by publication id")
                .is_none()
        );
    }

    #[test]
    fn crud_publisher_id_resolves_for_all_file_variants() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let publication = create_pdf_publication(pool.as_ref(), work.work_id);

        let frontcover_new_file = make_new_frontcover_file(
            work.work_id,
            format!("10.1234/{}/cover.jpg", Uuid::new_v4()),
        );
        let publication_new_file = make_new_publication_file(
            publication.publication_id,
            format!("10.1234/{}/publication.pdf", Uuid::new_v4()),
        );

        assert_eq!(
            frontcover_new_file.publisher_id(pool.as_ref()).unwrap(),
            publisher.publisher_id
        );
        assert_eq!(
            publication_new_file.publisher_id(pool.as_ref()).unwrap(),
            publisher.publisher_id
        );

        let frontcover_file =
            File::create(pool.as_ref(), &frontcover_new_file).expect("Failed to create file");
        let publication_upload = FileUpload::create(
            pool.as_ref(),
            &make_new_publication_upload(publication.publication_id, "pdf"),
        )
        .expect("Failed to create file upload");

        assert_eq!(
            frontcover_file.publisher_id(pool.as_ref()).unwrap(),
            publisher.publisher_id
        );
        assert_eq!(
            publication_upload.publisher_id(pool.as_ref()).unwrap(),
            publisher.publisher_id
        );

        let invalid_new_file = NewFile {
            file_type: FileType::Frontcover,
            work_id: None,
            publication_id: None,
            object_key: "invalid.jpg".to_string(),
            cdn_url: "https://cdn.example.org/invalid.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            bytes: 1,
            sha256: TEST_SHA256_HEX.to_string(),
        };
        assert_eq!(
            invalid_new_file.publisher_id(pool.as_ref()).unwrap_err(),
            ThothError::FileMissingWorkOrPublicationId
        );

        let invalid_upload = NewFileUpload {
            file_type: FileType::Publication,
            work_id: None,
            publication_id: None,
            declared_mime_type: "application/pdf".to_string(),
            declared_extension: "pdf".to_string(),
            declared_sha256: TEST_SHA256_HEX.to_string(),
        };
        assert_eq!(
            invalid_upload.publisher_id(pool.as_ref()).unwrap_err(),
            ThothError::FileUploadMissingWorkOrPublicationId
        );
    }

    #[test]
    fn crud_file_upload_helpers_load_scope_and_canonical_key() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let publication = create_pdf_publication(pool.as_ref(), work.work_id);

        let publication_upload = FileUpload::create(
            pool.as_ref(),
            &make_new_publication_upload(publication.publication_id, "pdf"),
        )
        .expect("Failed to create publication upload");
        let frontcover_upload = FileUpload::create(
            pool.as_ref(),
            &make_new_frontcover_upload(work.work_id, "jpg"),
        )
        .expect("Failed to create frontcover upload");

        let ctx = test_context(pool.clone(), "file-user");

        let (loaded_work, loaded_publication) = publication_upload
            .load_scope(&ctx)
            .expect("Failed to load publication upload scope");
        assert_eq!(loaded_work.work_id, work.work_id);
        assert_eq!(
            loaded_publication
                .expect("Expected publication to be loaded")
                .publication_id,
            publication.publication_id
        );

        let (loaded_work, loaded_publication) = frontcover_upload
            .load_scope(&ctx)
            .expect("Failed to load frontcover upload scope");
        assert_eq!(loaded_work.work_id, work.work_id);
        assert!(loaded_publication.is_none());

        let doi = Doi::from_str("https://doi.org/10.1234/AbC/Def").expect("Failed to parse DOI");
        assert_eq!(
            publication_upload.canonical_key(&doi),
            "10.1234/abc/def.pdf"
        );
        assert_eq!(
            frontcover_upload.canonical_key(&doi),
            "10.1234/abc/def_frontcover.jpg"
        );
    }

    #[test]
    fn crud_persist_file_record_creates_and_updates_existing_file() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("file-user", Role::CdnWrite, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let upload = FileUpload::create(
            pool.as_ref(),
            &make_new_frontcover_upload(work.work_id, "jpg"),
        )
        .expect("Failed to create upload");

        let first_key = "10.1234/abc/def_frontcover.jpg";
        let first_url = "https://cdn.example.org/10.1234/abc/def_frontcover.jpg";

        let (created_file, old_key) = upload
            .persist_file_record(&ctx, first_key, first_url, "image/jpeg", 1024)
            .expect("Failed to create initial file record");
        assert!(old_key.is_none());
        assert_eq!(created_file.object_key, first_key);
        assert_eq!(created_file.cdn_url, first_url);
        assert_eq!(created_file.mime_type, "image/jpeg");
        assert_eq!(created_file.bytes, 1024);

        let second_key = "10.1234/abc/def_frontcover_v2.jpg";
        let second_url = "https://cdn.example.org/10.1234/abc/def_frontcover_v2.jpg";

        let (updated_file, old_key) = upload
            .persist_file_record(&ctx, second_key, second_url, "image/webp", 2048)
            .expect("Failed to update existing file record");
        assert_eq!(old_key.as_deref(), Some(first_key));
        assert_eq!(updated_file.file_id, created_file.file_id);
        assert_eq!(updated_file.object_key, second_key);
        assert_eq!(updated_file.cdn_url, second_url);
        assert_eq!(updated_file.mime_type, "image/webp");
        assert_eq!(updated_file.bytes, 2048);

        let persisted = File::from_work_id(pool.as_ref(), &work.work_id)
            .expect("Failed to reload file by work id")
            .expect("Expected persisted frontcover");
        assert_eq!(persisted.file_id, created_file.file_id);
        assert_eq!(persisted.object_key, second_key);
    }

    #[test]
    fn crud_sync_related_metadata_updates_work_cover_url_for_frontcover() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("file-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let upload = FileUpload::create(
            pool.as_ref(),
            &make_new_frontcover_upload(work.work_id, "jpg"),
        )
        .expect("Failed to create upload");

        let cover_url = "https://cdn.example.org/10.1234/abc/def_frontcover.jpg";
        upload
            .sync_related_metadata(&ctx, &work, cover_url)
            .expect("Failed to sync frontcover metadata");

        let refreshed_work = Work::from_id(pool.as_ref(), &work.work_id)
            .expect("Failed to reload work after metadata sync");
        assert_eq!(refreshed_work.cover_url.as_deref(), Some(cover_url));
    }
}
