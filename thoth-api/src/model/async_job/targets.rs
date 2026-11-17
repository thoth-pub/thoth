//! Target holds and applied evidence (spec section 13).
//!
//! A hold row makes one job the only holder of a target key (S5). Every
//! transaction that inserts or deletes a hold for key `k` first holds
//! `pg_advisory_xact_lock(NS_HOLD, hashtext(k))` (rule H): claims and
//! recovery only try it, every other path waits for it after its job row
//! lock, ascending by lock key. Applied evidence moves only under the hold,
//! fenced by the writing attempt, and never backwards except by a staff
//! `ACCEPT_REGRESSION` (S6).

use std::collections::{BTreeMap, HashSet};

use chrono::{DateTime, Utc};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::sql_types::{Array, Int4, Int8, Nullable, Text, Timestamptz, Uuid as SqlUuid};
use diesel::{sql_query, QueryableByName};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{is_identifier, lock_keys, BoolRow, JobRow, RevisionMode, NS_HOLD};

/// One per-target completion entry (section 13.3). `order` is carried as
/// `i128` so the engine itself makes the `bigint` range check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceEntry {
    pub target_key: String,
    pub order: Option<i128>,
    pub fingerprint: Option<String>,
    pub source_ref: Option<String>,
}

impl EvidenceEntry {
    /// A key-only entry (`SUPERSEDED`, `COMPLETED`).
    pub fn key_only(target_key: impl Into<String>) -> Self {
        EvidenceEntry {
            target_key: target_key.into(),
            order: None,
            fingerprint: None,
            source_ref: None,
        }
    }
}

/// The applied state of one of a job's targets, as `read_target_state_on`
/// returns it. Every field after `target_key` is `None` when no evidence has
/// ever been written for the target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, QueryableByName)]
pub struct TargetState {
    #[diesel(sql_type = Text)]
    pub target_key: String,
    #[diesel(sql_type = Nullable<Int8>)]
    pub applied_order: Option<i64>,
    #[diesel(sql_type = Nullable<Text>)]
    pub applied_fingerprint: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub applied_source_ref: Option<String>,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    pub last_job_id: Option<Uuid>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub regressed_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Int8>)]
    pub regressed_from_order: Option<i64>,
    #[diesel(sql_type = Nullable<Text>)]
    pub regressed_from_fingerprint: Option<String>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub updated_at: Option<DateTime<Utc>>,
}

/// The applied state of every target of a job, ascending by key, including
/// targets without a state row.
pub(crate) fn read_states(conn: &mut PgConnection, job_id: Uuid) -> QueryResult<Vec<TargetState>> {
    sql_query(
        "SELECT t.target_key, s.applied_order, s.applied_fingerprint, s.applied_source_ref, \
                s.last_job_id, s.regressed_at, s.regressed_from_order, \
                s.regressed_from_fingerprint, s.updated_at \
           FROM public.async_job_target t \
           LEFT JOIN public.async_effect_target_state s ON s.target_key = t.target_key \
          WHERE t.async_job_id = $1 \
          ORDER BY t.target_key",
    )
    .bind::<SqlUuid, _>(job_id)
    .load::<TargetState>(conn)
}

// ---------------------------------------------------------------------------
// Holds (section 13.1, rule H of section 11)
// ---------------------------------------------------------------------------

/// The `NS_HOLD` lock keys of `keys`, ascending, duplicates removed.
fn hold_lock_keys(conn: &mut PgConnection, keys: &[String]) -> QueryResult<Vec<i32>> {
    Ok(lock_keys(conn, keys)?
        .into_iter()
        .map(|row| row.key)
        .collect())
}

/// Rule H, waiting form: for worker writes, reconciliation resolution and
/// operator commands, after the job row lock.
pub(crate) fn lock_holds_waiting(conn: &mut PgConnection, keys: &[String]) -> QueryResult<()> {
    for key in hold_lock_keys(conn, keys)? {
        sql_query("SELECT pg_advisory_xact_lock($1, $2)")
            .bind::<Int4, _>(NS_HOLD)
            .bind::<Int4, _>(key)
            .execute(conn)?;
    }
    Ok(())
}

