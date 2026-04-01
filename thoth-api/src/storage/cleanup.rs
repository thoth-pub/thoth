use log::{info, warn};
use std::time::{Duration, Instant};
use thoth_errors::ThothResult;
use uuid::Uuid;

use crate::db::PgPool;
use crate::model::{
    additional_resource::AdditionalResource,
    file::{File, FileCleanupCandidate},
    imprint::Imprint,
    publication::Publication,
    work::Work,
    work_featured_video::WorkFeaturedVideo,
    Crud,
};

use super::{
    cleanup_object_best_effort, AwsErrorContext, CleanupObjectOutcome, CloudFrontClient, S3Client,
    StorageConfig,
};

const SLOW_OPERATION_WARN_THRESHOLD_MS: u128 = 5_000;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct CleanupCounters {
    pub keys_total: usize,
    pub keys_deleted: usize,
    pub keys_absent: usize,
    pub keys_failed: usize,
}

impl CleanupCounters {
    pub(super) fn record(&mut self, outcome: CleanupObjectOutcome) {
        self.keys_total += 1;
        match outcome {
            CleanupObjectOutcome::Deleted => self.keys_deleted += 1,
            CleanupObjectOutcome::AlreadyAbsent => self.keys_absent += 1,
            CleanupObjectOutcome::Failed => self.keys_failed += 1,
        }
    }
}

pub(super) fn duration_ms(duration: Duration) -> u128 {
    duration.as_millis()
}

pub(super) fn elapsed_ms(started: Instant) -> u128 {
    duration_ms(started.elapsed())
}

fn aws_log_fields(
    context: Option<&AwsErrorContext>,
) -> (String, String, String, String, String, &'static str) {
    match context {
        Some(context) => (
            context.code.clone().unwrap_or_default(),
            context.message.clone().unwrap_or_default(),
            context
                .http_status
                .map(|status| status.to_string())
                .unwrap_or_default(),
            context.request_id.clone().unwrap_or_default(),
            context.extended_request_id.clone().unwrap_or_default(),
            context.retryable_classification,
        ),
        None => (
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            "",
        ),
    }
}

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

pub fn additional_resource_cleanup_plan(
    db: &PgPool,
    additional_resource: &AdditionalResource,
) -> ThothResult<Option<FileCleanupPlan>> {
    let candidates = File::cleanup_candidates_for_additional_resource(
        db,
        &additional_resource.additional_resource_id,
    )?;
    if candidates.is_empty() {
        return Ok(None);
    }

    let work = Work::from_id(db, &additional_resource.work_id)?;
    let Some(storage_config) = resolve_storage_config(
        db,
        &work,
        "additional_resource",
        additional_resource.additional_resource_id,
    )?
    else {
        return Ok(None);
    };

    Ok(Some(FileCleanupPlan {
        entity_type: "additional_resource",
        entity_id: additional_resource.additional_resource_id,
        storage_config,
        candidates,
    }))
}

