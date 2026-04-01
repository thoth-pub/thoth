use super::*;
use crate::db::PgPool;
use crate::model::{
    file::{File, FileType, NewFile, NewFileUpload},
    imprint::{Imprint, NewImprint},
    tests::db::{create_imprint, create_publication, create_publisher, create_work, setup_test_db},
    Crud,
};
use std::time::Duration;
use thoth_errors::ThothError;
use uuid::Uuid;

#[test]
fn storage_config_from_imprint_returns_values() {
    let imprint = Imprint {
        s3_bucket: Some("bucket".to_string()),
        cdn_domain: Some("cdn.example.org".to_string()),
        cloudfront_dist_id: Some("dist-123".to_string()),
        ..Default::default()
    };

    let config = StorageConfig::from_imprint(&imprint).expect("Expected storage config");
    assert_eq!(config.s3_bucket, "bucket");
    assert_eq!(config.cdn_domain, "cdn.example.org");
    assert_eq!(config.cloudfront_dist_id, "dist-123");
}

#[test]
fn storage_config_from_imprint_requires_all_fields() {
    let imprint = Imprint {
        s3_bucket: Some("bucket".to_string()),
        cdn_domain: None,
        cloudfront_dist_id: Some("dist-123".to_string()),
        ..Default::default()
    };

    let err = StorageConfig::from_imprint(&imprint)
        .err()
        .expect("Expected missing config error");
    assert_eq!(
        err,
        ThothError::InternalError("Imprint is not configured for file hosting".to_string())
    );
}

#[test]
fn temp_key_prefixes_uploads() {
    let upload_id = Uuid::parse_str("6f4e7ad7-8e68-4c1e-8efc-49f7c59b0c88").unwrap();
    assert_eq!(temp_key(&upload_id), format!("uploads/{}", upload_id));
}

#[test]
fn canonical_publication_key_lowercases_parts() {
    let key = canonical_publication_key("10.1234", "AbC/Def", "PDF");
    assert_eq!(key, "10.1234/abc/def.pdf");
}

#[test]
fn canonical_frontcover_key_lowercases_parts() {
    let key = canonical_frontcover_key("10.1234", "AbC/Def", "PNG");
    assert_eq!(key, "10.1234/abc/def_frontcover.png");
}

#[test]
fn canonical_resource_key_uses_resource_subpath() {
    let resource_id = Uuid::parse_str("0f97fb46-4ed2-4bc0-98dd-f2f8ce0ebe11").unwrap();
    let key = canonical_resource_key("10.1234", "AbC/Def", &resource_id, "MP4");
    assert_eq!(
        key,
        "10.1234/abc/def/resources/0f97fb46-4ed2-4bc0-98dd-f2f8ce0ebe11.mp4"
    );
}

#[test]
fn build_cdn_url_normalizes_domain_and_key() {
    let https_url = build_cdn_url("https://cdn.example.org/", "/files/doc.pdf");
    assert_eq!(https_url, "https://cdn.example.org/files/doc.pdf");

    let http_url = build_cdn_url("http://cdn.example.org", "files/doc.pdf");
    assert_eq!(http_url, "https://cdn.example.org/files/doc.pdf");
}

fn build_tkhd_box_v0(width: u32, height: u32) -> Vec<u8> {
    let mut tkhd = vec![0u8; 92];
    tkhd[0..4].copy_from_slice(&(92u32).to_be_bytes());
    tkhd[4..8].copy_from_slice(b"tkhd");
    tkhd[8] = 0; // version 0
    tkhd[9..12].copy_from_slice(&[0, 0, 7]); // flags
    tkhd[84..88].copy_from_slice(&(width << 16).to_be_bytes());
    tkhd[88..92].copy_from_slice(&(height << 16).to_be_bytes());
    tkhd
}

#[test]
fn parse_mp4_track_header_dimensions_extracts_size() {
    let mut payload = vec![0u8; 32];
    payload.extend_from_slice(&build_tkhd_box_v0(1280, 720));

    let parsed = parse_mp4_track_header_dimensions(&payload);
    assert_eq!(parsed, Some((1280, 720)));
}

#[test]
fn parse_mp4_track_header_dimensions_prefers_non_zero_video_track() {
    let mut payload = build_tkhd_box_v0(0, 0);
    payload.extend_from_slice(&build_tkhd_box_v0(640, 360));

    let parsed = parse_mp4_track_header_dimensions(&payload);
    assert_eq!(parsed, Some((640, 360)));
}