/// Rule H, non-waiting form: for claims and recovery. `false` as soon as one
/// key's lock is not granted.
pub(crate) fn try_lock_holds(conn: &mut PgConnection, keys: &[String]) -> QueryResult<bool> {
    for key in hold_lock_keys(conn, keys)? {
        let granted = sql_query("SELECT pg_try_advisory_xact_lock($1, $2) AS value")
            .bind::<Int4, _>(NS_HOLD)
            .bind::<Int4, _>(key)
            .get_result::<BoolRow>(conn)?;
        if !granted.value {
            return Ok(false);
        }
    }
    Ok(true)
}

#[derive(QueryableByName)]
struct KeyRow {
    #[diesel(sql_type = Text)]
    target_key: String,
}

/// The job's target keys that it does not hold yet (a job resumed from
/// `WAITING` keeps its holds), ascending.
pub(crate) fn unheld_target_keys(
    conn: &mut PgConnection,
    job_id: Uuid,
) -> QueryResult<Vec<String>> {
    Ok(sql_query(
        "SELECT t.target_key FROM public.async_job_target t \
          WHERE t.async_job_id = $1 \
            AND NOT EXISTS (SELECT 1 FROM public.async_target_hold h \
                             WHERE h.target_key = t.target_key AND h.async_job_id = $1) \
          ORDER BY t.target_key",
    )
    .bind::<SqlUuid, _>(job_id)
    .load::<KeyRow>(conn)?
    .into_iter()
    .map(|row| row.target_key)
    .collect())
}

/// Insert holds; returns how many were inserted (fewer than `keys` means
/// another job holds one of them).
pub(crate) fn insert_holds(
    conn: &mut PgConnection,
    job_id: Uuid,
    keys: &[String],
) -> QueryResult<usize> {
    if keys.is_empty() {
        return Ok(0);
    }
    sql_query(
        "INSERT INTO public.async_target_hold (target_key, async_job_id) \
         SELECT k, $2 FROM unnest($1::text[]) AS k \
         ON CONFLICT (target_key) DO NOTHING",
    )
    .bind::<Array<Text>, _>(keys)
    .bind::<SqlUuid, _>(job_id)
    .execute(conn)
}

/// Delete every hold of the job (a releasing transition, after its state
/// rows).
pub(crate) fn delete_holds(conn: &mut PgConnection, job_id: Uuid) -> QueryResult<usize> {
    sql_query("DELETE FROM public.async_target_hold WHERE async_job_id = $1")
        .bind::<SqlUuid, _>(job_id)
        .execute(conn)
}

// ---------------------------------------------------------------------------
// Applied evidence (section 13.3)
// ---------------------------------------------------------------------------

/// The stored applied state of one target, locked `FOR UPDATE`.
#[derive(Debug, Clone, QueryableByName)]
pub(crate) struct StateRow {
    #[diesel(sql_type = Text)]
    pub target_key: String,
    #[diesel(sql_type = Nullable<Int8>)]
    pub applied_order: Option<i64>,
    #[diesel(sql_type = Text)]
    pub applied_fingerprint: String,
}

/// Lock the existing state rows of `keys` `FOR UPDATE`, ascending by key.
pub(crate) fn lock_state_rows(
    conn: &mut PgConnection,
    keys: &[String],
) -> QueryResult<BTreeMap<String, StateRow>> {
    if keys.is_empty() {
        return Ok(BTreeMap::new());
    }
    Ok(sql_query(
        "SELECT target_key, applied_order, applied_fingerprint \
           FROM public.async_effect_target_state \
          WHERE target_key = ANY($1::text[]) \
          ORDER BY target_key \
            FOR UPDATE",
    )
    .bind::<Array<Text>, _>(keys)
    .load::<StateRow>(conn)?
    .into_iter()
    .map(|row| (row.target_key.clone(), row))
    .collect())
}

/// One state row write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StateWrite {
    pub target_key: String,
    pub order: Option<i64>,
    pub fingerprint: String,
    pub source_ref: Option<String>,
}

