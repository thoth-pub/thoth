//! Reads and operational signals (spec sections 10.7 and 15).
//!
//! Every read checks that it runs READ COMMITTED and returns stored JSON as
//! its `jsonb` text. No read returns a claim token (S4): jobs come back as
//! [`AsyncJob`] and attempts as [`AsyncJobAttempt`], neither of which has
//! one. The signals are set-based queries over bounded windows, each served
//! by one of the engine's indexes.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{
    Array, BigInt, Double, Int2, Int8, Nullable, Text, Timestamptz, Uuid as SqlUuid,
};
use diesel::QueryableByName;
use serde::Serialize;
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use super::registry::AsyncJobRegistry;
use super::{
    check_isolation, clock_now, db, read_job, AsyncJob, AsyncJobAttempt, AsyncJobStatus,
    AsyncJobTransition, JobRow, ATTEMPT_COLUMNS, JOB_COLUMNS, LIST_LIMIT_MAX,
};

/// How many blocked jobs one kind's signal lists (the count is exact).
pub(crate) const BLOCKED_LIST_LIMIT: i64 = 100;

// ---------------------------------------------------------------------------
// Single-job reads
// ---------------------------------------------------------------------------

/// One job, or `ASYNC_JOB_NOT_FOUND`.
pub fn get_job(conn: &mut PgConnection, job_id: Uuid) -> ThothResult<AsyncJob> {
    check_isolation(conn)?;
    read_job(conn, job_id)?
        .map(AsyncJob::from)
        .ok_or(ThothError::AsyncJobNotFound)
}

/// The job's attempts, by attempt number.
pub fn job_attempts(conn: &mut PgConnection, job_id: Uuid) -> ThothResult<Vec<AsyncJobAttempt>> {
    check_isolation(conn)?;
    db(sql_query(format!(
        "SELECT {ATTEMPT_COLUMNS} FROM public.async_job_attempt a \
          WHERE a.async_job_id = $1 ORDER BY a.attempt_number"
    ))
    .bind::<SqlUuid, _>(job_id)
    .load::<AsyncJobAttempt>(conn))
}

/// The job's transition log, in order.
pub fn job_transitions(
    conn: &mut PgConnection,
    job_id: Uuid,
) -> ThothResult<Vec<AsyncJobTransition>> {
    check_isolation(conn)?;
    db(sql_query(
        "SELECT async_job_transition_id, async_job_id, from_status, to_status, cause, \
                async_job_attempt_id, async_control_command_id, occurred_at \
           FROM public.async_job_transition \
          WHERE async_job_id = $1 ORDER BY async_job_transition_id",
    )
    .bind::<SqlUuid, _>(job_id)
    .load::<AsyncJobTransition>(conn))
}

// ---------------------------------------------------------------------------
// The job list (keyset, newest first)
// ---------------------------------------------------------------------------

/// Filters of [`list_jobs`]: kinds OR, statuses OR, one target key and one
/// correlation id, ANDed across filters. Empty lists do not filter.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AsyncJobFilter {
    pub kinds: Vec<String>,
    pub statuses: Vec<AsyncJobStatus>,
    pub target_key: Option<String>,
    pub correlation_id: Option<Uuid>,
}

/// A keyset position: list the jobs strictly after it, newest first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JobCursor {
    pub created_at: DateTime<Utc>,
    pub async_job_id: Uuid,
}

impl From<&AsyncJob> for JobCursor {
    fn from(job: &AsyncJob) -> Self {
        JobCursor {
            created_at: job.created_at,
            async_job_id: job.async_job_id,
        }
    }
}