const TEST_SHA256_HEX: &str = "444b138b41e3c48ca505b1740091b0c93ce9a71c7c9d24956e6cf8716f1aad7e";

fn create_hosting_imprint(
    pool: &PgPool,
    publisher: &crate::model::publisher::Publisher,
) -> Imprint {
    Imprint::create(
        pool,
        &NewImprint {
            publisher_id: publisher.publisher_id,
            imprint_name: format!("Hosting Imprint {}", Uuid::new_v4()),
            imprint_url: None,
            crossmark_doi: None,
            s3_bucket: Some("bucket-example".to_string()),
            cdn_domain: Some("cdn.example.org".to_string()),
            cloudfront_dist_id: Some("dist-example".to_string()),
            default_currency: None,
            default_place: None,
            default_locale: None,
        },
    )
    .expect("Failed to create hosting imprint")
}

fn create_additional_resource(
    pool: &PgPool,
    work_id: Uuid,
) -> crate::model::additional_resource::AdditionalResource {
    use crate::model::additional_resource::{
        AdditionalResource, NewAdditionalResource, ResourceType,
    };

    AdditionalResource::create(
        pool,
        &NewAdditionalResource {
            work_id,
            title: format!("Resource {}", Uuid::new_v4()),
            description: None,
            attribution: None,
            resource_type: ResourceType::Dataset,
            doi: None,
            handle: None,
            url: None,
            date: None,
            resource_ordinal: 1,
        },
    )
    .expect("Failed to create additional resource")
}

fn create_work_featured_video(
    pool: &PgPool,
    work_id: Uuid,
) -> crate::model::work_featured_video::WorkFeaturedVideo {
    use crate::model::work_featured_video::{NewWorkFeaturedVideo, WorkFeaturedVideo};

    WorkFeaturedVideo::create(
        pool,
        &NewWorkFeaturedVideo {
            work_id,
            title: Some("Featured video".to_string()),
            url: None,
            width: 560,
            height: 315,
        },
    )
    .expect("Failed to create featured video")
}

#[test]
fn work_cleanup_plan_returns_none_without_candidates() {
    let (_guard, pool) = setup_test_db();
    let publisher = create_publisher(pool.as_ref());
    let imprint = create_imprint(pool.as_ref(), &publisher);
    let work = create_work(pool.as_ref(), &imprint);

    let plan = work_cleanup_plan(pool.as_ref(), &work).expect("Failed to build cleanup plan");
    assert!(plan.is_none());
}

#[test]
fn publication_cleanup_plan_returns_none_without_candidates() {
    let (_guard, pool) = setup_test_db();
    let publisher = create_publisher(pool.as_ref());
    let imprint = create_imprint(pool.as_ref(), &publisher);
    let work = create_work(pool.as_ref(), &imprint);
    let publication = create_publication(pool.as_ref(), &work);

    let plan = publication_cleanup_plan(pool.as_ref(), &publication)
        .expect("Failed to build publication cleanup plan");
    assert!(plan.is_none());
}

#[test]
fn work_cleanup_plan_includes_storage_config_and_candidates() {
    let (_guard, pool) = setup_test_db();
    let publisher = create_publisher(pool.as_ref());
    let imprint = create_hosting_imprint(pool.as_ref(), &publisher);
    let work = create_work(pool.as_ref(), &imprint);

    let object_key = format!("10.1234/{}/cover.jpg", Uuid::new_v4());
    File::create(
        pool.as_ref(),
        &NewFile {
            file_type: FileType::Frontcover,
            work_id: Some(work.work_id),
            publication_id: None,
            additional_resource_id: None,
            work_featured_video_id: None,
            object_key: object_key.clone(),
            cdn_url: format!("https://cdn.example.org/{object_key}"),
            mime_type: "image/jpeg".to_string(),
            bytes: 1024,
            sha256: TEST_SHA256_HEX.to_string(),
        },
    )
    .expect("Failed to create file");
    let pending_upload = crate::model::file::FileUpload::create(
        pool.as_ref(),
        &NewFileUpload {
            file_type: FileType::Frontcover,
            work_id: Some(work.work_id),
            publication_id: None,
            additional_resource_id: None,
            work_featured_video_id: None,
            declared_mime_type: "image/jpeg".to_string(),
            declared_extension: "jpg".to_string(),
            declared_sha256: TEST_SHA256_HEX.to_string(),
        },
    )
    .expect("Failed to create pending upload");

    let plan = work_cleanup_plan(pool.as_ref(), &work)
        .expect("Failed to build work cleanup plan")
        .expect("Expected cleanup plan");

    assert_eq!(plan.entity_type, "work");
    assert_eq!(plan.entity_id, work.work_id);
    assert_eq!(plan.storage_config.s3_bucket, "bucket-example");
    assert_eq!(plan.storage_config.cloudfront_dist_id, "dist-example");
    assert_eq!(plan.candidates.len(), 2);
    assert!(plan
        .candidates
        .iter()
        .any(|candidate| candidate.object_key == object_key));
    assert!(plan
        .candidates
        .iter()
        .any(|candidate| candidate.object_key == temp_key(&pending_upload.file_upload_id)));
}