/// Upsert state rows ascending by key, attributed to the writing job and its
/// open attempt. The trigger alone maintains `regressed_*` and `updated_at`.
pub(crate) fn write_states(
    conn: &mut PgConnection,
    writes: &[StateWrite],
    job_id: Uuid,
    attempt_id: Uuid,
) -> QueryResult<()> {
    let mut sorted: Vec<&StateWrite> = writes.iter().collect();
    sorted.sort_by(|a, b| a.target_key.cmp(&b.target_key));
    for write in sorted {
        sql_query(
            "INSERT INTO public.async_effect_target_state \
                 (target_key, applied_order, applied_fingerprint, applied_source_ref, \
                  last_job_id, last_attempt_id, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, clock_timestamp()) \
             ON CONFLICT (target_key) DO UPDATE SET \
                 applied_order = EXCLUDED.applied_order, \
                 applied_fingerprint = EXCLUDED.applied_fingerprint, \
                 applied_source_ref = EXCLUDED.applied_source_ref, \
                 last_job_id = EXCLUDED.last_job_id, \
                 last_attempt_id = EXCLUDED.last_attempt_id",
        )
        .bind::<Text, _>(&write.target_key)
        .bind::<Nullable<Int8>, _>(write.order)
        .bind::<Text, _>(&write.fingerprint)
        .bind::<Nullable<Text>, _>(write.source_ref.as_deref())
        .bind::<SqlUuid, _>(job_id)
        .bind::<SqlUuid, _>(attempt_id)
        .execute(conn)?;
    }
    Ok(())
}

/// Which row of the outcome table a call's entries follow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EntryRule {
    /// `EFFECT_APPLIED`, and the `SUCCEEDED` of T16 and T18.
    EffectApplied,
    NoOp,
    Superseded,
    Completed,
}

impl EntryRule {
    fn key_only(self) -> bool {
        matches!(self, EntryRule::Superseded | EntryRule::Completed)
    }
}

/// The shape checks of section 13.3 that need no database: U+0000 in any
/// entry string, a duplicate `targetKey`, an `order` outside the `bigint`
/// range, a fingerprint or `sourceRef` outside the identifier format, and a
/// field the outcome and revision mode require that is missing or one they do
/// not list that is present. Returns the fault's description.
pub(crate) fn check_entry_shapes(
    entries: &[EvidenceEntry],
    rule: EntryRule,
    mode: RevisionMode,
) -> Result<(), String> {
    let mut seen = HashSet::new();
    for entry in entries {
        let strings = [
            Some(entry.target_key.as_str()),
            entry.fingerprint.as_deref(),
            entry.source_ref.as_deref(),
        ];
        if strings.iter().flatten().any(|value| value.contains('\0')) {
            return Err("an entry contains U+0000".to_string());
        }
        if !seen.insert(entry.target_key.as_str()) {
            return Err("the entries repeat a targetKey".to_string());
        }
        if let Some(order) = entry.order {
            if order < i128::from(i64::MIN) || order > i128::from(i64::MAX) {
                return Err("an entry order is outside the bigint range".to_string());
            }
        }
        if let Some(fingerprint) = &entry.fingerprint {
            if !is_identifier(fingerprint) {
                return Err("an entry fingerprint is outside the identifier format".to_string());
            }
        }
        if let Some(source_ref) = &entry.source_ref {
            if !is_identifier(source_ref) {
                return Err("an entry sourceRef is outside the identifier format".to_string());
            }
        }
        let revision_bound = mode == RevisionMode::RevisionBound;
        let (order_allowed, fingerprint_allowed, source_ref_allowed) = if rule.key_only() {
            (false, false, false)
        } else {
            (revision_bound, true, rule == EntryRule::EffectApplied)
        };
        // Required: the fingerprint for every non-key-only row, and the order
        // for REVISION_BOUND.
        if !rule.key_only() && entry.fingerprint.is_none() {
            return Err("an entry lacks its fingerprint".to_string());
        }
        if !rule.key_only() && revision_bound && entry.order.is_none() {
            return Err("an entry lacks its order".to_string());
        }
        if entry.order.is_some() && !order_allowed {
            return Err("an entry carries an order its outcome does not list".to_string());
        }
        if entry.fingerprint.is_some() && !fingerprint_allowed {
            return Err("an entry carries a fingerprint its outcome does not list".to_string());
        }
        if entry.source_ref.is_some() && !source_ref_allowed {
            return Err("an entry carries a sourceRef its outcome does not list".to_string());
        }
    }
    Ok(())
}

/// A semantic fault in the entries (section 13.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SemanticFault {
    /// A stored `applied_order` greater than the entry's order, for
    /// `EFFECT_APPLIED` (and reconciliation `SUCCEEDED`).
    Regression,
    /// Any other break of the table, the key set included.
    Invalid(String),
}

