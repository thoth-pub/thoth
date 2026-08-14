//! The named domain operations of the durable distribution-job lifecycle
//! (`BE-04`).
//!
//! `Crud` is deliberately **not** implemented for `distribution_job`,
//! `distribution_job_target` or `distribution_job_attempt`. There is no generic
//! create/update/delete surface for durable jobs: the functions here are the
//! only supported writes, and each one implements exactly one transition of the
//! approved state machine.
//!
//! Every mechanism here is programme-local. There is no generic job framework,
//! no universal lease abstraction and no reusable cross-programme claim
//! protocol (`ADR-0008` sections 3.4 and 3.5).
//!
//! Nothing in this module performs dissemination, opens a network connection to
//! a distribution platform, reads or writes a publication file, generates a feed
//! or deposit, or invokes an adapter.

use std::collections::HashMap;

use diesel::deserialize::QueryableByName;
use diesel::pg::PgConnection;
use diesel::sql_types::{Array, BigInt, Bool, Integer, Nullable, Text, Uuid as SqlUuid};
use diesel::{Connection, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use super::{
    ClaimedDistributionJob, DistributionJob, DistributionJobAttempt, DistributionJobKind,
    DistributionJobPayload, DistributionJobStatus, DistributionJobTarget,
    DISTRIBUTION_JOB_CLAIM_MAX_BATCH, DISTRIBUTION_JOB_ERROR_CODE_MAX_CHARS,
    DISTRIBUTION_JOB_ERROR_DETAIL_MAX_CHARS, DISTRIBUTION_JOB_LEASE_MAX_SECONDS,
    DISTRIBUTION_JOB_LEASE_MIN_SECONDS, DISTRIBUTION_JOB_LEASE_RECOVERY_BATCH,
    DISTRIBUTION_JOB_MAX_ATTEMPTS, DISTRIBUTION_JOB_RETRY_BASE_SECONDS,
    DISTRIBUTION_JOB_RETRY_MAX_SECONDS,
};
use crate::db::PgPool;
use crate::model::publisher_distribution_platform::DistributionPlatform;
use crate::schema::{distribution_job, distribution_job_attempt, distribution_job_target};

/// Every column of `distribution_job`, in DDL order, qualified to `j`.
///
/// The claim statement's projection is the whole job row, which is what lets
/// the claim payload be resolved without a separate "read the jobs I just
/// claimed" statement (specification section 12.3, statement 2 merged into
/// statement 1).
const JOB_COLUMNS_QUALIFIED: &str = "\
    j.distribution_job_id, j.kind, j.publisher_id, j.work_id, j.activation_id, \
    j.status, j.deduplication_key, j.attempt_count, j.available_at, j.claim_token, \
    j.claimed_by, j.claimed_at, j.lease_expires_at, j.completed_at, \
    j.cancellation_reason, j.last_error_code, j.last_error_detail, j.created_at, \
    j.updated_at";

/// The same columns projected out of the `claimed` CTE.
const JOB_COLUMNS_FROM_CLAIMED: &str = "\
    c.distribution_job_id, c.kind, c.publisher_id, c.work_id, c.activation_id, \
    c.status, c.deduplication_key, c.attempt_count, c.available_at, c.claim_token, \
    c.claimed_by, c.claimed_at, c.lease_expires_at, c.completed_at, \
    c.cancellation_reason, c.last_error_code, c.last_error_detail, c.created_at, \
    c.updated_at";

/// The claim statement's row: the whole job plus the ordinal of the attempt this
/// claim started.
#[derive(diesel::QueryableByName)]
struct ClaimRow {
    #[diesel(embed)]
    job: DistributionJob,
    #[diesel(sql_type = Integer)]
    attempt_number: i32,
}

// ---------------------------------------------------------------------------
// Creation, inside the BE-03 coordinator transaction
// ---------------------------------------------------------------------------

/// Create the durable onboarding job for one qualifying activation, and its
/// target rows, on the caller's connection and inside the caller's transaction.
///
/// This is called only from `BE-03`'s single service-configuration write
/// coordinator, at specification section 10.1 step 9b. It opens **no**
/// transaction of its own, adds no savepoint, no hook, no callback and no
/// after-the-fact best-effort path: a job therefore cannot exist without the
/// desired-state change that justified it, and that change cannot commit
/// without the job.
///
/// Idempotency is enforced by the database, not by an application
/// check-then-insert: `ON CONFLICT ON CONSTRAINT
/// distribution_job_deduplication_key_key DO NOTHING` means a repeated
/// observation of the same logical activation is a silent, correct no-op that
/// writes no target rows and raises no error.
///
/// `targets` must be non-empty; the caller computes it and only calls this
/// function for a qualifying activation, which is what enforces "no logical job
/// exists with zero targets" by construction.
pub(crate) fn create_back_catalogue_job_on(
    connection: &mut PgConnection,
    publisher_id: Uuid,
    activation_id: Uuid,
    targets: &[DistributionPlatform],
) -> ThothResult<Option<Uuid>> {
    debug_assert!(
        !targets.is_empty(),
        "a distribution job is never created with an empty target set"
    );
    if targets.is_empty() {
        return Ok(None);
    }

    let deduplication_key =
        DistributionJob::back_catalogue_deduplication_key(publisher_id, activation_id);

    let created: Option<Uuid> = diesel::insert_into(distribution_job::table)
        .values((
            distribution_job::kind.eq(DistributionJobKind::PublisherBackCatalogue),
            distribution_job::publisher_id.eq(publisher_id),
            distribution_job::activation_id.eq(activation_id),
            distribution_job::deduplication_key.eq(&deduplication_key),
            distribution_job::status.eq(DistributionJobStatus::Pending),
            distribution_job::attempt_count.eq(0),
            distribution_job::available_at.eq(current_timestamp()),
        ))
        .on_conflict(diesel::pg::upsert::on_constraint(
            "distribution_job_deduplication_key_key",
        ))
        .do_nothing()
        .returning(distribution_job::distribution_job_id)
        .get_result::<Uuid>(connection)
        .optional()?;

    let Some(distribution_job_id) = created else {
        // A job already exists for this logical activation. Write nothing,
        // raise nothing, continue.
        return Ok(None);
    };

    // One multi-row INSERT, in canonical platform order.
    let rows: Vec<_> = targets
        .iter()
        .map(|platform| {
            (
                distribution_job_target::distribution_job_id.eq(distribution_job_id),
                distribution_job_target::platform.eq(*platform),
            )
        })
        .collect();
    diesel::insert_into(distribution_job_target::table)
        .values(&rows)
        .execute(connection)?;

    Ok(Some(distribution_job_id))
}

/// Cancel the `PENDING` jobs of one publisher that target a group whose
/// assignment has just been disabled (`T8`), on the caller's connection and
/// inside the caller's transaction.
///
/// `RUNNING` jobs are deliberately **not** touched: external work may be in
/// flight and cancelling cannot undo an upload. A running job whose lease later
/// expires returns to `PENDING` within budget, where the claim eligibility
/// predicate makes it unclaimable so it waits visibly for an operator rather
/// than silently resuming.
///
/// Jobs of other publishers are never touched: the statement is scoped by
/// `publisher_id`, under the publisher row lock the coordinator already holds.
pub(crate) fn cancel_pending_jobs_for_disabled_group_on(
    connection: &mut PgConnection,
    publisher_id: Uuid,
    disabled_members: &[DistributionPlatform],
) -> ThothResult<usize> {
    if disabled_members.is_empty() {
        return Ok(0);
    }
    let members: Vec<String> = disabled_members
        .iter()
        .map(|platform| platform.to_string())
        .collect();

    diesel::sql_query(
        "UPDATE distribution_job j \
         SET status = 'CANCELLED', \
             completed_at = CURRENT_TIMESTAMP, \
             cancellation_reason = 'ASSIGNMENT_DISABLED' \
         WHERE j.publisher_id = $1 \
           AND j.status = 'PENDING' \
           AND EXISTS ( \
               SELECT 1 FROM distribution_job_target t \
               WHERE t.distribution_job_id = j.distribution_job_id \
                 AND t.platform = ANY($2::text[]::public.distribution_platform[]) \
           )",
    )
    .bind::<SqlUuid, _>(publisher_id)
    .bind::<Array<Text>, _>(members)
    .execute(connection)
    .map_err(Into::into)
}

// ---------------------------------------------------------------------------
// Worker operations
// ---------------------------------------------------------------------------

/// Claim a bounded batch of due distribution jobs for one worker (`T1`).
///
/// One transaction on one connection performs, in order:
///
/// 1. bounded lease recovery (`T5a`/`T5b`), so a worker's request also recovers
///    work orphaned by a crashed worker;
/// 2. the single atomic claim statement, which selects, claims, records one
///    attempt per claimed job and **returns exactly the rows it claimed**;
/// 3. two set-based payload statements for targets and attempts.
///
/// The total is a constant four statements for a claim of any size: there is no
/// per-job, per-target or per-attempt loop, and no second claim query or
/// read-back of "recently claimed" rows.
///
/// `worker` is the authenticated machine identity, never a caller-supplied
/// input. `limit` and `lease_seconds` are **clamped** rather than rejected: a
/// worker that asks for slightly too much should still make bounded progress,
/// because erroring would put a long-running automated process into a retry loop
/// that delivers nothing. Fail-closed applies to authorization and to state
/// transitions, not to a sizing argument.
pub(crate) fn claim_distribution_jobs(
    db: &PgPool,
    worker: &str,
    limit: i32,
    lease_seconds: i32,
    kinds: &[DistributionJobKind],
) -> ThothResult<Vec<ClaimedDistributionJob>> {
    if limit <= 0 {
        // An explicit request for nothing claims nothing, and performs no
        // database work at all.
        return Ok(Vec::new());
    }
    let batch = limit.min(DISTRIBUTION_JOB_CLAIM_MAX_BATCH);
    let lease = lease_seconds.clamp(
        DISTRIBUTION_JOB_LEASE_MIN_SECONDS,
        DISTRIBUTION_JOB_LEASE_MAX_SECONDS,
    );
    let kind_labels: Vec<String> = kinds.iter().map(|kind| kind.to_string()).collect();
    let worker = worker.to_string();

    let mut connection = db.get()?;
    connection.transaction(|connection| {
        // Step A. Lease recovery precedes selection.
        recover_expired_leases(connection)?;

        // Step B. The claim, as one atomic statement.
        let claim_sql = format!(
            "WITH eligible AS ( \
                 SELECT j.distribution_job_id \
                 FROM distribution_job j \
                 WHERE j.status = 'PENDING' \
                   AND j.available_at <= CURRENT_TIMESTAMP \
                   AND j.attempt_count < $1 \
                   AND (cardinality($2::text[]) = 0 \
                        OR j.kind = ANY($2::text[]::public.distribution_job_kind[])) \
                   AND NOT EXISTS ( \
                       SELECT 1 \
                       FROM distribution_job_target t \
                       WHERE t.distribution_job_id = j.distribution_job_id \
                         AND NOT EXISTS ( \
                             SELECT 1 \
                             FROM publisher_distribution_platform p \
                             WHERE p.publisher_id = j.publisher_id \
                               AND p.platform = t.platform \
                               AND p.enabled \
                               AND p.activation_id = j.activation_id \
                         ) \
                   ) \
                 ORDER BY j.available_at ASC, j.distribution_job_id ASC \
                 FOR UPDATE OF j SKIP LOCKED \
                 LIMIT $3 \
             ), \
             claimed AS ( \
                 UPDATE distribution_job j \
                 SET status = 'RUNNING', \
                     claim_token = public.uuid_generate_v4(), \
                     claimed_by = $4, \
                     claimed_at = CURRENT_TIMESTAMP, \
                     lease_expires_at = CURRENT_TIMESTAMP + ($5 * interval '1 second'), \
                     attempt_count = j.attempt_count + 1 \
                 FROM eligible e \
                 WHERE j.distribution_job_id = e.distribution_job_id \
                 RETURNING {JOB_COLUMNS_QUALIFIED} \
             ), \
             inserted_attempts AS ( \
                 INSERT INTO distribution_job_attempt \
                     (distribution_job_id, attempt_number, claim_token, claimed_by, started_at) \
                 SELECT c.distribution_job_id, c.attempt_count, c.claim_token, \
                        c.claimed_by, c.claimed_at \
                 FROM claimed c \
                 RETURNING distribution_job_id, attempt_number \
             ) \
             SELECT {JOB_COLUMNS_FROM_CLAIMED}, a.attempt_number \
             FROM claimed c \
             JOIN inserted_attempts a ON a.distribution_job_id = c.distribution_job_id \
             ORDER BY c.available_at ASC, c.distribution_job_id ASC"
        );

        let rows: Vec<ClaimRow> = diesel::sql_query(claim_sql)
            .bind::<Integer, _>(DISTRIBUTION_JOB_MAX_ATTEMPTS)
            .bind::<Array<Text>, _>(kind_labels)
            .bind::<BigInt, _>(i64::from(batch))
            .bind::<Text, _>(worker)
            .bind::<Integer, _>(lease)
            .load(connection)?;

        if rows.is_empty() {
            // Zero claims is not an error and not a null row: it is an empty
            // result, and it issues no payload statements at all.
            return Ok(Vec::new());
        }

        // Statements 3 and 4. Two set-based reads over the whole claimed
        // identity set, bounded by `DISTRIBUTION_JOB_CLAIM_MAX_BATCH`. There is
        // deliberately no per-job, per-target or per-attempt loop, and the
        // request-local `RequestLoaders` are not used on this path.
        let job_ids: Vec<Uuid> = rows.iter().map(|row| row.job.distribution_job_id).collect();
        let mut targets = partition_by_job(targets_for_jobs(connection, &job_ids)?, |target| {
            target.distribution_job_id
        });
        let mut attempts = partition_by_job(attempts_for_jobs(connection, &job_ids)?, |attempt| {
            attempt.distribution_job_id
        });

        rows.into_iter()
            .map(|row| {
                // Both are non-null on a RUNNING row by
                // `distribution_job_claim_state_check`; the fallible mapping
                // exists so a violated invariant fails closed rather than
                // panicking.
                let claim_token = row.job.claim_token.ok_or(ThothError::InternalError(
                    "claimed distribution job has no claim token".to_string(),
                ))?;
                let lease_expires_at = row.job.lease_expires_at.ok_or(
                    ThothError::InternalError("claimed distribution job has no lease".to_string()),
                )?;
                let job_id = row.job.distribution_job_id;
                Ok(ClaimedDistributionJob {
                    job: DistributionJobPayload::preloaded(
                        row.job,
                        targets.remove(&job_id).unwrap_or_default(),
                        attempts.remove(&job_id).unwrap_or_default(),
                    ),
                    claim_token,
                    lease_expires_at,
                    attempt_number: row.attempt_number,
                })
            })
            .collect()
    })
}

/// Recover every expired lease this call is willing to handle (`T5a`/`T5b`).
///
/// Expiry is not self-executing: PostgreSQL runs no timer, and `BE-04` adds no
/// scheduler, background task or cron. Recovery happens exactly when it can be
/// useful — when a worker is asking for work.
///
/// The `T5a`/`T5b` split is decided **inside the statement** from the row's own
/// `attempt_count`, so no read-then-decide race exists, and `attempt_count` is
/// written in neither branch: the abandoned attempt already consumed its
/// ordinal at claim time, and rewriting the count here would either refund a
/// used attempt or double-count it.
///
/// `FOR UPDATE SKIP LOCKED` means two workers racing recovery of the same job
/// preserve exactly one transition: the second never sees the row while the
/// first holds it, and afterwards the row no longer satisfies
/// `status = 'RUNNING'`.
fn recover_expired_leases(connection: &mut PgConnection) -> ThothResult<usize> {
    diesel::sql_query(
        "WITH expired AS ( \
             SELECT distribution_job_id, claim_token, attempt_count \
             FROM distribution_job \
             WHERE status = 'RUNNING' \
               AND lease_expires_at <= CURRENT_TIMESTAMP \
             ORDER BY lease_expires_at ASC, distribution_job_id ASC \
             FOR UPDATE SKIP LOCKED \
             LIMIT $1 \
         ), \
         closed AS ( \
             UPDATE distribution_job_attempt a \
             SET finished_at = CURRENT_TIMESTAMP, \
                 result = 'ABANDONED' \
             FROM expired e \
             WHERE a.claim_token = e.claim_token \
               AND a.finished_at IS NULL \
             RETURNING a.distribution_job_id \
         ) \
         UPDATE distribution_job j \
         SET status = CASE \
                 WHEN e.attempt_count >= $2 THEN 'FAILED'::public.distribution_job_status \
                 ELSE 'PENDING'::public.distribution_job_status \
             END, \
             claim_token = NULL, \
             claimed_by = NULL, \
             claimed_at = NULL, \
             lease_expires_at = NULL, \
             available_at = CASE \
                 WHEN e.attempt_count >= $2 THEN j.available_at \
                 ELSE CURRENT_TIMESTAMP \
             END, \
             completed_at = CASE \
                 WHEN e.attempt_count >= $2 THEN CURRENT_TIMESTAMP \
                 ELSE NULL \
             END \
         FROM expired e \
         WHERE j.distribution_job_id = e.distribution_job_id",
    )
    .bind::<BigInt, _>(i64::from(DISTRIBUTION_JOB_LEASE_RECOVERY_BATCH))
    .bind::<Integer, _>(DISTRIBUTION_JOB_MAX_ATTEMPTS)
    .execute(connection)
    .map_err(Into::into)
}

/// Record successful completion of a claimed job (`T2`).
///
/// Clears every claim field, clears `last_error_*` and closes the open attempt
/// with `result = 'SUCCEEDED'`.
pub(crate) fn complete_distribution_job(
    db: &PgPool,
    distribution_job_id: Uuid,
    claim_token: Uuid,
) -> ThothResult<DistributionJob> {
    let mut connection = db.get()?;
    connection.transaction(|connection| {
        let sql = format!(
            "UPDATE distribution_job j \
             SET status = 'SUCCEEDED', \
                 completed_at = CURRENT_TIMESTAMP, \
                 claim_token = NULL, \
                 claimed_by = NULL, \
                 claimed_at = NULL, \
                 lease_expires_at = NULL, \
                 last_error_code = NULL, \
                 last_error_detail = NULL \
             WHERE j.distribution_job_id = $1 \
               AND j.status = 'RUNNING' \
               AND j.claim_token = $2 \
             RETURNING {JOB_COLUMNS_QUALIFIED}"
        );
        let updated: Option<DistributionJob> = diesel::sql_query(sql)
            .bind::<SqlUuid, _>(distribution_job_id)
            .bind::<SqlUuid, _>(claim_token)
            .get_result(connection)
            .optional()?;

        let Some(job) = updated else {
            return Err(classify_worker_write_failure(
                connection,
                distribution_job_id,
            ));
        };

        close_open_attempt(connection, claim_token, "SUCCEEDED", None, None)?;
        Ok(job)
    })
}

/// Record failure of a claimed job, optionally scheduling a retry (`T3`/`T4`).
///
/// The branch is decided in the one statement from the row's own
/// `attempt_count`: a retryable failure within budget returns the job to
/// `PENDING` with a computed absolute `available_at`, and anything else
/// terminalizes it to `FAILED`. `attempt_count` is not rewritten — the
/// increment belongs to the claim alone, so the attempt ordinal and the count
/// can never disagree.
///
/// `error_code` is validated **before** any state transition is attempted, so a
/// malformed code changes no job or attempt state and leaves the caller's claim
/// token valid.
pub(crate) fn fail_distribution_job(
    db: &PgPool,
    distribution_job_id: Uuid,
    claim_token: Uuid,
    error_code: &str,
    error_detail: Option<&str>,
    retryable: bool,
) -> ThothResult<DistributionJob> {
    // Before any transaction is opened, and therefore before any state
    // transition is attempted.
    validate_error_code(error_code)?;
    let detail = error_detail.and_then(sanitize_error_detail);
    let error_code = error_code.to_string();

    let mut connection = db.get()?;
    connection.transaction(|connection| {
        let sql = format!(
            "UPDATE distribution_job j \
             SET status = CASE \
                     WHEN $3 AND j.attempt_count < $4 \
                     THEN 'PENDING'::public.distribution_job_status \
                     ELSE 'FAILED'::public.distribution_job_status \
                 END, \
                 completed_at = CASE \
                     WHEN $3 AND j.attempt_count < $4 THEN NULL \
                     ELSE CURRENT_TIMESTAMP \
                 END, \
                 available_at = CASE \
                     WHEN $3 AND j.attempt_count < $4 \
                     THEN CURRENT_TIMESTAMP + ( \
                         LEAST( \
                             $5::numeric * (2::numeric ^ (j.attempt_count - 1)::numeric), \
                             $6::numeric \
                         )::bigint * interval '1 second' \
                     ) \
                     ELSE j.available_at \
                 END, \
                 claim_token = NULL, \
                 claimed_by = NULL, \
                 claimed_at = NULL, \
                 lease_expires_at = NULL, \
                 last_error_code = $7, \
                 last_error_detail = $8 \
             WHERE j.distribution_job_id = $1 \
               AND j.status = 'RUNNING' \
               AND j.claim_token = $2 \
             RETURNING {JOB_COLUMNS_QUALIFIED}"
        );
        let updated: Option<DistributionJob> = diesel::sql_query(sql)
            .bind::<SqlUuid, _>(distribution_job_id)
            .bind::<SqlUuid, _>(claim_token)
            .bind::<Bool, _>(retryable)
            .bind::<Integer, _>(DISTRIBUTION_JOB_MAX_ATTEMPTS)
            .bind::<BigInt, _>(DISTRIBUTION_JOB_RETRY_BASE_SECONDS)
            .bind::<BigInt, _>(DISTRIBUTION_JOB_RETRY_MAX_SECONDS)
            .bind::<Text, _>(error_code.clone())
            .bind::<Nullable<Text>, _>(detail.clone())
            .get_result(connection)
            .optional()?;

        let Some(job) = updated else {
            return Err(classify_worker_write_failure(
                connection,
                distribution_job_id,
            ));
        };

        close_open_attempt(connection, claim_token, "FAILED", Some(error_code), detail)?;
        Ok(job)
    })
}

/// Cancel a pending or running job administratively (`T6`/`T7`).
///
/// One atomic statement takes the row, closes any open attempt with
/// `result = 'CANCELLED'` and clears every claim field, which invalidates the
/// holder's token immediately. A `PENDING` job has no token, so no attempt
/// matches and none is closed.
///
/// `last_error_*` are neither set nor cleared: cancellation is a withdrawal, not
/// a failure report.
///
/// This deletes no job row, no target row and no attempt row, and it cannot undo
/// an external delivery already performed.
pub(crate) fn cancel_distribution_job(
    db: &PgPool,
    distribution_job_id: Uuid,
) -> ThothResult<DistributionJob> {
    let mut connection = db.get()?;
    connection.transaction(|connection| {
        let sql = format!(
            "WITH target AS ( \
                 SELECT distribution_job_id, claim_token \
                 FROM distribution_job \
                 WHERE distribution_job_id = $1 \
                   AND status IN ('PENDING', 'RUNNING') \
                 FOR UPDATE \
             ), \
             closed AS ( \
                 UPDATE distribution_job_attempt a \
                 SET finished_at = CURRENT_TIMESTAMP, \
                     result = 'CANCELLED' \
                 FROM target t \
                 WHERE a.claim_token = t.claim_token \
                   AND a.finished_at IS NULL \
                 RETURNING a.distribution_job_id \
             ) \
             UPDATE distribution_job j \
             SET status = 'CANCELLED', \
                 completed_at = CURRENT_TIMESTAMP, \
                 cancellation_reason = 'ADMINISTRATIVE', \
                 claim_token = NULL, \
                 claimed_by = NULL, \
                 claimed_at = NULL, \
                 lease_expires_at = NULL \
             FROM target t \
             WHERE j.distribution_job_id = t.distribution_job_id \
             RETURNING {JOB_COLUMNS_QUALIFIED}"
        );
        let updated: Option<DistributionJob> = diesel::sql_query(sql)
            .bind::<SqlUuid, _>(distribution_job_id)
            .get_result(connection)
            .optional()?;

        updated.ok_or_else(|| classify_worker_write_failure(connection, distribution_job_id))
    })
}

/// Close the one open attempt bound to `claim_token`.
///
/// `UNIQUE (claim_token)` binds a token to exactly one attempt row for all time,
/// and the `finished_at IS NULL` predicate makes closure write-once: a repeat
/// affects zero rows and can never close a newer attempt.
fn close_open_attempt(
    connection: &mut PgConnection,
    claim_token: Uuid,
    result: &str,
    error_code: Option<String>,
    error_detail: Option<String>,
) -> ThothResult<usize> {
    let sql = format!(
        "UPDATE distribution_job_attempt \
         SET finished_at = CURRENT_TIMESTAMP, \
             result = '{result}'::public.distribution_job_attempt_result, \
             error_code = $2, \
             error_detail = $3 \
         WHERE claim_token = $1 \
           AND finished_at IS NULL"
    );
    diesel::sql_query(sql)
        .bind::<SqlUuid, _>(claim_token)
        .bind::<Nullable<Text>, _>(error_code)
        .bind::<Nullable<Text>, _>(error_detail)
        .execute(connection)
        .map_err(Into::into)
}

/// Classify a worker write that affected zero rows, reading the row inside the
/// same transaction.
///
/// `PENDING` and "held by another worker" deliberately produce the **same**
/// error: distinguishing them would tell a caller whether another worker
/// currently holds the job, which it has no need to know and which is exactly
/// the information that makes a stale caller retry aggressively.
fn classify_worker_write_failure(
    connection: &mut PgConnection,
    distribution_job_id: Uuid,
) -> ThothError {
    match distribution_job::table
        .filter(distribution_job::distribution_job_id.eq(distribution_job_id))
        .select(distribution_job::status)
        .first::<DistributionJobStatus>(connection)
        .optional()
    {
        Ok(None) => ThothError::EntityNotFound,
        Ok(Some(status)) if status.is_terminal() => {
            ThothError::DistributionJobAlreadyTerminal(status.to_string())
        }
        Ok(Some(_)) => ThothError::StaleDistributionJobClaim,
        Err(error) => error.into(),
    }
}

// ---------------------------------------------------------------------------
// Worker input contracts
// ---------------------------------------------------------------------------

/// Validate a worker-reported classification code.
///
/// The code is **validated rather than truncated**, because a truncated
/// classification code is worse than no code: clients switch on it. The
/// rejection changes no job or attempt state, and the returned error never
/// echoes the rejected value.
pub(crate) fn validate_error_code(error_code: &str) -> ThothResult<()> {
    let mut characters = error_code.chars();
    let valid = match characters.next() {
        Some(first) if first.is_ascii_uppercase() => characters.all(|character| {
            character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_'
        }),
        _ => false,
    };
    if !valid || error_code.chars().count() > DISTRIBUTION_JOB_ERROR_CODE_MAX_CHARS {
        return Err(ThothError::InvalidDistributionJobErrorCode);
    }
    Ok(())
}

/// Sanitize and bound a worker-reported diagnostic.
///
/// Exactly: remove ASCII control characters other than newline and tab, trim
/// leading and trailing whitespace, then keep the first
/// [`DISTRIBUTION_JOB_ERROR_DETAIL_MAX_CHARS`] **Unicode scalar values** — using
/// character boundaries, never a byte slice, so no partial UTF-8 sequence can be
/// produced.
///
/// The detail is **truncated, never rejected**: a worker reporting a genuine
/// failure must not have its report refused over length, because the diagnostic
/// is best-effort while the state transition is not.
///
/// The server does **not** attempt to detect or scrub secrets from free text,
/// and this function does not claim to. Pattern-matching for credentials
/// produces false negatives that create false assurance and false positives that
/// destroy diagnostics; the contract is an obligation on the writer, enforced at
/// review of `DIS-02`.
pub(crate) fn sanitize_error_detail(error_detail: &str) -> Option<String> {
    let stripped: String = error_detail
        .chars()
        .filter(|character| !character.is_control() || *character == '\n' || *character == '\t')
        .collect();
    let trimmed = stripped.trim();
    let truncated: String = trimmed
        .chars()
        .take(DISTRIBUTION_JOB_ERROR_DETAIL_MAX_CHARS)
        .collect();
    if truncated.is_empty() {
        None
    } else {
        Some(truncated)
    }
}

// ---------------------------------------------------------------------------
// Set-based reads for the staff report's DataLoaders and the claim payload
// ---------------------------------------------------------------------------

/// The latest `PUBLISHER_BACK_CATALOGUE` job of every requested publisher, in
/// one set-based statement.
///
/// The `DISTINCT ON` order is a **total** order — `publisher_id`, then
/// `created_at DESC`, then the job id `DESC` — so the selected row is
/// deterministic even for two jobs sharing a `created_at`.
/// `distribution_job_publisher_latest_idx` supports exactly this lookup.
pub(crate) fn latest_back_catalogue_jobs(
    connection: &mut PgConnection,
    publisher_ids: &[Uuid],
) -> ThothResult<Vec<DistributionJob>> {
    if publisher_ids.is_empty() {
        return Ok(Vec::new());
    }
    distribution_job::table
        .filter(distribution_job::publisher_id.eq_any(publisher_ids))
        .filter(distribution_job::kind.eq(DistributionJobKind::PublisherBackCatalogue))
        .distinct_on(distribution_job::publisher_id)
        .order((
            distribution_job::publisher_id.asc(),
            distribution_job::created_at.desc(),
            distribution_job::distribution_job_id.desc(),
        ))
        .load::<DistributionJob>(connection)
        .map_err(Into::into)
}

/// Every target of every requested job, in one set-based statement, ordered so
/// that partitioning per parent preserves canonical platform order.
pub(crate) fn targets_for_jobs(
    connection: &mut PgConnection,
    job_ids: &[Uuid],
) -> ThothResult<Vec<DistributionJobTarget>> {
    if job_ids.is_empty() {
        return Ok(Vec::new());
    }
    distribution_job_target::table
        .filter(distribution_job_target::distribution_job_id.eq_any(job_ids))
        .order((
            distribution_job_target::distribution_job_id.asc(),
            distribution_job_target::platform.asc(),
        ))
        .load::<DistributionJobTarget>(connection)
        .map_err(Into::into)
}

/// Every attempt of every requested job, in one set-based statement, most
/// recent first within each parent.
///
/// Loading them whole is safe by construction rather than by hope: attempts are
/// hard-bounded at five per job by `distribution_job_attempt_count_check`.
pub(crate) fn attempts_for_jobs(
    connection: &mut PgConnection,
    job_ids: &[Uuid],
) -> ThothResult<Vec<DistributionJobAttempt>> {
    if job_ids.is_empty() {
        return Ok(Vec::new());
    }
    distribution_job_attempt::table
        .filter(distribution_job_attempt::distribution_job_id.eq_any(job_ids))
        .order((
            distribution_job_attempt::distribution_job_id.asc(),
            distribution_job_attempt::attempt_number.desc(),
        ))
        .load::<DistributionJobAttempt>(connection)
        .map_err(Into::into)
}

/// Partition rows carrying a parent job id into a map, preserving input order
/// within each parent.
pub(crate) fn partition_by_job<T, F>(rows: Vec<T>, key: F) -> HashMap<Uuid, Vec<T>>
where
    F: Fn(&T) -> Uuid,
{
    let mut partitioned: HashMap<Uuid, Vec<T>> = HashMap::new();
    for row in rows {
        partitioned.entry(key(&row)).or_default().push(row);
    }
    partitioned
}

/// `CURRENT_TIMESTAMP`, which is transaction-scoped rather than statement-scoped.
fn current_timestamp() -> diesel::expression::SqlLiteral<diesel::sql_types::Timestamptz> {
    diesel::dsl::sql::<diesel::sql_types::Timestamptz>("CURRENT_TIMESTAMP")
}