/// Jobs keyset-paged by `(created_at, async_job_id)` descending, at most
/// `limit` (1 to 100).
pub fn list_jobs(
    conn: &mut PgConnection,
    filter: &AsyncJobFilter,
    after: Option<JobCursor>,
    limit: i64,
) -> ThothResult<Vec<AsyncJob>> {
    if !(1..=LIST_LIMIT_MAX).contains(&limit) {
        return Err(ThothError::AsyncJobPayloadInvalid(
            "limit must be between 1 and 100".to_string(),
        ));
    }
    let texts = filter.kinds.iter().chain(filter.target_key.as_ref());
    if texts.into_iter().any(|value| value.contains('\0')) {
        return Err(ThothError::AsyncJobPayloadInvalid(
            "a filter contains U+0000".to_string(),
        ));
    }
    check_isolation(conn)?;
    let statuses: Vec<&str> = filter.statuses.iter().map(|s| s.as_str()).collect();
    let rows = db(sql_query(format!(
        "SELECT {JOB_COLUMNS} FROM public.async_job j \
          WHERE (cardinality($1::text[]) = 0 OR j.kind = ANY($1::text[])) \
            AND (cardinality($2::text[]) = 0 OR j.status::text = ANY($2::text[])) \
            AND ($3::text IS NULL OR EXISTS ( \
                   SELECT 1 FROM public.async_job_target t \
                    WHERE t.async_job_id = j.async_job_id AND t.target_key = $3::text)) \
            AND ($4::uuid IS NULL OR j.correlation_id = $4::uuid) \
            AND ($5::timestamptz IS NULL \
                 OR (j.created_at, j.async_job_id) < ($5::timestamptz, $6::uuid)) \
          ORDER BY j.created_at DESC, j.async_job_id DESC \
          LIMIT $7"
    ))
    .bind::<Array<Text>, _>(&filter.kinds)
    .bind::<Array<Text>, _>(&statuses)
    .bind::<Nullable<Text>, _>(filter.target_key.as_deref())
    .bind::<Nullable<SqlUuid>, _>(filter.correlation_id)
    .bind::<Nullable<Timestamptz>, _>(after.map(|c| c.created_at))
    .bind::<Nullable<SqlUuid>, _>(after.map(|c| c.async_job_id))
    .bind::<BigInt, _>(limit)
    .load::<JobRow>(conn))?;
    Ok(rows.into_iter().map(AsyncJob::from).collect())
}

// ---------------------------------------------------------------------------
// Kind signals (section 15)
// ---------------------------------------------------------------------------

/// A ready job that waits because another job holds one of its targets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BlockedJob {
    pub async_job_id: Uuid,
    pub target_key: String,
    pub holder_job_id: Uuid,
    pub holder_kind: String,
}

/// A target whose accepted regression awaits an ordinary write that reaches
/// the pre-acceptance revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RegressedTarget {
    pub target_key: String,
    pub applied_order: Option<i64>,
    pub applied_fingerprint: String,
    pub regressed_at: DateTime<Utc>,
    pub regressed_from_order: Option<i64>,
    pub regressed_from_fingerprint: Option<String>,
}

/// The operational signals of one kind (section 15). Counts are exact; lists
/// are bounded.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct KindSignals {
    pub kind: String,
    /// The kind is in this binary's registry.
    pub known: bool,
    /// Claimable now (section 9.1).
    pub ready_count: i64,
    pub oldest_ready_at: Option<DateTime<Utc>>,
    pub oldest_ready_age_seconds: Option<i64>,
    pub running_count: i64,
    pub running_past_lease_count: i64,
    pub retry_scheduled_count: i64,
    pub waiting_count: i64,
    pub oldest_waiting_deadline: Option<DateTime<Utc>>,
    pub reconciliation_required_count: i64,
    /// Since the oldest RR job last entered RR.
    pub oldest_reconciliation_required_age_seconds: Option<i64>,
    /// `RECONCILIATION_EXHAUSTED`: unclaimed RR jobs at the reconciliation
    /// cap, which only staff can resolve.
    pub reconciliation_exhausted_count: i64,
    /// Ready jobs blocked by a target held by another job.
    pub blocked_count: i64,
    /// At most 100 of them, each with the key's holder.
    pub blocked: Vec<BlockedJob>,
    /// Rows past their lease or waiting deadline for more than 10 minutes.
    pub overdue_count: i64,
    /// Rows with a current recovery-failure entry.
    pub recovery_failure_count: i64,
    pub oldest_recovery_failure_at: Option<DateTime<Utc>>,
    pub recovery_failure_codes: Vec<String>,
    /// Ready jobs whose payload version no executor polled for in the last
    /// hour.
    pub ready_unpolled_count: i64,
    pub unpolled_versions: Vec<i16>,
    /// Non-terminal jobs whose kind is unknown to this binary.
    pub non_terminal_unknown_kind_count: i64,
    /// Non-terminal jobs of a known kind whose version is not executable.
    pub non_terminal_version_not_executable_count: i64,
    /// Targets last written by this kind with `regressed_at` set.
    pub regressed_targets: Vec<RegressedTarget>,
    pub succeeded_last_24h: i64,
    pub failed_last_24h: i64,
    pub cancelled_last_24h: i64,
    /// Over attempts of this kind finished in the last 24 hours.
    pub attempt_duration_p50_seconds: Option<f64>,
    pub attempt_duration_p95_seconds: Option<f64>,
}

