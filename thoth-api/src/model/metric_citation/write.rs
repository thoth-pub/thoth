//! Citation write path (`CITATIONS-01`, thoth#1074), slice C2.
//!
//! Four protected operations, all under the existing managed-DRIVER claim
//! (`claim_metric_source_units`) and all requiring `METRICS_INGEST_SERVICE`
//! (checked by the resolver) and the pinned publisher's `METRICS_COLLECT`
//! capability (checked here from canonical state):
//!
//! ```text
//! claim_metric_source_units        existing; leases one account's checkpoint
//!   -> begin_citation_refresh      create or recover the one RUNNING refresh
//!   -> citation_refresh_works      (query) the next works to look up
//!   -> submit_citation_batch       links and per-work outcomes, replay safe
//!   -> complete_citation_refresh   all withdrawal, as a guarded sweep
//! ```
//!
//! Every operation locks the account's checkpoint first (`require_live_claim`),
//! then the account, source, platform and publisher `FOR SHARE`
//! (`check_eligibility`), the lifecycle's fixed order. A stale, foreign or
//! expired token fails closed before anything is read or written.
//!
//! The checkpoint's `cursor` is never written: it is a closed manifest type
//! whose decode failure would stop every later claim. Progress is recorded in
//! `last_discovered_at` and `last_completed_at` only.
//!
//! Withdrawal happens only in [`complete_citation_refresh`], never per batch,
//! and only for a work that was matched, fully enumerated (the distinct ids
//! seen equal the declared total) and that passes the fixed server guard.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use chrono::NaiveDate;
use diesel::pg::PgConnection;
use diesel::sql_types::{
    Array, BigInt, Bool, Date, Integer, Jsonb, Nullable, SmallInt, Text, Uuid as SqlUuid,
};
use diesel::{sql_query, Connection, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

use super::MetricCitationRefreshStatus;
use crate::db::PgPool;
use crate::model::metric_ingestion_lifecycle::{
    check_eligibility, require_live_claim, LifecycleResult, MetricLifecycleError,
};
use crate::model::metric_source::{MetricSource, MetricSourceAcquisitionType};
use crate::model::metric_source_account::MetricSourceAccount;
use crate::schema::{metric_source, metric_source_account};

/// The only source driver these operations serve.
pub const CITATION_DRIVER_KEY: &str = "openalex-citations";
/// Works one batch may carry.
pub const MAX_WORKS_PER_BATCH: usize = 50;
/// Links one batch may carry, all works together.
pub const MAX_LINKS_PER_BATCH: usize = 2_000;
/// Serialized payload bytes one batch may carry.
pub const MAX_BATCH_PAYLOAD_BYTES: usize = 1_048_576;
/// Works one claim may look up before `done` is reported.
pub const MAX_WORKS_PER_REFRESH: i64 = 400;
/// Largest page `citation_refresh_works` returns.
pub const MAX_WORKS_PAGE: i32 = 200;
/// A work looked up more recently than this is not asked for again.
pub const REFRESH_MIN_AGE_DAYS: i32 = 7;
/// The guard: a previous count of at most this many is never guarded.
pub const GUARD_SMALL_COUNT: i32 = 2;
/// The guard: the links seen must be at least this percentage of the previous
/// count, else nothing is withdrawn.
pub const GUARD_MIN_PERCENT: i64 = 50;

const MAX_ID_CHARS: usize = 200;
const MAX_TITLE_CHARS: usize = 1_000;
const MAX_VENUE_CHARS: usize = 500;
const MAX_TYPE_CHARS: usize = 64;
const MAX_AUTHOR_CHARS: usize = 200;
const MAX_AUTHOR_NAMES: usize = 5;
const MAX_URL_CHARS: usize = 2_048;
const MAX_DOI_CHARS: usize = 300;
/// Names are dropped from the end until the encoded names fit, so the stored
/// object stays under the 2,048-byte CHECK on its column size.
const MAX_AUTHOR_NAME_BYTES: usize = 1_400;
const MAX_BATCH_KEY_CHARS: usize = 200;
const ABSENT_KEY: i64 = -1;

// --------------------------------------------------------------------------
// Transport inputs and outputs
// --------------------------------------------------------------------------

/// What the driver found for one work.
#[derive(juniper::GraphQLEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[graphql(description = "What the driver found for one work")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CitationLookupStatus {
    #[graphql(description = "The source knows the work; its citing works were read")]
    Matched,
    #[graphql(description = "The source does not know the work's DOI")]
    NotFound,
    #[graphql(
        description = "The driver could not finish the lookup (transport or upstream failure)"
    )]
    Failed,
}

/// One citing work as the source reports it.
#[derive(juniper::GraphQLInputObject, Debug, Clone, PartialEq, Eq, Serialize)]
#[graphql(
    description = "One work that cites a Thoth work, as the source reports it. All text is plain text"
)]
pub struct CitationLinkInput {
    #[graphql(
        description = "The source's own id of the citing work. Non-blank and at most 200 characters after sanitising, else the link is refused but still counted as read"
    )]
    pub citing_external_id: String,
    #[graphql(description = "DOI of the citing work, with or without a doi.org prefix")]
    pub citing_doi: Option<String>,
    pub citing_title: Option<String>,
    pub citing_type: Option<String>,
    pub citing_publication_year: Option<i32>,
    pub citing_publication_date: Option<NaiveDate>,
    pub citing_venue: Option<String>,
    #[graphql(description = "Up to the first five author names")]
    pub citing_author_names: Option<Vec<String>>,
    #[graphql(description = "Total number of authors the source reports")]
    pub citing_authors_total: Option<i32>,
    #[graphql(description = "https URL of the citing work")]
    pub citing_url: Option<String>,
}

/// One work's outcome and links in a batch.
#[derive(juniper::GraphQLInputObject, Debug, Clone, PartialEq, Eq, Serialize)]
#[graphql(description = "One work's lookup outcome and the citing works read in this batch")]
pub struct CitationWorkEntryInput {
    pub work_id: Uuid,
    pub match_status: CitationLookupStatus,
    #[graphql(description = "The source's id of the cited work. Required when MATCHED")]
    pub external_work_id: Option<String>,
    #[graphql(description = "The source's own cited-by count, if reported")]
    pub reported_count: Option<i32>,
    #[graphql(description = "Citing works read in this batch. Empty unless MATCHED")]
    pub links: Vec<CitationLinkInput>,
    #[graphql(
        description = "True only when the driver has now read every citing work of this work, across all of its batches"
    )]
    pub enumeration_complete: bool,
    #[graphql(
        description = "When complete: the number of distinct citing ids read in all, including ones that could not be sent. Required when complete"
    )]
    pub enumerated_total: Option<i32>,
}

#[derive(juniper::GraphQLInputObject, Debug, Clone, PartialEq, Eq)]
#[graphql(description = "A claimed source account and its lease token")]
pub struct BeginCitationRefreshInput {
    pub source_account_code: String,
    pub lease_token: Uuid,
}

#[derive(juniper::GraphQLInputObject, Debug, Clone, PartialEq, Eq)]
#[graphql(description = "Which works of a running refresh to look up next")]
pub struct CitationRefreshWorksInput {
    pub source_account_code: String,
    pub lease_token: Uuid,
    pub refresh_id: Uuid,
    #[graphql(description = "Page size, 1 to 200")]
    pub limit: Option<i32>,
    #[graphql(description = "The nextAfter value of the previous page")]
    pub after: Option<String>,
}