/// The semantic rules of section 13.3's outcome table, over entries that
/// passed [`check_entry_shapes`] and the state rows locked for the job's
/// targets. Returns the state writes the outcome makes.
pub(crate) fn check_entries_semantic(
    entries: &[EvidenceEntry],
    rule: EntryRule,
    job: &JobRow,
    states: &BTreeMap<String, StateRow>,
) -> Result<Vec<StateWrite>, SemanticFault> {
    let entry_keys: HashSet<&str> = entries.iter().map(|e| e.target_key.as_str()).collect();
    let job_keys: HashSet<&str> = job.target_keys.iter().map(String::as_str).collect();
    if entry_keys != job_keys {
        return Err(SemanticFault::Invalid(
            "the entries do not match the job's targets".to_string(),
        ));
    }
    let revision_bound = job.revision_mode == RevisionMode::RevisionBound;
    let mut sorted: Vec<&EvidenceEntry> = entries.iter().collect();
    sorted.sort_by(|a, b| a.target_key.cmp(&b.target_key));
    let mut writes = Vec::new();
    for entry in sorted {
        let stored = states.get(&entry.target_key);
        // Shapes are checked, so these conversions cannot fail.
        let order = entry.order.map(|order| order as i64);
        match rule {
            EntryRule::Completed => {}
            EntryRule::Superseded => {
                let superseded = matches!(
                    (stored.and_then(|s| s.applied_order), job.source_order),
                    (Some(applied), Some(source)) if applied > source
                );
                if !superseded {
                    return Err(SemanticFault::Invalid(
                        "SUPERSEDED needs a stored order greater than the job's".to_string(),
                    ));
                }
            }
            EntryRule::EffectApplied | EntryRule::NoOp => {
                let fingerprint = entry.fingerprint.clone().unwrap_or_default();
                if revision_bound {
                    if order != job.source_order
                        || Some(fingerprint.as_str()) != job.source_fingerprint.as_deref()
                    {
                        return Err(SemanticFault::Invalid(
                            "an entry does not carry the job's source revision".to_string(),
                        ));
                    }
                    if let Some(stored) = stored {
                        match (stored.applied_order, order) {
                            (Some(applied), Some(order)) if applied > order => {
                                return Err(if rule == EntryRule::EffectApplied {
                                    SemanticFault::Regression
                                } else {
                                    SemanticFault::Invalid(
                                        "a newer revision is stored; report SUPERSEDED".to_string(),
                                    )
                                });
                            }
                            (Some(applied), Some(order))
                                if applied == order
                                    && stored.applied_fingerprint != fingerprint =>
                            {
                                return Err(SemanticFault::Invalid(
                                    "the stored order has another fingerprint".to_string(),
                                ));
                            }
                            _ => {}
                        }
                    }
                }
                let unchanged = stored.is_some_and(|stored| {
                    stored.applied_order == order && stored.applied_fingerprint == fingerprint
                });
                if rule == EntryRule::NoOp && unchanged {
                    continue;
                }
                writes.push(StateWrite {
                    target_key: entry.target_key.clone(),
                    order,
                    fingerprint,
                    source_ref: entry.source_ref.clone(),
                });
            }
        }
    }
    Ok(writes)
}

/// The canonical JSON text of an entry list for a command digest (section
/// 10): sorted by `targetKey`, every field present. The database renders it
/// as `jsonb::text` before hashing, so key order and spacing do not matter.
pub(crate) fn entries_json(entries: &[EvidenceEntry]) -> String {
    let mut sorted: Vec<&EvidenceEntry> = entries.iter().collect();
    sorted.sort_by(|a, b| a.target_key.cmp(&b.target_key));
    let items: Vec<String> = sorted
        .into_iter()
        .map(|entry| {
            format!(
                "{{\"targetKey\":{},\"order\":{},\"fingerprint\":{},\"sourceRef\":{}}}",
                json_string(&entry.target_key),
                entry
                    .order
                    .map(|order| order.to_string())
                    .unwrap_or_else(|| "null".to_string()),
                entry
                    .fingerprint
                    .as_deref()
                    .map(json_string)
                    .unwrap_or_else(|| "null".to_string()),
                entry
                    .source_ref
                    .as_deref()
                    .map(json_string)
                    .unwrap_or_else(|| "null".to_string()),
            )
        })
        .collect();
    format!("[{}]", items.join(","))
}

/// A JSON string literal.
pub(crate) fn json_string(value: &str) -> String {
    serde_json::to_string(value).expect("a string always serialises")
}