impl KindSignals {
    fn new(kind: &str, known: bool) -> Self {
        KindSignals {
            kind: kind.to_string(),
            known,
            ready_count: 0,
            oldest_ready_at: None,
            oldest_ready_age_seconds: None,
            running_count: 0,
            running_past_lease_count: 0,
            retry_scheduled_count: 0,
            waiting_count: 0,
            oldest_waiting_deadline: None,
            reconciliation_required_count: 0,
            oldest_reconciliation_required_age_seconds: None,
            reconciliation_exhausted_count: 0,
            blocked_count: 0,
            blocked: Vec::new(),
            overdue_count: 0,
            recovery_failure_count: 0,
            oldest_recovery_failure_at: None,
            recovery_failure_codes: Vec::new(),
            ready_unpolled_count: 0,
            unpolled_versions: Vec::new(),
            non_terminal_unknown_kind_count: 0,
            non_terminal_version_not_executable_count: 0,
            regressed_targets: Vec::new(),
            succeeded_last_24h: 0,
            failed_last_24h: 0,
            cancelled_last_24h: 0,
            attempt_duration_p50_seconds: None,
            attempt_duration_p95_seconds: None,
        }
    }
}

/// "Claimable now" (section 9.1) over alias `j` and the instant `$1`.
const READY: &str = "((j.status IN ('PENDING', 'RETRY_SCHEDULED') \
       AND j.execution_claim_count < j.max_execution_claims) \
      OR (j.status = 'WAITING' AND j.waiting_deadline > $1)) \
     AND j.available_at <= $1";

#[derive(QueryableByName)]
struct KindRow {
    #[diesel(sql_type = Text)]
    kind: String,
}

#[derive(QueryableByName)]
struct StatusRow {
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = BigInt)]
    ready_count: i64,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    oldest_ready_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = BigInt)]
    running_count: i64,
    #[diesel(sql_type = BigInt)]
    running_past_lease_count: i64,
    #[diesel(sql_type = BigInt)]
    retry_scheduled_count: i64,
    #[diesel(sql_type = BigInt)]
    waiting_count: i64,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    oldest_waiting_deadline: Option<DateTime<Utc>>,
    #[diesel(sql_type = BigInt)]
    reconciliation_required_count: i64,
    #[diesel(sql_type = BigInt)]
    reconciliation_exhausted_count: i64,
    #[diesel(sql_type = BigInt)]
    overdue_count: i64,
}

#[derive(QueryableByName)]
struct OldestRow {
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    oldest: Option<DateTime<Utc>>,
}

#[derive(QueryableByName)]
struct CountByKind {
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = BigInt)]
    count: i64,
}

#[derive(QueryableByName)]
struct BlockedRow {
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = SqlUuid)]
    async_job_id: Uuid,
    #[diesel(sql_type = Text)]
    target_key: String,
    #[diesel(sql_type = SqlUuid)]
    holder_job_id: Uuid,
    #[diesel(sql_type = Text)]
    holder_kind: String,
}

#[derive(QueryableByName)]
struct FailureRow {
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = BigInt)]
    count: i64,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    oldest: Option<DateTime<Utc>>,
    #[diesel(sql_type = Array<Text>)]
    codes: Vec<String>,
}

#[derive(QueryableByName)]
struct UnpolledRow {
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = BigInt)]
    count: i64,
    #[diesel(sql_type = Array<Int2>)]
    versions: Vec<i16>,
}