#[derive(juniper::GraphQLInputObject, Debug, Clone, PartialEq, Eq)]
#[graphql(
    description = "One batch of a running refresh: at most 50 works, 2000 links and 1 MiB. A work may span several batches"
)]
pub struct SubmitCitationBatchInput {
    pub source_account_code: String,
    pub lease_token: Uuid,
    pub refresh_id: Uuid,
    #[graphql(
        description = "Idempotency key, unique within the refresh. Non-blank, at most 200 characters"
    )]
    pub batch_key: String,
    pub works: Vec<CitationWorkEntryInput>,
}

#[derive(juniper::GraphQLInputObject, Debug, Clone, PartialEq, Eq)]
#[graphql(description = "Which refresh to complete, and under which claim")]
pub struct CompleteCitationRefreshInput {
    pub source_account_code: String,
    pub lease_token: Uuid,
    pub refresh_id: Uuid,
}

/// A running or finished refresh.
#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq, Eq)]
#[graphql(description = "One claimed citation refresh run")]
pub struct CitationRefresh {
    pub refresh_id: Uuid,
    pub status: MetricCitationRefreshStatus,
    pub works_seen: i32,
    pub works_matched: i32,
    pub works_not_found: i32,
    pub links_upserted: i32,
    pub links_withdrawn: i32,
    pub links_refused: i32,
    pub links_merged: i32,
}

/// One work to look up.
#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq, Eq)]
#[graphql(description = "One work to look up at the source")]
pub struct CitationRefreshWork {
    pub work_id: Uuid,
    #[graphql(description = "The work's DOI, lower-case, without a doi.org prefix")]
    pub doi: String,
}

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq, Eq)]
#[graphql(description = "A page of works to look up")]
pub struct CitationRefreshWorksPage {
    pub works: Vec<CitationRefreshWork>,
    #[graphql(
        description = "True when no qualifying work remains or this claim's cap of 400 works is reached; the driver then completes the refresh"
    )]
    pub done: bool,
    #[graphql(description = "Pass as `after` for the next page; null when there is none")]
    pub next_after: Option<String>,
}

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq, Eq)]
#[graphql(description = "The committed outcome of one citation batch")]
pub struct CitationBatchResult {
    pub refresh_id: Uuid,
    pub batch_key: String,
    #[graphql(description = "True when this exact batch had already been committed")]
    pub replayed: bool,
    pub works_accepted: i32,
    pub links_upserted: i32,
    pub links_refused: i32,
    pub links_merged: i32,
}

// --------------------------------------------------------------------------
// Boundary sanitising (Thoth is authoritative; the driver mirrors it)
// --------------------------------------------------------------------------

fn is_bidi_control(c: char) -> bool {
    matches!(c, '\u{061C}' | '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
}

/// Strip NUL, C0/C1 controls and bidi controls, turn every other whitespace
/// into one space and collapse runs, normalise to NFC. No truncation.
fn sanitise(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut pending_space = false;
    for c in raw.chars() {
        if c.is_whitespace() {
            pending_space = true;
        } else if c == '\0' || c.is_control() || is_bidi_control(c) {
            continue;
        } else {
            if pending_space && !out.is_empty() {
                out.push(' ');
            }
            pending_space = false;
            out.push(c);
        }
    }
    out.nfc().collect()
}

/// Sanitise, then truncate to `max_chars` characters. `None` when nothing is
/// left.
fn clean_text(raw: &str, max_chars: usize) -> Option<String> {
    let cleaned = sanitise(raw);
    if cleaned.is_empty() {
        return None;
    }
    Some(cleaned.chars().take(max_chars).collect())
}

/// A DOI in the stored shape: lower-case, no resolver prefix, `10.` first.
fn normalise_doi(raw: &str) -> Option<String> {
    let cleaned = sanitise(raw).to_lowercase();
    let mut doi = cleaned.trim();
    for prefix in [
        "https://doi.org/",
        "http://doi.org/",
        "https://dx.doi.org/",
        "http://dx.doi.org/",
        "doi:",
    ] {
        if let Some(rest) = doi.strip_prefix(prefix) {
            doi = rest.trim();
            break;
        }
    }
    let shape = static_regex(&DOI_SHAPE, r"^10\.[^\s]+$");
    (shape.is_match(doi) && doi.chars().count() <= MAX_DOI_CHARS).then(|| doi.to_string())
}

static DOI_SHAPE: OnceLock<Regex> = OnceLock::new();
static URL_SHAPE: OnceLock<Regex> = OnceLock::new();

fn static_regex(cell: &'static OnceLock<Regex>, pattern: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pattern).expect("a constant pattern compiles"))
}

/// An https URL without userinfo, within the length bound; else `None`.
fn clean_url(raw: &str) -> Option<String> {
    let url = sanitise(raw);
    if url.chars().count() > MAX_URL_CHARS {
        return None;
    }
    static_regex(&URL_SHAPE, r"^https://[^/?#@\s]+([/?#]\S*)?$")
        .is_match(&url)
        .then_some(url)
}

fn author_object(names: &[String], total: Option<i32>) -> Option<JsonValue> {
    let mut kept: Vec<String> = names
        .iter()
        .filter_map(|name| clean_text(name, MAX_AUTHOR_CHARS))
        .take(MAX_AUTHOR_NAMES)
        .collect();
    while kept.iter().map(String::len).sum::<usize>() > MAX_AUTHOR_NAME_BYTES {
        kept.pop();
    }
    let total = total
        .filter(|n| *n >= 0)
        .unwrap_or(0)
        .max(kept.len() as i32);
    if kept.is_empty() && total == 0 {
        return None;
    }
    Some(json!({ "names": kept, "total": total }))
}

/// The citing work's date for ordering: its publication date, else the first
/// day of its year, else 0001-01-01.
fn sort_date(date: Option<NaiveDate>, year: Option<i16>) -> NaiveDate {
    date.or_else(|| year.and_then(|y| NaiveDate::from_ymd_opt(i32::from(y), 1, 1)))
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(1, 1, 1).expect("0001-01-01 is a date"))
}

/// A year within the range any real publication has, else `None`.
fn plausible_year(year: Option<i32>) -> Option<i16> {
    year.filter(|y| (1000..=2999).contains(y))
        .and_then(|y| i16::try_from(y).ok())
}

// --------------------------------------------------------------------------
// Authority
// --------------------------------------------------------------------------

struct CitationAuthority {
    source_id: Uuid,
    source_account_id: Uuid,
    publisher_id: Uuid,
    checkpoint_id: Uuid,
}

fn token_hash(lease_token: Uuid) -> String {
    hex::encode(Sha256::digest(lease_token.to_string().as_bytes()))
}

/// Lock the claim and revalidate the route, in the lifecycle's lock order.
fn citation_authority(
    connection: &mut PgConnection,
    source_account_code: &str,
    lease_token: Uuid,
) -> LifecycleResult<CitationAuthority> {
    let source_account_id: Uuid = metric_source_account::table
        .filter(metric_source_account::code.eq(source_account_code))
        .select(metric_source_account::source_account_id)
        .first(connection)
        .optional()?
        .ok_or(MetricLifecycleError::SourceAccountNotFound)?;
    let checkpoint_id = require_live_claim(connection, source_account_id, lease_token)?;
    let account: MetricSourceAccount = metric_source_account::table
        .find(source_account_id)
        .for_share()
        .first(connection)?;
    let source: MetricSource = metric_source::table
        .find(account.source_id)
        .for_share()
        .first(connection)?;
    if source.acquisition_type != MetricSourceAcquisitionType::Driver
        || source.driver_key.as_deref() != Some(CITATION_DRIVER_KEY)
    {
        return Err(MetricLifecycleError::SourceNotEligible);
    }
    let route = check_eligibility(connection, &source, &account, true)?;
    Ok(CitationAuthority {
        source_id: source.source_id,
        source_account_id,
        publisher_id: route.publisher_id,
        checkpoint_id,
    })
}