#[test]
fn publication_cleanup_plan_includes_pending_upload_candidate() {
    let (_guard, pool) = setup_test_db();
    let publisher = create_publisher(pool.as_ref());
    let imprint = create_hosting_imprint(pool.as_ref(), &publisher);
    let work = create_work(pool.as_ref(), &imprint);
    let publication = create_publication(pool.as_ref(), &work);

    let pending_upload = crate::model::file::FileUpload::create(
        pool.as_ref(),
        &NewFileUpload {
            file_type: FileType::Publication,
            work_id: None,
            publication_id: Some(publication.publication_id),
            additional_resource_id: None,
            work_featured_video_id: None,
            declared_mime_type: "application/pdf".to_string(),
            declared_extension: "pdf".to_string(),
            declared_sha256: TEST_SHA256_HEX.to_string(),
        },
    )
    .expect("Failed to create pending upload");

    let plan = publication_cleanup_plan(pool.as_ref(), &publication)
        .expect("Failed to build publication cleanup plan")
        .expect("Expected cleanup plan");

    assert_eq!(plan.entity_type, "publication");
    assert_eq!(plan.entity_id, publication.publication_id);
    assert_eq!(plan.storage_config.s3_bucket, "bucket-example");
    assert_eq!(plan.storage_config.cloudfront_dist_id, "dist-example");
    assert_eq!(plan.candidates.len(), 1);
    assert_eq!(
        plan.candidates[0].object_key,
        temp_key(&pending_upload.file_upload_id)
    );
}

#[test]
fn additional_resource_cleanup_plan_includes_pending_upload_candidate() {
    let (_guard, pool) = setup_test_db();
    let publisher = create_publisher(pool.as_ref());
    let imprint = create_hosting_imprint(pool.as_ref(), &publisher);
    let work = create_work(pool.as_ref(), &imprint);
    let additional_resource = create_additional_resource(pool.as_ref(), work.work_id);

    let pending_upload = crate::model::file::FileUpload::create(
        pool.as_ref(),
        &NewFileUpload {
            file_type: FileType::AdditionalResource,
            work_id: None,
            publication_id: None,
            additional_resource_id: Some(additional_resource.additional_resource_id),
            work_featured_video_id: None,
            declared_mime_type: "application/json".to_string(),
            declared_extension: "json".to_string(),
            declared_sha256: TEST_SHA256_HEX.to_string(),
        },
    )
    .expect("Failed to create pending additional-resource upload");

    let plan = additional_resource_cleanup_plan(pool.as_ref(), &additional_resource)
        .expect("Failed to build additional-resource cleanup plan")
        .expect("Expected cleanup plan");

    assert_eq!(plan.entity_type, "additional_resource");
    assert_eq!(plan.entity_id, additional_resource.additional_resource_id);
    assert_eq!(plan.storage_config.s3_bucket, "bucket-example");
    assert_eq!(plan.storage_config.cloudfront_dist_id, "dist-example");
    assert_eq!(plan.candidates.len(), 1);
    assert_eq!(
        plan.candidates[0].object_key,
        temp_key(&pending_upload.file_upload_id)
    );
}