pub fn work_featured_video_cleanup_plan(
    db: &PgPool,
    work_featured_video: &WorkFeaturedVideo,
) -> ThothResult<Option<FileCleanupPlan>> {
    let candidates = File::cleanup_candidates_for_work_featured_video(
        db,
        &work_featured_video.work_featured_video_id,
    )?;
    if candidates.is_empty() {
        return Ok(None);
    }

    let work = Work::from_id(db, &work_featured_video.work_id)?;
    let Some(storage_config) = resolve_storage_config(
        db,
        &work,
        "work_featured_video",
        work_featured_video.work_featured_video_id,
    )?
    else {
        return Ok(None);
    };

    Ok(Some(FileCleanupPlan {
        entity_type: "work_featured_video",
        entity_id: work_featured_video.work_featured_video_id,
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
    let cleanup_run_id = Uuid::new_v4();
    let mut counters = CleanupCounters::default();

    for candidate in &plan.candidates {
        let report = cleanup_object_best_effort(
            s3_client,
            cloudfront_client,
            &plan.storage_config.s3_bucket,
            &plan.storage_config.cloudfront_dist_id,
            &candidate.object_key,
        )
        .await;
        counters.record(report.outcome);

        let (
            delete_error_code,
            delete_error_message,
            delete_http_status,
            delete_request_id,
            delete_extended_request_id,
            delete_retryable_classification,
        ) = aws_log_fields(report.delete_error.as_ref());
        let delete_slow = report.delete_ms > SLOW_OPERATION_WARN_THRESHOLD_MS;
        let delete_base_log = format!(
            "cleanup_run_id={} entity_type={} entity_id={} file_type={} object_key={} bucket={} cloudfront_dist_id={} phase=delete result={} delete_ms={} aws_error_code={} aws_error_message={:?} aws_http_status={} request_id={} extended_request_id={} retryable_classification={}",
            cleanup_run_id,
            plan.entity_type,
            plan.entity_id,
            candidate.file_type,
            candidate.object_key,
            plan.storage_config.s3_bucket,
            plan.storage_config.cloudfront_dist_id,
            report.delete_outcome.as_str(),
            report.delete_ms,
            delete_error_code,
            delete_error_message,
            delete_http_status,
            delete_request_id,
            delete_extended_request_id,
            delete_retryable_classification
        );
        if report.delete_outcome == CleanupObjectOutcome::Failed {
            warn!("{delete_base_log}");
        } else {
            info!("{delete_base_log}");
        }
        if delete_slow {
            warn!(
                "cleanup_run_id={} entity_type={} entity_id={} file_type={} object_key={} bucket={} cloudfront_dist_id={} phase=delete slow_operation=true operation_ms={} threshold_ms={}",
                cleanup_run_id,
                plan.entity_type,
                plan.entity_id,
                candidate.file_type,
                candidate.object_key,
                plan.storage_config.s3_bucket,
                plan.storage_config.cloudfront_dist_id,
                report.delete_ms,
                SLOW_OPERATION_WARN_THRESHOLD_MS
            );
        }

        match report.invalidate_ms {
            Some(invalidate_ms) => {
                let (
                    invalidate_error_code,
                    invalidate_error_message,
                    invalidate_http_status,
                    invalidate_request_id,
                    invalidate_extended_request_id,
                    invalidate_retryable_classification,
                ) = aws_log_fields(report.invalidate_error.as_ref());
                let invalidate_result = if report.invalidate_error.is_some() {
                    "failed"
                } else {
                    "invalidated"
                };
                let invalidate_base_log = format!(
                    "cleanup_run_id={} entity_type={} entity_id={} file_type={} object_key={} bucket={} cloudfront_dist_id={} phase=invalidate result={} invalidate_ms={} aws_error_code={} aws_error_message={:?} aws_http_status={} request_id={} extended_request_id={} retryable_classification={}",
                    cleanup_run_id,
                    plan.entity_type,
                    plan.entity_id,
                    candidate.file_type,
                    candidate.object_key,
                    plan.storage_config.s3_bucket,
                    plan.storage_config.cloudfront_dist_id,
                    invalidate_result,
                    invalidate_ms,
                    invalidate_error_code,
                    invalidate_error_message,
                    invalidate_http_status,
                    invalidate_request_id,
                    invalidate_extended_request_id,
                    invalidate_retryable_classification
                );
                if report.invalidate_error.is_some() {
                    warn!("{invalidate_base_log}");
                } else {
                    info!("{invalidate_base_log}");
                }
                if invalidate_ms > SLOW_OPERATION_WARN_THRESHOLD_MS {
                    warn!(
                        "cleanup_run_id={} entity_type={} entity_id={} file_type={} object_key={} bucket={} cloudfront_dist_id={} phase=invalidate slow_operation=true operation_ms={} threshold_ms={}",
                        cleanup_run_id,
                        plan.entity_type,
                        plan.entity_id,
                        candidate.file_type,
                        candidate.object_key,
                        plan.storage_config.s3_bucket,
                        plan.storage_config.cloudfront_dist_id,
                        invalidate_ms,
                        SLOW_OPERATION_WARN_THRESHOLD_MS
                    );
                }
            }
            None => {
                info!(
                    "cleanup_run_id={} entity_type={} entity_id={} file_type={} object_key={} bucket={} cloudfront_dist_id={} phase=invalidate result=skipped reason=delete_failed",
                    cleanup_run_id,
                    plan.entity_type,
                    plan.entity_id,
                    candidate.file_type,
                    candidate.object_key,
                    plan.storage_config.s3_bucket,
                    plan.storage_config.cloudfront_dist_id
                );
            }
        }
    }

    info!(
        "cleanup_run_id={} entity_type={} entity_id={} cleanup_completed=true keys_total={} keys_deleted={} keys_absent={} keys_failed={} duration_ms={}",
        cleanup_run_id,
        plan.entity_type,
        plan.entity_id,
        counters.keys_total,
        counters.keys_deleted,
        counters.keys_absent,
        counters.keys_failed,
        elapsed_ms(started)
    );
}

pub fn run_cleanup_plan_sync(
    s3_client: &S3Client,
    cloudfront_client: &CloudFrontClient,
    plan: FileCleanupPlan,
) {
    futures::executor::block_on(run_cleanup_plan(s3_client, cloudfront_client, plan));
}