#[derive(diesel::QueryableByName)]
struct RefreshRow {
    #[diesel(sql_type = SqlUuid)]
    refresh_id: Uuid,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Text)]
    claim_token_hash: String,
    #[diesel(sql_type = Integer)]
    works_seen: i32,
    #[diesel(sql_type = Integer)]
    works_matched: i32,
    #[diesel(sql_type = Integer)]
    works_not_found: i32,
    #[diesel(sql_type = Integer)]
    links_upserted: i32,
    #[diesel(sql_type = Integer)]
    links_withdrawn: i32,
    #[diesel(sql_type = Integer)]
    links_refused: i32,
    #[diesel(sql_type = Integer)]
    links_merged: i32,
}

impl RefreshRow {
    fn into_output(self) -> LifecycleResult<CitationRefresh> {
        let status = match self.status.as_str() {
            "RUNNING" => MetricCitationRefreshStatus::Running,
            "COMPLETED" => MetricCitationRefreshStatus::Completed,
            "COMPLETED_WITH_ERRORS" => MetricCitationRefreshStatus::CompletedWithErrors,
            "FAILED" => MetricCitationRefreshStatus::Failed,
            _ => {
                return Err(MetricLifecycleError::Ingestion(
                    crate::model::metric_ingestion::MetricIngestionErrorCode::InternalStateInconsistency,
                ))
            }
        };
        Ok(CitationRefresh {
            refresh_id: self.refresh_id,
            status,
            works_seen: self.works_seen,
            works_matched: self.works_matched,
            works_not_found: self.works_not_found,
            links_upserted: self.links_upserted,
            links_withdrawn: self.links_withdrawn,
            links_refused: self.links_refused,
            links_merged: self.links_merged,
        })
    }
}

const REFRESH_COLUMNS: &str = "refresh_id, status::text AS status, claim_token_hash, works_seen, \
     works_matched, works_not_found, links_upserted, links_withdrawn, links_refused, links_merged";

/// Load one refresh of this account `FOR UPDATE`.
fn lock_refresh(
    connection: &mut PgConnection,
    authority: &CitationAuthority,
    refresh_id: Uuid,
) -> LifecycleResult<RefreshRow> {
    sql_query(format!(
        "SELECT {REFRESH_COLUMNS} FROM metric_citation_refresh \
          WHERE refresh_id = $1 AND source_account_id = $2 FOR UPDATE"
    ))
    .bind::<SqlUuid, _>(refresh_id)
    .bind::<SqlUuid, _>(authority.source_account_id)
    .get_result(connection)
    .optional()?
    .ok_or(MetricLifecycleError::CitationRefreshNotFound)
}

/// A refresh this token may still write to.
fn lock_running_refresh(
    connection: &mut PgConnection,
    authority: &CitationAuthority,
    refresh_id: Uuid,
    lease_token: Uuid,
) -> LifecycleResult<RefreshRow> {
    let row = lock_refresh(connection, authority, refresh_id)?;
    if row.status != "RUNNING" || row.claim_token_hash != token_hash(lease_token) {
        return Err(MetricLifecycleError::CitationRefreshNotRunning);
    }
    Ok(row)
}

// --------------------------------------------------------------------------
// begin_citation_refresh
// --------------------------------------------------------------------------

/// Create, or recover, the one RUNNING refresh of a claim.
///
/// A refresh still RUNNING under an earlier token (its lease expired) is set
/// FAILED first; its committed batches stay committed and its seen ids are
/// removed. Repeating the call under the same claim returns the same refresh.
pub fn begin_citation_refresh(
    db: &PgPool,
    input: &BeginCitationRefreshInput,
) -> LifecycleResult<CitationRefresh> {
    let mut connection = db.get()?;
    connection.transaction(|connection| {
        let authority =
            citation_authority(connection, &input.source_account_code, input.lease_token)?;
        let hash = token_hash(input.lease_token);

        let superseded: Vec<Uuid> = sql_query(
            "UPDATE metric_citation_refresh \
                SET status = 'FAILED', completed_at = transaction_timestamp(), \
                    last_error = 'superseded by a later claim' \
              WHERE source_account_id = $1 AND status = 'RUNNING' AND claim_token_hash <> $2 \
          RETURNING refresh_id",
        )
        .bind::<SqlUuid, _>(authority.source_account_id)
        .bind::<Text, _>(&hash)
        .load::<RefreshId>(connection)?
        .into_iter()
        .map(|row| row.refresh_id)
        .collect();
        if !superseded.is_empty() {
            sql_query("DELETE FROM metric_citation_refresh_seen WHERE refresh_id = ANY($1)")
                .bind::<Array<SqlUuid>, _>(&superseded)
                .execute(connection)?;
        }

        let existing: Option<RefreshRow> = sql_query(format!(
            "SELECT {REFRESH_COLUMNS} FROM metric_citation_refresh \
              WHERE source_account_id = $1 AND status = 'RUNNING' AND claim_token_hash = $2"
        ))
        .bind::<SqlUuid, _>(authority.source_account_id)
        .bind::<Text, _>(&hash)
        .get_result(connection)
        .optional()?;
        if let Some(row) = existing {
            return row.into_output();
        }

        let row: RefreshRow = sql_query(format!(
            "INSERT INTO metric_citation_refresh (source_account_id, claim_token_hash) \
             VALUES ($1, $2) RETURNING {REFRESH_COLUMNS}"
        ))
        .bind::<SqlUuid, _>(authority.source_account_id)
        .bind::<Text, _>(&hash)
        .get_result(connection)?;
        sql_query(
            "UPDATE metric_source_checkpoint SET last_discovered_at = transaction_timestamp() \
              WHERE source_checkpoint_id = $1",
        )
        .bind::<SqlUuid, _>(authority.checkpoint_id)
        .execute(connection)?;
        row.into_output()
    })
}

#[derive(diesel::QueryableByName)]
struct RefreshId {
    #[diesel(sql_type = SqlUuid)]
    refresh_id: Uuid,
}

// --------------------------------------------------------------------------
// citation_refresh_works
// --------------------------------------------------------------------------

#[derive(diesel::QueryableByName)]
struct QueueRow {
    #[diesel(sql_type = SqlUuid)]
    work_id: Uuid,
    #[diesel(sql_type = Text)]
    doi: String,
    #[diesel(sql_type = BigInt)]
    sort_key: i64,
}

fn parse_after(after: &str) -> LifecycleResult<(i64, Uuid)> {
    let (key, id) = after
        .split_once(':')
        .ok_or(MetricLifecycleError::LifecycleLimitExceeded)?;
    if after.len() > 64 {
        return Err(MetricLifecycleError::LifecycleLimitExceeded);
    }
    Ok((
        key.parse::<i64>()
            .map_err(|_| MetricLifecycleError::LifecycleLimitExceeded)?,
        Uuid::parse_str(id).map_err(|_| MetricLifecycleError::LifecycleLimitExceeded)?,
    ))
}