#[derive(QueryableByName)]
struct VersionRow {
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = Int2)]
    payload_version: i16,
    #[diesel(sql_type = BigInt)]
    count: i64,
}

#[derive(QueryableByName)]
struct RegressedRow {
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = Text)]
    target_key: String,
    #[diesel(sql_type = Nullable<Int8>)]
    applied_order: Option<i64>,
    #[diesel(sql_type = Text)]
    applied_fingerprint: String,
    #[diesel(sql_type = Timestamptz)]
    regressed_at: DateTime<Utc>,
    #[diesel(sql_type = Nullable<Int8>)]
    regressed_from_order: Option<i64>,
    #[diesel(sql_type = Nullable<Text>)]
    regressed_from_fingerprint: Option<String>,
}

#[derive(QueryableByName)]
struct TerminalRow {
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = BigInt)]
    succeeded: i64,
    #[diesel(sql_type = BigInt)]
    failed: i64,
    #[diesel(sql_type = BigInt)]
    cancelled: i64,
}

#[derive(QueryableByName)]
struct DurationRow {
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = Nullable<Double>)]
    p50: Option<f64>,
    #[diesel(sql_type = Nullable<Double>)]
    p95: Option<f64>,
}

fn age_seconds(now: DateTime<Utc>, since: Option<DateTime<Utc>>) -> Option<i64> {
    since.map(|since| (now - since).num_seconds().max(0))
}