#[test]
fn work_featured_video_cleanup_plan_includes_pending_upload_candidate() {
    let (_guard, pool) = setup_test_db();
    let publisher = create_publisher(pool.as_ref());
    let imprint = create_hosting_imprint(pool.as_ref(), &publisher);
    let work = create_work(pool.as_ref(), &imprint);
    let featured_video = create_work_featured_video(pool.as_ref(), work.work_id);

    let pending_upload = crate::model::file::FileUpload::create(
        pool.as_ref(),
        &NewFileUpload {
            file_type: FileType::WorkFeaturedVideo,
            work_id: None,
            publication_id: None,
            additional_resource_id: None,
            work_featured_video_id: Some(featured_video.work_featured_video_id),
            declared_mime_type: "video/mp4".to_string(),
            declared_extension: "mp4".to_string(),
            declared_sha256: TEST_SHA256_HEX.to_string(),
        },
    )
    .expect("Failed to create pending featured-video upload");

    let plan = work_featured_video_cleanup_plan(pool.as_ref(), &featured_video)
        .expect("Failed to build featured-video cleanup plan")
        .expect("Expected cleanup plan");

    assert_eq!(plan.entity_type, "work_featured_video");
    assert_eq!(plan.entity_id, featured_video.work_featured_video_id);
    assert_eq!(plan.storage_config.s3_bucket, "bucket-example");
    assert_eq!(plan.storage_config.cloudfront_dist_id, "dist-example");
    assert_eq!(plan.candidates.len(), 1);
    assert_eq!(
        plan.candidates[0].object_key,
        temp_key(&pending_upload.file_upload_id)
    );
}

#[test]
fn run_cleanup_plan_sync_is_noop_for_empty_candidates() {
    let runtime = tokio::runtime::Runtime::new().expect("Failed to build Tokio runtime");
    let s3_client = runtime.block_on(create_s3_client(
        "test-access-key",
        "test-secret-key",
        "us-east-1",
    ));
    let cloudfront_client = runtime.block_on(create_cloudfront_client(
        "test-access-key",
        "test-secret-key",
        "us-east-1",
    ));

    run_cleanup_plan_sync(
        &s3_client,
        &cloudfront_client,
        FileCleanupPlan {
            entity_type: "work",
            entity_id: Uuid::new_v4(),
            storage_config: StorageConfig {
                s3_bucket: "bucket-example".to_string(),
                cdn_domain: "cdn.example.org".to_string(),
                cloudfront_dist_id: "dist-example".to_string(),
            },
            candidates: vec![],
        },
    );
}

fn build_aws_error_context(code: Option<&str>, http_status: Option<u16>) -> AwsErrorContext {
    AwsErrorContext {
        code: code.map(ToOwned::to_owned),
        message: Some("test error".to_string()),
        http_status,
        request_id: Some("req-id".to_string()),
        extended_request_id: Some("ext-id".to_string()),
        retryable_classification: "service_error",
    }
}

#[test]
fn classify_delete_error_marks_missing_object_as_absent() {
    let missing_key = build_aws_error_context(Some("NoSuchKey"), None);
    assert_eq!(
        classify_delete_error(&missing_key),
        CleanupObjectOutcome::AlreadyAbsent
    );

    let not_found = build_aws_error_context(None, Some(404));
    assert_eq!(
        classify_delete_error(&not_found),
        CleanupObjectOutcome::AlreadyAbsent
    );
}

#[test]
fn classify_delete_error_marks_other_service_errors_as_failed() {
    let access_denied = build_aws_error_context(Some("AccessDenied"), Some(403));
    assert_eq!(
        classify_delete_error(&access_denied),
        CleanupObjectOutcome::Failed
    );
}

#[test]
fn cleanup_counters_track_deleted_absent_and_failed_results() {
    let mut counters = super::cleanup::CleanupCounters::default();
    counters.record(CleanupObjectOutcome::Deleted);
    counters.record(CleanupObjectOutcome::AlreadyAbsent);
    counters.record(CleanupObjectOutcome::Failed);
    counters.record(CleanupObjectOutcome::Deleted);

    assert_eq!(counters.keys_total, 4);
    assert_eq!(counters.keys_deleted, 2);
    assert_eq!(counters.keys_absent, 1);
    assert_eq!(counters.keys_failed, 1);
}

#[test]
fn duration_ms_helper_reports_milliseconds() {
    assert_eq!(
        super::cleanup::duration_ms(Duration::from_millis(1500)),
        1500
    );
    assert_eq!(super::cleanup::duration_ms(Duration::from_secs(2)), 2000);
}
