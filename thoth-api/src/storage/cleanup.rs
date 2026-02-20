use log::{info, warn};
use std::time::Instant;
use thoth_errors::ThothResult;
use uuid::Uuid;

use crate::db::PgPool;
use crate::model::{
    file::{File, FileCleanupCandidate},
    imprint::Imprint,
    publication::Publication,
    work::Work,
    Crud,
};

use super::{cleanup_object_best_effort, CloudFrontClient, S3Client, StorageConfig};

pub struct FileCleanupPlan {
    pub entity_type: &'static str,
    pub entity_id: Uuid,
    pub storage_config: StorageConfig,
    pub candidates: Vec<FileCleanupCandidate>,
}

fn resolve_storage_config(
    db: &PgPool,
    work: &Work,
    entity_type: &'static str,
    entity_id: Uuid,
) -> ThothResult<Option<StorageConfig>> {
    let imprint = Imprint::from_id(db, &work.imprint_id)?;
    match StorageConfig::from_imprint(&imprint) {
        Ok(storage_config) => Ok(Some(storage_config)),
        Err(error) => {
            warn!(
                "entity_type={entity_type} entity_id={entity_id} cleanup_skipped=true reason=\"missing_storage_config\" error=\"{error}\"",
            );
            Ok(None)
        }
    }
}

pub fn work_cleanup_plan(db: &PgPool, work: &Work) -> ThothResult<Option<FileCleanupPlan>> {
    let candidates = File::cleanup_candidates_for_work(db, &work.work_id)?;
    if candidates.is_empty() {
        return Ok(None);
    }

    let Some(storage_config) = resolve_storage_config(db, work, "work", work.work_id)? else {
        return Ok(None);
    };

    Ok(Some(FileCleanupPlan {
        entity_type: "work",
        entity_id: work.work_id,
        storage_config,
        candidates,
    }))
}

pub fn publication_cleanup_plan(
    db: &PgPool,
    publication: &Publication,
) -> ThothResult<Option<FileCleanupPlan>> {
    let candidates = File::cleanup_candidates_for_publication(db, &publication.publication_id)?;
    if candidates.is_empty() {
        return Ok(None);
    }

    let work = Work::from_id(db, &publication.work_id)?;
    let Some(storage_config) =
        resolve_storage_config(db, &work, "publication", publication.publication_id)?
    else {
        return Ok(None);
    };

    Ok(Some(FileCleanupPlan {
        entity_type: "publication",
        entity_id: publication.publication_id,
        storage_config,
        candidates,
    }))
}

pub async fn run_cleanup_plan(
    s3_client: &S3Client,
    cloudfront_client: &CloudFrontClient,
    plan: FileCleanupPlan,
) {
    let started = Instant::now();
    let mut deleted = 0usize;
    let mut failed = 0usize;

    for candidate in &plan.candidates {
        let log_context = format!(
            "entity_type={} entity_id={} file_type={} object_key={} bucket={} cloudfront_dist_id={}",
            plan.entity_type,
            plan.entity_id,
            candidate.file_type,
            candidate.object_key,
            plan.storage_config.s3_bucket,
            plan.storage_config.cloudfront_dist_id
        );
        if cleanup_object_best_effort(
            s3_client,
            cloudfront_client,
            &plan.storage_config.s3_bucket,
            &plan.storage_config.cloudfront_dist_id,
            &candidate.object_key,
            &log_context,
        )
        .await
        {
            deleted += 1;
        } else {
            failed += 1;
        }
    }

    info!(
        "entity_type={} entity_id={} cleanup_completed=true keys_total={} keys_deleted={} keys_failed={} duration_ms={}",
        plan.entity_type,
        plan.entity_id,
        plan.candidates.len(),
        deleted,
        failed,
        started.elapsed().as_millis()
    );
}

pub fn run_cleanup_plan_sync(
    s3_client: &S3Client,
    cloudfront_client: &CloudFrontClient,
    plan: FileCleanupPlan,
) {
    futures::executor::block_on(run_cleanup_plan(s3_client, cloudfront_client, plan));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        file::{FileType, NewFile},
        imprint::NewImprint,
        tests::db::{
            create_imprint, create_publication, create_publisher, create_work, setup_test_db,
        },
    };
    use crate::storage::{create_cloudfront_client, create_s3_client};

    const TEST_SHA256_HEX: &str =
        "444b138b41e3c48ca505b1740091b0c93ce9a71c7c9d24956e6cf8716f1aad7e";

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
            },
        )
        .expect("Failed to create hosting imprint")
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

        let plan = work_cleanup_plan(pool.as_ref(), &work)
            .expect("Failed to build work cleanup plan")
            .expect("Expected cleanup plan");

        assert_eq!(plan.entity_type, "work");
        assert_eq!(plan.entity_id, work.work_id);
        assert_eq!(plan.storage_config.s3_bucket, "bucket-example");
        assert_eq!(plan.storage_config.cloudfront_dist_id, "dist-example");
        assert_eq!(plan.candidates.len(), 1);
        assert_eq!(plan.candidates[0].object_key, object_key);
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
}