/// The signals of every kind present in the registry or in the table, by
/// kind.
pub fn kind_signals(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
) -> ThothResult<Vec<KindSignals>> {
    check_isolation(conn)?;
    let now = db(clock_now(conn))?;
    let mut signals: BTreeMap<String, KindSignals> = registry
        .policies()
        .iter()
        .map(|policy| (policy.kind.to_string(), KindSignals::new(policy.kind, true)))
        .collect();
    for row in db(sql_query("SELECT DISTINCT kind FROM public.async_job").load::<KindRow>(conn))? {
        let known = registry.get(&row.kind).is_some();
        signals
            .entry(row.kind.clone())
            .or_insert_with(|| KindSignals::new(&row.kind, known));
    }

    // States of non-terminal jobs.
    let rows = db(sql_query(format!(
        "SELECT j.kind, \
                count(*) FILTER (WHERE {READY}) AS ready_count, \
                min(j.available_at) FILTER (WHERE {READY}) AS oldest_ready_at, \
                count(*) FILTER (WHERE j.status = 'RUNNING') AS running_count, \
                count(*) FILTER (WHERE j.status = 'RUNNING' AND j.lease_expires_at <= $1) \
                    AS running_past_lease_count, \
                count(*) FILTER (WHERE j.status = 'RETRY_SCHEDULED') AS retry_scheduled_count, \
                count(*) FILTER (WHERE j.status = 'WAITING') AS waiting_count, \
                min(j.waiting_deadline) FILTER (WHERE j.status = 'WAITING') \
                    AS oldest_waiting_deadline, \
                count(*) FILTER (WHERE j.status = 'RECONCILIATION_REQUIRED') \
                    AS reconciliation_required_count, \
                count(*) FILTER (WHERE j.status = 'RECONCILIATION_REQUIRED' \
                                   AND j.claim_token IS NULL \
                                   AND j.reconciliation_attempt_count \
                                       >= j.max_reconciliation_attempts) \
                    AS reconciliation_exhausted_count, \
                count(*) FILTER (WHERE (j.claim_token IS NOT NULL \
                                        AND j.lease_expires_at <= $1 - interval '10 minutes') \
                                    OR (j.status = 'WAITING' \
                                        AND j.waiting_deadline <= $1 - interval '10 minutes')) \
                    AS overdue_count \
           FROM public.async_job j \
          WHERE j.status IN ('PENDING', 'RUNNING', 'WAITING', 'RETRY_SCHEDULED', \
                             'RECONCILIATION_REQUIRED') \
          GROUP BY j.kind"
    ))
    .bind::<Timestamptz, _>(now)
    .load::<StatusRow>(conn))?;
    for row in rows {
        if let Some(s) = signals.get_mut(&row.kind) {
            s.ready_count = row.ready_count;
            s.oldest_ready_at = row.oldest_ready_at;
            s.oldest_ready_age_seconds = age_seconds(now, row.oldest_ready_at);
            s.running_count = row.running_count;
            s.running_past_lease_count = row.running_past_lease_count;
            s.retry_scheduled_count = row.retry_scheduled_count;
            s.waiting_count = row.waiting_count;
            s.oldest_waiting_deadline = row.oldest_waiting_deadline;
            s.reconciliation_required_count = row.reconciliation_required_count;
            s.reconciliation_exhausted_count = row.reconciliation_exhausted_count;
            s.overdue_count = row.overdue_count;
        }
    }

    // RR age: since each RR job last entered RR (the transition log).
    let rows = db(sql_query(
        "SELECT j.kind, min(e.entered_at) AS oldest \
           FROM public.async_job j \
           CROSS JOIN LATERAL ( \
                SELECT max(x.occurred_at) AS entered_at FROM public.async_job_transition x \
                 WHERE x.async_job_id = j.async_job_id \
                   AND x.to_status = 'RECONCILIATION_REQUIRED') e \
          WHERE j.status = 'RECONCILIATION_REQUIRED' \
          GROUP BY j.kind",
    )
    .load::<OldestRow>(conn))?;
    for row in rows {
        if let Some(s) = signals.get_mut(&row.kind) {
            s.oldest_reconciliation_required_age_seconds = age_seconds(now, row.oldest);
        }
    }

    // Ready jobs blocked by another job's hold, with the holder's kind.
    let blocked_from = format!(
        "FROM public.async_job j \
           JOIN public.async_job_target t ON t.async_job_id = j.async_job_id \
           JOIN public.async_target_hold h \
             ON h.target_key = t.target_key AND h.async_job_id <> j.async_job_id \
           JOIN public.async_job hj ON hj.async_job_id = h.async_job_id \
          WHERE {READY}"
    );
    let rows = db(sql_query(format!(
        "SELECT j.kind, count(DISTINCT j.async_job_id) AS count {blocked_from} GROUP BY j.kind"
    ))
    .bind::<Timestamptz, _>(now)
    .load::<CountByKind>(conn))?;
    for row in rows {
        if let Some(s) = signals.get_mut(&row.kind) {
            s.blocked_count = row.count;
        }
    }
    let rows = db(sql_query(format!(
        "SELECT kind, async_job_id, target_key, holder_job_id, holder_kind FROM ( \
            SELECT j.kind, j.async_job_id, t.target_key, h.async_job_id AS holder_job_id, \
                   hj.kind AS holder_kind, \
                   row_number() OVER (PARTITION BY j.kind \
                                      ORDER BY j.available_at, j.async_job_id, t.target_key) \
                       AS rank \
              {blocked_from}) b \
          WHERE rank <= $2 \
          ORDER BY kind, rank"
    ))
    .bind::<Timestamptz, _>(now)
    .bind::<BigInt, _>(BLOCKED_LIST_LIMIT)
    .load::<BlockedRow>(conn))?;
    for row in rows {
        if let Some(s) = signals.get_mut(&row.kind) {
            s.blocked.push(BlockedJob {
                async_job_id: row.async_job_id,
                target_key: row.target_key,
                holder_job_id: row.holder_job_id,
                holder_kind: row.holder_kind,
            });
        }
    }

    // Current recovery-failure entries (section 7.9): the job is due and the
    // entry is from its current due period.
    let rows = db(sql_query(
        "SELECT j.kind, count(*) AS count, min(f.first_failed_at) AS oldest, \
                array_agg(DISTINCT f.error_code ORDER BY f.error_code) AS codes \
           FROM public.async_recovery_failure f \
           JOIN public.async_job j ON j.async_job_id = f.async_job_id \
          WHERE ((j.claim_token IS NOT NULL AND j.lease_expires_at <= $1) \
                 OR (j.status = 'WAITING' AND j.waiting_deadline <= $1)) \
            AND f.last_failed_at >= COALESCE(j.lease_expires_at, j.waiting_deadline) \
          GROUP BY j.kind",
    )
    .bind::<Timestamptz, _>(now)
    .load::<FailureRow>(conn))?;
    for row in rows {
        if let Some(s) = signals.get_mut(&row.kind) {
            s.recovery_failure_count = row.count;
            s.oldest_recovery_failure_at = row.oldest;
            s.recovery_failure_codes = row.codes;
        }
    }

    // Ready jobs whose version no executor polled for in the last hour.
    let rows = db(sql_query(format!(
        "SELECT j.kind, count(*) AS count, \
                array_agg(DISTINCT j.payload_version ORDER BY j.payload_version) AS versions \
           FROM public.async_job j \
          WHERE {READY} \
            AND NOT EXISTS ( \
                  SELECT 1 FROM public.async_executor_poll p \
                   WHERE p.kind = j.kind AND p.purpose = 'EXECUTION' \
                     AND p.last_poll_at >= $1 - interval '1 hour' \
                     AND j.payload_version = ANY(p.payload_versions)) \
          GROUP BY j.kind"
    ))
    .bind::<Timestamptz, _>(now)
    .load::<UnpolledRow>(conn))?;
    for row in rows {
        if let Some(s) = signals.get_mut(&row.kind) {
            s.ready_unpolled_count = row.count;
            s.unpolled_versions = row.versions;
        }
    }

    // Non-terminal jobs whose kind is unknown or whose version is not
    // executable: premature removals from the registry.
    let rows = db(sql_query(
        "SELECT j.kind, j.payload_version, count(*) AS count \
           FROM public.async_job j \
          WHERE j.status IN ('PENDING', 'RUNNING', 'WAITING', 'RETRY_SCHEDULED', \
                             'RECONCILIATION_REQUIRED') \
          GROUP BY j.kind, j.payload_version",
    )
    .load::<VersionRow>(conn))?;
    for row in rows {
        let policy = registry.get(&row.kind);
        if let Some(s) = signals.get_mut(&row.kind) {
            match policy {
                None => s.non_terminal_unknown_kind_count += row.count,
                Some(policy) if !policy.is_executable_version(row.payload_version) => {
                    s.non_terminal_version_not_executable_count += row.count
                }
                Some(_) => {}
            }
        }
    }

    // Accepted regressions awaiting an ordinary write.
    let rows = db(sql_query(
        "SELECT j.kind, s.target_key, s.applied_order, s.applied_fingerprint, s.regressed_at, \
                s.regressed_from_order, s.regressed_from_fingerprint \
           FROM public.async_effect_target_state s \
           JOIN public.async_job j ON j.async_job_id = s.last_job_id \
          WHERE s.regressed_at IS NOT NULL \
          ORDER BY j.kind, s.target_key",
    )
    .load::<RegressedRow>(conn))?;
    for row in rows {
        if let Some(s) = signals.get_mut(&row.kind) {
            s.regressed_targets.push(RegressedTarget {
                target_key: row.target_key,
                applied_order: row.applied_order,
                applied_fingerprint: row.applied_fingerprint,
                regressed_at: row.regressed_at,
                regressed_from_order: row.regressed_from_order,
                regressed_from_fingerprint: row.regressed_from_fingerprint,
            });
        }
    }

    // Terminal counts in the last 24 hours. `updated_at >= terminal_at`, so
    // the `(status, updated_at)` index bounds the scan.
    let rows = db(sql_query(
        "SELECT j.kind, \
                count(*) FILTER (WHERE j.status = 'SUCCEEDED') AS succeeded, \
                count(*) FILTER (WHERE j.status = 'FAILED') AS failed, \
                count(*) FILTER (WHERE j.status = 'CANCELLED') AS cancelled \
           FROM public.async_job j \
          WHERE j.status IN ('SUCCEEDED', 'FAILED', 'CANCELLED') \
            AND j.updated_at >= $1 - interval '24 hours' \
            AND j.terminal_at >= $1 - interval '24 hours' \
          GROUP BY j.kind",
    )
    .bind::<Timestamptz, _>(now)
    .load::<TerminalRow>(conn))?;
    for row in rows {
        if let Some(s) = signals.get_mut(&row.kind) {
            s.succeeded_last_24h = row.succeeded;
            s.failed_last_24h = row.failed;
            s.cancelled_last_24h = row.cancelled;
        }
    }

    // Attempt durations over attempts finished in the last 24 hours.
    let rows = db(sql_query(
        "SELECT j.kind, \
                percentile_cont(0.5) WITHIN GROUP ( \
                    ORDER BY EXTRACT(EPOCH FROM a.finished_at - a.started_at)::double precision) \
                    AS p50, \
                percentile_cont(0.95) WITHIN GROUP ( \
                    ORDER BY EXTRACT(EPOCH FROM a.finished_at - a.started_at)::double precision) \
                    AS p95 \
           FROM public.async_job_attempt a \
           JOIN public.async_job j ON j.async_job_id = a.async_job_id \
          WHERE a.finished_at IS NOT NULL AND a.finished_at >= $1 - interval '24 hours' \
          GROUP BY j.kind",
    )
    .bind::<Timestamptz, _>(now)
    .load::<DurationRow>(conn))?;
    for row in rows {
        if let Some(s) = signals.get_mut(&row.kind) {
            s.attempt_duration_p50_seconds = row.p50;
            s.attempt_duration_p95_seconds = row.p95;
        }
    }

    Ok(signals.into_values().collect())
}