/// The next works of the claimed publisher to look up.
///
/// A qualifying work has a DOI, is not withdrawn or cancelled, and was not
/// attempted in the last seven days (a work submitted in this refresh, of any
/// outcome, was attempted just now, so a loop terminates and a poison DOI is
/// never asked twice). Order is never-attempted first, then oldest attempt,
/// then work id. `done` is true when nothing qualifying remains or the
/// claim's cap of 400 works is reached.
pub fn citation_refresh_works(
    db: &PgPool,
    input: &CitationRefreshWorksInput,
) -> LifecycleResult<CitationRefreshWorksPage> {
    let limit = input
        .limit
        .unwrap_or(MAX_WORKS_PAGE)
        .clamp(1, MAX_WORKS_PAGE);
    let after = input.after.as_deref().map(parse_after).transpose()?;
    let mut connection = db.get()?;
    connection.transaction(|connection| {
        let authority = citation_authority(connection, &input.source_account_code, input.lease_token)?;
        let refresh = lock_running_refresh(connection, &authority, input.refresh_id, input.lease_token)?;
        let remaining = (MAX_WORKS_PER_REFRESH - i64::from(refresh.works_seen)).max(0);
        if remaining == 0 {
            return Ok(CitationRefreshWorksPage {
                works: Vec::new(),
                done: true,
                next_after: None,
            });
        }
        let page = i64::from(limit).min(remaining);
        let (after_key, after_id) = after.unwrap_or((i64::MIN, Uuid::nil()));
        let rows: Vec<QueueRow> = sql_query(format!(
            "SELECT w.work_id, \
                    lower(regexp_replace(w.doi, '^(https?://(dx\\.)?doi\\.org/|doi:)', '', 'i')) AS doi, \
                    CASE WHEN s.last_attempted_at IS NULL THEN {ABSENT_KEY}::bigint \
                         ELSE (extract(epoch FROM s.last_attempted_at) * 1000000)::bigint END AS sort_key \
               FROM work w \
               JOIN imprint i ON i.imprint_id = w.imprint_id \
               LEFT JOIN metric_citation_work_state s \
                      ON s.cited_work_id = w.work_id AND s.source_id = $2 \
              WHERE i.publisher_id = $1 \
                AND w.doi IS NOT NULL \
                AND w.work_status NOT IN ('withdrawn', 'cancelled') \
                AND (s.last_attempted_at IS NULL \
                     OR s.last_attempted_at < transaction_timestamp() - ($3 * interval '1 day')) \
                AND (CASE WHEN s.last_attempted_at IS NULL THEN {ABSENT_KEY}::bigint \
                          ELSE (extract(epoch FROM s.last_attempted_at) * 1000000)::bigint END, \
                     w.work_id) > ($4, $5) \
              ORDER BY sort_key, w.work_id \
              LIMIT $6"
        ))
        .bind::<SqlUuid, _>(authority.publisher_id)
        .bind::<SqlUuid, _>(authority.source_id)
        .bind::<Integer, _>(REFRESH_MIN_AGE_DAYS)
        .bind::<BigInt, _>(after_key)
        .bind::<SqlUuid, _>(after_id)
        .bind::<BigInt, _>(page + 1)
        .load(connection)?;
        let more = rows.len() as i64 > page;
        let rows: Vec<QueueRow> = rows.into_iter().take(page as usize).collect();
        let next_after = (more && !rows.is_empty())
            .then(|| rows.last().map(|r| format!("{}:{}", r.sort_key, r.work_id)))
            .flatten();
        let reaches_cap = i64::from(refresh.works_seen) + rows.len() as i64 >= MAX_WORKS_PER_REFRESH;
        Ok(CitationRefreshWorksPage {
            done: !more || reaches_cap,
            works: rows
                .into_iter()
                .map(|r| CitationRefreshWork {
                    work_id: r.work_id,
                    doi: r.doi,
                })
                .collect(),
            next_after,
        })
    })
}

// --------------------------------------------------------------------------
// submit_citation_batch
// --------------------------------------------------------------------------

/// What one batch recorded about one work: all `complete_citation_refresh`
/// needs to decide, kept in the batch's committed outcome.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct WorkOutcome {
    work_id: Uuid,
    status: CitationLookupStatus,
    complete: bool,
    declared_total: Option<i32>,
    /// Ids read but not storable (blank or over-long): counted, never stored.
    unidentified: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct BatchOutcome {
    works: Vec<WorkOutcome>,
    works_accepted: i32,
    links_upserted: i32,
    links_refused: i32,
    links_merged: i32,
}

fn validate_batch(input: &SubmitCitationBatchInput) -> LifecycleResult<()> {
    use MetricLifecycleError::LifecycleLimitExceeded as Limit;
    let key = sanitise(&input.batch_key);
    if key.is_empty() || key.chars().count() > MAX_BATCH_KEY_CHARS || key != input.batch_key {
        return Err(Limit);
    }
    if input.works.is_empty() || input.works.len() > MAX_WORKS_PER_BATCH {
        return Err(Limit);
    }
    let links: usize = input.works.iter().map(|w| w.links.len()).sum();
    if links > MAX_LINKS_PER_BATCH {
        return Err(Limit);
    }
    let payload = serde_json::to_vec(&input.works).map_err(|_| Limit)?;
    if payload.len() > MAX_BATCH_PAYLOAD_BYTES {
        return Err(Limit);
    }
    for work in &input.works {
        if work.reported_count.is_some_and(|n| n < 0)
            || work.enumerated_total.is_some_and(|n| n < 0)
        {
            return Err(Limit);
        }
        match work.match_status {
            CitationLookupStatus::Matched => {
                let id = work
                    .external_work_id
                    .as_deref()
                    .and_then(|s| clean_text(s, usize::MAX));
                if id.is_none_or(|id| id.chars().count() > MAX_ID_CHARS) {
                    return Err(Limit);
                }
                if work.enumeration_complete && work.enumerated_total.is_none() {
                    return Err(Limit);
                }
            }
            CitationLookupStatus::NotFound | CitationLookupStatus::Failed => {
                if !work.links.is_empty() {
                    return Err(Limit);
                }
            }
        }
    }
    Ok(())
}