// ---------------------------------------------------------------------------
// Executor activity (section 15)
// ---------------------------------------------------------------------------

/// One executor identity's activity on one kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, QueryableByName)]
pub struct ExecutorActivity {
    #[diesel(sql_type = Text)]
    pub executor_name: String,
    #[diesel(sql_type = Text)]
    pub executor_principal: String,
    #[diesel(sql_type = Text)]
    pub kind: String,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub last_execution_poll_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub last_execution_claim_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub last_reconciliation_poll_at: Option<DateTime<Utc>>,
    /// The latest heartbeat of its open attempts of the kind.
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub latest_open_heartbeat_at: Option<DateTime<Utc>>,
}

/// Per `(executor_name, executor_principal, kind)`: the last poll and claim
/// of its `EXECUTION` row, the last poll of its `RECONCILIATION` row, and
/// the latest heartbeat of its open attempts.
pub fn executor_activity(conn: &mut PgConnection) -> ThothResult<Vec<ExecutorActivity>> {
    check_isolation(conn)?;
    db(sql_query(
        "WITH open_attempts AS ( \
             SELECT a.claimed_by AS executor_name, a.executor_principal, j.kind, \
                    max(a.last_heartbeat_at) AS latest_heartbeat \
               FROM public.async_job_attempt a \
               JOIN public.async_job j ON j.async_job_id = a.async_job_id \
              WHERE a.finished_at IS NULL \
                AND a.purpose IN ('EXECUTION', 'RECONCILIATION') \
              GROUP BY a.claimed_by, a.executor_principal, j.kind), \
         identities AS ( \
             SELECT executor_name, executor_principal, kind FROM public.async_executor_poll \
             UNION \
             SELECT executor_name, executor_principal, kind FROM open_attempts) \
         SELECT i.executor_name, i.executor_principal, i.kind, \
                e.last_poll_at AS last_execution_poll_at, \
                e.last_claim_at AS last_execution_claim_at, \
                r.last_poll_at AS last_reconciliation_poll_at, \
                o.latest_heartbeat AS latest_open_heartbeat_at \
           FROM identities i \
           LEFT JOIN public.async_executor_poll e \
             ON e.executor_name = i.executor_name \
            AND e.executor_principal = i.executor_principal \
            AND e.kind = i.kind AND e.purpose = 'EXECUTION' \
           LEFT JOIN public.async_executor_poll r \
             ON r.executor_name = i.executor_name \
            AND r.executor_principal = i.executor_principal \
            AND r.kind = i.kind AND r.purpose = 'RECONCILIATION' \
           LEFT JOIN open_attempts o \
             ON o.executor_name = i.executor_name \
            AND o.executor_principal = i.executor_principal \
            AND o.kind = i.kind \
          ORDER BY i.executor_name, i.executor_principal, i.kind",
    )
    .load::<ExecutorActivity>(conn))
}