fn payload_hash(input: &SubmitCitationBatchInput) -> LifecycleResult<String> {
    let bytes = serde_json::to_vec(&(input.refresh_id, &input.batch_key, &input.works))
        .map_err(|_| MetricLifecycleError::LifecycleLimitExceeded)?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

#[derive(diesel::QueryableByName)]
struct LinkRow {
    #[diesel(sql_type = SqlUuid)]
    link_id: Uuid,
    #[diesel(sql_type = Text)]
    citing_external_id: String,
}

#[derive(diesel::QueryableByName)]
struct DoiRow {
    #[diesel(sql_type = SqlUuid)]
    work_id: Uuid,
    #[diesel(sql_type = Nullable<Text>)]
    doi: Option<String>,
}

/// One citing work, sanitised and ready to store.
struct CleanLink {
    id: String,
    doi: Option<String>,
    title: Option<String>,
    kind: Option<String>,
    year: Option<i16>,
    date: Option<NaiveDate>,
    venue: Option<String>,
    authors: Option<JsonValue>,
    url: Option<String>,
}

fn clean_link(link: &CitationLinkInput) -> Option<CleanLink> {
    let id = clean_text(&link.citing_external_id, usize::MAX)?;
    if id.chars().count() > MAX_ID_CHARS {
        return None;
    }
    let year = plausible_year(link.citing_publication_year);
    Some(CleanLink {
        id,
        doi: link.citing_doi.as_deref().and_then(normalise_doi),
        title: link
            .citing_title
            .as_deref()
            .and_then(|t| clean_text(t, MAX_TITLE_CHARS)),
        kind: link
            .citing_type
            .as_deref()
            .and_then(|t| clean_text(t, MAX_TYPE_CHARS)),
        year,
        date: link.citing_publication_date,
        venue: link
            .citing_venue
            .as_deref()
            .and_then(|t| clean_text(t, MAX_VENUE_CHARS)),
        authors: author_object(
            link.citing_author_names.as_deref().unwrap_or(&[]),
            link.citing_authors_total,
        ),
        url: link.citing_url.as_deref().and_then(clean_url),
    })
}

enum LinkResult {
    Stored,
    Merged,
}

/// Store one sanitised link of a matched work, following the id-mapping rule.
fn store_link(
    connection: &mut PgConnection,
    authority: &CitationAuthority,
    refresh_id: Uuid,
    work_id: Uuid,
    link: &CleanLink,
) -> LifecycleResult<LinkResult> {
    let existing_by_id: Option<LinkRow> = sql_query(
        "SELECT link_id, citing_external_id \
           FROM metric_citation_link \
          WHERE cited_work_id = $1 AND source_id = $2 AND citing_external_id = $3 FOR UPDATE",
    )
    .bind::<SqlUuid, _>(work_id)
    .bind::<SqlUuid, _>(authority.source_id)
    .bind::<Text, _>(&link.id)
    .get_result(connection)
    .optional()?;
    let active_by_doi: Option<LinkRow> = match &link.doi {
        Some(doi) => sql_query(
            "SELECT link_id, citing_external_id FROM metric_citation_link \
              WHERE cited_work_id = $1 AND source_id = $2 AND citing_doi = $3 \
                AND status = 'ACTIVE' FOR UPDATE",
        )
        .bind::<SqlUuid, _>(work_id)
        .bind::<SqlUuid, _>(authority.source_id)
        .bind::<Text, _>(doi)
        .get_result(connection)
        .optional()?,
        None => None,
    };

    let touch = |connection: &mut PgConnection, link_id: Uuid| -> LifecycleResult<()> {
        sql_query(
            "UPDATE metric_citation_link \
                SET last_seen_at = transaction_timestamp(), last_refresh_id = $2 \
              WHERE link_id = $1",
        )
        .bind::<SqlUuid, _>(link_id)
        .bind::<SqlUuid, _>(refresh_id)
        .execute(connection)?;
        Ok(())
    };

    // Another ACTIVE row already carries this DOI under a different id.
    if let Some(winner) = active_by_doi
        .as_ref()
        .filter(|row| row.citing_external_id != link.id)
    {
        touch(connection, winner.link_id)?;
        let old_id_seen_this_refresh: bool = {
            #[derive(diesel::QueryableByName)]
            struct Seen {
                #[diesel(sql_type = Bool)]
                seen: bool,
            }
            sql_query(
                "SELECT EXISTS (SELECT 1 FROM metric_citation_refresh_seen \
                                 WHERE refresh_id = $1 AND work_id = $2 \
                                   AND citing_external_id = $3) AS seen",
            )
            .bind::<SqlUuid, _>(refresh_id)
            .bind::<SqlUuid, _>(work_id)
            .bind::<Text, _>(&winner.citing_external_id)
            .get_result::<Seen>(connection)?
            .seen
        };
        if !old_id_seen_this_refresh && existing_by_id.is_none() {
            // The source re-ided the work: the existing row takes the new id.
            sql_query("UPDATE metric_citation_link SET citing_external_id = $2 WHERE link_id = $1")
                .bind::<SqlUuid, _>(winner.link_id)
                .bind::<Text, _>(&link.id)
                .execute(connection)?;
            update_link_fields(connection, winner.link_id, link)?;
            return Ok(LinkResult::Stored);
        }
        return Ok(LinkResult::Merged);
    }

    if let Some(row) = existing_by_id {
        // Same id (and, when it has one, same DOI owner): update, touch and
        // reactivate. A WITHDRAWN row whose DOI is now another ACTIVE row's
        // was handled above as merged.
        update_link_fields(connection, row.link_id, link)?;
        touch(connection, row.link_id)?;
        return Ok(LinkResult::Stored);
    }

    sql_query(
        "INSERT INTO metric_citation_link \
           (cited_work_id, source_id, citing_external_id, citing_doi, citing_title, citing_type, \
            citing_publication_year, citing_publication_date, citing_sort_date, citing_venue, \
            citing_authors, citing_url, last_refresh_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
    )
    .bind::<SqlUuid, _>(work_id)
    .bind::<SqlUuid, _>(authority.source_id)
    .bind::<Text, _>(&link.id)
    .bind::<Nullable<Text>, _>(&link.doi)
    .bind::<Nullable<Text>, _>(&link.title)
    .bind::<Nullable<Text>, _>(&link.kind)
    .bind::<Nullable<SmallInt>, _>(link.year)
    .bind::<Nullable<Date>, _>(link.date)
    .bind::<Date, _>(sort_date(link.date, link.year))
    .bind::<Nullable<Text>, _>(&link.venue)
    .bind::<Nullable<Jsonb>, _>(&link.authors)
    .bind::<Nullable<Text>, _>(&link.url)
    .bind::<SqlUuid, _>(refresh_id)
    .execute(connection)?;
    Ok(LinkResult::Stored)
}

fn update_link_fields(
    connection: &mut PgConnection,
    link_id: Uuid,
    link: &CleanLink,
) -> LifecycleResult<()> {
    sql_query(
        "UPDATE metric_citation_link SET status = 'ACTIVE', withdrawn_at = NULL, \
            citing_doi = $2, citing_title = $3, citing_type = $4, citing_publication_year = $5, \
            citing_publication_date = $6, citing_sort_date = $7, citing_venue = $8, \
            citing_authors = $9, citing_url = $10 \
          WHERE link_id = $1",
    )
    .bind::<SqlUuid, _>(link_id)
    .bind::<Nullable<Text>, _>(&link.doi)
    .bind::<Nullable<Text>, _>(&link.title)
    .bind::<Nullable<Text>, _>(&link.kind)
    .bind::<Nullable<SmallInt>, _>(link.year)
    .bind::<Nullable<Date>, _>(link.date)
    .bind::<Date, _>(sort_date(link.date, link.year))
    .bind::<Nullable<Text>, _>(&link.venue)
    .bind::<Nullable<Jsonb>, _>(&link.authors)
    .bind::<Nullable<Text>, _>(&link.url)
    .execute(connection)?;
    Ok(())
}

/// Commit one batch of a running refresh.
///
/// The batch only adds or updates links (ACTIVE, touched by this refresh,
/// reactivating a WITHDRAWN one and keeping its `first_seen_at`); it never
/// withdraws. A clash of two ids on one DOI never fails the batch: the existing
/// ACTIVE row wins, else the lowest id (links are processed in id order), and
/// the other id is recorded as seen and counted in `links_merged`. Replaying the
/// same key and payload returns the committed outcome; the same key with
/// another payload is refused.
pub fn submit_citation_batch(
    db: &PgPool,
    input: &SubmitCitationBatchInput,
) -> LifecycleResult<CitationBatchResult> {
    validate_batch(input)?;
    let hash = payload_hash(input)?;
    let mut connection = db.get()?;
    connection.transaction(|connection| {
        let authority = citation_authority(connection, &input.source_account_code, input.lease_token)?;
        let _refresh = lock_running_refresh(connection, &authority, input.refresh_id, input.lease_token)?;

        #[derive(diesel::QueryableByName)]
        struct StoredBatch {
            #[diesel(sql_type = Text)]
            payload_hash: String,
            #[diesel(sql_type = Jsonb)]
            outcome: JsonValue,
        }
        let stored: Option<StoredBatch> = sql_query(
            "SELECT payload_hash, outcome FROM metric_citation_batch \
              WHERE refresh_id = $1 AND batch_key = $2",
        )
        .bind::<SqlUuid, _>(input.refresh_id)
        .bind::<Text, _>(&input.batch_key)
        .get_result(connection)
        .optional()?;
        if let Some(stored) = stored {
            if stored.payload_hash != hash {
                return Err(MetricLifecycleError::CitationBatchIdempotencyMismatch);
            }
            let outcome: BatchOutcome = serde_json::from_value(stored.outcome)
                .map_err(|_| internal_inconsistency())?;
            return Ok(batch_result(input, &outcome, true));
        }

        // Every work must be the claimed publisher's.
        let distinct: BTreeSet<Uuid> = input.works.iter().map(|w| w.work_id).collect();
        let ids: Vec<Uuid> = distinct.iter().copied().collect();
        let owned: Vec<DoiRow> = sql_query(
            "SELECT w.work_id, w.doi FROM work w JOIN imprint i ON i.imprint_id = w.imprint_id \
              WHERE w.work_id = ANY($1) AND i.publisher_id = $2",
        )
        .bind::<Array<SqlUuid>, _>(&ids)
        .bind::<SqlUuid, _>(authority.publisher_id)
        .load(connection)?;
        if owned.len() != ids.len() {
            return Err(MetricLifecycleError::CitationWorkNotInPublisher);
        }
        let own_doi: BTreeMap<Uuid, Option<String>> = owned
            .into_iter()
            .map(|row| (row.work_id, row.doi.as_deref().and_then(normalise_doi)))
            .collect();

        let mut outcome = BatchOutcome::default();
        let mut new_works = 0i32;
        let mut new_matched = 0i32;
        let mut new_not_found = 0i32;
        for entry in &input.works {
            let first_time = !work_in_refresh(connection, input.refresh_id, entry.work_id)?
                && !outcome.works.iter().any(|w| w.work_id == entry.work_id);
            if first_time {
                new_works += 1;
                match entry.match_status {
                    CitationLookupStatus::Matched => new_matched += 1,
                    CitationLookupStatus::NotFound => new_not_found += 1,
                    CitationLookupStatus::Failed => {}
                }
            }
            record_work_state(connection, &authority, entry)?;

            let mut unidentified = 0i32;
            let mut cleaned: Vec<CleanLink> = Vec::new();
            for link in &entry.links {
                match clean_link(link) {
                    Some(clean) => cleaned.push(clean),
                    None => {
                        unidentified += 1;
                        outcome.links_refused += 1;
                    }
                }
            }
            cleaned.sort_by(|a, b| a.id.cmp(&b.id));
            for link in &cleaned {
                sql_query(
                    "INSERT INTO metric_citation_refresh_seen (refresh_id, work_id, citing_external_id) \
                     VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
                )
                .bind::<SqlUuid, _>(input.refresh_id)
                .bind::<SqlUuid, _>(entry.work_id)
                .bind::<Text, _>(&link.id)
                .execute(connection)?;
                if own_doi.get(&entry.work_id).and_then(Option::as_ref) == link.doi.as_ref()
                    && link.doi.is_some()
                {
                    outcome.links_refused += 1; // self-citation: seen, never stored
                    continue;
                }
                match store_link(connection, &authority, input.refresh_id, entry.work_id, link)? {
                    LinkResult::Stored => outcome.links_upserted += 1,
                    LinkResult::Merged => outcome.links_merged += 1,
                }
            }
            outcome.works.push(WorkOutcome {
                work_id: entry.work_id,
                status: entry.match_status,
                complete: entry.enumeration_complete
                    && entry.match_status == CitationLookupStatus::Matched,
                declared_total: entry.enumerated_total,
                unidentified,
            });
        }
        outcome.works_accepted = input.works.len() as i32;

        sql_query(
            "INSERT INTO metric_citation_batch (refresh_id, batch_key, payload_hash, outcome) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind::<SqlUuid, _>(input.refresh_id)
        .bind::<Text, _>(&input.batch_key)
        .bind::<Text, _>(&hash)
        .bind::<Jsonb, _>(serde_json::to_value(&outcome).map_err(|_| internal_inconsistency())?)
        .execute(connection)?;
        sql_query(
            "UPDATE metric_citation_refresh SET \
                works_seen = works_seen + $2, works_matched = works_matched + $3, \
                works_not_found = works_not_found + $4, links_upserted = links_upserted + $5, \
                links_refused = links_refused + $6, links_merged = links_merged + $7 \
              WHERE refresh_id = $1",
        )
        .bind::<SqlUuid, _>(input.refresh_id)
        .bind::<Integer, _>(new_works)
        .bind::<Integer, _>(new_matched)
        .bind::<Integer, _>(new_not_found)
        .bind::<Integer, _>(outcome.links_upserted)
        .bind::<Integer, _>(outcome.links_refused)
        .bind::<Integer, _>(outcome.links_merged)
        .execute(connection)?;
        Ok(batch_result(input, &outcome, false))
    })
}

fn internal_inconsistency() -> MetricLifecycleError {
    MetricLifecycleError::Ingestion(
        crate::model::metric_ingestion::MetricIngestionErrorCode::InternalStateInconsistency,
    )
}

fn batch_result(
    input: &SubmitCitationBatchInput,
    outcome: &BatchOutcome,
    replayed: bool,
) -> CitationBatchResult {
    CitationBatchResult {
        refresh_id: input.refresh_id,
        batch_key: input.batch_key.clone(),
        replayed,
        works_accepted: outcome.works_accepted,
        links_upserted: outcome.links_upserted,
        links_refused: outcome.links_refused,
        links_merged: outcome.links_merged,
    }
}

/// Whether an earlier committed batch of this refresh already carried the work.
fn work_in_refresh(
    connection: &mut PgConnection,
    refresh_id: Uuid,
    work_id: Uuid,
) -> LifecycleResult<bool> {
    #[derive(diesel::QueryableByName)]
    struct Present {
        #[diesel(sql_type = Bool)]
        present: bool,
    }
    Ok(sql_query(
        "SELECT EXISTS (SELECT 1 FROM metric_citation_batch b, \
                               jsonb_array_elements(b.outcome -> 'works') w \
                         WHERE b.refresh_id = $1 AND w ->> 'workId' = $2) AS present",
    )
    .bind::<SqlUuid, _>(refresh_id)
    .bind::<Text, _>(work_id.to_string())
    .get_result::<Present>(connection)?
    .present)
}

/// The work-state effects of one entry: lookup identity and attempt times
/// only. Links, withdrawal and the enumerated count are not touched here.
fn record_work_state(
    connection: &mut PgConnection,
    authority: &CitationAuthority,
    entry: &CitationWorkEntryInput,
) -> LifecycleResult<()> {
    match entry.match_status {
        CitationLookupStatus::Matched => {
            let external = entry
                .external_work_id
                .as_deref()
                .and_then(|s| clean_text(s, MAX_ID_CHARS))
                .ok_or(MetricLifecycleError::LifecycleLimitExceeded)?;
            sql_query(
                "INSERT INTO metric_citation_work_state \
                   (cited_work_id, source_id, match_status, last_matched_at, external_work_id, \
                    reported_count, last_attempted_at) \
                 VALUES ($1, $2, 'MATCHED', transaction_timestamp(), $3, $4, transaction_timestamp()) \
                 ON CONFLICT (cited_work_id, source_id) DO UPDATE SET \
                    match_status = 'MATCHED', last_matched_at = transaction_timestamp(), \
                    external_work_id = EXCLUDED.external_work_id, \
                    reported_count = EXCLUDED.reported_count, \
                    last_attempted_at = transaction_timestamp()",
            )
            .bind::<SqlUuid, _>(entry.work_id)
            .bind::<SqlUuid, _>(authority.source_id)
            .bind::<Text, _>(external)
            .bind::<Nullable<Integer>, _>(entry.reported_count)
            .execute(connection)?;
        }
        CitationLookupStatus::NotFound => {
            // After a MATCHED the links and last matched values stay and the
            // work reads "unconfirmed"; a never-matched work is simply NOT_FOUND.
            sql_query(
                "INSERT INTO metric_citation_work_state \
                   (cited_work_id, source_id, match_status, last_attempted_at, last_refreshed_at) \
                 VALUES ($1, $2, 'NOT_FOUND', transaction_timestamp(), transaction_timestamp()) \
                 ON CONFLICT (cited_work_id, source_id) DO UPDATE SET \
                    last_attempted_at = transaction_timestamp(), \
                    last_error_at = CASE WHEN metric_citation_work_state.last_matched_at IS NOT NULL \
                                         THEN transaction_timestamp() \
                                         ELSE metric_citation_work_state.last_error_at END, \
                    unconfirmed = metric_citation_work_state.last_matched_at IS NOT NULL, \
                    last_refreshed_at = CASE WHEN metric_citation_work_state.last_matched_at IS NULL \
                                             THEN transaction_timestamp() \
                                             ELSE metric_citation_work_state.last_refreshed_at END",
            )
            .bind::<SqlUuid, _>(entry.work_id)
            .bind::<SqlUuid, _>(authority.source_id)
            .execute(connection)?;
        }
        CitationLookupStatus::Failed => {
            sql_query(
                "INSERT INTO metric_citation_work_state \
                   (cited_work_id, source_id, match_status, last_attempted_at, last_error_at, unconfirmed) \
                 VALUES ($1, $2, 'NOT_FOUND', transaction_timestamp(), transaction_timestamp(), TRUE) \
                 ON CONFLICT (cited_work_id, source_id) DO UPDATE SET \
                    last_attempted_at = transaction_timestamp(), \
                    last_error_at = transaction_timestamp(), unconfirmed = TRUE",
            )
            .bind::<SqlUuid, _>(entry.work_id)
            .bind::<SqlUuid, _>(authority.source_id)
            .execute(connection)?;
        }
    }
    Ok(())
}

// --------------------------------------------------------------------------
// complete_citation_refresh
// --------------------------------------------------------------------------

#[derive(Default)]
struct FoldedWork {
    matched: bool,
    failed: bool,
    incomplete: bool,
    declared: Option<i32>,
    unidentified: i64,
}

#[derive(diesel::QueryableByName)]
struct Count {
    #[diesel(sql_type = BigInt)]
    n: i64,
}

#[derive(diesel::QueryableByName)]
struct WorkStateRow {
    #[diesel(sql_type = Nullable<Integer>)]
    enumerated_count: Option<i32>,
    #[diesel(sql_type = Nullable<Integer>)]
    guard_held_count: Option<i32>,
}

/// Close a running refresh: the only place a link is withdrawn.
///
/// For each work that was MATCHED, fully enumerated (distinct ids seen equal
/// the declared total, counting ids that could not be stored) and passes the
/// fixed server guard, ACTIVE links not touched by this refresh become
/// WITHDRAWN. A work that is not complete is only marked `unconfirmed`. The
/// guard passes when the previous enumerated count was absent or at most 2, or
/// the links seen are at least 50% of it, or the same lower count was seen on
/// the previous complete refresh; otherwise nothing is withdrawn and the
/// refresh ends COMPLETED_WITH_ERRORS. Progress goes to `last_completed_at`
/// and the lease is released; the cursor is never written. The lease release
/// means a repeated call is refused as a stale claim; the driver reads the
/// outcome from the first response.
pub fn complete_citation_refresh(
    db: &PgPool,
    input: &CompleteCitationRefreshInput,
) -> LifecycleResult<CitationRefresh> {
    let mut connection = db.get()?;
    connection.transaction(|connection| {
        let authority =
            citation_authority(connection, &input.source_account_code, input.lease_token)?;
        let refresh =
            lock_running_refresh(connection, &authority, input.refresh_id, input.lease_token)?;

        let outcomes: Vec<JsonValue> = {
            #[derive(diesel::QueryableByName)]
            struct Outcome {
                #[diesel(sql_type = Jsonb)]
                outcome: JsonValue,
            }
            sql_query(
                "SELECT outcome FROM metric_citation_batch WHERE refresh_id = $1 \
                  ORDER BY committed_at, batch_key",
            )
            .bind::<SqlUuid, _>(input.refresh_id)
            .load::<Outcome>(connection)?
            .into_iter()
            .map(|row| row.outcome)
            .collect()
        };
        let mut folded: BTreeMap<Uuid, FoldedWork> = BTreeMap::new();
        for value in outcomes {
            let batch: BatchOutcome =
                serde_json::from_value(value).map_err(|_| internal_inconsistency())?;
            for work in batch.works {
                let entry = folded.entry(work.work_id).or_default();
                match work.status {
                    CitationLookupStatus::Matched => {
                        entry.matched = true;
                        entry.unidentified += i64::from(work.unidentified);
                        if work.complete {
                            entry.declared = work.declared_total;
                        } else {
                            entry.incomplete = true;
                        }
                    }
                    CitationLookupStatus::Failed => entry.failed = true,
                    CitationLookupStatus::NotFound => {}
                }
            }
        }

        let mut withdrawn_total = 0i32;
        for (work_id, work) in &folded {
            if !work.matched {
                continue;
            }
            let seen_ids = sql_query(
                "SELECT count(*) AS n FROM metric_citation_refresh_seen \
                  WHERE refresh_id = $1 AND work_id = $2",
            )
            .bind::<SqlUuid, _>(input.refresh_id)
            .bind::<SqlUuid, _>(*work_id)
            .get_result::<Count>(connection)?
            .n;
            let complete = !work.failed
                && !work.incomplete
                && work
                    .declared
                    .is_some_and(|d| i64::from(d) == seen_ids + work.unidentified);
            if !complete {
                mark_unconfirmed(connection, &authority, *work_id)?;
                continue;
            }
            let touched = sql_query(
                "SELECT count(*) AS n FROM metric_citation_link \
                  WHERE cited_work_id = $1 AND source_id = $2 AND status = 'ACTIVE' \
                    AND last_refresh_id = $3",
            )
            .bind::<SqlUuid, _>(*work_id)
            .bind::<SqlUuid, _>(authority.source_id)
            .bind::<SqlUuid, _>(input.refresh_id)
            .get_result::<Count>(connection)?
            .n;
            let state: WorkStateRow = sql_query(
                "SELECT enumerated_count, guard_held_count FROM metric_citation_work_state \
                  WHERE cited_work_id = $1 AND source_id = $2 FOR UPDATE",
            )
            .bind::<SqlUuid, _>(*work_id)
            .bind::<SqlUuid, _>(authority.source_id)
            .get_result(connection)?;
            let guard_holds = match state.enumerated_count {
                None => true,
                Some(previous) if previous <= GUARD_SMALL_COUNT => true,
                Some(previous) => {
                    touched * 100 >= i64::from(previous) * GUARD_MIN_PERCENT
                        || state
                            .guard_held_count
                            .is_some_and(|held| i64::from(held) == touched)
                }
            };
            if !guard_holds {
                sql_query(
                    "UPDATE metric_citation_work_state SET guard_held_count = $3, \
                        unconfirmed = TRUE, last_error_at = transaction_timestamp() \
                      WHERE cited_work_id = $1 AND source_id = $2",
                )
                .bind::<SqlUuid, _>(*work_id)
                .bind::<SqlUuid, _>(authority.source_id)
                .bind::<Integer, _>(touched as i32)
                .execute(connection)?;
                continue;
            }
            let withdrawn = sql_query(
                "UPDATE metric_citation_link SET status = 'WITHDRAWN', \
                        withdrawn_at = transaction_timestamp() \
                  WHERE cited_work_id = $1 AND source_id = $2 AND status = 'ACTIVE' \
                    AND last_refresh_id IS DISTINCT FROM $3",
            )
            .bind::<SqlUuid, _>(*work_id)
            .bind::<SqlUuid, _>(authority.source_id)
            .bind::<SqlUuid, _>(input.refresh_id)
            .execute(connection)?;
            withdrawn_total += withdrawn as i32;
            sql_query(
                "UPDATE metric_citation_work_state SET enumerated_count = $3, \
                    guard_held_count = NULL, unconfirmed = FALSE, \
                    last_refreshed_at = transaction_timestamp(), last_refresh_id = $4 \
                  WHERE cited_work_id = $1 AND source_id = $2",
            )
            .bind::<SqlUuid, _>(*work_id)
            .bind::<SqlUuid, _>(authority.source_id)
            .bind::<Integer, _>(touched as i32)
            .bind::<SqlUuid, _>(input.refresh_id)
            .execute(connection)?;
        }

        let unconfirmed = sql_query(
            "SELECT count(*) AS n FROM metric_citation_work_state s \
              WHERE s.source_id = $1 AND s.unconfirmed AND s.cited_work_id = ANY($3) \
                AND s.last_attempted_at >= (SELECT started_at FROM metric_citation_refresh \
                                              WHERE refresh_id = $2)",
        )
        .bind::<SqlUuid, _>(authority.source_id)
        .bind::<SqlUuid, _>(input.refresh_id)
        .bind::<Array<SqlUuid>, _>(folded.keys().copied().collect::<Vec<Uuid>>())
        .get_result::<Count>(connection)?
        .n;
        let status = if unconfirmed > 0 {
            "COMPLETED_WITH_ERRORS"
        } else {
            "COMPLETED"
        };
        let closed: RefreshRow = sql_query(format!(
            "UPDATE metric_citation_refresh SET status = $2::metric_citation_refresh_status, \
                    completed_at = transaction_timestamp(), links_withdrawn = $3 \
              WHERE refresh_id = $1 RETURNING {REFRESH_COLUMNS}"
        ))
        .bind::<SqlUuid, _>(input.refresh_id)
        .bind::<Text, _>(status)
        .bind::<Integer, _>(withdrawn_total + refresh.links_withdrawn)
        .get_result(connection)?;
        sql_query("DELETE FROM metric_citation_refresh_seen WHERE refresh_id = $1")
            .bind::<SqlUuid, _>(input.refresh_id)
            .execute(connection)?;
        sql_query(
            "UPDATE metric_source_checkpoint \
                SET last_completed_at = transaction_timestamp(), \
                    lease_owner = NULL, lease_expires_at = NULL \
              WHERE source_checkpoint_id = $1",
        )
        .bind::<SqlUuid, _>(authority.checkpoint_id)
        .execute(connection)?;
        closed.into_output()
    })
}

fn mark_unconfirmed(
    connection: &mut PgConnection,
    authority: &CitationAuthority,
    work_id: Uuid,
) -> LifecycleResult<()> {
    sql_query(
        "UPDATE metric_citation_work_state SET unconfirmed = TRUE, \
            last_error_at = transaction_timestamp() \
          WHERE cited_work_id = $1 AND source_id = $2",
    )
    .bind::<SqlUuid, _>(work_id)
    .bind::<SqlUuid, _>(authority.source_id)
    .execute(connection)?;
    Ok(())
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn sanitise_strips_controls_and_bidi_collapses_whitespace_and_normalises() {
        assert_eq!(sanitise("a\u{0}b\u{7}c"), "abc");
        assert_eq!(sanitise("  a \t\n b\u{a0}c  "), "a b c");
        assert_eq!(sanitise("x\u{202E}y\u{2066}z\u{200F}"), "xyz");
        assert_eq!(sanitise("e\u{301}"), "\u{e9}");
        assert_eq!(sanitise("\u{85}a\u{9f}"), "a");
    }

    #[test]
    fn text_is_truncated_by_characters_after_cleaning() {
        let long = "\u{e9}".repeat(1_100);
        assert_eq!(
            clean_text(&long, MAX_TITLE_CHARS).unwrap().chars().count(),
            1_000
        );
        assert_eq!(clean_text(" \u{0} ", 10), None);
    }

    #[test]
    fn dois_are_lowercased_unprefixed_and_shape_checked() {
        assert_eq!(
            normalise_doi("https://doi.org/10.1234/ABC"),
            Some("10.1234/abc".into())
        );
        assert_eq!(normalise_doi("DOI: 10.1/x"), Some("10.1/x".into()));
        assert_eq!(
            normalise_doi("doi:10.1/x<y>+[z]"),
            Some("10.1/x<y>+[z]".into())
        );
        assert_eq!(normalise_doi("11.1/x"), None);
        assert_eq!(normalise_doi(""), None);
    }

    #[test]
    fn urls_must_be_https_without_userinfo_within_the_bound() {
        assert!(clean_url("https://example.org/a?b=c@d").is_some());
        assert_eq!(clean_url("http://example.org"), None);
        assert_eq!(clean_url("https://user@example.org"), None);
        assert_eq!(clean_url("https://exa mple.org"), None);
        assert_eq!(
            clean_url(&format!("https://e.org/{}", "a".repeat(2_100))),
            None
        );
    }

    #[test]
    fn authors_are_capped_and_sized_for_the_column_check() {
        let names: Vec<String> = (0..9)
            .map(|i| format!("{i}{}", "\u{4e2d}".repeat(200)))
            .collect();
        let object = author_object(&names, Some(9)).unwrap();
        let kept = object["names"].as_array().unwrap();
        assert!(kept.len() <= 5);
        assert!(
            kept.iter()
                .map(|n| n.as_str().unwrap().len())
                .sum::<usize>()
                <= 1_400
        );
        assert_eq!(object["total"], 9);
        assert_eq!(author_object(&[], None), None);
    }

    #[test]
    fn sort_date_falls_back_to_year_then_the_epoch_floor() {
        let d = NaiveDate::from_ymd_opt(2020, 5, 17);
        assert_eq!(sort_date(d, Some(1999)), d.unwrap());
        assert_eq!(
            sort_date(None, Some(1999)),
            NaiveDate::from_ymd_opt(1999, 1, 1).unwrap()
        );
        assert_eq!(
            sort_date(None, None),
            NaiveDate::from_ymd_opt(1, 1, 1).unwrap()
        );
    }
}
