//! `BE-06` Crossref profile evidence: canonical DOI identity, deposit
//! timestamps, the permit state machine, reservations, finalisation, voids,
//! reconciliation and the version floor.
//!
//! Every database test runs against a real disposable PostgreSQL with both
//! BE-06 migrations applied. Nothing here contacts Crossref.

use std::str::FromStr;

use diesel::sql_types::{BigInt, Nullable, Text};
use diesel::{PgConnection, QueryableByName, RunQueryDsl};

use crate::model::tests::db as test_db;
use crate::model::Doi;

#[derive(QueryableByName)]
struct OptionalTextRow {
    #[diesel(sql_type = Nullable<Text>)]
    value: Option<String>,
}

#[derive(QueryableByName)]
struct OptionalBigIntRow {
    #[diesel(sql_type = Nullable<BigInt>)]
    value: Option<i64>,
}

fn migration_2_up() -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/20260911_v1.10.0/up.sql"
    ))
    .expect("read Migration 2")
}

fn marked_block<'a>(sql: &'a str, begin: &str, end: &str) -> &'a str {
    sql.split_once(begin)
        .unwrap_or_else(|| panic!("Migration 2 carries {begin}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("Migration 2 carries {end}"))
        .0
}

/// Decode one PostgreSQL `E'...'` literal as the corpus writes it: `\n`,
/// `\uXXXX`, `\\` and `''` are the only escapes used.
fn decode_escape_literal(literal: &str) -> String {
    let body = literal
        .strip_prefix("E'")
        .and_then(|rest| rest.strip_suffix('\''))
        .unwrap_or_else(|| panic!("not an E'' literal: {literal}"));
    let mut out = String::new();
    let mut chars = body.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some('n') => out.push('\n'),
                Some('\\') => out.push('\\'),
                Some('u') => {
                    let hex: String = (0..4).filter_map(|_| chars.next()).collect();
                    let code = u32::from_str_radix(&hex, 16).expect("\\u escape");
                    out.push(char::from_u32(code).expect("valid scalar"));
                }
                other => panic!("unsupported escape {other:?} in {literal}"),
            },
            '\'' => {
                assert_eq!(chars.next(), Some('\''), "a quote is doubled in {literal}");
                out.push('\'');
            }
            other => out.push(other),
        }
    }
    out
}

/// The 40-value canonicalisation corpus embedded in Migration 2's assertions,
/// as `(raw, expected)` with `expected` `None` for a rejected value.
fn doi_corpus() -> Vec<(String, Option<String>)> {
    let sql = migration_2_up();
    marked_block(&sql, "-- BE06_DOI_CORPUS_BEGIN", "-- BE06_DOI_CORPUS_END")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let tuple = line
                .trim_end_matches(',')
                .strip_prefix('(')
                .and_then(|rest| rest.strip_suffix(')'))
                .unwrap_or_else(|| panic!("corpus row is a tuple: {line}"));
            let (raw, expected) = tuple
                .rsplit_once(", ")
                .unwrap_or_else(|| panic!("corpus row has two fields: {line}"));
            let expected = match expected {
                "NULL" => None,
                quoted => Some(
                    quoted
                        .strip_prefix('\'')
                        .and_then(|rest| rest.strip_suffix('\''))
                        .unwrap_or_else(|| panic!("expected value is quoted: {line}"))
                        .to_string(),
                ),
            };
            (decode_escape_literal(raw), expected)
        })
        .collect()
}

fn canonical_doi(connection: &mut PgConnection, raw: &str) -> Option<String> {
    diesel::sql_query("SELECT public.crossref_canonical_doi($1) AS value")
        .bind::<Text, _>(raw)
        .get_result::<OptionalTextRow>(connection)
        .expect("crossref_canonical_doi")
        .value
}

// ---------------------------------------------------------------------------
// §16.3 canonical DOI identity (R52B section 25.10)
// ---------------------------------------------------------------------------

#[test]
fn the_canonicalisation_corpus_is_total_exact_and_never_over_accepts() {
    let corpus = doi_corpus();
    assert_eq!(corpus.len(), 40, "the corpus has 40 values");
    let accepted = corpus
        .iter()
        .filter(|(raw, _)| Doi::from_str(raw).is_ok())
        .count();
    assert_eq!(
        accepted, 22,
        "22 corpus values are accepted by the released parser"
    );

    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let mut totality_gaps = Vec::new();
    let mut over_acceptances = Vec::new();
    for (raw, expected) in &corpus {
        let canonical = canonical_doi(&mut connection, raw);
        assert_eq!(
            &canonical, expected,
            "the migration's expectation for {raw:?}"
        );
        match Doi::from_str(raw) {
            Ok(doi) => {
                // Exactness: the canonical identity is the lower-cased released stored form.
                match &canonical {
                    Some(value) => assert_eq!(value, &doi.to_lowercase_string(), "{raw:?}"),
                    None => totality_gaps.push(raw.clone()),
                }
            }
            Err(_) => {
                if canonical.is_some() {
                    over_acceptances.push(raw.clone());
                }
            }
        }
    }
    assert!(totality_gaps.is_empty(), "totality gaps: {totality_gaps:?}");
    assert!(
        over_acceptances.is_empty(),
        "over-acceptances: {over_acceptances:?}"
    );
}

#[test]
fn case_scheme_www_and_dx_variants_canonicalise_to_one_value() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let expected = Some("https://doi.org/10.12345/abc".to_string());
    for raw in [
        "10.12345/ABC",
        "http://doi.org/10.12345/abc",
        "https://doi.org/10.12345/AbC",
        "https://www.doi.org/10.12345/abc",
        "http://dx.doi.org/10.12345/abc",
        "HTTPS://DX.DOI.ORG/10.12345/aBc",
        "WwW.Dx.DoI.oRg/10.12345/abc",
    ] {
        assert_eq!(canonical_doi(&mut connection, raw), expected, "{raw}");
    }
}

// ---------------------------------------------------------------------------
// §17.2 calendar arithmetic (R52B T194-T196, T259-T260)
// ---------------------------------------------------------------------------

fn timestamp_vectors() -> Vec<(i64, i64)> {
    let sql = migration_2_up();
    marked_block(&sql, "-- BE06_TS_VECTORS_BEGIN", "-- BE06_TS_VECTORS_END")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let tuple = line
                .trim_end_matches(',')
                .strip_prefix('(')
                .and_then(|rest| rest.strip_suffix(')'))
                .unwrap_or_else(|| panic!("vector is a tuple: {line}"));
            let (input, successor) = tuple.split_once(", ").expect("two fields");
            let parse = |value: &str| {
                value
                    .strip_suffix("::bigint")
                    .expect("typed literal")
                    .parse::<i64>()
                    .expect("integer")
            };
            (parse(input), parse(successor))
        })
        .collect()
}

fn ts_next(connection: &mut PgConnection, value: i64) -> Result<Option<i64>, String> {
    diesel::sql_query("SELECT public.crossref_ts_next($1) AS value")
        .bind::<BigInt, _>(value)
        .get_result::<OptionalBigIntRow>(connection)
        .map(|row| row.value)
        .map_err(|error| match error {
            diesel::result::Error::DatabaseError(_, info) => info.message().to_string(),
            other => other.to_string(),
        })
}

#[test]
fn the_thirteen_boundary_vectors_have_valid_calendar_successors_where_naive_plus_one_mostly_fails()
{
    let vectors = timestamp_vectors();
    assert_eq!(vectors.len(), 13);
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let mut naive_invalid = 0;
    for (input, successor) in &vectors {
        assert_eq!(
            ts_next(&mut connection, *input),
            Ok(Some(*successor)),
            "crossref_ts_next({input})"
        );
        assert!(successor > input);
        let naive = input + 1;
        let decoded = diesel::sql_query("SELECT public.crossref_ts_decode($1)::text AS value")
            .bind::<BigInt, _>(naive)
            .get_result::<OptionalTextRow>(&mut connection)
            .expect("decode")
            .value;
        if decoded.is_none() {
            naive_invalid += 1;
        } else {
            assert_eq!(naive, *successor, "a valid naive +1 is the successor");
        }
    }
    assert_eq!(
        naive_invalid, 11,
        "naive +1 is invalid for 11 of the 13 vectors"
    );
}

#[test]
fn the_domain_maximum_has_no_successor_and_overflow_raises_a_stable_code() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    assert_eq!(
        ts_next(&mut connection, 99991231235959998),
        Ok(Some(99991231235959999))
    );
    let message = ts_next(&mut connection, 99991231235959999).expect_err("overflow");
    assert!(
        message.starts_with("CROSSREF_TIMESTAMP_OVERFLOW: "),
        "{message}"
    );
}

// ---------------------------------------------------------------------------
// R52B section 16.4 and Amendment 3 section 9.6: the four reservations
// ---------------------------------------------------------------------------

use thoth_errors::ThothError;
use uuid::Uuid;

use crate::model::crossref_write_permit::crud as permit_crud;
use crate::model::crossref_write_permit::{
    CrossrefWritePermitState, CrossrefWriteReservation, CrossrefWriteRoute,
};
use crate::model::distribution_job::crud as job_crud;
use crate::model::publisher_distribution_platform::DistributionPlatform;
use crate::model::work_upsert::crud as work_upsert_crud;
use crate::model::work_upsert::tests as fx;

/// A claimed `WORK_UPSERT` job: `(publisher, imprint, activation, work, job, claim_token)`.
fn claimed_work_upsert(
    pool: &crate::db::PgPool,
    connection: &mut PgConnection,
) -> (Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let (publisher, imprint, activation, work, job) = fx::claimable_job(pool, connection);
    fx::enable_execution(pool);
    let claimed = fx::claim(pool);
    let token = claimed
        .iter()
        .find(|claimed| claimed.job.job.distribution_job_id == job)
        .map(|claimed| claimed.claim_token)
        .expect("this job was claimed");
    (publisher, imprint, activation, work, job, token)
}

fn reserve_work_upsert(
    pool: &crate::db::PgPool,
    job: Uuid,
    token: Uuid,
) -> Result<CrossrefWriteReservation, ThothError> {
    permit_crud::reserve_work_upsert_crossref_write(pool, job, token)
}

fn permit_row(connection: &mut PgConnection, permit: Uuid) -> String {
    fx::texts(
        connection,
        &format!(
            "SELECT route::text || '|' || state::text || '|' || coalesce(job_identity::text, '-') || '|' \
                 || coalesce(attempt_identity::text, '-') || '|' || coalesce(permit_generation::text, '-') || '|' \
                 || source_generation_witness::text || '|' || doi_set_cardinality::text || '|' \
                 || coalesce(operator_authorization_reference, '-') || '|' || xmin::text AS value \
             FROM crossref_write_permit WHERE permit_id = '{permit}'"
        ),
    )
    .remove(0)
}

fn work_doi(connection: &mut PgConnection, work: Uuid) -> String {
    fx::texts(
        connection,
        &format!("SELECT lower(doi) AS value FROM work WHERE work_id = '{work}'"),
    )
    .remove(0)
}

#[test]
fn a_work_upsert_reservation_derives_everything_server_side() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (publisher, _imprint, _activation, work, job, token) =
        claimed_work_upsert(pool.as_ref(), &mut connection);
    let generation = fx::generation_of(&mut connection, work).expect("generation");

    let before = fx::texts(
        &mut connection,
        "SELECT (floor(extract(epoch FROM clock_timestamp()) * 1000))::text AS value",
    )
    .remove(0);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserved");
    let _ = before;
    // R12: the public binding projection.
    assert_eq!(reservation.publisher_identity, publisher);
    assert_eq!(reservation.root_work_identity, work);
    assert_eq!(reservation.dois, vec![work_doi(&mut connection, work)]);
    assert_eq!(reservation.crossref_timestamp.to_string().len(), 17);
    assert_eq!(
        reservation.doi_batch_id,
        format!("{work}_{}", reservation.crossref_timestamp)
    );
    assert_eq!(
        fx::texts(&mut connection, &format!("SELECT reservation_token::text AS value FROM crossref_write_permit WHERE permit_id = '{}'", reservation.permit_id)),
        vec![reservation.reservation_token.to_string()]
    );
    let attempt = fx::texts(&mut connection, &format!("SELECT distribution_job_attempt_id::text AS value FROM distribution_job_attempt WHERE distribution_job_id = '{job}'")).remove(0);
    let row = permit_row(&mut connection, reservation.permit_id);
    assert!(
        row.starts_with(&format!(
            "WORK_UPSERT|RESERVED|{job}|{attempt}|{generation}|{generation}|1|-|"
        )),
        "{row}"
    );
    assert_eq!(
        fx::texts(&mut connection, &format!(
            "SELECT (publisher_id = '{publisher}' AND distribution_job_id = '{job}' AND distribution_job_attempt_id = '{attempt}')::text AS value \
             FROM crossref_write_permit WHERE permit_id = '{}'", reservation.permit_id
        )),
        vec!["true"]
    );
    assert_eq!(
        fx::generation_of(&mut connection, work),
        Some(generation),
        "never bumped"
    );
    // The attempt is not fenced by a reservation.
    assert_eq!(fx::texts(&mut connection, &format!("SELECT (fenced_at IS NULL)::text AS value FROM distribution_job_attempt WHERE distribution_job_id = '{job}'")), vec!["true"]);

    // R6: one permit per attempt, ever, in RESERVED and VOIDED.
    assert_eq!(
        reserve_work_upsert(pool.as_ref(), job, token).map(|r| r.permit_id),
        Err(ThothError::CrossrefPermitAttemptAlreadyReserved)
    );
    fx::execute(&mut connection, &format!(
        "UPDATE crossref_write_permit SET state = 'VOIDED', void_reason = 'OWNER_ABANDONED', void_detail = 'd', closed_at = now() WHERE permit_id = '{}'",
        reservation.permit_id
    ));
    let refused = reserve_work_upsert(pool.as_ref(), job, token).expect_err("still refused");
    assert_eq!(refused, ThothError::CrossrefPermitAttemptAlreadyReserved);
    let message = refused.to_string();
    assert!(
        !message.contains(&reservation.permit_id.to_string())
            && !message.contains("VOIDED")
            && !message.contains(&reservation.reservation_token.to_string())
    );
    assert_eq!(
        fx::count(
            &mut connection,
            "SELECT count(*) AS count FROM crossref_write_permit"
        ),
        1
    );
}

#[test]
fn r1_r2_stale_and_wrong_kind_claims_are_refused_before_any_lock() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (_publisher, _imprint, _activation, work, job, token) =
        claimed_work_upsert(pool.as_ref(), &mut connection);

    assert_eq!(
        reserve_work_upsert(pool.as_ref(), Uuid::new_v4(), token).map(|r| r.permit_id),
        Err(ThothError::CrossrefPermitClaimStale)
    );
    assert_eq!(
        reserve_work_upsert(pool.as_ref(), job, Uuid::new_v4()).map(|r| r.permit_id),
        Err(ThothError::CrossrefPermitClaimStale)
    );
    // A PENDING job (the claim returned by lease recovery) is stale too.
    fx::execute(&mut connection, &format!("UPDATE distribution_job SET lease_expires_at = now() - interval '1 second' WHERE distribution_job_id = '{job}'"));
    fx::execute(&mut connection, "SET session_replication_role = replica; UPDATE work_upsert_control SET execution_enabled = false; SET session_replication_role = origin");
    assert!(
        fx::claim(pool.as_ref()).is_empty(),
        "recovery ran; the job is PENDING"
    );
    assert_eq!(
        reserve_work_upsert(pool.as_ref(), job, token).map(|r| r.permit_id),
        Err(ThothError::CrossrefPermitClaimStale)
    );
    fx::execute(
        &mut connection,
        "UPDATE work_upsert_control SET execution_enabled = true",
    );
    assert_eq!(
        fx::count(
            &mut connection,
            "SELECT count(*) AS count FROM crossref_write_permit"
        ),
        0
    );

    // R2: a valid claim on a back-catalogue job, on the WORK_UPSERT route, and the reverse.
    let (back_publisher, back_imprint) = fx::publisher_and_imprint(pool.as_ref());
    let back_activation = fx::cover_crossref(&mut connection, back_publisher);
    let unit = fx::insert_eligible_work(&mut connection, back_imprint, Uuid::new_v4());
    fx::execute(&mut connection, &format!(
        "INSERT INTO distribution_job (kind, publisher_id, activation_id, deduplication_key) \
         VALUES ('PUBLISHER_BACK_CATALOGUE', '{back_publisher}', '{back_activation}', 'PUBLISHER_BACK_CATALOGUE:{back_publisher}:{back_activation}'); \
         INSERT INTO distribution_job_target (distribution_job_id, platform) \
         SELECT distribution_job_id, 'CROSSREF' FROM distribution_job WHERE kind = 'PUBLISHER_BACK_CATALOGUE'"
    ));
    let outer =
        job_crud::claim_distribution_jobs(pool.as_ref(), "legacy", 10, 900, &[]).expect("claim");
    assert_eq!(outer.len(), 1);
    let (outer_job, outer_token) = (outer[0].job.job.distribution_job_id, outer[0].claim_token);
    assert_eq!(
        reserve_work_upsert(pool.as_ref(), outer_job, outer_token).map(|r| r.permit_id),
        Err(ThothError::CrossrefReservationJobKindMismatch)
    );
    let claimed = fx::claim(pool.as_ref());
    assert_eq!(claimed.len(), 1);
    assert_eq!(
        permit_crud::reserve_back_catalogue_crossref_write(
            pool.as_ref(),
            claimed[0].job.job.distribution_job_id,
            claimed[0].claim_token,
            work
        )
        .map(|r| r.permit_id),
        Err(ThothError::CrossrefReservationJobKindMismatch)
    );

    // The back-catalogue route: the unit belongs to the outer job's publisher.
    let unit_reservation = permit_crud::reserve_back_catalogue_crossref_write(
        pool.as_ref(),
        outer_job,
        outer_token,
        unit,
    )
    .expect("unit");
    assert_eq!(unit_reservation.publisher_identity, back_publisher);
    assert!(permit_row(&mut connection, unit_reservation.permit_id)
        .starts_with(&format!("PUBLISHER_BACK_CATALOGUE|RESERVED|{outer_job}|")));
    // R5: a unit of another publisher is a mismatch, and no 0 row is written.
    fx::uncover(&mut connection, work);
    assert_eq!(
        permit_crud::reserve_back_catalogue_crossref_write(
            pool.as_ref(),
            outer_job,
            outer_token,
            work
        )
        .map(|r| r.permit_id),
        Err(ThothError::CrossrefUnitPublisherMismatch)
    );
    assert_eq!(fx::generation_of(&mut connection, work), None);
    assert_eq!(
        permit_crud::reserve_back_catalogue_crossref_write(
            pool.as_ref(),
            outer_job,
            outer_token,
            Uuid::new_v4()
        )
        .map(|r| r.permit_id),
        Err(ThothError::CrossrefRootWorkNotFound)
    );
}

#[test]
fn r3_r4_fence_preconditions_refuse_without_writing() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (publisher, _imprint, _activation, work, job, token) =
        claimed_work_upsert(pool.as_ref(), &mut connection);
    let refused = |expected: ThothError| {
        assert_eq!(
            reserve_work_upsert(pool.as_ref(), job, token).map(|r| r.permit_id),
            Err(expected)
        );
    };

    // R4: clause 5, the Work moved.
    let (_other, other_imprint) = fx::publisher_and_imprint(pool.as_ref());
    let home = fx::texts(
        &mut connection,
        &format!("SELECT imprint_id::text AS value FROM work WHERE work_id = '{work}'"),
    )
    .remove(0);
    fx::execute(
        &mut connection,
        &format!("UPDATE work SET imprint_id = '{other_imprint}' WHERE work_id = '{work}'"),
    );
    refused(ThothError::CrossrefBindingMovedRetry);
    fx::execute(
        &mut connection,
        &format!("UPDATE work SET imprint_id = '{home}' WHERE work_id = '{work}'"),
    );

    // R3: clause 6, the assignment disabled, then re-enabled under a new activation.
    fx::disable_crossref(&mut connection, publisher);
    refused(ThothError::CrossrefBindingMovedRetry);
    let activation = fx::texts(&mut connection, &format!("SELECT activation_id::text AS value FROM publisher_distribution_platform WHERE publisher_id = '{publisher}'")).remove(0);
    fx::cover_crossref(&mut connection, publisher);
    refused(ThothError::CrossrefBindingMovedRetry);
    fx::execute(&mut connection, &format!("UPDATE publisher_distribution_platform SET activation_id = '{activation}' WHERE publisher_id = '{publisher}'"));

    // Clause 7: no admission row.
    fx::execute(&mut connection, "SET session_replication_role = replica; DELETE FROM work_upsert_admission; SET session_replication_role = origin");
    refused(ThothError::WorkUpsertProfileNotAdmitted);
    work_upsert_crud::admit_crossref_work_upsert(pool.as_ref(), publisher, "EV", "admin")
        .expect("readmit");

    // Clause 8: execution paused.
    work_upsert_crud::set_work_upsert_execution(
        pool.as_ref(),
        DistributionPlatform::Crossref,
        false,
    )
    .expect("pause");
    refused(ThothError::WorkUpsertExecutionNotPermitted);
    work_upsert_crud::set_work_upsert_execution(
        pool.as_ref(),
        DistributionPlatform::Crossref,
        true,
    )
    .expect("resume");

    // Clause 9: the Work is not eligible now.
    fx::execute(
        &mut connection,
        &format!("UPDATE publication SET isbn = NULL WHERE work_id = '{work}'"),
    );
    refused(ThothError::WorkUpsertExecutionNotPermitted);

    assert_eq!(
        fx::count(
            &mut connection,
            "SELECT count(*) AS count FROM crossref_write_permit"
        ),
        0
    );
}

#[test]
fn jobless_routes_require_coverage_membership_and_a_reference() {
    let failing = test_db::failing_pool();
    for blank in ["", " ", "\u{00A0}"] {
        assert_eq!(
            permit_crud::reserve_manual_recovery_crossref_write(&failing, Uuid::new_v4(), blank)
                .map(|r| r.permit_id),
            Err(ThothError::CrossrefManualRecoveryRequiresReference)
        );
    }

    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (publisher, imprint) = fx::publisher_and_imprint(pool.as_ref());
    let work = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    assert_eq!(
        permit_crud::reserve_legacy_scheduled_crossref_write(pool.as_ref(), Uuid::new_v4())
            .map(|r| r.permit_id),
        Err(ThothError::CrossrefRootWorkNotFound)
    );
    assert_eq!(
        permit_crud::reserve_legacy_scheduled_crossref_write(pool.as_ref(), work)
            .map(|r| r.permit_id),
        Err(ThothError::CrossrefPublisherNotCovered)
    );
    fx::cover_crossref(&mut connection, publisher);

    // The zero-row witness on an untouched Work.
    fx::uncover(&mut connection, work);
    let legacy =
        permit_crud::reserve_legacy_scheduled_crossref_write(pool.as_ref(), work).expect("legacy");
    assert_eq!(fx::generation_of(&mut connection, work), Some(0));
    assert!(permit_row(&mut connection, legacy.permit_id)
        .starts_with("LEGACY_SCHEDULED|RESERVED|-|-|-|0|1|-|"));

    // An overlapping reservation on any route is blocked; a manual one records its reference.
    assert_eq!(
        permit_crud::reserve_manual_recovery_crossref_write(pool.as_ref(), work, " INC-42 ")
            .map(|r| r.permit_id),
        Err(ThothError::CrossrefPermitBlocked)
    );
    fx::execute(&mut connection, &format!(
        "UPDATE crossref_write_permit SET state = 'VOIDED', void_reason = 'OWNER_ABANDONED', void_detail = 'd', closed_at = now() WHERE permit_id = '{}'",
        legacy.permit_id
    ));
    let manual =
        permit_crud::reserve_manual_recovery_crossref_write(pool.as_ref(), work, " INC-42 ")
            .expect("manual");
    assert!(permit_row(&mut connection, manual.permit_id)
        .starts_with("MANUAL_RECOVERY|RESERVED|-|-|-|0|1| INC-42 |"));
    assert!(
        manual.crossref_timestamp >= legacy.crossref_timestamp,
        "a VOIDED permit is not history"
    );

    // Empty membership: a root with a DOI but no landing page, and no children.
    let bare = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    fx::execute(
        &mut connection,
        &format!("UPDATE work SET landing_page = NULL WHERE work_id = '{bare}'"),
    );
    assert_eq!(
        permit_crud::reserve_legacy_scheduled_crossref_write(pool.as_ref(), bare)
            .map(|r| r.permit_id),
        Err(ThothError::CrossrefPermitEmptyDoiSet)
    );
    assert_eq!(
        fx::count(
            &mut connection,
            "SELECT count(*) AS count FROM crossref_write_permit"
        ),
        2
    );
}

#[test]
fn every_overlap_shape_but_disjoint_is_blocked() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (publisher, imprint) = fx::publisher_and_imprint(pool.as_ref());
    fx::cover_crossref(&mut connection, publisher);

    // Parents whose memberships are built from shared chapters A, B, C, D.
    // Children that are not book chapters may belong to several parents, which is
    // what lets one DOI appear in several memberships.
    let chapter = |connection: &mut PgConnection| {
        fx::insert_eligible_work(connection, imprint, Uuid::new_v4())
    };
    let parent_of = |connection: &mut PgConnection, chapters: &[Uuid]| {
        let parent = fx::insert_eligible_work(connection, imprint, Uuid::new_v4());
        fx::execute(
            connection,
            &format!("UPDATE work SET landing_page = NULL WHERE work_id = '{parent}'"),
        );
        for (ordinal, child) in chapters.iter().enumerate() {
            fx::relate_child(connection, parent, *child, ordinal as i32 + 1);
        }
        parent
    };
    let [a, b, c, d] = [
        chapter(&mut connection),
        chapter(&mut connection),
        chapter(&mut connection),
        chapter(&mut connection),
    ];
    let held = parent_of(&mut connection, &[a, b]);
    let reservation =
        permit_crud::reserve_legacy_scheduled_crossref_write(pool.as_ref(), held).expect("held");
    assert_eq!(reservation.dois.len(), 2);
    let mut sorted = reservation.dois.clone();
    sorted.sort();
    assert_eq!(reservation.dois, sorted, "ascending by code point");

    for (shape, chapters, blocked) in [
        ("identical", vec![a, b], true),
        ("subset", vec![a], true),
        ("superset", vec![a, b, c], true),
        ("one-member overlap", vec![b, c], true),
        ("multi-member overlap", vec![a, b, d], true),
        ("disjoint", vec![c, d], false),
    ] {
        let parent = parent_of(&mut connection, &chapters);
        let result = permit_crud::reserve_legacy_scheduled_crossref_write(pool.as_ref(), parent);
        if blocked {
            assert_eq!(
                result.map(|r| r.permit_id),
                Err(ThothError::CrossrefPermitBlocked),
                "{shape}"
            );
        } else {
            assert!(result.is_ok(), "{shape}: {result:?}");
        }
    }
    assert_eq!(
        fx::count(
            &mut connection,
            "SELECT count(*) AS count FROM crossref_write_permit"
        ),
        2
    );
    assert_eq!(
        fx::count(
            &mut connection,
            "SELECT count(*) AS count FROM crossref_write_permit WHERE state = 'RESERVED'"
        ),
        2
    );
    let _ = CrossrefWritePermitState::Reserved;
    let _ = CrossrefWriteRoute::LegacyScheduled;
}

#[test]
fn t253_the_reservation_reads_the_clock_once_after_the_keys_and_the_floor() {
    let crud = include_str!("crud.rs");
    assert_eq!(crud.matches("crossref_ts_now()").count(), 1);
    let clock = crud.find("crossref_ts_now()").expect("clock");
    let keys = crud.find("pg_advisory_xact_lock(").expect("keys");
    let floor = crud
        .find("FOR SHARE")
        .map(|_| crud.find("work_crossref_version_floor").expect("floor"))
        .expect("floor share");
    assert!(
        keys < clock && floor < clock,
        "the one clock read follows K and F"
    );
}

// ---------------------------------------------------------------------------
// R52B section 16.6 and Amendment 3 section 9.7: finalisation
// ---------------------------------------------------------------------------

use crate::model::crossref_write_permit::crud::FinaliseCrossrefWrite;
use crate::model::crossref_write_permit::{
    CrossrefFinalisationOutcome as Finalised, CrossrefVoidReason,
};
use crate::model::distribution_job::{DistributionJobCancellationReason, DistributionJobStatus};

const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn presentation(
    reservation: &CrossrefWriteReservation,
    claim_token: Option<Uuid>,
) -> FinaliseCrossrefWrite {
    FinaliseCrossrefWrite {
        permit_id: reservation.permit_id,
        reservation_token: reservation.reservation_token,
        claim_token,
        observed_dois: reservation.dois.clone(),
        observed_doi_batch_id: reservation.doi_batch_id.clone(),
        observed_crossref_timestamp: reservation.crossref_timestamp,
        payload_digest: DIGEST.to_string(),
    }
}

fn allow(_route: CrossrefWriteRoute) -> ThothResult<()> {
    Ok(())
}

fn finalise(
    pool: &crate::db::PgPool,
    input: &FinaliseCrossrefWrite,
) -> ThothResult<crate::model::crossref_write_permit::CrossrefFinalisationResult> {
    permit_crud::finalise_crossref_write(pool, input, &allow)
}

fn fingerprint(connection: &mut PgConnection) -> String {
    fx::texts(
        connection,
        "SELECT coalesce((SELECT string_agg(permit_id::text || state::text || xmin::text, ',' ORDER BY permit_id) FROM crossref_write_permit), '') || '#' \
             || coalesce((SELECT string_agg(distribution_job_id::text || status::text || xmin::text, ',' ORDER BY distribution_job_id) FROM distribution_job), '') || '#' \
             || coalesce((SELECT string_agg(distribution_job_attempt_id::text || coalesce(fenced_at::text, '-') || xmin::text, ',' ORDER BY distribution_job_attempt_id) FROM distribution_job_attempt), '') AS value",
    )
    .remove(0)
}

use thoth_errors::ThothResult;

#[test]
fn a_work_upsert_finalisation_authorises_and_fences_in_one_transaction() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (publisher, _imprint, _activation, _work, job, token) =
        claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let input = presentation(&reservation, Some(token));

    let result = finalise(pool.as_ref(), &input).expect("finalised");
    assert_eq!(
        (result.outcome, result.void_reason),
        (Finalised::Authorized, None)
    );
    assert_eq!(
        result.permit.permit.state,
        CrossrefWritePermitState::Authorized
    );
    assert_eq!(result.permit.permit.payload_digest.as_deref(), Some(DIGEST));
    assert!(result.permit.permit.authorized_at.is_some());
    assert_eq!(result.permit.permit.publisher_identity, publisher);
    assert_eq!(result.permit.dois, reservation.dois);
    assert_eq!(
        fx::texts(&mut connection, &format!("SELECT (fenced_at IS NOT NULL)::text AS value FROM distribution_job_attempt WHERE distribution_job_id = '{job}'")),
        vec!["true"]
    );

    // C2: an identical retry replays and writes nothing; a different digest is refused.
    let before = fingerprint(&mut connection);
    let replay = finalise(pool.as_ref(), &input).expect("replay");
    assert_eq!(replay.outcome, Finalised::Authorized);
    assert_eq!(fingerprint(&mut connection), before);
    let different = FinaliseCrossrefWrite {
        payload_digest: "f".repeat(64),
        ..input.clone()
    };
    assert_eq!(
        finalise(pool.as_ref(), &different).map(|r| r.outcome),
        Err(ThothError::CrossrefPermitIllegalTransition)
    );
    // A malformed digest is refused before any lock, even on replay.
    let upper = FinaliseCrossrefWrite {
        payload_digest: DIGEST.to_uppercase(),
        ..input.clone()
    };
    assert_eq!(
        finalise(pool.as_ref(), &upper).map(|r| r.outcome),
        Err(ThothError::CrossrefPayloadDigestInvalid)
    );
    assert_eq!(fingerprint(&mut connection), before);
    // Request authorization precedes everything but the route read.
    let deny = |_route: CrossrefWriteRoute| -> ThothResult<()> { Err(ThothError::Unauthorised) };
    assert_eq!(
        permit_crud::finalise_crossref_write(pool.as_ref(), &upper, &deny).map(|r| r.outcome),
        Err(ThothError::Unauthorised)
    );
    assert_eq!(
        permit_crud::finalise_crossref_write(pool.as_ref(), &input, &deny).map(|r| r.outcome),
        Err(ThothError::Unauthorised)
    );
    let missing = FinaliseCrossrefWrite {
        permit_id: Uuid::new_v4(),
        ..input.clone()
    };
    assert_eq!(
        finalise(pool.as_ref(), &missing).map(|r| r.outcome),
        Err(ThothError::CrossrefPermitNotFound)
    );
}

#[test]
fn group_c_refusals_change_nothing() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (_publisher, _imprint, _activation, _work, job, token) =
        claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let input = presentation(&reservation, Some(token));
    let before = fingerprint(&mut connection);

    // C1: the reservation token.
    let wrong = FinaliseCrossrefWrite {
        reservation_token: Uuid::new_v4(),
        ..input.clone()
    };
    assert_eq!(
        finalise(pool.as_ref(), &wrong).map(|r| r.outcome),
        Err(ThothError::CrossrefPermitRequiresReservationToken)
    );
    // C4: an absent or stale claim token on a job-linked route.
    let absent = FinaliseCrossrefWrite {
        claim_token: None,
        ..input.clone()
    };
    assert_eq!(
        finalise(pool.as_ref(), &absent).map(|r| r.outcome),
        Err(ThothError::CrossrefPermitClaimStale)
    );
    let stale = FinaliseCrossrefWrite {
        claim_token: Some(Uuid::new_v4()),
        ..input.clone()
    };
    assert_eq!(
        finalise(pool.as_ref(), &stale).map(|r| r.outcome),
        Err(ThothError::CrossrefPermitClaimStale)
    );
    assert_eq!(fingerprint(&mut connection), before);

    // C4 through administrative cancellation: the permit stays RESERVED and untouched.
    job_crud::cancel_distribution_job(pool.as_ref(), job).expect("cancel");
    let permit_before = permit_row(&mut connection, reservation.permit_id);
    assert_eq!(
        finalise(pool.as_ref(), &input).map(|r| r.outcome),
        Err(ThothError::CrossrefPermitClaimStale)
    );
    assert_eq!(
        permit_row(&mut connection, reservation.permit_id),
        permit_before
    );

    // A job-less route sent a claim token.
    let (publisher, imprint) = fx::publisher_and_imprint(pool.as_ref());
    fx::cover_crossref(&mut connection, publisher);
    let work = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let legacy =
        permit_crud::reserve_legacy_scheduled_crossref_write(pool.as_ref(), work).expect("legacy");
    assert_eq!(
        finalise(pool.as_ref(), &presentation(&legacy, Some(Uuid::new_v4()))).map(|r| r.outcome),
        Err(ThothError::CrossrefPermitClaimStale)
    );
    // C3: an owner-voided reservation is not RESERVED.
    fx::execute(&mut connection, &format!(
        "UPDATE crossref_write_permit SET state = 'VOIDED', void_reason = 'OWNER_ABANDONED', void_detail = 'd', closed_at = now() WHERE permit_id = '{}'",
        legacy.permit_id
    ));
    assert_eq!(
        finalise(pool.as_ref(), &presentation(&legacy, None)).map(|r| r.outcome),
        Err(ThothError::CrossrefPermitIllegalTransition)
    );
}

#[test]
fn group_a_voids_the_reservation_retryably_and_leaves_the_job_running() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");

    let voided =
        |pool: &crate::db::PgPool, input: &FinaliseCrossrefWrite, reason: CrossrefVoidReason| {
            let result = finalise(pool, input).expect("a void is a result");
            assert_eq!(
                (result.outcome, result.void_reason),
                (Finalised::VoidedRetryable, Some(reason))
            );
            assert_eq!(result.permit.permit.state, CrossrefWritePermitState::Voided);
            // C2: the voided finalisation replays its recorded outcome.
            let replay = finalise(pool, input).expect("replay");
            assert_eq!(
                (replay.outcome, replay.void_reason),
                (Finalised::VoidedRetryable, Some(reason))
            );
        };

    // A1: a source edit after the reservation.
    let (_p, _i, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    fx::execute(
        &mut connection,
        &format!("UPDATE work SET place = 'Cambridge' WHERE work_id = '{work}'"),
    );
    voided(
        pool.as_ref(),
        &presentation(&reservation, Some(token)),
        CrossrefVoidReason::SourceChangedDuringPreparation,
    );
    assert_eq!(
        fx::texts(&mut connection, &format!(
            "SELECT j.status::text || '|' || (a.finished_at IS NULL)::text || '|' || (a.fenced_at IS NULL)::text AS value \
             FROM distribution_job j JOIN distribution_job_attempt a ON a.distribution_job_id = j.distribution_job_id WHERE j.distribution_job_id = '{job}'"
        )),
        vec!["RUNNING|true|true"]
    );

    // A3, A4, A5 on job-less routes.
    let (publisher, imprint) = fx::publisher_and_imprint(pool.as_ref());
    fx::cover_crossref(&mut connection, publisher);
    for (mutate, reason) in [
        (0, CrossrefVoidReason::ArtifactDoiSetMismatch),
        (1, CrossrefVoidReason::ArtifactBatchIdMismatch),
        (2, CrossrefVoidReason::ArtifactTimestampMismatch),
    ] {
        let work = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
        let legacy = permit_crud::reserve_legacy_scheduled_crossref_write(pool.as_ref(), work)
            .expect("legacy");
        let mut input = presentation(&legacy, None);
        match mutate {
            0 => input
                .observed_dois
                .push("https://doi.org/10.12345/extra".to_string()),
            1 => input.observed_doi_batch_id.push('x'),
            _ => input.observed_crossref_timestamp += 1,
        }
        voided(pool.as_ref(), &input, reason);
    }
    // A3's canonicalisation: case and prefix variants of the reserved DOIs match.
    let work = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let legacy =
        permit_crud::reserve_legacy_scheduled_crossref_write(pool.as_ref(), work).expect("legacy");
    let mut input = presentation(&legacy, None);
    input.observed_dois = legacy
        .dois
        .iter()
        .map(|doi| {
            doi.replace("https://doi.org/", "HTTP://DX.DOI.ORG/")
                .to_uppercase()
        })
        .collect();
    assert_eq!(
        finalise(pool.as_ref(), &input).map(|r| r.outcome),
        Ok(Finalised::Authorized)
    );

    // A2: membership changed without a counted event (capture bypassed).
    let root = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let child = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let legacy =
        permit_crud::reserve_legacy_scheduled_crossref_write(pool.as_ref(), root).expect("legacy");
    fx::execute(&mut connection, "SET session_replication_role = replica");
    fx::relate_child(&mut connection, root, child, 1);
    fx::execute(&mut connection, "SET session_replication_role = origin");
    voided(
        pool.as_ref(),
        &presentation(&legacy, None),
        CrossrefVoidReason::DoiMembershipChanged,
    );
}

#[test]
fn group_a0_fence_clauses_7_to_9_void_retryably() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    for case in 0..3 {
        fx::execute(
            &mut connection,
            "UPDATE work_upsert_control SET execution_enabled = true WHERE capture_enabled",
        );
        let (publisher, _i, _a, work, job, token) =
            claimed_work_upsert(pool.as_ref(), &mut connection);
        let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
        let reason = match case {
            0 => {
                fx::execute(&mut connection, &format!("SET session_replication_role = replica; DELETE FROM work_upsert_admission WHERE publisher_id = '{publisher}'; SET session_replication_role = origin"));
                CrossrefVoidReason::ProfileNotAdmitted
            }
            1 => {
                work_upsert_crud::set_work_upsert_execution(
                    pool.as_ref(),
                    DistributionPlatform::Crossref,
                    false,
                )
                .expect("pause");
                CrossrefVoidReason::ExecutionNotPermitted
            }
            _ => {
                fx::execute(&mut connection, &format!("SET session_replication_role = replica; UPDATE publication SET isbn = NULL WHERE work_id = '{work}'; SET session_replication_role = origin"));
                CrossrefVoidReason::Ineligible
            }
        };
        let result =
            finalise(pool.as_ref(), &presentation(&reservation, Some(token))).expect("result");
        assert_eq!(
            (result.outcome, result.void_reason),
            (Finalised::VoidedRetryable, Some(reason)),
            "case {case}"
        );
        assert_eq!(
            fx::texts(&mut connection, &format!("SELECT status::text AS value FROM distribution_job WHERE distribution_job_id = '{job}'")),
            vec!["RUNNING"]
        );
    }
}

#[test]
fn group_b_retires_the_job_and_replaces_it_only_under_the_held_publisher() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");

    // B2: the Work moved to another publisher: retired, no replacement.
    let (_publisher, _i, _a, work, job, token) =
        claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let (_other, other_imprint) = fx::publisher_and_imprint(pool.as_ref());
    fx::execute(
        &mut connection,
        &format!("UPDATE work SET imprint_id = '{other_imprint}' WHERE work_id = '{work}'"),
    );
    let result = finalise(pool.as_ref(), &presentation(&reservation, Some(token))).expect("result");
    assert_eq!(
        (result.outcome, result.void_reason),
        (
            Finalised::VoidedJobRetired,
            Some(CrossrefVoidReason::BindingSuperseded)
        )
    );
    assert_eq!(
        fx::job_summary(&mut connection, work),
        vec!["CANCELLED|BINDING_SUPERSEDED|1|1|false|false|CROSSREF"]
    );
    assert_eq!(
        fx::texts(&mut connection, &format!("SELECT result::text AS value FROM distribution_job_attempt WHERE distribution_job_id = '{job}'")),
        vec!["CANCELLED"]
    );
    assert_eq!(
        job_crud::fail_distribution_job(
            pool.as_ref(),
            job,
            token,
            "CROSSREF_PERMIT_VOIDED_RETRYABLE",
            None,
            true
        )
        .map(|j| j.status),
        Err(ThothError::DistributionJobAlreadyTerminal(
            "CANCELLED".to_string()
        ))
    );
    // C2 precedes C4: the voided finalisation replays its recorded outcome.
    let replay = finalise(pool.as_ref(), &presentation(&reservation, Some(token))).expect("replay");
    assert_eq!(
        (replay.outcome, replay.void_reason),
        (
            Finalised::VoidedJobRetired,
            Some(CrossrefVoidReason::BindingSuperseded)
        )
    );

    // B3 with an activation change under the same, admitted publisher: a replacement.
    let (publisher, _i, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    fx::cover_crossref(&mut connection, publisher);
    work_upsert_crud::admit_crossref_work_upsert(pool.as_ref(), publisher, "EV-NEW", "admin")
        .expect("admit");
    fx::set_generation(&mut connection, work, 2);
    let result = finalise(pool.as_ref(), &presentation(&reservation, Some(token))).expect("result");
    assert_eq!(
        (result.outcome, result.void_reason),
        (
            Finalised::VoidedJobRetired,
            Some(CrossrefVoidReason::BindingSuperseded)
        )
    );
    assert_eq!(
        fx::job_summary(&mut connection, work),
        vec![
            "CANCELLED|BINDING_SUPERSEDED|1|1|false|true|CROSSREF",
            "PENDING|-|2|2|true|false|CROSSREF",
        ]
    );

    // B3, the target disabled: ASSIGNMENT_DISABLED and no replacement.
    let (publisher, _i, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    fx::disable_crossref(&mut connection, publisher);
    let result = finalise(pool.as_ref(), &presentation(&reservation, Some(token))).expect("result");
    assert_eq!(
        (result.outcome, result.void_reason),
        (
            Finalised::VoidedJobRetired,
            Some(CrossrefVoidReason::AssignmentDisabled)
        )
    );
    assert_eq!(
        fx::job_summary(&mut connection, work),
        vec!["CANCELLED|ASSIGNMENT_DISABLED|1|1|false|false|CROSSREF"]
    );
    let _ = (
        DistributionJobCancellationReason::AssignmentDisabled,
        DistributionJobStatus::Cancelled,
    );

    // A job-less route: VOIDED_RETRYABLE.
    let (publisher, imprint) = fx::publisher_and_imprint(pool.as_ref());
    fx::cover_crossref(&mut connection, publisher);
    let work = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let legacy =
        permit_crud::reserve_legacy_scheduled_crossref_write(pool.as_ref(), work).expect("legacy");
    fx::disable_crossref(&mut connection, publisher);
    let result = finalise(pool.as_ref(), &presentation(&legacy, None)).expect("result");
    assert_eq!(
        (result.outcome, result.void_reason),
        (
            Finalised::VoidedRetryable,
            Some(CrossrefVoidReason::AssignmentDisabled)
        )
    );
}

#[test]
fn only_finalisation_moves_a_permit_to_authorized() {
    let crud = include_str!("crud.rs");
    assert_eq!(
        crud.matches("state.eq(CrossrefWritePermitState::Authorized)")
            .count(),
        1,
        "exactly one code path issues RESERVED -> AUTHORIZED"
    );
    let finalise = crud
        .split_once("pub fn finalise_crossref_write(")
        .expect("finalise")
        .1;
    let authorise = crud
        .find("state.eq(CrossrefWritePermitState::Authorized)")
        .expect("authorise");
    assert!(crud.len() - finalise.len() < authorise);
}

// ---------------------------------------------------------------------------
// R52B sections 16.7-16.8 and Amendment 3 section 9.8: outcome reports, voids,
// reconciliation and the internal clearance
// ---------------------------------------------------------------------------

use crate::model::crossref_write_permit::{
    CrossrefReconciliationState as Recon, CrossrefWriteOutcome as Outcome,
};

/// A legacy permit finalised to AUTHORIZED: `(reservation, publisher, work)`.
fn authorized_legacy(
    pool: &crate::db::PgPool,
    connection: &mut PgConnection,
) -> (CrossrefWriteReservation, Uuid, Uuid) {
    let (publisher, imprint) = fx::publisher_and_imprint(pool);
    fx::cover_crossref(connection, publisher);
    let work = fx::insert_eligible_work(connection, imprint, Uuid::new_v4());
    let reservation =
        permit_crud::reserve_legacy_scheduled_crossref_write(pool, work).expect("reserve");
    assert_eq!(
        finalise(pool, &presentation(&reservation, None)).map(|r| r.outcome),
        Ok(Finalised::Authorized)
    );
    (reservation, publisher, work)
}

fn state_of(connection: &mut PgConnection, permit: Uuid) -> String {
    fx::texts(
        connection,
        &format!(
            "SELECT state::text || '|' || coalesce(reconciliation_state::text, '-') || '|' \
                 || (provider_reported_at IS NOT NULL)::text || '|' || (closed_at IS NOT NULL)::text || '|' \
                 || coalesce(void_reason::text, '-') || '|' || coalesce(void_detail, '-') || '|' \
                 || coalesce(void_authorization_reference, '-') || '|' \
                 || coalesce(reconciliation_annotation_reference, '-') || '|' \
                 || coalesce(reconciliation_authorization_reference, '-') AS value \
             FROM crossref_write_permit WHERE permit_id = '{permit}'"
        ),
    )
    .remove(0)
}

#[test]
fn the_outcome_report_moves_an_authorized_permit_once() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let deny = |_route: CrossrefWriteRoute| -> ThothResult<()> { Err(ThothError::Unauthorised) };

    let (reservation, _p, _w) = authorized_legacy(pool.as_ref(), &mut connection);
    let report = |token, outcome, authorize: &dyn Fn(CrossrefWriteRoute) -> ThothResult<()>| {
        permit_crud::report_crossref_write(
            pool.as_ref(),
            reservation.permit_id,
            token,
            outcome,
            authorize,
        )
    };
    assert_eq!(
        report(reservation.reservation_token, Outcome::Accepted, &deny).map(|p| p.permit.state),
        Err(ThothError::Unauthorised)
    );
    assert_eq!(
        report(Uuid::new_v4(), Outcome::Accepted, &allow).map(|p| p.permit.state),
        Err(ThothError::CrossrefPermitRequiresReservationToken)
    );
    let reported =
        report(reservation.reservation_token, Outcome::Accepted, &allow).expect("reported");
    assert_eq!(reported.permit.state, CrossrefWritePermitState::Accepted);
    assert_eq!(
        state_of(&mut connection, reservation.permit_id),
        "ACCEPTED|-|true|true|-|-|-|-|-"
    );
    // A repeat after a lost response is refused, for the role and token holder only.
    assert_eq!(
        report(reservation.reservation_token, Outcome::Accepted, &allow).map(|p| p.permit.state),
        Err(ThothError::CrossrefPermitIllegalTransition)
    );
    assert_eq!(
        report(reservation.reservation_token, Outcome::Accepted, &deny).map(|p| p.permit.state),
        Err(ThothError::Unauthorised)
    );
    assert_eq!(
        permit_crud::report_crossref_write(
            pool.as_ref(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Outcome::Accepted,
            &allow
        )
        .map(|p| p.permit.state),
        Err(ThothError::CrossrefPermitNotFound)
    );

    // INDETERMINATE is reported but not closed, and blocks.
    let (reservation, _p, work) = authorized_legacy(pool.as_ref(), &mut connection);
    permit_crud::report_crossref_write(
        pool.as_ref(),
        reservation.permit_id,
        reservation.reservation_token,
        Outcome::Indeterminate,
        &allow,
    )
    .expect("reported");
    assert_eq!(
        state_of(&mut connection, reservation.permit_id),
        "INDETERMINATE|-|true|false|-|-|-|-|-"
    );
    assert_eq!(
        permit_crud::reserve_legacy_scheduled_crossref_write(pool.as_ref(), work)
            .map(|r| r.permit_id),
        Err(ThothError::CrossrefPermitBlocked)
    );
    // A RESERVED permit cannot be reported.
    let (publisher, imprint) = fx::publisher_and_imprint(pool.as_ref());
    fx::cover_crossref(&mut connection, publisher);
    let fresh = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let reserved = permit_crud::reserve_legacy_scheduled_crossref_write(pool.as_ref(), fresh)
        .expect("reserve");
    assert_eq!(
        permit_crud::report_crossref_write(
            pool.as_ref(),
            reserved.permit_id,
            reserved.reservation_token,
            Outcome::NoneAttempted,
            &allow
        )
        .map(|p| p.permit.state),
        Err(ThothError::CrossrefPermitIllegalTransition)
    );

    // NONE_ATTEMPTED history makes the next reservation strictly later.
    let (reservation, _p, work) = authorized_legacy(pool.as_ref(), &mut connection);
    permit_crud::report_crossref_write(
        pool.as_ref(),
        reservation.permit_id,
        reservation.reservation_token,
        Outcome::NoneAttempted,
        &allow,
    )
    .expect("reported");
    let next =
        permit_crud::reserve_legacy_scheduled_crossref_write(pool.as_ref(), work).expect("next");
    assert!(next.crossref_timestamp > reservation.crossref_timestamp);
}

#[test]
fn both_voids_release_only_a_reserved_permit() {
    let failing = test_db::failing_pool();
    for blank in [
        "",
        " ",
        "\t\n\r\u{0b}\u{0c}",
        "\u{0085}",
        "\u{00A0}",
        "\u{1680}",
        "\u{2003}",
        "\u{2028}",
        "\u{202F}",
        "\u{205F}",
        "\u{3000}",
        "\u{001C}",
        "\u{001F}",
        "\u{0001}",
        "\u{007F}",
        "\u{009F}",
    ] {
        assert_eq!(
            permit_crud::void_crossref_write_reservation_as_superuser(
                &failing,
                Uuid::new_v4(),
                blank,
                "REF"
            )
            .map(|p| p.permit.state),
            Err(ThothError::CrossrefPermitVoidRequiresDetail)
        );
        assert_eq!(
            permit_crud::void_crossref_write_reservation_as_superuser(
                &failing,
                Uuid::new_v4(),
                "detail",
                blank
            )
            .map(|p| p.permit.state),
            Err(ThothError::CrossrefVoidRequiresAuthorizationReference)
        );
        assert_eq!(
            permit_crud::reconcile_crossref_write_permit(
                &failing,
                Uuid::new_v4(),
                Outcome::Accepted,
                Recon::Reconciled,
                blank
            )
            .map(|p| p.permit.state),
            Err(ThothError::CrossrefReconciliationRequiresReference)
        );
    }

    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (publisher, imprint) = fx::publisher_and_imprint(pool.as_ref());
    fx::cover_crossref(&mut connection, publisher);
    let work = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let reservation =
        permit_crud::reserve_legacy_scheduled_crossref_write(pool.as_ref(), work).expect("reserve");
    let owner = |token, detail: &str| {
        permit_crud::void_crossref_write_reservation(
            pool.as_ref(),
            reservation.permit_id,
            token,
            detail,
            &allow,
        )
    };
    // Role, then the blank detail, then the token and the state.
    let deny = |_route: CrossrefWriteRoute| -> ThothResult<()> { Err(ThothError::Unauthorised) };
    assert_eq!(
        permit_crud::void_crossref_write_reservation(
            pool.as_ref(),
            reservation.permit_id,
            reservation.reservation_token,
            " ",
            &deny
        )
        .map(|p| p.permit.state),
        Err(ThothError::Unauthorised)
    );
    assert_eq!(
        owner(reservation.reservation_token, " ").map(|p| p.permit.state),
        Err(ThothError::CrossrefPermitVoidRequiresDetail)
    );
    assert_eq!(
        owner(Uuid::new_v4(), "gone").map(|p| p.permit.state),
        Err(ThothError::CrossrefPermitRequiresReservationToken)
    );
    let voided = owner(reservation.reservation_token, " worker crashed ").expect("voided");
    assert_eq!(voided.permit.state, CrossrefWritePermitState::Voided);
    assert_eq!(
        state_of(&mut connection, reservation.permit_id),
        "VOIDED|-|false|true|OWNER_ABANDONED| worker crashed |-|-|-"
    );
    assert_eq!(
        owner(reservation.reservation_token, "again").map(|p| p.permit.state),
        Err(ThothError::CrossrefPermitVoidRequiresReserved)
    );

    // The superuser void needs no token and persists its reference.
    let again = permit_crud::reserve_legacy_scheduled_crossref_write(pool.as_ref(), work)
        .expect("an overlapping reservation succeeds after the void");
    let cleaned = permit_crud::void_crossref_write_reservation_as_superuser(
        pool.as_ref(),
        again.permit_id,
        "cleanup",
        "INC-42",
    )
    .expect("cleaned");
    assert_eq!(
        cleaned.permit.void_authorization_reference.as_deref(),
        Some("INC-42")
    );
    assert_eq!(
        state_of(&mut connection, again.permit_id),
        "VOIDED|-|false|true|OPERATOR_CLEANUP|cleanup|INC-42|-|-"
    );
    assert_eq!(
        permit_crud::void_crossref_write_reservation_as_superuser(
            pool.as_ref(),
            Uuid::new_v4(),
            "cleanup",
            "INC-42"
        )
        .map(|p| p.permit.state),
        Err(ThothError::CrossrefPermitNotFound)
    );

    // Neither void touches an AUTHORIZED permit.
    let (authorized, _p, _w) = authorized_legacy(pool.as_ref(), &mut connection);
    assert_eq!(
        permit_crud::void_crossref_write_reservation(
            pool.as_ref(),
            authorized.permit_id,
            authorized.reservation_token,
            "no",
            &allow
        )
        .map(|p| p.permit.state),
        Err(ThothError::CrossrefPermitVoidRequiresReserved)
    );
    assert_eq!(
        permit_crud::void_crossref_write_reservation_as_superuser(
            pool.as_ref(),
            authorized.permit_id,
            "no",
            "REF"
        )
        .map(|p| p.permit.state),
        Err(ThothError::CrossrefPermitVoidRequiresReserved)
    );
}

#[test]
fn reconciliation_follows_the_section_16_8_table_exactly() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let reconcile = |permit, outcome, state, reference: &str| {
        permit_crud::reconcile_crossref_write_permit(
            pool.as_ref(),
            permit,
            outcome,
            state,
            reference,
        )
        .map(|p| p.permit.state)
    };

    // AUTHORIZED -> ACCEPTED, reconciled; provider_reported_at stays NULL.
    let (a, _p, _w) = authorized_legacy(pool.as_ref(), &mut connection);
    assert_eq!(
        reconcile(a.permit_id, Outcome::Accepted, Recon::Reconciled, "R-1"),
        Ok(CrossrefWritePermitState::Accepted)
    );
    assert_eq!(
        state_of(&mut connection, a.permit_id),
        "ACCEPTED|RECONCILED|false|true|-|-|-|-|R-1"
    );
    // A repeated confirmation returns the unchanged permit.
    let before = permit_row(&mut connection, a.permit_id);
    assert_eq!(
        reconcile(a.permit_id, Outcome::Accepted, Recon::Reconciled, "R-2"),
        Ok(CrossrefWritePermitState::Accepted)
    );
    assert_eq!(permit_row(&mut connection, a.permit_id), before);
    assert_eq!(
        reconcile(
            a.permit_id,
            Outcome::NoneAttempted,
            Recon::Reconciled,
            "R-2"
        ),
        Err(ThothError::CrossrefPermitIllegalTransition)
    );

    // AUTHORIZED -> INDETERMINATE annotated; a second annotation refused; then resolved.
    let (b, _p, _w) = authorized_legacy(pool.as_ref(), &mut connection);
    assert_eq!(
        reconcile(
            b.permit_id,
            Outcome::Indeterminate,
            Recon::ReconciliationImpossible,
            "ANN-1"
        ),
        Ok(CrossrefWritePermitState::Indeterminate)
    );
    assert_eq!(
        state_of(&mut connection, b.permit_id),
        "INDETERMINATE|RECONCILIATION_IMPOSSIBLE|false|false|-|-|-|ANN-1|-"
    );
    assert_eq!(
        reconcile(
            b.permit_id,
            Outcome::Indeterminate,
            Recon::ReconciliationRequired,
            "ANN-2"
        ),
        Err(ThothError::CrossrefPermitIllegalTransition)
    );
    assert_eq!(
        reconcile(
            b.permit_id,
            Outcome::NoneAttempted,
            Recon::Reconciled,
            "RES-1"
        ),
        Ok(CrossrefWritePermitState::NoneAttempted)
    );
    assert_eq!(
        state_of(&mut connection, b.permit_id),
        "NONE_ATTEMPTED|RECONCILED|false|true|-|-|-|ANN-1|RES-1"
    );

    // A reported INDETERMINATE annotated once (S2), then resolved; the report time stays.
    let (c, _p, _w) = authorized_legacy(pool.as_ref(), &mut connection);
    permit_crud::report_crossref_write(
        pool.as_ref(),
        c.permit_id,
        c.reservation_token,
        Outcome::Indeterminate,
        &allow,
    )
    .expect("report");
    assert_eq!(
        reconcile(
            c.permit_id,
            Outcome::Indeterminate,
            Recon::ReconciliationRequired,
            "ANN"
        ),
        Ok(CrossrefWritePermitState::Indeterminate)
    );
    assert_eq!(
        reconcile(c.permit_id, Outcome::Accepted, Recon::Reconciled, "RES"),
        Ok(CrossrefWritePermitState::Accepted)
    );
    assert_eq!(
        state_of(&mut connection, c.permit_id),
        "ACCEPTED|RECONCILED|true|true|-|-|-|ANN|RES"
    );

    // Mismatched presentations and reservations are refused.
    let (d, _p, _w) = authorized_legacy(pool.as_ref(), &mut connection);
    assert_eq!(
        reconcile(
            d.permit_id,
            Outcome::Accepted,
            Recon::ReconciliationRequired,
            "X"
        ),
        Err(ThothError::CrossrefPermitIllegalTransition)
    );
    assert_eq!(
        reconcile(d.permit_id, Outcome::Indeterminate, Recon::Reconciled, "X"),
        Err(ThothError::CrossrefPermitIllegalTransition)
    );
    let (publisher, imprint) = fx::publisher_and_imprint(pool.as_ref());
    fx::cover_crossref(&mut connection, publisher);
    let work = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let reserved =
        permit_crud::reserve_legacy_scheduled_crossref_write(pool.as_ref(), work).expect("reserve");
    assert_eq!(
        reconcile(
            reserved.permit_id,
            Outcome::Accepted,
            Recon::Reconciled,
            "X"
        ),
        Err(ThothError::CrossrefPermitIllegalTransition)
    );
    assert_eq!(
        reconcile(Uuid::new_v4(), Outcome::Accepted, Recon::Reconciled, "X"),
        Err(ThothError::CrossrefPermitNotFound)
    );
}

#[test]
fn a_fenced_abandonment_is_cleared_only_by_reconciliation_truth() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (_p, _i, _a, _work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    assert_eq!(
        finalise(pool.as_ref(), &presentation(&reservation, Some(token))).map(|r| r.outcome),
        Ok(Finalised::Authorized)
    );

    // The worker crashes after finalisation: the lease lapses and recovery runs.
    fx::execute(&mut connection, &format!("UPDATE distribution_job SET lease_expires_at = now() - interval '1 second' WHERE distribution_job_id = '{job}'"));
    assert!(
        fx::claim(pool.as_ref()).is_empty(),
        "recovered, then refused: fenced abandonment and a blocking permit"
    );
    let attempt = |connection: &mut PgConnection| {
        fx::texts(connection, &format!(
            "SELECT result::text || '|' || (fenced_at IS NOT NULL)::text || '|' || coalesce(recovery_clearance_reference, '-') AS value \
             FROM distribution_job_attempt WHERE distribution_job_id = '{job}' ORDER BY attempt_number"
        ))
    };
    assert_eq!(attempt(&mut connection), vec!["ABANDONED|true|-"]);

    // An annotation clears nothing.
    permit_crud::reconcile_crossref_write_permit(
        pool.as_ref(),
        reservation.permit_id,
        Outcome::Indeterminate,
        Recon::ReconciliationImpossible,
        "ANN-7",
    )
    .expect("annotate");
    assert_eq!(attempt(&mut connection), vec!["ABANDONED|true|-"]);
    assert!(fx::claim(pool.as_ref()).is_empty());

    // The resolution clears the attempt in the same transaction; the job is claimable.
    permit_crud::reconcile_crossref_write_permit(
        pool.as_ref(),
        reservation.permit_id,
        Outcome::NoneAttempted,
        Recon::Reconciled,
        "RES-7",
    )
    .expect("resolve");
    assert_eq!(attempt(&mut connection), vec!["ABANDONED|true|RES-7"]);
    let reclaimed = fx::claim(pool.as_ref());
    assert_eq!(reclaimed.len(), 1);
    assert_eq!(reclaimed[0].attempt_number, 2);

    // No public operation other than reconciliation clears a fenced abandonment.
    let crud = include_str!("crud.rs");
    assert_eq!(crud.matches("clear_fenced_abandonment(").count(), 1);
    let substrate = include_str!("../work_upsert/crud.rs");
    assert_eq!(substrate.matches("fn clear_fenced_abandonment(").count(), 1);
}

#[test]
fn confirmation_of_a_reported_outcome_clears_the_attempt() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (_p, _i, _a, _work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    finalise(pool.as_ref(), &presentation(&reservation, Some(token))).expect("finalise");
    permit_crud::report_crossref_write(
        pool.as_ref(),
        reservation.permit_id,
        reservation.reservation_token,
        Outcome::Accepted,
        &allow,
    )
    .expect("report");
    fx::execute(&mut connection, &format!("UPDATE distribution_job SET lease_expires_at = now() - interval '1 second' WHERE distribution_job_id = '{job}'"));
    assert!(
        fx::claim(pool.as_ref()).is_empty(),
        "the fenced abandonment blocks"
    );
    permit_crud::reconcile_crossref_write_permit(
        pool.as_ref(),
        reservation.permit_id,
        Outcome::Accepted,
        Recon::Reconciled,
        "CONF-1",
    )
    .expect("confirm");
    assert_eq!(
        fx::claim(pool.as_ref()).len(),
        1,
        "confirmation cleared the attempt"
    );
}

#[test]
fn a_back_catalogue_unit_is_deposited_at_most_once_per_outer_job() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (publisher, imprint) = fx::publisher_and_imprint(pool.as_ref());
    let activation = fx::cover_crossref(&mut connection, publisher);
    let unit = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    fx::execute(&mut connection, &format!(
        "INSERT INTO distribution_job (kind, publisher_id, activation_id, deduplication_key) \
         VALUES ('PUBLISHER_BACK_CATALOGUE', '{publisher}', '{activation}', 'PUBLISHER_BACK_CATALOGUE:{publisher}:{activation}'); \
         INSERT INTO distribution_job_target (distribution_job_id, platform) \
         SELECT distribution_job_id, 'CROSSREF' FROM distribution_job WHERE kind = 'PUBLISHER_BACK_CATALOGUE'"
    ));
    let outer =
        job_crud::claim_distribution_jobs(pool.as_ref(), "legacy", 10, 900, &[]).expect("claim");
    let (outer_job, outer_token) = (outer[0].job.job.distribution_job_id, outer[0].claim_token);
    let first = permit_crud::reserve_back_catalogue_crossref_write(
        pool.as_ref(),
        outer_job,
        outer_token,
        unit,
    )
    .expect("unit");
    assert_eq!(
        finalise(pool.as_ref(), &presentation(&first, Some(outer_token))).map(|r| r.outcome),
        Ok(Finalised::Authorized)
    );
    permit_crud::report_crossref_write(
        pool.as_ref(),
        first.permit_id,
        first.reservation_token,
        Outcome::Accepted,
        &allow,
    )
    .expect("report");
    assert_eq!(
        permit_crud::reserve_back_catalogue_crossref_write(
            pool.as_ref(),
            outer_job,
            outer_token,
            unit
        )
        .map(|r| r.permit_id),
        Err(ThothError::CrossrefUnitAlreadyDepositedInJob)
    );
}

// ---------------------------------------------------------------------------
// Amendment 3 section 9.9: the version floor advance (F1-F3, F5-F12, F14, F15,
// F19, F21, F22)
// ---------------------------------------------------------------------------

use crate::model::crossref_write_permit::crud::AdvanceCrossrefVersionFloor;

const TARGET: i64 = 99_999_999_999_999;

fn advance_input(g6_attempt_id: Uuid) -> AdvanceCrossrefVersionFloor {
    AdvanceCrossrefVersionFloor {
        target_value: TARGET,
        g6_attempt_id,
        observation_id: Uuid::new_v4(),
        authorization_reference: " G7-AUTH-1 ".to_string(),
        authorization_register_digest: "a".repeat(64),
    }
}

fn floor_state(connection: &mut PgConnection) -> String {
    fx::texts(
        connection,
        "SELECT (SELECT floor_value::text || '|' || xmin::text FROM work_crossref_version_floor) || '#' \
             || coalesce((SELECT string_agg(audit_id::text || xmin::text, ',') FROM crossref_version_floor_audit), '') AS value",
    )
    .remove(0)
}

#[test]
fn f1_f2_f3_f15_a_single_advance_writes_exactly_one_audit_row() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let attempt = Uuid::new_v4();
    let input = advance_input(attempt);
    let before = fx::texts(&mut connection, "SELECT now()::text AS value").remove(0);
    let advance = permit_crud::advance_crossref_version_floor(pool.as_ref(), &input, "user-g7")
        .expect("advanced");
    assert_eq!((advance.before_value, advance.after_value), (0, TARGET));
    assert_eq!(
        (advance.g6_attempt_id, advance.observation_id),
        (attempt, input.observation_id)
    );
    assert_eq!(
        advance.authorization_reference, " G7-AUTH-1 ",
        "stored verbatim"
    );
    assert_eq!(advance.authorization_register_digest, "a".repeat(64));
    assert_eq!(advance.actor, "user-g7");
    let _ = before;
    assert_eq!(
        fx::texts(
            &mut connection,
            "SELECT floor_value::text AS value FROM work_crossref_version_floor"
        ),
        vec![TARGET.to_string()]
    );
    assert_eq!(
        fx::texts(&mut connection, "SELECT audit_id::text || '|' || mutation_kind || '|' || g7_authorization_reference || '|' || actor AS value FROM crossref_version_floor_audit"),
        vec![format!("{}|ADVANCE_VERSION_FLOOR| G7-AUTH-1 |user-g7", advance.audit_id)]
    );
    // F15: the audit row is append-only.
    for statement in [
        "UPDATE crossref_version_floor_audit SET actor = 'x'",
        "DELETE FROM crossref_version_floor_audit",
        "TRUNCATE crossref_version_floor_audit",
    ] {
        assert_eq!(
            fx::refusal(&mut connection, statement),
            "CROSSREF_VERSION_FLOOR_AUDIT_APPEND_ONLY",
            "{statement}"
        );
    }
    // F3 and F5/F19.
    let state = floor_state(&mut connection);
    assert_eq!(
        permit_crud::advance_crossref_version_floor(pool.as_ref(), &input, "user-g7")
            .map(|a| a.audit_id),
        Err(ThothError::CrossrefVersionFloorAlreadyAdvanced)
    );
    assert_eq!(
        permit_crud::advance_crossref_version_floor(
            pool.as_ref(),
            &advance_input(Uuid::new_v4()),
            "user-g7"
        )
        .map(|a| a.audit_id),
        Err(ThothError::CrossrefVersionFloorBindingMismatch)
    );
    assert_eq!(
        floor_state(&mut connection),
        state,
        "F14/F19: refusals change nothing"
    );
}

#[test]
fn f6_to_f11_f21_argument_refusals_precede_every_database_access() {
    let failing = test_db::failing_pool();
    let refused = |input: AdvanceCrossrefVersionFloor| {
        permit_crud::advance_crossref_version_floor(&failing, &input, "user").map(|a| a.audit_id)
    };
    for target in [
        0,
        1,
        99_999_999_999_998,
        100_000_000_000_000,
        20_260_914_120_000_000,
        i64::MAX,
    ] {
        assert_eq!(
            refused(AdvanceCrossrefVersionFloor {
                target_value: target,
                ..advance_input(Uuid::new_v4())
            }),
            Err(ThothError::CrossrefVersionFloorTargetInvalid),
            "{target}"
        );
    }
    for blank in [
        "",
        " ",
        "\t\n\r\u{0b}\u{0c}",
        "\u{00A0}\u{2003}\u{3000}",
        "\u{001F}",
        "\u{0001}",
    ] {
        assert_eq!(
            refused(AdvanceCrossrefVersionFloor {
                authorization_reference: blank.to_string(),
                ..advance_input(Uuid::new_v4())
            }),
            Err(ThothError::CrossrefVersionFloorRequiresAuthorizationReference),
            "{blank:?}"
        );
    }
    for digest in [
        String::new(),
        "a".repeat(63),
        "a".repeat(65),
        "A".repeat(64),
        format!("{}g", "a".repeat(63)),
        format!("{} ", "a".repeat(63)),
        format!("{}\u{FF11}", "a".repeat(63)),
    ] {
        assert_eq!(
            refused(AdvanceCrossrefVersionFloor {
                authorization_register_digest: digest.clone(),
                ..advance_input(Uuid::new_v4())
            }),
            Err(ThothError::CrossrefVersionFloorRegisterDigestInvalid),
            "{digest:?}"
        );
    }
    // All three invalid: the target first.
    assert_eq!(
        refused(AdvanceCrossrefVersionFloor {
            target_value: 1,
            authorization_reference: String::new(),
            authorization_register_digest: String::new(),
            ..advance_input(Uuid::new_v4())
        }),
        Err(ThothError::CrossrefVersionFloorTargetInvalid)
    );

    // F21: while another connection holds the floor lock, an argument refusal returns without waiting.
    let (_guard, pool) = test_db::setup_test_db();
    let mut holder = pool.get().expect("holder");
    fx::execute(
        &mut holder,
        "BEGIN; SELECT floor_value FROM work_crossref_version_floor FOR UPDATE",
    );
    let started = std::time::Instant::now();
    assert_eq!(
        permit_crud::advance_crossref_version_floor(
            pool.as_ref(),
            &AdvanceCrossrefVersionFloor {
                authorization_register_digest: "x".into(),
                ..advance_input(Uuid::new_v4())
            },
            "user"
        )
        .map(|a| a.audit_id),
        Err(ThothError::CrossrefVersionFloorRegisterDigestInvalid)
    );
    assert!(started.elapsed() < std::time::Duration::from_secs(2));
    fx::execute(&mut holder, "ROLLBACK");
}

#[test]
fn f12_f22_a_blocking_permit_refuses_not_drained_before_binding_mismatch() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (publisher, imprint) = fx::publisher_and_imprint(pool.as_ref());
    fx::cover_crossref(&mut connection, publisher);

    // F12: each blocking state refuses NOT_DRAINED; resolved, the advance succeeds.
    let work = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let reserved =
        permit_crud::reserve_legacy_scheduled_crossref_write(pool.as_ref(), work).expect("reserve");
    let attempt = Uuid::new_v4();
    let state = floor_state(&mut connection);
    assert_eq!(
        permit_crud::advance_crossref_version_floor(pool.as_ref(), &advance_input(attempt), "user")
            .map(|a| a.audit_id),
        Err(ThothError::CrossrefVersionFloorNotDrained)
    );
    assert_eq!(
        finalise(pool.as_ref(), &presentation(&reserved, None)).map(|r| r.outcome),
        Ok(Finalised::Authorized)
    );
    assert_eq!(
        permit_crud::advance_crossref_version_floor(pool.as_ref(), &advance_input(attempt), "user")
            .map(|a| a.audit_id),
        Err(ThothError::CrossrefVersionFloorNotDrained)
    );
    permit_crud::report_crossref_write(
        pool.as_ref(),
        reserved.permit_id,
        reserved.reservation_token,
        Outcome::Indeterminate,
        &allow,
    )
    .expect("report");
    assert_eq!(
        permit_crud::advance_crossref_version_floor(pool.as_ref(), &advance_input(attempt), "user")
            .map(|a| a.audit_id),
        Err(ThothError::CrossrefVersionFloorNotDrained)
    );
    assert_eq!(floor_state(&mut connection), state);
    permit_crud::reconcile_crossref_write_permit(
        pool.as_ref(),
        reserved.permit_id,
        Outcome::NoneAttempted,
        Recon::Reconciled,
        "RES",
    )
    .expect("resolve");
    permit_crud::advance_crossref_version_floor(pool.as_ref(), &advance_input(attempt), "user")
        .expect("advanced");

    // F22: the floor at target and a blocking permit present.
    let work = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let blocking = permit_crud::reserve_legacy_scheduled_crossref_write(pool.as_ref(), work)
        .expect("reserve after the advance");
    assert!(blocking.crossref_timestamp > TARGET);
    let state = floor_state(&mut connection);
    let fresh = Uuid::new_v4();
    assert_eq!(
        permit_crud::advance_crossref_version_floor(pool.as_ref(), &advance_input(fresh), "user")
            .map(|a| a.audit_id),
        Err(ThothError::CrossrefVersionFloorNotDrained)
    );
    assert_eq!(
        permit_crud::advance_crossref_version_floor(pool.as_ref(), &advance_input(attempt), "user")
            .map(|a| a.audit_id),
        Err(ThothError::CrossrefVersionFloorAlreadyAdvanced),
        "F18"
    );
    permit_crud::void_crossref_write_reservation_as_superuser(
        pool.as_ref(),
        blocking.permit_id,
        "cleanup",
        "REF",
    )
    .expect("void");
    assert_eq!(
        permit_crud::advance_crossref_version_floor(pool.as_ref(), &advance_input(fresh), "user")
            .map(|a| a.audit_id),
        Err(ThothError::CrossrefVersionFloorBindingMismatch)
    );
    let after = floor_state(&mut connection);
    assert_eq!(
        after.split('#').next(),
        state.split('#').next(),
        "floor unchanged"
    );
    assert_eq!(
        fx::count(
            &mut connection,
            "SELECT count(*) AS count FROM crossref_version_floor_audit"
        ),
        1
    );
}

// ---------------------------------------------------------------------------
// R52B sections 10.8, 11.5, 12.3, 14.4 and 18.10: completion, failure and
// cancellation guards and the T2 successor (Amendment 3 section 10.3, EB2)
// ---------------------------------------------------------------------------

fn job_state(connection: &mut PgConnection, job: Uuid) -> String {
    fx::texts(
        connection,
        &format!(
            "SELECT j.status::text || '|' || coalesce(j.cancellation_reason::text, '-') || '|' \
                 || coalesce((SELECT string_agg(coalesce(a.result::text, 'OPEN'), ',' ORDER BY a.attempt_number) \
                                FROM distribution_job_attempt a WHERE a.distribution_job_id = j.distribution_job_id), '-') AS value \
             FROM distribution_job j WHERE j.distribution_job_id = '{job}'"
        ),
    )
    .remove(0)
}

#[test]
fn completion_requires_the_fence_and_an_accepted_permit_at_the_claimed_generation() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (_p, _i, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);

    assert_eq!(
        job_crud::complete_distribution_job(pool.as_ref(), job, token).map(|j| j.status),
        Err(ThothError::WorkUpsertCompletionRequiresFence)
    );
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    assert_eq!(
        job_crud::fail_distribution_job(
            pool.as_ref(),
            job,
            token,
            "CROSSREF_ARTIFACT_REFUSED",
            None,
            true
        )
        .map(|j| j.status),
        Err(ThothError::AttemptHasOpenReservation)
    );
    finalise(pool.as_ref(), &presentation(&reservation, Some(token))).expect("finalise");
    assert_eq!(
        job_crud::complete_distribution_job(pool.as_ref(), job, token).map(|j| j.status),
        Err(ThothError::WorkUpsertCompletionRequiresAcceptedPermit)
    );
    assert_eq!(
        job_crud::fail_distribution_job(
            pool.as_ref(),
            job,
            token,
            "CROSSREF_PROVIDER_INDETERMINATE",
            None,
            false
        )
        .map(|j| j.status),
        Err(ThothError::AttemptHasAuthorizedPermit)
    );
    // Administrative cancellation of a fenced attempt is refused and writes nothing.
    let before = fingerprint(&mut connection);
    assert_eq!(
        job_crud::cancel_distribution_job(pool.as_ref(), job).map(|j| j.status),
        Err(ThothError::WorkUpsertCancellationRefusedFencedAttempt)
    );
    assert_eq!(fingerprint(&mut connection), before);
    assert_eq!(job_state(&mut connection, job), "RUNNING|-|OPEN");

    permit_crud::report_crossref_write(
        pool.as_ref(),
        reservation.permit_id,
        reservation.reservation_token,
        Outcome::Accepted,
        &allow,
    )
    .expect("report");
    let completed =
        job_crud::complete_distribution_job(pool.as_ref(), job, token).expect("completed");
    assert_eq!(completed.status, DistributionJobStatus::Succeeded);
    assert_eq!(job_state(&mut connection, job), "SUCCEEDED|-|SUCCEEDED");
    assert_eq!(
        fx::job_summary(&mut connection, work).len(),
        1,
        "no successor without newer residue"
    );
}

#[test]
fn t2_creates_a_successor_only_under_the_held_publisher() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");

    let deposit = |pool: &crate::db::PgPool, connection: &mut PgConnection| {
        let (publisher, imprint, activation, work, job, token) =
            claimed_work_upsert(pool, connection);
        let reservation = reserve_work_upsert(pool, job, token).expect("reserve");
        finalise(pool, &presentation(&reservation, Some(token))).expect("finalise");
        permit_crud::report_crossref_write(
            pool,
            reservation.permit_id,
            reservation.reservation_token,
            Outcome::Accepted,
            &allow,
        )
        .expect("report");
        (publisher, imprint, activation, work, job, token)
    };

    // An edit after the claim: the successor is created at the current generation.
    let (_publisher, _imprint, _activation, work, job, token) =
        deposit(pool.as_ref(), &mut connection);
    fx::execute(
        &mut connection,
        &format!("UPDATE work SET place = 'Leeds' WHERE work_id = '{work}'"),
    );
    job_crud::complete_distribution_job(pool.as_ref(), job, token).expect("completed");
    assert_eq!(
        fx::job_summary(&mut connection, work),
        vec![
            "SUCCEEDED|-|1|1|false|true|CROSSREF",
            "PENDING|-|2|2|true|false|CROSSREF"
        ]
    );

    // A move to another publisher: no successor; the residue is the drain's.
    let (_publisher, _imprint, _activation, work, job, token) =
        deposit(pool.as_ref(), &mut connection);
    let (_other, other_imprint) = fx::publisher_and_imprint(pool.as_ref());
    fx::execute(
        &mut connection,
        &format!("UPDATE work SET imprint_id = '{other_imprint}' WHERE work_id = '{work}'"),
    );
    job_crud::complete_distribution_job(pool.as_ref(), job, token).expect("completed");
    assert_eq!(
        fx::job_summary(&mut connection, work),
        vec!["SUCCEEDED|-|1|1|false|false|CROSSREF"]
    );

    // Execution paused: T2 still creates the successor (it takes no gate).
    let (_publisher, _imprint, _activation, work, job, token) =
        deposit(pool.as_ref(), &mut connection);
    work_upsert_crud::set_work_upsert_execution(
        pool.as_ref(),
        DistributionPlatform::Crossref,
        false,
    )
    .expect("pause");
    fx::execute(
        &mut connection,
        &format!("UPDATE work SET place = 'York' WHERE work_id = '{work}'"),
    );
    job_crud::complete_distribution_job(pool.as_ref(), job, token).expect("completed while paused");
    assert_eq!(fx::job_summary(&mut connection, work).len(), 2);
}

#[test]
fn an_unfenced_cancellation_touches_no_permit_and_the_retry_projects_the_current_generation() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");

    // C2/T197: cancelling a job whose attempt holds only a RESERVED permit.
    let (_p, _i, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let permit_before = permit_row(&mut connection, reservation.permit_id);
    let cancelled = job_crud::cancel_distribution_job(pool.as_ref(), job).expect("cancelled");
    assert_eq!(cancelled.status, DistributionJobStatus::Cancelled);
    assert_eq!(
        permit_row(&mut connection, reservation.permit_id),
        permit_before
    );
    assert_eq!(
        permit_crud::reserve_legacy_scheduled_crossref_write(pool.as_ref(), work)
            .map(|r| r.permit_id),
        Err(ThothError::CrossrefPermitBlocked)
    );

    // T224: VOIDED_RETRYABLE, then a retryable failure, then the next claim.
    let (_p, _i, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    fx::execute(
        &mut connection,
        &format!("UPDATE work SET place = 'Bath' WHERE work_id = '{work}'"),
    );
    assert_eq!(
        finalise(pool.as_ref(), &presentation(&reservation, Some(token))).map(|r| r.outcome),
        Ok(Finalised::VoidedRetryable)
    );
    let failed = job_crud::fail_distribution_job(
        pool.as_ref(),
        job,
        token,
        "CROSSREF_PERMIT_VOIDED_RETRYABLE",
        None,
        true,
    )
    .expect("failed");
    assert_eq!(failed.status, DistributionJobStatus::Pending);
    fx::execute(
        &mut connection,
        &format!(
            "UPDATE distribution_job SET available_at = now() WHERE distribution_job_id = '{job}'"
        ),
    );
    let claimed = fx::claim(pool.as_ref());
    let again = claimed
        .iter()
        .find(|c| c.job.job.distribution_job_id == job)
        .expect("reclaimed");
    // The released payload orders attempts newest first.
    let attempts = again.job.preloaded_attempts.clone().expect("attempts");
    assert_eq!(attempts.first().map(|a| a.attempt_number), Some(2));
    assert_eq!(
        attempts.first().and_then(|a| a.claimed_generation),
        fx::generation_of(&mut connection, work)
    );
    assert_eq!(attempts.first().and_then(|a| a.claimed_generation), Some(2));
}

#[test]
fn an_outer_back_catalogue_attempt_cannot_close_over_an_open_unit_permit() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (publisher, imprint) = fx::publisher_and_imprint(pool.as_ref());
    let activation = fx::cover_crossref(&mut connection, publisher);
    let units: Vec<Uuid> = (0..3)
        .map(|_| fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4()))
        .collect();
    fx::execute(&mut connection, &format!(
        "INSERT INTO distribution_job (kind, publisher_id, activation_id, deduplication_key) \
         VALUES ('PUBLISHER_BACK_CATALOGUE', '{publisher}', '{activation}', 'PUBLISHER_BACK_CATALOGUE:{publisher}:{activation}'); \
         INSERT INTO distribution_job_target (distribution_job_id, platform) \
         SELECT distribution_job_id, 'CROSSREF' FROM distribution_job WHERE kind = 'PUBLISHER_BACK_CATALOGUE'"
    ));
    let outer =
        job_crud::claim_distribution_jobs(pool.as_ref(), "legacy", 10, 900, &[]).expect("claim");
    let (job, token) = (outer[0].job.job.distribution_job_id, outer[0].claim_token);

    // RESERVED.
    let reserved =
        permit_crud::reserve_back_catalogue_crossref_write(pool.as_ref(), job, token, units[0])
            .expect("unit");
    assert_eq!(
        job_crud::complete_distribution_job(pool.as_ref(), job, token).map(|j| j.status),
        Err(ThothError::AttemptHasOpenReservation)
    );
    assert_eq!(
        job_crud::fail_distribution_job(pool.as_ref(), job, token, "X", None, false)
            .map(|j| j.status),
        Err(ThothError::AttemptHasOpenReservation)
    );
    // AUTHORIZED.
    finalise(pool.as_ref(), &presentation(&reserved, Some(token))).expect("finalise");
    assert_eq!(
        job_crud::complete_distribution_job(pool.as_ref(), job, token).map(|j| j.status),
        Err(ThothError::AttemptHasAuthorizedPermit)
    );
    assert_eq!(
        job_crud::fail_distribution_job(pool.as_ref(), job, token, "X", None, true)
            .map(|j| j.status),
        Err(ThothError::AttemptHasAuthorizedPermit)
    );
    // INDETERMINATE: the outer attempt cannot terminally close, but may retry.
    permit_crud::report_crossref_write(
        pool.as_ref(),
        reserved.permit_id,
        reserved.reservation_token,
        Outcome::Indeterminate,
        &allow,
    )
    .expect("report");
    assert_eq!(
        job_crud::complete_distribution_job(pool.as_ref(), job, token).map(|j| j.status),
        Err(ThothError::OuterAttemptHasOpenPermits)
    );
    assert_eq!(
        job_crud::fail_distribution_job(pool.as_ref(), job, token, "X", None, false)
            .map(|j| j.status),
        Err(ThothError::OuterAttemptHasOpenPermits)
    );
    assert_eq!(
        job_crud::fail_distribution_job(pool.as_ref(), job, token, "X", None, true)
            .map(|j| j.status),
        Ok(DistributionJobStatus::Pending)
    );

    // Outer cancellation touches no unit permit.
    let outer = job_crud::claim_distribution_jobs(pool.as_ref(), "legacy", 10, 900, &[]);
    let _ = outer;
    fx::execute(
        &mut connection,
        &format!(
            "UPDATE distribution_job SET available_at = now() WHERE distribution_job_id = '{job}'"
        ),
    );
    let outer =
        job_crud::claim_distribution_jobs(pool.as_ref(), "legacy", 10, 900, &[]).expect("reclaim");
    let token = outer[0].claim_token;
    let second =
        permit_crud::reserve_back_catalogue_crossref_write(pool.as_ref(), job, token, units[1])
            .expect("unit");
    let before = permit_row(&mut connection, second.permit_id);
    job_crud::cancel_distribution_job(pool.as_ref(), job).expect("cancel");
    assert_eq!(permit_row(&mut connection, second.permit_id), before);
}

#[test]
fn x4_every_be06_statement_in_the_shared_job_operations_uses_the_scoped_conversion() {
    let crud = include_str!("../distribution_job/crud.rs");
    let functions = [
        "work_upsert_completion_guard",
        "complete_work_upsert_successor",
        "attempt_permit_guard",
        "work_upsert_cancellation_guard",
    ];
    for name in functions {
        let body = crud
            .split_once(&format!("fn {name}("))
            .unwrap_or_else(|| panic!("{name}"))
            .1
            .split_once("\n}\n")
            .expect("end")
            .0;
        // Every call that takes the connection is a statement or a substrate
        // helper; the helpers that already convert return ThothResult.
        let self_converting = ["work_upsert_create_job(", "attempt_permit_guard("];
        let calls = body.matches("(connection").count()
            - self_converting
                .iter()
                .map(|helper| body.matches(helper).count())
                .sum::<usize>();
        assert!(calls > 0, "{name} runs statements");
        let converted = body.matches(".work_upsert()").count();
        assert!(
            converted >= calls,
            "{name}: {calls} statement calls but {converted} conversions"
        );
        assert!(!body.contains("map_err(Into::into)"), "{name}");
    }
    for released in [
        "pub(crate) fn complete_distribution_job(",
        "pub(crate) fn fail_distribution_job(",
        "pub(crate) fn cancel_distribution_job(",
    ] {
        let body = crud
            .split_once(released)
            .expect(released)
            .1
            .split_once("\n}\n")
            .expect("end")
            .0;
        assert!(
            functions
                .iter()
                .any(|name| body.contains(&format!("{name}("))),
            "{released} calls its BE-06 guard"
        );
    }
}

// ---------------------------------------------------------------------------
// Amendment 3 section 4.4 entries 11-14: permit, floor and safety reports
// ---------------------------------------------------------------------------

#[test]
fn reports_11_to_14_list_permits_unresolved_permits_the_floor_and_drain() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    assert_eq!(
        permit_crud::crossref_blocking_write_permit_count(pool.as_ref()),
        Ok(0)
    );
    assert_eq!(permit_crud::crossref_drained(pool.as_ref()), Ok(true));

    let (_p, _i, _a, _work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let (authorized, _publisher, _w) = authorized_legacy(pool.as_ref(), &mut connection);
    permit_crud::report_crossref_write(
        pool.as_ref(),
        authorized.permit_id,
        authorized.reservation_token,
        Outcome::NoneAttempted,
        &allow,
    )
    .expect("report");
    assert_eq!(
        permit_crud::crossref_blocking_write_permit_count(pool.as_ref()),
        Ok(1)
    );
    assert_eq!(permit_crud::crossref_drained(pool.as_ref()), Ok(false));

    let attempt = fx::texts(&mut connection, &format!("SELECT distribution_job_attempt_id::text AS value FROM distribution_job_attempt WHERE distribution_job_id = '{job}'")).remove(0);
    let filter = permit_crud::CrossrefWritePermitFilter {
        job_identity: Some(job),
        attempt_identity: Some(Uuid::parse_str(&attempt).expect("uuid")),
        ..Default::default()
    };
    let found =
        permit_crud::crossref_write_permits(pool.as_ref(), &filter, 100, 0).expect("report");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].permit.permit_id, reservation.permit_id);
    assert_eq!(found[0].dois, reservation.dois);
    assert_eq!(
        permit_crud::crossref_write_permits(pool.as_ref(), &Default::default(), 100, 0)
            .map(|p| p.len()),
        Ok(2)
    );

    let unresolved = permit_crud::crossref_unresolved_permits(pool.as_ref()).expect("report");
    assert_eq!(
        unresolved
            .iter()
            .map(|p| p.permit.permit_id)
            .collect::<Vec<_>>(),
        vec![reservation.permit_id]
    );

    let floor = permit_crud::crossref_version_floor(pool.as_ref()).expect("report");
    assert_eq!((floor.floor_value, floor.advances.len()), (0, 0));
}

// ---------------------------------------------------------------------------------------------------------------------
// Amendment 3 section 11.2: R5, R6, R9, R11, R12, B3, B6, P1, P2.
// ---------------------------------------------------------------------------------------------------------------------

use crate::model::crossref_write_permit::CrossrefWriteOutcome;
use crate::model::Crud;

/// A claimed outer back-catalogue job over a covered publisher: `(publisher, imprint, job, token)`.
fn claimed_back_catalogue(
    pool: &crate::db::PgPool,
    connection: &mut PgConnection,
) -> (Uuid, Uuid, Uuid, Uuid) {
    let (publisher, imprint) = fx::publisher_and_imprint(pool);
    let activation = fx::cover_crossref(connection, publisher);
    fx::execute(connection, &format!(
        "INSERT INTO distribution_job (kind, publisher_id, activation_id, deduplication_key) \
         VALUES ('PUBLISHER_BACK_CATALOGUE', '{publisher}', '{activation}', 'PUBLISHER_BACK_CATALOGUE:{publisher}:{activation}'); \
         INSERT INTO distribution_job_target (distribution_job_id, platform) \
         SELECT distribution_job_id, 'CROSSREF' FROM distribution_job WHERE kind = 'PUBLISHER_BACK_CATALOGUE' AND publisher_id = '{publisher}'"
    ));
    let outer = job_crud::claim_distribution_jobs(pool, "legacy", 10, 900, &[]).expect("claim");
    let claimed = outer
        .into_iter()
        .find(|claimed| claimed.job.job.publisher_id == publisher)
        .expect("the outer job");
    (
        publisher,
        imprint,
        claimed.job.job.distribution_job_id,
        claimed.claim_token,
    )
}

fn load_job(
    connection: &mut PgConnection,
    job: Uuid,
) -> crate::model::distribution_job::DistributionJob {
    use crate::schema::distribution_job;
    use diesel::QueryDsl;
    distribution_job::table
        .find(job)
        .first(connection)
        .expect("job")
}

fn permit_count(connection: &mut PgConnection) -> i64 {
    fx::count(
        connection,
        "SELECT count(*) AS count FROM crossref_write_permit",
    )
}

#[test]
fn r5_a_unit_moved_out_after_the_outer_claim_is_a_publisher_mismatch() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (_publisher, imprint, job, token) = claimed_back_catalogue(pool.as_ref(), &mut connection);
    let unit = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let (_other_publisher, other_imprint) = fx::publisher_and_imprint(pool.as_ref());
    fx::execute(
        &mut connection,
        &format!("UPDATE work SET imprint_id = '{other_imprint}' WHERE work_id = '{unit}'"),
    );
    fx::uncover(&mut connection, unit);
    assert_eq!(
        permit_crud::reserve_back_catalogue_crossref_write(pool.as_ref(), job, token, unit)
            .map(|r| r.permit_id),
        Err(ThothError::CrossrefUnitPublisherMismatch)
    );
    assert_eq!(
        fx::generation_of(&mut connection, unit),
        None,
        "no 0 row is written"
    );
    assert_eq!(permit_count(&mut connection), 0);
}

/// Bring a fresh claimed attempt's one permit to `state`; returns `(job, claim token, permit, reservation token)`.
fn attempt_with_permit_in(
    pool: &crate::db::PgPool,
    connection: &mut PgConnection,
    state: CrossrefWritePermitState,
) -> (Uuid, Uuid, Uuid, Uuid) {
    let (_publisher, _imprint, _activation, _work, job, token) =
        claimed_work_upsert(pool, connection);
    let reservation = reserve_work_upsert(pool, job, token).expect("reserve");
    let (permit, reservation_token) = (reservation.permit_id, reservation.reservation_token);
    match state {
        CrossrefWritePermitState::Reserved => {}
        CrossrefWritePermitState::Voided => {
            permit_crud::void_crossref_write_reservation(
                pool,
                permit,
                reservation_token,
                "released by the worker",
                &allow,
            )
            .expect("void");
        }
        other => {
            finalise(pool, &presentation(&reservation, Some(token))).expect("finalise");
            let outcome = match other {
                CrossrefWritePermitState::Authorized => None,
                CrossrefWritePermitState::Indeterminate => {
                    Some(CrossrefWriteOutcome::Indeterminate)
                }
                CrossrefWritePermitState::Accepted => Some(CrossrefWriteOutcome::Accepted),
                CrossrefWritePermitState::NoneAttempted => {
                    Some(CrossrefWriteOutcome::NoneAttempted)
                }
                _ => unreachable!(),
            };
            if let Some(outcome) = outcome {
                permit_crud::report_crossref_write(
                    pool,
                    permit,
                    reservation_token,
                    outcome,
                    &allow,
                )
                .expect("report");
            }
        }
    }
    assert_eq!(
        state_of(connection, permit)
            .split('|')
            .next()
            .expect("state")
            .to_string(),
        format!("{state:?}")
            .chars()
            .fold(String::new(), |mut s, c| {
                if c.is_uppercase() && !s.is_empty() {
                    s.push('_');
                }
                s.push(c.to_ascii_uppercase());
                s
            }),
        "the permit is in the state under test"
    );
    (job, token, permit, reservation_token)
}

const PERMIT_STATES: [CrossrefWritePermitState; 6] = [
    CrossrefWritePermitState::Reserved,
    CrossrefWritePermitState::Voided,
    CrossrefWritePermitState::Authorized,
    CrossrefWritePermitState::Indeterminate,
    CrossrefWritePermitState::Accepted,
    CrossrefWritePermitState::NoneAttempted,
];

#[test]
fn r6_a_second_reservation_on_an_attempt_is_refused_in_every_permit_state() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    for state in PERMIT_STATES {
        let (job, token, permit, reservation_token) =
            attempt_with_permit_in(pool.as_ref(), &mut connection, state);
        let before = fingerprint(&mut connection);
        let refused = reserve_work_upsert(pool.as_ref(), job, token);
        assert_eq!(
            refused.as_ref().map(|r| r.permit_id),
            Err(&ThothError::CrossrefPermitAttemptAlreadyReserved),
            "{state:?}"
        );
        assert_eq!(
            fingerprint(&mut connection),
            before,
            "{state:?}: nothing written"
        );
        let message = refused.expect_err("refused").to_string();
        for disclosed in [
            permit.to_string(),
            reservation_token.to_string(),
            format!("{state:?}"),
            "RESERVED".to_string(),
            "VOIDED".to_string(),
            "AUTHORIZED".to_string(),
        ] {
            assert!(
                !message.to_lowercase().contains(&disclosed.to_lowercase()),
                "{state:?}: the refusal discloses {disclosed}: {message}"
            );
        }
        assert_eq!(
            fx::count(
                &mut connection,
                &format!("SELECT count(*) AS count FROM crossref_write_permit WHERE job_identity = '{job}'")
            ),
            1,
            "{state:?}: one permit per attempt"
        );
    }
}

#[test]
fn r9_a_deleted_work_makes_the_claim_stale_and_a_jobless_root_not_found() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (_publisher, imprint, _activation, work, job, token) =
        claimed_work_upsert(pool.as_ref(), &mut connection);
    let legacy_root = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let loaded = crate::model::work::Work::from_id(pool.as_ref(), &work).expect("work");
    loaded
        .delete(pool.as_ref())
        .expect("the protocol retires the running unfenced job");
    assert_eq!(
        reserve_work_upsert(pool.as_ref(), job, token).map(|r| r.permit_id),
        Err(ThothError::CrossrefPermitClaimStale)
    );
    let loaded = crate::model::work::Work::from_id(pool.as_ref(), &legacy_root).expect("work");
    loaded.delete(pool.as_ref()).expect("delete");
    assert_eq!(
        permit_crud::reserve_legacy_scheduled_crossref_write(pool.as_ref(), legacy_root)
            .map(|r| r.permit_id),
        Err(ThothError::CrossrefRootWorkNotFound)
    );
    assert_eq!(
        permit_crud::reserve_manual_recovery_crossref_write(pool.as_ref(), legacy_root, "INC-9")
            .map(|r| r.permit_id),
        Err(ThothError::CrossrefRootWorkNotFound)
    );
    assert_eq!(permit_count(&mut connection), 0);
}

#[test]
fn r11_no_reservation_or_finalisation_refusal_message_carries_database_text() {
    let refusals = [
        ThothError::CrossrefPermitClaimStale,
        ThothError::CrossrefReservationJobKindMismatch,
        ThothError::CrossrefBindingMovedRetry,
        ThothError::CrossrefUnitPublisherMismatch,
        ThothError::CrossrefPermitAttemptAlreadyReserved,
        ThothError::CrossrefRootWorkNotFound,
        ThothError::CrossrefPermitBlocked,
        ThothError::CrossrefPayloadDigestInvalid,
        ThothError::CrossrefPermitNotFound,
        ThothError::CrossrefPermitIllegalTransition,
        ThothError::CrossrefManualRecoveryRequiresReference,
        ThothError::WorkUpsertDatabaseFailure,
    ];
    for refusal in refusals {
        let message = refusal.to_string();
        let lower = message.to_lowercase();
        for forbidden in [
            "select",
            "insert",
            "update ",
            "delete ",
            "violates",
            "duplicate key",
            "constraint",
            "index",
            "trigger",
            "sqlstate",
            "postgres",
            "diesel",
            "pq:",
            "relation",
            "_idx",
            "_check",
            "_fkey",
            "crossref_write_permit",
            "distribution_job",
        ] {
            assert!(
                !lower.contains(forbidden),
                "{refusal:?}: {forbidden} in {message}"
            );
        }
    }
}

#[test]
fn r12_b3_b6_the_binding_projection_on_every_route() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");

    // WORK_UPSERT: the job's publisher and work identity.
    let (publisher, imprint, _activation, work, job, token) =
        claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let job_row = fx::texts(
        &mut connection,
        &format!("SELECT publisher_id::text || '|' || work_identity::text || '|' || coalesce(execution_profile::text, '-') AS value FROM distribution_job WHERE distribution_job_id = '{job}'"),
    )
    .remove(0);
    assert_eq!(job_row, format!("{publisher}|{work}|CROSSREF"), "B6");
    assert_eq!(reservation.publisher_identity, publisher);
    assert_eq!(reservation.root_work_identity, work);
    let authorized =
        finalise(pool.as_ref(), &presentation(&reservation, Some(token))).expect("finalise");
    assert_eq!(authorized.permit.permit.publisher_identity, publisher, "B3");
    let loaded_job = load_job(&mut connection, job);
    assert_eq!(
        loaded_job.execution_profile,
        Some(DistributionPlatform::Crossref),
        "B6"
    );

    // PUBLISHER_BACK_CATALOGUE: the outer job's publisher, the supplied root, no profile.
    let (back_publisher, back_imprint, outer_job, outer_token) =
        claimed_back_catalogue(pool.as_ref(), &mut connection);
    let unit = fx::insert_eligible_work(&mut connection, back_imprint, Uuid::new_v4());
    let unit_reservation = permit_crud::reserve_back_catalogue_crossref_write(
        pool.as_ref(),
        outer_job,
        outer_token,
        unit,
    )
    .expect("unit");
    assert_eq!(unit_reservation.publisher_identity, back_publisher);
    assert_eq!(unit_reservation.root_work_identity, unit);
    let outer = load_job(&mut connection, outer_job);
    assert_eq!(outer.execution_profile, None, "B6");
    assert_eq!(
        finalise(
            pool.as_ref(),
            &presentation(&unit_reservation, Some(outer_token))
        )
        .expect("finalise")
        .permit
        .permit
        .publisher_identity,
        back_publisher,
        "B3"
    );

    // The jobless routes: the root's current publisher and the supplied root.
    let legacy_root = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let legacy = permit_crud::reserve_legacy_scheduled_crossref_write(pool.as_ref(), legacy_root)
        .expect("legacy");
    let manual_root = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let manual =
        permit_crud::reserve_manual_recovery_crossref_write(pool.as_ref(), manual_root, "INC-1")
            .expect("manual");
    for (reservation, root) in [(&legacy, legacy_root), (&manual, manual_root)] {
        assert_eq!(reservation.publisher_identity, publisher);
        assert_eq!(reservation.root_work_identity, root);
        assert_eq!(
            finalise(pool.as_ref(), &presentation(reservation, None))
                .expect("finalise")
                .permit
                .permit
                .publisher_identity,
            publisher,
            "B3"
        );
    }

    // Every permit row's publisher_identity equals the identity its reservation returned (B3).
    assert_eq!(
        fx::texts(
            &mut connection,
            "SELECT string_agg(publisher_identity::text, ',' ORDER BY publisher_identity::text) AS value FROM crossref_write_permit"
        ),
        vec![{
            let mut identities = [
                publisher.to_string(),
                back_publisher.to_string(),
                publisher.to_string(),
                publisher.to_string(),
            ];
            identities.sort();
            identities.join(",")
        }]
    );
    // B6 over the whole table.
    assert_eq!(
        fx::count(
            &mut connection,
            "SELECT count(*) AS count FROM distribution_job WHERE (kind = 'WORK_UPSERT') <> (execution_profile IS NOT DISTINCT FROM 'CROSSREF')"
        ),
        0
    );
}

#[test]
fn p1_a_malformed_digest_is_refused_before_any_lock_and_changes_nothing() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (_publisher, _imprint, _activation, _work, job, token) =
        claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let before = fingerprint(&mut connection);

    // Another session holds the permit row: a refusal that took no lock returns at once.
    let mut holder = pool.get().expect("holder");
    fx::execute(
        &mut holder,
        &format!(
            "BEGIN; SELECT 1 FROM crossref_write_permit WHERE permit_id = '{}' FOR UPDATE",
            reservation.permit_id
        ),
    );
    let hex63 = &DIGEST[..63];
    for digest in [
        DIGEST.to_uppercase(),
        hex63.to_string(),
        format!("{DIGEST}0"),
        format!("{hex63}g"),
        format!("{hex63} "),
        format!("{hex63}\u{FF10}"),
        format!("{hex63}\u{00E9}"),
        String::new(),
    ] {
        let input = FinaliseCrossrefWrite {
            payload_digest: digest.clone(),
            ..presentation(&reservation, Some(token))
        };
        let pool_for_call = pool.clone();
        let call =
            std::thread::spawn(move || finalise(pool_for_call.as_ref(), &input).map(|r| r.outcome));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !call.is_finished() {
            if std::time::Instant::now() > deadline {
                fx::execute(&mut holder, "ROLLBACK");
                panic!("{digest:?}: the refusal waited on the permit lock");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(
            call.join().expect("call"),
            Err(ThothError::CrossrefPayloadDigestInvalid),
            "{digest:?}"
        );
    }
    fx::execute(&mut holder, "ROLLBACK");
    assert_eq!(fingerprint(&mut connection), before);
}

#[test]
fn p2_the_lower_case_digest_authorises_and_an_upper_cased_replay_is_malformed() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (_publisher, _imprint, _activation, _work, job, token) =
        claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let input = presentation(&reservation, Some(token));
    assert_eq!(
        finalise(pool.as_ref(), &input).map(|r| r.outcome),
        Ok(Finalised::Authorized)
    );
    let before = fingerprint(&mut connection);
    assert_eq!(
        finalise(pool.as_ref(), &input).map(|r| r.outcome),
        Ok(Finalised::Authorized),
        "the same digest replays"
    );
    let upper = FinaliseCrossrefWrite {
        payload_digest: DIGEST.to_uppercase(),
        ..input.clone()
    };
    assert_eq!(
        finalise(pool.as_ref(), &upper).map(|r| r.outcome),
        Err(ThothError::CrossrefPayloadDigestInvalid),
        "not CROSSREF_PERMIT_ILLEGAL_TRANSITION"
    );
    assert_eq!(fingerprint(&mut connection), before);
}

// ---------------------------------------------------------------------------------------------------------------------
// R52B section 25.17: the race rows T205-T223 against the implementation's functions, each in both orders, with the
// waits read from pg_locks (section 29).
// ---------------------------------------------------------------------------------------------------------------------

use crate::model::work_upsert::tests::race::{self, interleave, PausePoint};

fn state_name(connection: &mut PgConnection, permit: Uuid) -> String {
    state_of(connection, permit)
        .split('|')
        .next()
        .expect("state")
        .to_string()
}

fn owner_void(
    pool: &crate::db::PgPool,
    permit: Uuid,
    token: Uuid,
) -> ThothResult<CrossrefWritePermitState> {
    permit_crud::void_crossref_write_reservation(pool, permit, token, "released", &allow)
        .map(|p| p.permit.state)
}

#[test]
fn t205_finalise_and_the_route_owner_void_in_both_orders() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");

    // Finalise first, paused holding X: the void waits on the permit row and is then refused.
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let input = presentation(&reservation, Some(token));
    let (permit, reservation_token) = (reservation.permit_id, reservation.reservation_token);
    let (finalised, voided, transcript) = interleave(
        &pool,
        PausePoint::install("t205a", "BEFORE UPDATE", "crossref_write_permit"),
        move |pool| finalise(pool, &input).map(|r| r.outcome),
        move |pool| owner_void(pool, permit, reservation_token),
        true,
    );
    assert_eq!(finalised, Ok(Finalised::Authorized));
    assert_eq!(voided, Err(ThothError::CrossrefPermitVoidRequiresReserved));
    assert_eq!(transcript, vec!["transactionid:ShareLock"]);
    assert_eq!(state_name(&mut connection, permit), "AUTHORIZED");

    // The void first, paused holding X: finalisation waits at X and is then refused.
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let input = presentation(&reservation, Some(token));
    let (permit, reservation_token) = (reservation.permit_id, reservation.reservation_token);
    let (voided, finalised, transcript) = interleave(
        &pool,
        PausePoint::install("t205b", "BEFORE UPDATE", "crossref_write_permit"),
        move |pool| owner_void(pool, permit, reservation_token),
        move |pool| finalise(pool, &input).map(|r| r.outcome),
        true,
    );
    assert_eq!(voided, Ok(CrossrefWritePermitState::Voided));
    assert_eq!(finalised, Err(ThothError::CrossrefPermitIllegalTransition));
    assert_eq!(transcript, vec!["transactionid:ShareLock"]);
    assert_eq!(state_name(&mut connection, permit), "VOIDED");
}

#[test]
fn t206_finalise_and_the_superuser_void_in_both_orders() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let superuser_void = |pool: &crate::db::PgPool, permit: Uuid| {
        permit_crud::void_crossref_write_reservation_as_superuser(pool, permit, "cleanup", "INC-42")
            .map(|p| p.permit.state)
    };

    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let input = presentation(&reservation, Some(token));
    let permit = reservation.permit_id;
    let (finalised, voided, transcript) = interleave(
        &pool,
        PausePoint::install("t206a", "BEFORE UPDATE", "crossref_write_permit"),
        move |pool| finalise(pool, &input).map(|r| r.outcome),
        move |pool| superuser_void(pool, permit),
        true,
    );
    assert_eq!(finalised, Ok(Finalised::Authorized));
    assert_eq!(voided, Err(ThothError::CrossrefPermitVoidRequiresReserved));
    assert_eq!(transcript, vec!["transactionid:ShareLock"]);

    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let input = presentation(&reservation, Some(token));
    let permit = reservation.permit_id;
    let (voided, finalised, transcript) = interleave(
        &pool,
        PausePoint::install("t206b", "BEFORE UPDATE", "crossref_write_permit"),
        move |pool| superuser_void(pool, permit),
        move |pool| finalise(pool, &input).map(|r| r.outcome),
        true,
    );
    assert_eq!(voided, Ok(CrossrefWritePermitState::Voided));
    assert_eq!(finalised, Err(ThothError::CrossrefPermitIllegalTransition));
    assert_eq!(transcript, vec!["transactionid:ShareLock"]);
}

#[test]
fn t207_finalise_and_administrative_cancellation_in_both_orders() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");

    // Finalise first, paused holding J and A: the cancellation waits on J, then is refused by the fence.
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let input = presentation(&reservation, Some(token));
    let (finalised, cancelled, transcript) = interleave(
        &pool,
        PausePoint::install("t207a", "BEFORE UPDATE", "crossref_write_permit"),
        move |pool| finalise(pool, &input).map(|r| r.outcome),
        move |pool| job_crud::cancel_distribution_job(pool, job).map(|j| j.status),
        true,
    );
    assert_eq!(finalised, Ok(Finalised::Authorized));
    assert_eq!(
        cancelled,
        Err(ThothError::WorkUpsertCancellationRefusedFencedAttempt)
    );
    assert_eq!(transcript, vec!["transactionid:ShareLock"]);

    // The cancellation first, paused holding J: finalisation waits at J, then finds the claim stale.
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let permit = reservation.permit_id;
    let input = presentation(&reservation, Some(token));
    let before = permit_row(&mut connection, permit);
    let (cancelled, finalised, transcript) = interleave(
        &pool,
        PausePoint::install("t207b", "BEFORE UPDATE", "distribution_job"),
        move |pool| job_crud::cancel_distribution_job(pool, job).map(|j| j.status),
        move |pool| finalise(pool, &input).map(|r| r.outcome),
        true,
    );
    assert_eq!(cancelled, Ok(DistributionJobStatus::Cancelled));
    assert_eq!(finalised, Err(ThothError::CrossrefPermitClaimStale));
    assert_eq!(transcript, vec!["transactionid:ShareLock"]);
    assert_eq!(
        permit_row(&mut connection, permit),
        before,
        "permit untouched"
    );
}

#[test]
fn t208_finalise_and_work_deletion_in_both_orders() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let delete = |pool: &crate::db::PgPool, work: Uuid| {
        crate::model::work::Work::from_id(pool, &work)
            .expect("work")
            .delete(pool)
            .map(|w| w.work_id)
    };

    // Finalise first, paused holding W FOR SHARE: the deletion waits at W and is refused by the fence.
    let (_p, _i, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let input = presentation(&reservation, Some(token));
    let (finalised, deleted, transcript) = interleave(
        &pool,
        PausePoint::install("t208a", "BEFORE UPDATE", "crossref_write_permit"),
        move |pool| finalise(pool, &input).map(|r| r.outcome),
        move |pool| delete(pool, work),
        true,
    );
    assert_eq!(finalised, Ok(Finalised::Authorized));
    assert_eq!(deleted, Err(ThothError::WorkDeleteBlockedByFencedAttempt));
    assert!(!transcript.is_empty());

    // The deletion first, paused holding W FOR UPDATE: finalisation waits at W, then finds the claim stale.
    let (_p, _i, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let input = presentation(&reservation, Some(token));
    let (deleted, finalised, transcript) = interleave(
        &pool,
        PausePoint::install("t208b", "BEFORE DELETE", "work"),
        move |pool| delete(pool, work),
        move |pool| finalise(pool, &input).map(|r| r.outcome),
        true,
    );
    assert_eq!(deleted, Ok(work));
    assert_eq!(finalised, Err(ThothError::CrossrefPermitClaimStale));
    assert!(!transcript.is_empty());
}

fn expire_lease(connection: &mut PgConnection, job: Uuid) {
    fx::execute(
        connection,
        &format!("UPDATE distribution_job SET lease_expires_at = now() - interval '1 second' WHERE distribution_job_id = '{job}'"),
    );
}

fn attempt_row(connection: &mut PgConnection, job: Uuid) -> Vec<String> {
    fx::texts(
        connection,
        &format!(
            "SELECT coalesce(result::text, 'OPEN') || '|' || (fenced_at IS NOT NULL)::text || '|' \
                 || (recovery_cleared_at IS NOT NULL)::text AS value \
             FROM distribution_job_attempt WHERE distribution_job_id = '{job}' ORDER BY started_at"
        ),
    )
}

fn report(
    pool: &crate::db::PgPool,
    permit: Uuid,
    token: Uuid,
    outcome: Outcome,
) -> ThothResult<CrossrefWritePermitState> {
    permit_crud::report_crossref_write(pool, permit, token, outcome, &allow).map(|p| p.permit.state)
}

fn reconcile(
    pool: &crate::db::PgPool,
    permit: Uuid,
    outcome: Outcome,
    reference: &str,
) -> ThothResult<CrossrefWritePermitState> {
    permit_crud::reconcile_crossref_write_permit(
        pool,
        permit,
        outcome,
        Recon::Reconciled,
        reference,
    )
    .map(|p| p.permit.state)
}

#[test]
fn t209_finalise_and_lease_recovery_in_both_orders() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");

    // Finalise first, paused holding J with the lease lapsed: recovery skips the locked job without waiting.
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let input = presentation(&reservation, Some(token));
    expire_lease(&mut connection, job);
    let (finalised, claimed, transcript) = interleave(
        &pool,
        PausePoint::install("t209a", "BEFORE UPDATE", "crossref_write_permit"),
        move |pool| finalise(pool, &input).map(|r| r.outcome),
        move |pool| fx::claim(pool).len(),
        false,
    );
    assert_eq!(finalised, Ok(Finalised::Authorized));
    assert_eq!(claimed, 0);
    assert!(transcript.is_empty(), "recovery never waited");
    assert_eq!(attempt_row(&mut connection, job), vec!["OPEN|true|false"]);

    // Recovery first: the attempt is abandoned unfenced and finalisation finds the claim stale.
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    expire_lease(&mut connection, job);
    assert!(
        fx::claim(pool.as_ref()).is_empty(),
        "the RESERVED permit keeps the job unclaimable"
    );
    assert_eq!(
        finalise(pool.as_ref(), &presentation(&reservation, Some(token))).map(|r| r.outcome),
        Err(ThothError::CrossrefPermitClaimStale)
    );
    assert_eq!(
        attempt_row(&mut connection, job),
        vec!["ABANDONED|false|false"]
    );
    assert_eq!(
        state_name(&mut connection, reservation.permit_id),
        "RESERVED"
    );
}

#[test]
fn t210_t211_finalise_against_the_report_and_reconciliation_in_both_orders() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");

    // T210, finalise first: the report waits at X, then records ACCEPTED.
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let input = presentation(&reservation, Some(token));
    let (permit, reservation_token) = (reservation.permit_id, reservation.reservation_token);
    let (finalised, reported, transcript) = interleave(
        &pool,
        PausePoint::install("t210a", "BEFORE UPDATE", "crossref_write_permit"),
        move |pool| finalise(pool, &input).map(|r| r.outcome),
        move |pool| report(pool, permit, reservation_token, Outcome::Accepted),
        true,
    );
    assert_eq!(finalised, Ok(Finalised::Authorized));
    assert_eq!(reported, Ok(CrossrefWritePermitState::Accepted));
    assert_eq!(transcript, vec!["transactionid:ShareLock"]);
    // T210, the report first: refused on RESERVED, then finalisation authorises.
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    assert_eq!(
        report(
            pool.as_ref(),
            reservation.permit_id,
            reservation.reservation_token,
            Outcome::Accepted
        ),
        Err(ThothError::CrossrefPermitIllegalTransition)
    );
    assert_eq!(
        finalise(pool.as_ref(), &presentation(&reservation, Some(token))).map(|r| r.outcome),
        Ok(Finalised::Authorized)
    );

    // T211, finalise first: reconciliation waits at A, then records RECONCILED:ACCEPTED.
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let input = presentation(&reservation, Some(token));
    let permit = reservation.permit_id;
    let (finalised, reconciled, transcript) = interleave(
        &pool,
        PausePoint::install("t211a", "BEFORE UPDATE", "crossref_write_permit"),
        move |pool| finalise(pool, &input).map(|r| r.outcome),
        move |pool| reconcile(pool, permit, Outcome::Accepted, "R-211"),
        true,
    );
    assert_eq!(finalised, Ok(Finalised::Authorized));
    assert_eq!(reconciled, Ok(CrossrefWritePermitState::Accepted));
    assert!(!transcript.is_empty());
    assert!(state_of(&mut connection, permit).starts_with("ACCEPTED|RECONCILED|"));
    // T211, reconciliation first: refused on RESERVED, then finalisation authorises.
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    assert_eq!(
        reconcile(
            pool.as_ref(),
            reservation.permit_id,
            Outcome::Accepted,
            "R-211b"
        ),
        Err(ThothError::CrossrefPermitIllegalTransition)
    );
    assert_eq!(
        finalise(pool.as_ref(), &presentation(&reservation, Some(token))).map(|r| r.outcome),
        Ok(Finalised::Authorized)
    );
}

#[test]
fn t212_t213_completion_against_the_report_and_the_confirmation() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let complete = |pool: &crate::db::PgPool, job: Uuid, token: Uuid| {
        job_crud::complete_distribution_job(pool, job, token).map(|j| j.status)
    };

    // T212: completion while the report holds X, uncommitted, reads the permit by MVCC and is refused without
    // waiting; after the report commits, completion succeeds.
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    finalise(pool.as_ref(), &presentation(&reservation, Some(token))).expect("finalise");
    let (permit, reservation_token) = (reservation.permit_id, reservation.reservation_token);
    let (reported, completed, transcript) = interleave(
        &pool,
        PausePoint::install("t212", "AFTER UPDATE", "crossref_write_permit"),
        move |pool| report(pool, permit, reservation_token, Outcome::Accepted),
        move |pool| complete(pool, job, token),
        false,
    );
    assert_eq!(reported, Ok(CrossrefWritePermitState::Accepted));
    assert_eq!(
        completed,
        Err(ThothError::WorkUpsertCompletionRequiresAcceptedPermit)
    );
    assert!(transcript.is_empty());
    assert_eq!(
        complete(pool.as_ref(), job, token),
        Ok(DistributionJobStatus::Succeeded)
    );

    // T213, confirmation first, paused holding A: completion waits at A; both land.
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    finalise(pool.as_ref(), &presentation(&reservation, Some(token))).expect("finalise");
    report(
        pool.as_ref(),
        reservation.permit_id,
        reservation.reservation_token,
        Outcome::Accepted,
    )
    .expect("report");
    let permit = reservation.permit_id;
    let (confirmed, completed, transcript) = interleave(
        &pool,
        PausePoint::install("t213a", "BEFORE UPDATE", "crossref_write_permit"),
        move |pool| reconcile(pool, permit, Outcome::Accepted, "CONF-213a"),
        move |pool| complete(pool, job, token),
        true,
    );
    assert_eq!(confirmed, Ok(CrossrefWritePermitState::Accepted));
    assert_eq!(completed, Ok(DistributionJobStatus::Succeeded));
    assert!(!transcript.is_empty());
    assert!(state_of(&mut connection, permit).starts_with("ACCEPTED|RECONCILED|"));

    // T213, completion first, paused holding J and A: the confirmation waits at A; both land.
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    finalise(pool.as_ref(), &presentation(&reservation, Some(token))).expect("finalise");
    report(
        pool.as_ref(),
        reservation.permit_id,
        reservation.reservation_token,
        Outcome::Accepted,
    )
    .expect("report");
    let permit = reservation.permit_id;
    let (completed, confirmed, transcript) = interleave(
        &pool,
        PausePoint::install("t213b", "AFTER UPDATE", "distribution_job_attempt"),
        move |pool| complete(pool, job, token),
        move |pool| reconcile(pool, permit, Outcome::Accepted, "CONF-213b"),
        true,
    );
    assert_eq!(completed, Ok(DistributionJobStatus::Succeeded));
    assert_eq!(confirmed, Ok(CrossrefWritePermitState::Accepted));
    assert!(!transcript.is_empty());
}

#[test]
fn t214_reconciliation_and_lease_recovery_in_both_orders() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");

    // Recovery first: the fenced attempt is abandoned; reconciliation to NONE_ATTEMPTED clears it.
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    finalise(pool.as_ref(), &presentation(&reservation, Some(token))).expect("finalise");
    expire_lease(&mut connection, job);
    assert!(fx::claim(pool.as_ref()).is_empty());
    assert_eq!(
        attempt_row(&mut connection, job),
        vec!["ABANDONED|true|false"]
    );
    assert_eq!(
        reconcile(
            pool.as_ref(),
            reservation.permit_id,
            Outcome::NoneAttempted,
            "R-214a"
        ),
        Ok(CrossrefWritePermitState::NoneAttempted)
    );
    assert_eq!(
        attempt_row(&mut connection, job),
        vec!["ABANDONED|true|true"]
    );

    // Reconciliation first, paused holding A with the lease lapsed: recovery waits at A, then abandons; a later
    // confirmation clears.
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    finalise(pool.as_ref(), &presentation(&reservation, Some(token))).expect("finalise");
    expire_lease(&mut connection, job);
    let permit = reservation.permit_id;
    let (reconciled, claimed, transcript) = interleave(
        &pool,
        PausePoint::install("t214b", "BEFORE UPDATE", "crossref_write_permit"),
        move |pool| reconcile(pool, permit, Outcome::NoneAttempted, "R-214b"),
        move |pool| fx::claim(pool).len(),
        true,
    );
    assert_eq!(reconciled, Ok(CrossrefWritePermitState::NoneAttempted));
    assert!(!transcript.is_empty());
    let _ = claimed;
    assert_eq!(attempt_row(&mut connection, job)[0], "ABANDONED|true|false");
    assert_eq!(
        reconcile(pool.as_ref(), permit, Outcome::NoneAttempted, "R-214c"),
        Ok(CrossrefWritePermitState::NoneAttempted)
    );
    assert_eq!(attempt_row(&mut connection, job)[0], "ABANDONED|true|true");
}

#[test]
fn t215_reservation_and_the_floor_advance_at_three_points() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (publisher, imprint) = fx::publisher_and_imprint(pool.as_ref());
    fx::cover_crossref(&mut connection, publisher);
    let advance = |pool: &crate::db::PgPool| {
        permit_crud::advance_crossref_version_floor(
            pool,
            &advance_input(Uuid::new_v4()),
            "superuser",
        )
        .map(|a| a.after_value)
    };

    // F held by the reservation: the advance waits at F, then is refused NOT_DRAINED.
    let root = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let (reserved, advanced, transcript) = interleave(
        &pool,
        PausePoint::install("t215c", "BEFORE INSERT", "crossref_write_permit"),
        move |pool| permit_crud::reserve_legacy_scheduled_crossref_write(pool, root),
        advance,
        true,
    );
    let reserved = reserved.expect("reserved");
    assert_eq!(advanced, Err(ThothError::CrossrefVersionFloorNotDrained));
    assert_eq!(transcript, vec!["transactionid:ShareLock"]);
    permit_crud::void_crossref_write_reservation(
        pool.as_ref(),
        reserved.permit_id,
        reserved.reservation_token,
        "t215",
        &allow,
    )
    .expect("void");

    // K held by another reservation's keys: the advance proceeds without waiting.
    let mut holder = race::dedicated();
    use diesel::connection::SimpleConnection;
    holder
        .batch_execute(&format!(
            "BEGIN; SELECT pg_advisory_xact_lock(1948572001, hashtext('be06:crossref:doi:' || public.crossref_canonical_doi(doi))) FROM work WHERE work_id = '{root}'"
        ))
        .expect("hold K");
    assert_eq!(
        advance(pool.as_ref()),
        Ok(TARGET),
        "the advance takes no DOI key"
    );
    holder.batch_execute("ROLLBACK").expect("release K");

    // F held by the advance: a reservation waits at F, then allocates a 17-digit value above the floor.
    let (_guard2, pool2) = (0, pool.clone());
    let _ = _guard2;
    fx::execute(&mut connection, "SET session_replication_role = replica; UPDATE work_crossref_version_floor SET floor_value = 0; DELETE FROM crossref_version_floor_audit; SET session_replication_role = origin");
    let second_root = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let (advanced, reserved, transcript) = interleave(
        &pool2,
        PausePoint::install("t215b", "BEFORE INSERT", "crossref_version_floor_audit"),
        advance,
        move |pool| permit_crud::reserve_legacy_scheduled_crossref_write(pool, second_root),
        true,
    );
    assert_eq!(advanced, Ok(TARGET));
    let reserved = reserved.expect("reserved after the advance");
    assert!(reserved.crossref_timestamp > TARGET);
    assert_eq!(reserved.crossref_timestamp.to_string().len(), 17);
    assert_eq!(transcript, vec!["transactionid:ShareLock"]);
}

#[test]
fn t216_two_finalisations_of_one_permit_with_different_digests() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let first = presentation(&reservation, Some(token));
    let second = FinaliseCrossrefWrite {
        payload_digest: "e".repeat(64),
        ..first.clone()
    };
    let (a, b, transcript) = interleave(
        &pool,
        PausePoint::install("t216", "BEFORE UPDATE", "crossref_write_permit"),
        move |pool| finalise(pool, &first).map(|r| r.outcome),
        move |pool| finalise(pool, &second).map(|r| r.outcome),
        true,
    );
    assert_eq!(a, Ok(Finalised::Authorized));
    assert_eq!(b, Err(ThothError::CrossrefPermitIllegalTransition));
    assert_eq!(transcript.len(), 1);
    assert_eq!(
        state_of(&mut connection, reservation.permit_id)
            .split('|')
            .next(),
        Some("AUTHORIZED")
    );
}

/// Two parents sharing one non-chapter child, so their memberships overlap.
fn overlapping_parents(
    connection: &mut PgConnection,
    imprint: Uuid,
    shared: Uuid,
    others: [Uuid; 2],
) -> (Uuid, Uuid) {
    let mut parents = Vec::new();
    for other in others {
        let parent = fx::insert_eligible_work(connection, imprint, Uuid::new_v4());
        fx::execute(
            connection,
            &format!("UPDATE work SET landing_page = NULL WHERE work_id = '{parent}'"),
        );
        fx::relate_child(connection, parent, shared, 1);
        fx::relate_child(connection, parent, other, 2);
        parents.push(parent);
    }
    (parents[0], parents[1])
}

#[test]
fn t217_two_overlapping_reservations_serialise_at_the_first_shared_key() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (publisher, imprint) = fx::publisher_and_imprint(pool.as_ref());
    fx::cover_crossref(&mut connection, publisher);
    let shared = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let others = [
        fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4()),
        fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4()),
    ];
    let (first_root, second_root) = overlapping_parents(&mut connection, imprint, shared, others);
    let (first, second, transcript) = interleave(
        &pool,
        PausePoint::install("t217", "BEFORE INSERT", "crossref_write_permit"),
        move |pool| permit_crud::reserve_legacy_scheduled_crossref_write(pool, first_root),
        move |pool| permit_crud::reserve_legacy_scheduled_crossref_write(pool, second_root),
        true,
    );
    assert!(first.is_ok());
    assert_eq!(
        second.map(|r| r.permit_id),
        Err(ThothError::CrossrefPermitBlocked)
    );
    assert_eq!(transcript, vec!["advisory:ExclusiveLock"]);
}

#[test]
fn t218_a_real_hashtext_collision_is_acquired_in_key_order_without_deadlock() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    #[derive(QueryableByName)]
    struct Pair {
        #[diesel(sql_type = Text)]
        low: String,
        #[diesel(sql_type = Text)]
        high: String,
        #[diesel(sql_type = diesel::sql_types::Integer)]
        key: i32,
    }
    // A real collision of the DOI key: two canonical DOIs, one key.
    let pairs = diesel::sql_query(
        "SELECT min(d) AS low, max(d) AS high, h AS key FROM ( \
             SELECT 'https://doi.org/10.12345/t218-' || n AS d, \
                    hashtext('be06:crossref:doi:' || 'https://doi.org/10.12345/t218-' || n) AS h \
               FROM generate_series(1, 400000) n) s \
          GROUP BY h HAVING count(*) = 2 ORDER BY h",
    )
    .load::<Pair>(&mut connection)
    .expect("collisions");
    // A shared DOI strictly between them in code-point order, so string order and key order disagree.
    let (pair, shared_doi) = pairs
        .iter()
        .find_map(|pair| {
            let shared = format!("{}0", pair.low);
            (pair.low < shared && shared < pair.high).then_some((pair, shared))
        })
        .expect("a usable collision");
    let (publisher, imprint) = fx::publisher_and_imprint(pool.as_ref());
    fx::cover_crossref(&mut connection, publisher);
    let with_doi = |connection: &mut PgConnection, doi: &str| {
        let work = fx::insert_eligible_work(connection, imprint, Uuid::new_v4());
        fx::execute(
            connection,
            &format!("UPDATE work SET doi = '{doi}' WHERE work_id = '{work}'"),
        );
        work
    };
    let shared = with_doi(&mut connection, &shared_doi);
    let low = with_doi(&mut connection, &pair.low);
    let high = with_doi(&mut connection, &pair.high);
    let shared_key = fx::texts(
        &mut connection,
        &format!("SELECT hashtext('be06:crossref:doi:{shared_doi}')::text AS value"),
    )
    .remove(0)
    .parse::<i32>()
    .expect("key");
    assert_ne!(shared_key, pair.key);
    // Parent 1 = {shared, high}, parent 2 = {low, shared}: in string order parent 1 takes shared then the collided
    // key and parent 2 the collided key then shared — the withdrawn R52 order's deadlock.
    let (first_root, second_root) =
        overlapping_parents(&mut connection, imprint, shared, [high, low]);

    // Hold the larger key from a third session, so the first reservation takes the smaller key and waits.
    let (smaller, larger) = (shared_key.min(pair.key), shared_key.max(pair.key));
    let mut blocker = race::dedicated();
    use diesel::connection::SimpleConnection;
    blocker
        .batch_execute(&format!(
            "BEGIN; SELECT pg_advisory_xact_lock(1948572001, {larger})"
        ))
        .expect("blocker");
    let mut observer = race::dedicated();
    let first_pool = pool.clone();
    let first = std::thread::spawn(move || {
        permit_crud::reserve_legacy_scheduled_crossref_write(first_pool.as_ref(), first_root)
    });
    race::wait_for_waits(&mut observer, 1);
    let second_pool = pool.clone();
    let second = std::thread::spawn(move || {
        permit_crud::reserve_legacy_scheduled_crossref_write(second_pool.as_ref(), second_root)
    });
    let waits = race::wait_for_waits(&mut observer, 2);
    assert_eq!(
        waits,
        vec!["advisory:ExclusiveLock", "advisory:ExclusiveLock"]
    );
    // The second waits on the smaller key, which the first holds: both acquire in key order.
    assert_eq!(
        fx::texts(
            &mut observer,
            "SELECT string_agg(((objid::bigint # 2147483648) - 2147483648)::text || ':' || granted::text, ',' ORDER BY granted, ((objid::bigint # 2147483648) - 2147483648)) AS value \
             FROM pg_locks WHERE locktype = 'advisory' AND classid = 1948572001"
        ),
        vec![format!("{smaller}:false,{larger}:false,{smaller}:true,{larger}:true")]
    );
    blocker.batch_execute("ROLLBACK").expect("release");
    assert!(first.join().expect("first").is_ok());
    assert_eq!(
        second.join().expect("second").map(|r| r.permit_id),
        Err(ThothError::CrossrefPermitBlocked),
        "no 40P01: the second is refused on the shared DOI"
    );
}

/// An emitted chapter of `parent`: a DOI-bearing `book-chapter` with no edition.
fn chapter_of(connection: &mut PgConnection, imprint: Uuid, parent: Uuid) -> Uuid {
    let chapter = fx::insert_eligible_work(connection, imprint, Uuid::new_v4());
    fx::execute(
        connection,
        &format!("UPDATE work SET work_type = 'book-chapter', edition = NULL WHERE work_id = '{chapter}'"),
    );
    fx::relate_child(connection, parent, chapter, 1);
    chapter
}

#[test]
fn t219_the_witness_against_capture_at_its_pause_points() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    fx::enable_capture(&mut connection);

    // The reservation holds G: a child capture's commit-time flush waits for it, so the witness is the value
    // committed before its read, and the later finalisation voids SOURCE_CHANGED_DURING_PREPARATION.
    let (_p, imprint, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let child = chapter_of(&mut connection, imprint, work);
    let witness_before = fx::generation_of(&mut connection, work).expect("generation");
    let (reserved, edited, transcript) = interleave(
        &pool,
        PausePoint::install("t219a", "BEFORE INSERT", "crossref_write_permit"),
        move |pool| reserve_work_upsert(pool, job, token),
        move |pool| {
            let mut connection = pool.get().expect("editor");
            use diesel::connection::SimpleConnection;
            connection
                .batch_execute(&format!(
                    "UPDATE title SET title = 'Edited' WHERE work_id = '{child}'"
                ))
                .map_err(|e| e.to_string())
        },
        true,
    );
    let reserved = reserved.expect("reserved");
    assert_eq!(edited, Ok(()));
    assert_eq!(transcript, vec!["transactionid:ShareLock"]);
    assert_eq!(
        permit_row(&mut connection, reserved.permit_id)
            .split('|')
            .nth(5),
        Some(witness_before.to_string().as_str()),
        "the witness is the value committed before its read"
    );
    let result = finalise(pool.as_ref(), &presentation(&reserved, Some(token))).expect("finalise");
    assert_eq!(
        (result.outcome, result.void_reason),
        (
            Finalised::VoidedRetryable,
            Some(CrossrefVoidReason::SourceChangedDuringPreparation)
        )
    );

    // The capture holds G at its flush: the reservation waits, and its witness includes the edit.
    let (_p, imprint, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let child = chapter_of(&mut connection, imprint, work);
    let before = fx::generation_of(&mut connection, work).expect("generation");
    let (edited, reserved, transcript) = interleave(
        &pool,
        PausePoint::install("t219b", "AFTER UPDATE", "work_upsert_generation"),
        move |pool| {
            let mut connection = pool.get().expect("editor");
            use diesel::connection::SimpleConnection;
            connection
                .batch_execute(&format!(
                    "UPDATE title SET title = 'Edited' WHERE work_id = '{child}'"
                ))
                .map_err(|e| e.to_string())
        },
        move |pool| reserve_work_upsert(pool, job, token),
        true,
    );
    assert_eq!(edited, Ok(()));
    let reserved = reserved.expect("reserved");
    assert_eq!(transcript, vec!["transactionid:ShareLock"]);
    assert_eq!(
        permit_row(&mut connection, reserved.permit_id)
            .split('|')
            .nth(5),
        Some((before + 1).to_string().as_str())
    );
    assert_eq!(
        finalise(pool.as_ref(), &presentation(&reserved, Some(token))).map(|r| r.outcome),
        Ok(Finalised::Authorized)
    );
}

#[test]
fn t220_the_witness_against_work_deletion_in_both_orders() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (publisher, imprint) = fx::publisher_and_imprint(pool.as_ref());
    fx::cover_crossref(&mut connection, publisher);
    let delete = |pool: &crate::db::PgPool, work: Uuid| {
        crate::model::work::Work::from_id(pool, &work)
            .expect("work")
            .delete(pool)
            .map(|w| w.work_id)
    };

    // The reservation first, paused holding W FOR SHARE and G: the deletion waits, then deletes; the permit
    // survives and blocks.
    let root = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let (reserved, deleted, transcript) = interleave(
        &pool,
        PausePoint::install("t220a", "BEFORE INSERT", "crossref_write_permit"),
        move |pool| permit_crud::reserve_legacy_scheduled_crossref_write(pool, root),
        move |pool| delete(pool, root),
        true,
    );
    let reserved = reserved.expect("reserved");
    assert_eq!(deleted, Ok(root));
    assert!(!transcript.is_empty());
    assert_eq!(state_name(&mut connection, reserved.permit_id), "RESERVED");
    assert_eq!(
        fx::count(&mut connection, "SELECT count(*) AS count FROM crossref_write_permit WHERE public.crossref_is_blocking_write_permit(state, reconciliation_state)"),
        1
    );

    // The deletion first, paused holding W FOR UPDATE: the reservation waits, then finds no root; no row.
    let root = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let (deleted, reserved, transcript) = interleave(
        &pool,
        PausePoint::install("t220b", "BEFORE DELETE", "work"),
        move |pool| delete(pool, root),
        move |pool| {
            permit_crud::reserve_legacy_scheduled_crossref_write(pool, root).map(|r| r.permit_id)
        },
        true,
    );
    assert_eq!(deleted, Ok(root));
    assert_eq!(reserved, Err(ThothError::CrossrefRootWorkNotFound));
    assert!(!transcript.is_empty());
    assert_eq!(
        fx::count(
            &mut connection,
            "SELECT count(*) AS count FROM crossref_write_permit"
        ),
        1
    );
}

#[test]
fn t221_finalise_and_publisher_deletion_in_both_orders() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let delete_publisher = |pool: &crate::db::PgPool, publisher: Uuid| {
        crate::model::publisher::Publisher::from_id(pool, &publisher)
            .expect("publisher")
            .delete(pool)
            .map(|p| p.publisher_id)
    };

    let (publisher, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let input = presentation(&reservation, Some(token));
    let (finalised, deleted, transcript) = interleave(
        &pool,
        PausePoint::install("t221a", "BEFORE UPDATE", "crossref_write_permit"),
        move |pool| finalise(pool, &input).map(|r| r.outcome),
        move |pool| delete_publisher(pool, publisher),
        true,
    );
    assert_eq!(finalised, Ok(Finalised::Authorized));
    assert_eq!(deleted, Err(ThothError::WorkDeleteBlockedByFencedAttempt));
    assert!(!transcript.is_empty());

    let (publisher, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let input = presentation(&reservation, Some(token));
    let (deleted, finalised, transcript) = interleave(
        &pool,
        PausePoint::install("t221b", "BEFORE DELETE", "publisher"),
        move |pool| delete_publisher(pool, publisher),
        move |pool| finalise(pool, &input).map(|r| r.outcome),
        true,
    );
    assert_eq!(deleted, Ok(publisher));
    assert_eq!(finalised, Err(ThothError::CrossrefPermitClaimStale));
    assert!(!transcript.is_empty());
}

#[test]
fn t222_back_catalogue_reservation_and_outer_cancellation_in_both_orders() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");

    let (_publisher, imprint, job, token) = claimed_back_catalogue(pool.as_ref(), &mut connection);
    let unit = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let (reserved, cancelled, transcript) = interleave(
        &pool,
        PausePoint::install("t222a", "BEFORE INSERT", "crossref_write_permit"),
        move |pool| permit_crud::reserve_back_catalogue_crossref_write(pool, job, token, unit),
        move |pool| job_crud::cancel_distribution_job(pool, job).map(|j| j.status),
        true,
    );
    let reserved = reserved.expect("reserved");
    assert_eq!(cancelled, Ok(DistributionJobStatus::Cancelled));
    assert_eq!(transcript, vec!["transactionid:ShareLock"]);
    assert_eq!(
        state_name(&mut connection, reserved.permit_id),
        "RESERVED",
        "the unit permit is untouched"
    );

    let (_publisher, imprint, job, token) = claimed_back_catalogue(pool.as_ref(), &mut connection);
    let unit = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let (cancelled, reserved, transcript) = interleave(
        &pool,
        PausePoint::install("t222b", "BEFORE UPDATE", "distribution_job"),
        move |pool| job_crud::cancel_distribution_job(pool, job).map(|j| j.status),
        move |pool| {
            permit_crud::reserve_back_catalogue_crossref_write(pool, job, token, unit)
                .map(|r| r.permit_id)
        },
        true,
    );
    assert_eq!(cancelled, Ok(DistributionJobStatus::Cancelled));
    assert_eq!(reserved, Err(ThothError::CrossrefPermitClaimStale));
    assert!(!transcript.is_empty());
}

#[test]
fn t223_cross_route_overlapping_reservations() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");

    // Manual recovery holding its keys against a legacy reservation of the same root.
    let (publisher, imprint) = fx::publisher_and_imprint(pool.as_ref());
    fx::cover_crossref(&mut connection, publisher);
    let root = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let (manual, legacy, transcript) = interleave(
        &pool,
        PausePoint::install("t223a", "BEFORE INSERT", "crossref_write_permit"),
        move |pool| permit_crud::reserve_manual_recovery_crossref_write(pool, root, "INC-223"),
        move |pool| {
            permit_crud::reserve_legacy_scheduled_crossref_write(pool, root).map(|r| r.permit_id)
        },
        true,
    );
    assert!(manual.is_ok());
    assert_eq!(legacy, Err(ThothError::CrossrefPermitBlocked));
    assert!(!transcript.is_empty());

    // A back-catalogue unit holding its keys against the WORK_UPSERT reservation of the same Work.
    let (_p, _i, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let work_publisher = fx::texts(&mut connection, &format!("SELECT i.publisher_id::text AS value FROM work w JOIN imprint i USING (imprint_id) WHERE w.work_id = '{work}'")).remove(0);
    let activation = fx::texts(&mut connection, &format!("SELECT activation_id::text AS value FROM publisher_distribution_platform WHERE publisher_id = '{work_publisher}' AND platform = 'CROSSREF'")).remove(0);
    fx::execute(&mut connection, &format!(
        "INSERT INTO distribution_job (kind, publisher_id, activation_id, deduplication_key) \
         VALUES ('PUBLISHER_BACK_CATALOGUE', '{work_publisher}', '{activation}', 'PUBLISHER_BACK_CATALOGUE:{work_publisher}:{activation}'); \
         INSERT INTO distribution_job_target (distribution_job_id, platform) \
         SELECT distribution_job_id, 'CROSSREF' FROM distribution_job WHERE kind = 'PUBLISHER_BACK_CATALOGUE' AND publisher_id = '{work_publisher}'"
    ));
    let outer =
        job_crud::claim_distribution_jobs(pool.as_ref(), "legacy", 10, 900, &[]).expect("claim");
    let (outer_job, outer_token) = (outer[0].job.job.distribution_job_id, outer[0].claim_token);
    let (unit, work_upsert, transcript) = interleave(
        &pool,
        PausePoint::install("t223b", "BEFORE INSERT", "crossref_write_permit"),
        move |pool| {
            permit_crud::reserve_back_catalogue_crossref_write(pool, outer_job, outer_token, work)
        },
        move |pool| reserve_work_upsert(pool, job, token).map(|r| r.permit_id),
        true,
    );
    assert!(unit.is_ok());
    assert_eq!(work_upsert, Err(ThothError::CrossrefPermitBlocked));
    assert!(!transcript.is_empty());
}

// ---------------------------------------------------------------------------------------------------------------------
// R52B section 25.17 T237-T240 and section 25.18 T269: the permit state machine against the implementation's own
// migration — the insertion guard, the authorization preconditions against bypass rows, the 30 ordered state pairs,
// and the immutable and write-once fields (section 29).
// ---------------------------------------------------------------------------------------------------------------------

/// A legacy permit brought to `state` through the API.
fn legacy_permit_in(
    pool: &crate::db::PgPool,
    connection: &mut PgConnection,
    state: CrossrefWritePermitState,
) -> CrossrefWriteReservation {
    let (publisher, imprint) = fx::publisher_and_imprint(pool);
    fx::cover_crossref(connection, publisher);
    let root = fx::insert_eligible_work(connection, imprint, Uuid::new_v4());
    let reservation =
        permit_crud::reserve_legacy_scheduled_crossref_write(pool, root).expect("reserve");
    let (permit, token) = (reservation.permit_id, reservation.reservation_token);
    match state {
        CrossrefWritePermitState::Reserved => {}
        CrossrefWritePermitState::Voided => {
            owner_void(pool, permit, token).expect("void");
        }
        other => {
            finalise(pool, &presentation(&reservation, None)).expect("finalise");
            let outcome = match other {
                CrossrefWritePermitState::Indeterminate => Some(Outcome::Indeterminate),
                CrossrefWritePermitState::Accepted => Some(Outcome::Accepted),
                CrossrefWritePermitState::NoneAttempted => Some(Outcome::NoneAttempted),
                _ => None,
            };
            if let Some(outcome) = outcome {
                report(pool, permit, token, outcome).expect("report");
            }
        }
    }
    reservation
}

/// Run `statement` in a transaction that is always rolled back; `Ok` when it was accepted, else the message.
fn attempt_rolled_back(connection: &mut PgConnection, statement: &str) -> Result<(), String> {
    use diesel::connection::SimpleConnection;
    use diesel::Connection;
    let mut outcome = Ok(());
    let _ = connection.transaction::<(), diesel::result::Error, _>(|connection| {
        if let Err(error) = connection.batch_execute(statement) {
            outcome = Err(error.to_string());
        }
        Err(diesel::result::Error::RollbackTransaction)
    });
    outcome
}

const STATE_LABELS: [&str; 6] = [
    "RESERVED",
    "VOIDED",
    "AUTHORIZED",
    "INDETERMINATE",
    "ACCEPTED",
    "NONE_ATTEMPTED",
];

/// The writes an act moving a permit to `to` would make, one variant per owning act.
fn transition_variants(to: &str) -> Vec<String> {
    let digest = DIGEST;
    match to {
        "RESERVED" => vec![
            "state = 'RESERVED', payload_digest = NULL, authorized_at = NULL, provider_reported_at = NULL, \
             void_reason = NULL, void_detail = NULL, closed_at = NULL"
                .to_string(),
        ],
        "VOIDED" => vec![
            "state = 'VOIDED', void_reason = 'OWNER_ABANDONED', void_detail = 'x', closed_at = coalesce(closed_at, now())"
                .to_string(),
            "state = 'VOIDED', void_reason = 'OWNER_ABANDONED', void_detail = 'x', closed_at = coalesce(closed_at, now()), \
             payload_digest = NULL, authorized_at = NULL, provider_reported_at = NULL"
                .to_string(),
        ],
        "AUTHORIZED" => vec![format!(
            "state = 'AUTHORIZED', payload_digest = coalesce(payload_digest, '{digest}'), \
             authorized_at = coalesce(authorized_at, now())"
        )],
        "INDETERMINATE" => vec![
            format!(
                "state = 'INDETERMINATE', payload_digest = coalesce(payload_digest, '{digest}'), \
                 authorized_at = coalesce(authorized_at, now()), provider_reported_at = coalesce(provider_reported_at, now())"
            ),
            format!(
                "state = 'INDETERMINATE', payload_digest = coalesce(payload_digest, '{digest}'), \
                 authorized_at = coalesce(authorized_at, now()), reconciliation_state = 'RECONCILIATION_REQUIRED', \
                 reconciliation_annotation_reference = 'ANN', reconciliation_annotated_at = now()"
            ),
        ],
        closed => vec![
            format!(
                "state = '{closed}', payload_digest = coalesce(payload_digest, '{digest}'), \
                 authorized_at = coalesce(authorized_at, now()), provider_reported_at = coalesce(provider_reported_at, now()), \
                 closed_at = coalesce(closed_at, now())"
            ),
            format!(
                "state = '{closed}', payload_digest = coalesce(payload_digest, '{digest}'), \
                 authorized_at = coalesce(authorized_at, now()), reconciliation_state = 'RECONCILED', \
                 reconciliation_authorization_reference = 'REC', reconciled_at = now(), closed_at = coalesce(closed_at, now())"
            ),
        ],
    }
}

#[test]
fn t239_t269_exactly_the_seven_edges_of_the_30_ordered_pairs_are_permitted() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let mut permitted = Vec::new();
    let mut refused = 0;
    for (from_state, from) in PERMIT_STATES.iter().zip(STATE_LABELS) {
        let permit = legacy_permit_in(pool.as_ref(), &mut connection, *from_state).permit_id;
        assert_eq!(state_name(&mut connection, permit), from);
        for to in STATE_LABELS.iter().filter(|to| **to != from) {
            let outcomes: Vec<Result<(), String>> = transition_variants(to)
                .iter()
                .map(|set| {
                    attempt_rolled_back(
                        &mut connection,
                        &format!(
                            "UPDATE crossref_write_permit SET {set} WHERE permit_id = '{permit}'"
                        ),
                    )
                })
                .collect();
            if outcomes.iter().any(Result::is_ok) {
                permitted.push(format!("{from}->{to}"));
            } else {
                refused += 1;
                for outcome in outcomes {
                    let message = outcome.expect_err("refused");
                    assert!(
                        message.starts_with("CROSSREF_")
                            || message.contains("violates check constraint"),
                        "{from}->{to}: {message}"
                    );
                }
            }
        }
    }
    permitted.sort();
    assert_eq!(
        permitted,
        vec![
            "AUTHORIZED->ACCEPTED",
            "AUTHORIZED->INDETERMINATE",
            "AUTHORIZED->NONE_ATTEMPTED",
            "INDETERMINATE->ACCEPTED",
            "INDETERMINATE->NONE_ATTEMPTED",
            "RESERVED->AUTHORIZED",
            "RESERVED->VOIDED",
        ]
    );
    assert_eq!(refused, 23);
}

#[test]
fn t237_t269_the_insertion_guard_admits_only_a_clean_reserved_row() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let template = legacy_permit_in(
        pool.as_ref(),
        &mut connection,
        CrossrefWritePermitState::Reserved,
    );
    let copy = |overrides: &str| {
        format!(
            "INSERT INTO crossref_write_permit \
             SELECT (json_populate_record(p, json_build_object('permit_id', gen_random_uuid(), \
                                                               'reservation_token', gen_random_uuid(){overrides}))).* \
               FROM crossref_write_permit p WHERE p.permit_id = '{}'",
            template.permit_id
        )
    };
    let shapes = [
        ("VOIDED", ", 'state', 'VOIDED', 'void_reason', 'OWNER_ABANDONED', 'void_detail', 'x', 'closed_at', now()"),
        ("AUTHORIZED", &format!(", 'state', 'AUTHORIZED', 'payload_digest', '{DIGEST}', 'authorized_at', now()") as &str),
        ("INDETERMINATE", &format!(", 'state', 'INDETERMINATE', 'payload_digest', '{DIGEST}', 'authorized_at', now(), 'provider_reported_at', now()")),
        ("ACCEPTED", &format!(", 'state', 'ACCEPTED', 'payload_digest', '{DIGEST}', 'authorized_at', now(), 'provider_reported_at', now(), 'closed_at', now()")),
        ("NONE_ATTEMPTED", &format!(", 'state', 'NONE_ATTEMPTED', 'payload_digest', '{DIGEST}', 'authorized_at', now(), 'provider_reported_at', now(), 'closed_at', now()")),
        ("RESERVED with a digest", &format!(", 'payload_digest', '{DIGEST}'")),
        ("RESERVED with authorized_at", ", 'authorized_at', now()"),
        ("RESERVED with a void reason", ", 'void_reason', 'OWNER_ABANDONED'"),
        ("RESERVED with a reconciliation state", ", 'reconciliation_state', 'RECONCILIATION_REQUIRED'"),
    ];
    for (shape, overrides) in shapes {
        let refused = attempt_rolled_back(&mut connection, &copy(overrides)).expect_err(shape);
        assert!(
            refused.contains("CROSSREF_PERMIT_INITIAL_STATE_INVALID"),
            "{shape}: {refused}"
        );
    }
}

#[test]
fn t240_t269_immutable_write_once_and_non_restorable_fields() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let update = |connection: &mut PgConnection, permit: Uuid, set: &str| {
        attempt_rolled_back(
            connection,
            &format!("UPDATE crossref_write_permit SET {set} WHERE permit_id = '{permit}'"),
        )
    };

    // The issuance reference is immutable.
    let (publisher, imprint) = fx::publisher_and_imprint(pool.as_ref());
    fx::cover_crossref(&mut connection, publisher);
    let root = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    let manual = permit_crud::reserve_manual_recovery_crossref_write(pool.as_ref(), root, "INC-1")
        .expect("manual");
    assert!(update(
        &mut connection,
        manual.permit_id,
        "operator_authorization_reference = 'INC-2'"
    )
    .expect_err("immutable")
    .contains("CROSSREF_PERMIT_EVIDENCE_IMMUTABLE"));
    // The void reference is write-once.
    permit_crud::void_crossref_write_reservation_as_superuser(
        pool.as_ref(),
        manual.permit_id,
        "cleanup",
        "INC-3",
    )
    .expect("superuser void");
    assert!(update(
        &mut connection,
        manual.permit_id,
        "void_authorization_reference = 'INC-4'"
    )
    .expect_err("write-once")
    .contains("CROSSREF_PERMIT_WRITE_ONCE_FIELD"));

    // The payload digest is write-once.
    let authorized = legacy_permit_in(
        pool.as_ref(),
        &mut connection,
        CrossrefWritePermitState::Authorized,
    );
    assert!(update(
        &mut connection,
        authorized.permit_id,
        &format!("payload_digest = '{}'", "f".repeat(64))
    )
    .expect_err("write-once")
    .contains("CROSSREF_PERMIT_WRITE_ONCE_FIELD"));
    // The reconciliation reference is write-once.
    reconcile(
        pool.as_ref(),
        authorized.permit_id,
        Outcome::Accepted,
        "REC-1",
    )
    .expect("reconcile");
    assert!(update(
        &mut connection,
        authorized.permit_id,
        "reconciliation_authorization_reference = 'REC-2'"
    )
    .expect_err("write-once")
    .contains("CROSSREF_PERMIT_WRITE_ONCE_FIELD"));

    // A NULLed link is not restorable.
    let reserved = legacy_permit_in(
        pool.as_ref(),
        &mut connection,
        CrossrefWritePermitState::Reserved,
    );
    let publisher_id = fx::texts(
        &mut connection,
        &format!(
            "SELECT publisher_id::text AS value FROM crossref_write_permit WHERE permit_id = '{}'",
            reserved.permit_id
        ),
    )
    .remove(0);
    fx::execute(
        &mut connection,
        &format!(
            "UPDATE crossref_write_permit SET publisher_id = NULL WHERE permit_id = '{}'",
            reserved.permit_id
        ),
    );
    assert!(update(
        &mut connection,
        reserved.permit_id,
        &format!("publisher_id = '{publisher_id}'")
    )
    .expect_err("not restorable")
    .contains("CROSSREF_PERMIT_LINK_NOT_RESTORABLE"));
}

#[test]
fn t238_authorization_preconditions_hold_against_bypass_rows() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let authorize = |connection: &mut PgConnection, permit: Uuid| {
        attempt_rolled_back(
            connection,
            &format!(
                "UPDATE crossref_write_permit SET state = 'AUTHORIZED', payload_digest = '{DIGEST}', authorized_at = now() \
                 WHERE permit_id = '{permit}'"
            ),
        )
    };
    // A RESERVED copy of `source`, with its membership, written with every trigger bypassed.
    let bypass_copy = |connection: &mut PgConnection, source: Uuid, timestamp: Option<i64>| {
        let copy = Uuid::new_v4();
        let timestamp = timestamp.map_or("p.crossref_timestamp".to_string(), |t| t.to_string());
        fx::execute(connection, &format!(
            "SET session_replication_role = replica; \
             INSERT INTO crossref_write_permit \
             SELECT (json_populate_record(p, json_build_object( \
                        'permit_id', '{copy}', 'reservation_token', gen_random_uuid(), 'state', 'RESERVED', \
                        'crossref_timestamp', {timestamp}, 'doi_batch_id', 'bypass-{copy}', 'payload_digest', NULL, \
                        'authorized_at', NULL, 'provider_reported_at', NULL, 'closed_at', NULL, \
                        'reconciliation_state', NULL, 'reconciliation_authorization_reference', NULL, 'reconciled_at', NULL))).* \
               FROM crossref_write_permit p WHERE p.permit_id = '{source}'; \
             INSERT INTO crossref_write_permit_doi (permit_id, doi) \
             SELECT '{copy}', doi FROM crossref_write_permit_doi WHERE permit_id = '{source}'; \
             SET session_replication_role = origin;"
        ));
        copy
    };

    // Overlapping an AUTHORIZED permit.
    let held = legacy_permit_in(
        pool.as_ref(),
        &mut connection,
        CrossrefWritePermitState::Authorized,
    );
    let copy = bypass_copy(
        &mut connection,
        held.permit_id,
        Some(held.crossref_timestamp + 1),
    );
    assert!(authorize(&mut connection, copy)
        .expect_err("blocked")
        .contains("CROSSREF_PERMIT_BLOCKED"));

    // Below history: an older ACCEPTED permit at or above the candidate.
    let accepted = legacy_permit_in(
        pool.as_ref(),
        &mut connection,
        CrossrefWritePermitState::Accepted,
    );
    let copy = bypass_copy(
        &mut connection,
        accepted.permit_id,
        Some(accepted.crossref_timestamp),
    );
    assert!(authorize(&mut connection, copy)
        .expect_err("history")
        .contains("CROSSREF_TIMESTAMP_NOT_INCREASING"));

    // A false witness: the root changed after the reservation.
    let reserved = legacy_permit_in(
        pool.as_ref(),
        &mut connection,
        CrossrefWritePermitState::Reserved,
    );
    fx::execute(&mut connection, &format!(
        "UPDATE work SET place = 'Changed' WHERE work_id = (SELECT root_work_identity FROM crossref_write_permit WHERE permit_id = '{}')",
        reserved.permit_id
    ));
    assert!(authorize(&mut connection, reserved.permit_id)
        .expect_err("witness")
        .contains("CROSSREF_ARTIFACT_SOURCE_CHANGED"));

    // Unfenced: a WORK_UPSERT permit whose attempt was never fenced.
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let work_upsert = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    assert!(authorize(&mut connection, work_upsert.permit_id)
        .expect_err("fence")
        .contains("CROSSREF_PERMIT_AUTHORIZATION_REQUIRES_FENCE"));
}

// ---------------------------------------------------------------------------------------------------------------------
// R52B section 25.19: the held publisher, T270-T275, against the implementation, with the released BE-02 assignment
// writer and BE-03 coordinator as the concurrent writers.
// ---------------------------------------------------------------------------------------------------------------------

use crate::model::publisher_distribution_platform::PublisherDistributionPlatform;

fn imprint_of(connection: &mut PgConnection, work: Uuid) -> Uuid {
    Uuid::parse_str(
        &fx::texts(
            connection,
            &format!("SELECT imprint_id::text AS value FROM work WHERE work_id = '{work}'"),
        )
        .remove(0),
    )
    .expect("uuid")
}

/// A dedicated session holding an uncommitted editorial statement.
fn hold(statement: &str) -> PgConnection {
    use diesel::connection::SimpleConnection;
    let mut session = race::dedicated();
    session
        .batch_execute(&format!("BEGIN; {statement}"))
        .expect("held statement");
    session
}

fn commit(mut session: PgConnection) {
    use diesel::connection::SimpleConnection;
    session.batch_execute("COMMIT").expect("commit");
}

/// The released BE-03 coordinator disabling the publisher's `CROSSREF` assignment: `lock_publisher FOR UPDATE`, the
/// assignment write, the `PENDING` cancellation, the publisher `UPDATE` and its cascade over the publisher's Works.
fn disable_crossref(pool: &crate::db::PgPool, publisher: Uuid) -> ThothResult<()> {
    use crate::model::publisher_service_configuration::{
        crud::replace_publisher_service_configuration, PublisherServiceConfigurationSource,
        ReplacePublisherServiceConfigurationInput, ServiceConfigurationWriteContext,
    };
    let current = crate::model::publisher::Publisher::from_id(pool, &publisher)?;
    replace_publisher_service_configuration(
        pool,
        &ServiceConfigurationWriteContext {
            source: PublisherServiceConfigurationSource::SuperuserApi,
            actor: "be06-race",
            job_creation: crate::model::distribution_job::DistributionJobCreation::Off,
        },
        &ReplacePublisherServiceConfigurationInput {
            publisher_id: publisher,
            subscription_package: current.subscription_package,
            enabled_distribution_platforms: Vec::new(),
            expected_updated_at: current.service_configuration_updated_at,
        },
    )
    .map(|_| ())
}

#[test]
fn t270_the_root_moves_back_to_the_permit_publisher_while_finalisation_waits_for_w() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (p1, i1, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    // The root observed under P2 before any lock.
    let (_p2, i2) = fx::publisher_and_imprint(pool.as_ref());
    fx::execute(
        &mut connection,
        &format!("UPDATE work SET imprint_id = '{i2}' WHERE work_id = '{work}'"),
    );
    // An editor moves it back to P1 and holds W.
    let editor = hold(&format!(
        "UPDATE work SET imprint_id = '{i1}' WHERE work_id = '{work}'"
    ));
    let mut observer = race::dedicated();
    let input = presentation(&reservation, Some(token));
    let finaliser_pool = pool.clone();
    let finaliser = std::thread::spawn(move || finalise(finaliser_pool.as_ref(), &input));
    let waits = race::wait_for_waits(&mut observer, 1);
    assert_eq!(
        waits,
        vec!["transactionid:ShareLock"],
        "finalisation waits for W"
    );
    // A BE-02 writer disabling P1's CROSSREF assignment waits on P1, which finalisation holds.
    let writer_pool = pool.clone();
    let writer = std::thread::spawn(move || disable_crossref(writer_pool.as_ref(), p1));
    race::wait_for_waits(&mut observer, 2);
    race::assert_blocked(&writer);
    commit(editor);
    let result = finaliser.join().expect("finaliser").expect("finalised");
    assert_eq!(
        (result.outcome, result.void_reason),
        (
            Finalised::VoidedRetryable,
            Some(CrossrefVoidReason::SourceChangedDuringPreparation)
        ),
        "the binding re-resolved as P1 under W, and both moves were captured"
    );
    assert_eq!(result.permit.permit.publisher_identity, p1);
    assert_eq!(writer.join().expect("writer"), Ok(()));
}

#[test]
fn t272_drift_that_stays_and_an_ownership_move_against_finalisation_in_both_orders() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");

    // The root moved to P2 and P2 reconfigured before W: BINDING_SUPERSEDED, no assignment read, no replacement.
    let (_p1, _i1, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let (p2, i2) = fx::publisher_and_imprint(pool.as_ref());
    fx::cover_crossref(&mut connection, p2);
    fx::execute(
        &mut connection,
        &format!("UPDATE work SET imprint_id = '{i2}' WHERE work_id = '{work}'"),
    );
    let result =
        finalise(pool.as_ref(), &presentation(&reservation, Some(token))).expect("finalise");
    assert_eq!(
        (result.outcome, result.void_reason),
        (
            Finalised::VoidedJobRetired,
            Some(CrossrefVoidReason::BindingSuperseded)
        )
    );
    assert_eq!(
        fx::job_summary(&mut connection, work).len(),
        1,
        "no replacement"
    );

    // Finalisation first, paused holding W FOR SHARE: the move waits at W, finalisation authorises, the move commits.
    let (_p1, _i1, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let (_p2, i2) = fx::publisher_and_imprint(pool.as_ref());
    let input = presentation(&reservation, Some(token));
    let (finalised, moved, transcript) = interleave(
        &pool,
        PausePoint::install("t272a", "BEFORE UPDATE", "crossref_write_permit"),
        move |pool| finalise(pool, &input).map(|r| r.outcome),
        move |pool| {
            use diesel::connection::SimpleConnection;
            pool.get()
                .expect("mover")
                .batch_execute(&format!(
                    "UPDATE work SET imprint_id = '{i2}' WHERE work_id = '{work}'"
                ))
                .map_err(|e| e.to_string())
        },
        true,
    );
    assert_eq!(finalised, Ok(Finalised::Authorized));
    assert_eq!(moved, Ok(()));
    assert_eq!(transcript, vec!["transactionid:ShareLock"]);

    // The move first, held: finalisation waits at W, then refuses BINDING_SUPERSEDED.
    let (_p1, _i1, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let (_p2, i2) = fx::publisher_and_imprint(pool.as_ref());
    let mover = hold(&format!(
        "UPDATE work SET imprint_id = '{i2}' WHERE work_id = '{work}'"
    ));
    let mut observer = race::dedicated();
    let input = presentation(&reservation, Some(token));
    let finaliser_pool = pool.clone();
    let finaliser = std::thread::spawn(move || finalise(finaliser_pool.as_ref(), &input));
    assert_eq!(
        race::wait_for_waits(&mut observer, 1),
        vec!["transactionid:ShareLock"]
    );
    commit(mover);
    let result = finaliser.join().expect("finaliser").expect("finalised");
    assert_eq!(
        (result.outcome, result.void_reason),
        (
            Finalised::VoidedJobRetired,
            Some(CrossrefVoidReason::BindingSuperseded)
        )
    );
}

#[test]
fn t273_a_configuration_writer_against_finalisation_in_both_orders_and_an_activation_change() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");

    // Finalisation first: the writer waits at P1 and commits after AUTHORIZED.
    let (p1, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let input = presentation(&reservation, Some(token));
    let (finalised, written, transcript) = interleave(
        &pool,
        PausePoint::install("t273a", "BEFORE UPDATE", "crossref_write_permit"),
        move |pool| finalise(pool, &input).map(|r| r.outcome),
        move |pool| disable_crossref(pool, p1),
        true,
    );
    assert_eq!(finalised, Ok(Finalised::Authorized));
    assert_eq!(written, Ok(()));
    assert_eq!(transcript, vec!["transactionid:ShareLock"]);

    // The writer first, disabling, paused holding P1: finalisation waits at P1, then ASSIGNMENT_DISABLED.
    let (p1, _i, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let input = presentation(&reservation, Some(token));
    let (written, finalised, transcript) = interleave(
        &pool,
        PausePoint::install("t273b", "BEFORE UPDATE", "publisher_distribution_platform"),
        move |pool| disable_crossref(pool, p1),
        move |pool| finalise(pool, &input).map(|r| (r.outcome, r.void_reason)),
        true,
    );
    assert_eq!(written, Ok(()));
    assert_eq!(
        finalised,
        Ok((
            Finalised::VoidedJobRetired,
            Some(CrossrefVoidReason::AssignmentDisabled)
        ))
    );
    assert_eq!(transcript, vec!["transactionid:ShareLock"]);
    assert_eq!(
        fx::job_summary(&mut connection, work).len(),
        1,
        "no replacement"
    );

    // The writer first, changing the activation to an admitted one: retired BINDING_SUPERSEDED, replaced under P1
    // and the new activation.
    let (p1, _i, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    PublisherDistributionPlatform::disable(pool.as_ref(), p1, DistributionPlatform::Crossref)
        .expect("disable");
    PublisherDistributionPlatform::enable(pool.as_ref(), p1, DistributionPlatform::Crossref)
        .expect("re-enable under a new activation");
    work_upsert_crud::admit_crossref_work_upsert(pool.as_ref(), p1, "EV-273", "admin")
        .expect("admit the new activation");
    let result =
        finalise(pool.as_ref(), &presentation(&reservation, Some(token))).expect("finalise");
    assert_eq!(
        (result.outcome, result.void_reason),
        (
            Finalised::VoidedJobRetired,
            Some(CrossrefVoidReason::BindingSuperseded)
        )
    );
    let jobs = fx::texts(&mut connection, &format!(
        "SELECT status::text || '|' || publisher_id::text || '|' || (activation_id = (SELECT activation_id FROM publisher_distribution_platform WHERE publisher_id = '{p1}' AND platform = 'CROSSREF'))::text AS value \
         FROM distribution_job WHERE work_identity = '{work}' ORDER BY job_ordinal"
    ));
    assert_eq!(
        jobs,
        vec![
            format!("CANCELLED|{p1}|false"),
            format!("PENDING|{p1}|true")
        ]
    );
}

#[test]
fn t274_the_coordinator_of_the_drifted_publisher_waits_behind_finalisation() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (_p1, _i1, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let publisher = test_db::create_publisher(pool.as_ref());
    let imprint = test_db::create_imprint(pool.as_ref(), &publisher);
    let p2 = publisher.publisher_id;
    fx::execute(
        &mut connection,
        &format!(
            "UPDATE work SET imprint_id = '{}' WHERE work_id = '{work}'",
            imprint.imprint_id
        ),
    );
    let current =
        crate::model::publisher::Publisher::from_id(pool.as_ref(), &p2).expect("publisher");
    let expected = current.service_configuration_updated_at;
    // A package change: the coordinator's publisher UPDATE and its cascade over every Work of P2.
    let package = if current.subscription_package == crate::model::publisher::ThothPackage::Obelisk
    {
        crate::model::publisher::ThothPackage::Pyramid
    } else {
        crate::model::publisher::ThothPackage::Obelisk
    };
    let input = presentation(&reservation, Some(token));
    let (finalised, coordinated, transcript) = interleave(
        &pool,
        PausePoint::install("t274", "BEFORE UPDATE", "crossref_write_permit"),
        move |pool| finalise(pool, &input).map(|r| (r.outcome, r.void_reason)),
        move |pool| {
            use crate::model::publisher_service_configuration::{
                crud::replace_publisher_service_configuration, PublisherServiceConfigurationSource,
                ReplacePublisherServiceConfigurationInput, ServiceConfigurationWriteContext,
            };
            replace_publisher_service_configuration(
                pool,
                &ServiceConfigurationWriteContext {
                    source: PublisherServiceConfigurationSource::SuperuserApi,
                    actor: "t274",
                    job_creation: crate::model::distribution_job::DistributionJobCreation::Off,
                },
                &ReplacePublisherServiceConfigurationInput {
                    publisher_id: p2,
                    subscription_package: package,
                    enabled_distribution_platforms: Vec::new(),
                    expected_updated_at: expected,
                },
            )
            .map(|_| ())
        },
        true,
    );
    assert_eq!(
        finalised,
        Ok((
            Finalised::VoidedJobRetired,
            Some(CrossrefVoidReason::BindingSuperseded)
        ))
    );
    assert_eq!(
        coordinated,
        Ok(()),
        "no 40P01: the coordinator completed after finalisation"
    );
    assert_eq!(
        transcript,
        vec!["transactionid:ShareLock"],
        "the cascade waited on W(root)"
    );
}

/// A fenced, reported attempt with a successor due: `(publisher, imprint, work, job, token)`.
fn completable_with_successor_due(
    pool: &crate::db::PgPool,
    connection: &mut PgConnection,
) -> (Uuid, Uuid, Uuid, Uuid, Uuid) {
    let (publisher, imprint, _a, work, job, token) = claimed_work_upsert(pool, connection);
    let reservation = reserve_work_upsert(pool, job, token).expect("reserve");
    finalise(pool, &presentation(&reservation, Some(token))).expect("finalise");
    report(
        pool,
        reservation.permit_id,
        reservation.reservation_token,
        Outcome::Accepted,
    )
    .expect("report");
    fx::execute(
        connection,
        &format!("UPDATE work SET place = 'Due' WHERE work_id = '{work}'"),
    );
    (publisher, imprint, work, job, token)
}

#[test]
fn t275_t2_completion_against_an_ownership_move_and_the_writer_in_both_orders() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let complete = |pool: &crate::db::PgPool, job: Uuid, token: Uuid| {
        job_crud::complete_distribution_job(pool, job, token).map(|j| j.status)
    };

    // Completion first, paused at the successor's insert holding P1 and W: the move waits; the successor is created
    // under the held P1.
    let (p1, _i, work, job, token) = completable_with_successor_due(pool.as_ref(), &mut connection);
    let (_p2, i2) = fx::publisher_and_imprint(pool.as_ref());
    let (completed, moved, transcript) = interleave(
        &pool,
        PausePoint::install("t275a", "BEFORE INSERT", "distribution_job"),
        move |pool| complete(pool, job, token),
        move |pool| {
            use diesel::connection::SimpleConnection;
            pool.get()
                .expect("mover")
                .batch_execute(&format!(
                    "UPDATE work SET imprint_id = '{i2}' WHERE work_id = '{work}'"
                ))
                .map_err(|e| e.to_string())
        },
        true,
    );
    assert_eq!(completed, Ok(DistributionJobStatus::Succeeded));
    assert_eq!(moved, Ok(()));
    assert_eq!(transcript, vec!["transactionid:ShareLock"]);
    let successor = fx::texts(&mut connection, &format!(
        "SELECT status::text || '|' || publisher_id::text AS value FROM distribution_job WHERE work_identity = '{work}' AND job_ordinal = 2"
    ));
    assert_eq!(successor, vec![format!("PENDING|{p1}")]);

    // The move first, held: completion waits at W, re-resolves P2, creates no successor; the residue stays.
    let (_p1, _i, work, job, token) =
        completable_with_successor_due(pool.as_ref(), &mut connection);
    let (_p2, i2) = fx::publisher_and_imprint(pool.as_ref());
    let mover = hold(&format!(
        "UPDATE work SET imprint_id = '{i2}' WHERE work_id = '{work}'"
    ));
    let mut observer = race::dedicated();
    let completer_pool = pool.clone();
    let completer = std::thread::spawn(move || complete(completer_pool.as_ref(), job, token));
    assert_eq!(
        race::wait_for_waits(&mut observer, 1),
        vec!["transactionid:ShareLock"]
    );
    commit(mover);
    assert_eq!(
        completer.join().expect("completer"),
        Ok(DistributionJobStatus::Succeeded)
    );
    assert_eq!(
        fx::job_summary(&mut connection, work).len(),
        1,
        "no successor"
    );
    assert!(
        fx::count(&mut connection, &format!(
            "SELECT count(*) AS count FROM work_upsert_generation g WHERE g.work_id = '{work}' \
             AND g.source_generation > public.work_upsert_resolution(g.work_id, g.execution_profile)"
        )) == 1,
        "the generation above D stays residue"
    );

    // Completion first against the writer: the writer waits at P1, then cancels the PENDING successor.
    let (p1, _i, work, job, token) = completable_with_successor_due(pool.as_ref(), &mut connection);
    let (completed, written, transcript) = interleave(
        &pool,
        PausePoint::install("t275c", "BEFORE INSERT", "distribution_job"),
        move |pool| complete(pool, job, token),
        move |pool| disable_crossref(pool, p1),
        true,
    );
    assert_eq!(completed, Ok(DistributionJobStatus::Succeeded));
    assert_eq!(written, Ok(()));
    assert_eq!(transcript, vec!["transactionid:ShareLock"]);
    assert_eq!(
        fx::texts(&mut connection, &format!(
            "SELECT status::text || '|' || coalesce(cancellation_reason::text, '-') AS value FROM distribution_job WHERE work_identity = '{work}' AND job_ordinal = 2"
        )),
        vec!["CANCELLED|ASSIGNMENT_DISABLED"]
    );

    // The writer first, paused holding P1: completion waits at P1; no successor.
    let (p1, _i, work, job, token) = completable_with_successor_due(pool.as_ref(), &mut connection);
    let (written, completed, transcript) = interleave(
        &pool,
        PausePoint::install("t275d", "BEFORE UPDATE", "publisher_distribution_platform"),
        move |pool| disable_crossref(pool, p1),
        move |pool| complete(pool, job, token),
        true,
    );
    assert_eq!(written, Ok(()));
    assert_eq!(completed, Ok(DistributionJobStatus::Succeeded));
    assert_eq!(transcript, vec!["transactionid:ShareLock"]);
    assert_eq!(
        fx::job_summary(&mut connection, work).len(),
        1,
        "no successor"
    );
    let _ = imprint_of;
}

// ---------------------------------------------------------------------------------------------------------------------
// R52B section 25.20: the execution gate Q, T280-T289, against the implementation.
// ---------------------------------------------------------------------------------------------------------------------

const GATE: &str = "1948572002, hashtext('be06:work_upsert:execution:CROSSREF')";

fn pause_execution(pool: &crate::db::PgPool) -> ThothResult<bool> {
    work_upsert_crud::set_work_upsert_execution(pool, DistributionPlatform::Crossref, false)
        .map(|c| c.execution_enabled)
}

fn enable_execution(pool: &crate::db::PgPool) -> ThothResult<bool> {
    work_upsert_crud::set_work_upsert_execution(pool, DistributionPlatform::Crossref, true)
        .map(|c| c.execution_enabled)
}

fn gate_waiters(observer: &mut PgConnection) -> i64 {
    fx::count(
        observer,
        "SELECT count(*) AS count FROM pg_locks WHERE locktype = 'advisory' AND classid = 1948572002 AND NOT granted",
    )
}

#[test]
fn t280_the_pause_against_finalisation_in_both_orders() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");

    // Order A: finalisation holds Q SHARE after reading true; the pause waits on Q EXCLUSIVE.
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let input = presentation(&reservation, Some(token));
    let (finalised, paused, transcript) = interleave(
        &pool,
        PausePoint::install("t280a", "BEFORE UPDATE", "crossref_write_permit"),
        move |pool| finalise(pool, &input).map(|r| r.outcome),
        pause_execution,
        true,
    );
    assert_eq!(finalised, Ok(Finalised::Authorized));
    assert_eq!(paused, Ok(false));
    assert_eq!(transcript, vec!["advisory:ExclusiveLock"]);
    assert_eq!(attempt_row(&mut connection, job), vec!["OPEN|true|false"]);

    // Order B: the pause holds Q with its update uncommitted; finalisation waits on Q SHARE, then reads false.
    enable_execution(pool.as_ref()).expect("enable");
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let input = presentation(&reservation, Some(token));
    let permit = reservation.permit_id;
    let (paused, finalised, transcript) = interleave(
        &pool,
        PausePoint::install("t280b", "AFTER UPDATE", "work_upsert_control"),
        pause_execution,
        move |pool| finalise(pool, &input).map(|r| (r.outcome, r.void_reason)),
        true,
    );
    assert_eq!(paused, Ok(false));
    assert_eq!(
        finalised,
        Ok((
            Finalised::VoidedRetryable,
            Some(CrossrefVoidReason::ExecutionNotPermitted)
        ))
    );
    assert_eq!(transcript, vec!["advisory:ShareLock"]);
    assert_eq!(state_name(&mut connection, permit), "VOIDED");
    assert_eq!(attempt_row(&mut connection, job), vec!["OPEN|false|false"]);
    assert_eq!(
        job_state(&mut connection, job).split('|').next(),
        Some("RUNNING")
    );
}

#[test]
fn t281_the_pause_against_the_claim_in_both_orders() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");

    // The claim first, holding Q SHARE after its statement: the pause waits and returns after the claim's commit.
    let (_p, _i, _a, _w, job) = fx::claimable_job(pool.as_ref(), &mut connection);
    fx::enable_execution(pool.as_ref());
    let (claimed, paused, transcript) = interleave(
        &pool,
        PausePoint::install("t281a", "AFTER UPDATE", "distribution_job"),
        |pool| fx::claim(pool).len(),
        pause_execution,
        true,
    );
    assert_eq!(claimed, 1);
    assert_eq!(paused, Ok(false));
    assert_eq!(transcript, vec!["advisory:ExclusiveLock"]);
    let _ = job;

    // The pause first: the claim waits on Q SHARE behind it, reads false and returns nothing.
    enable_execution(pool.as_ref()).expect("enable");
    let (_p, _i, _a, _w, job) = fx::claimable_job(pool.as_ref(), &mut connection);
    let (paused, claimed, transcript) = interleave(
        &pool,
        PausePoint::install("t281b", "AFTER UPDATE", "work_upsert_control"),
        pause_execution,
        |pool| fx::claim(pool).len(),
        true,
    );
    assert_eq!(paused, Ok(false));
    assert_eq!(claimed, 0);
    assert_eq!(transcript, vec!["advisory:ShareLock"]);
    assert_eq!(
        job_state(&mut connection, job).split('|').next(),
        Some("PENDING")
    );
    assert_eq!(fx::count(&mut connection, &format!("SELECT count(*) AS count FROM distribution_job_attempt WHERE distribution_job_id = '{job}'")), 0);
}

#[test]
fn t282_the_pause_against_the_work_upsert_reservation_in_both_orders() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");

    // The reservation first, holding Q: the pause waits; finalising after the pause voids EXECUTION_NOT_PERMITTED.
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let (reserved, paused, transcript) = interleave(
        &pool,
        PausePoint::install("t282a", "BEFORE INSERT", "crossref_write_permit"),
        move |pool| reserve_work_upsert(pool, job, token),
        pause_execution,
        true,
    );
    let reserved = reserved.expect("reserved");
    assert_eq!(paused, Ok(false));
    assert_eq!(transcript, vec!["advisory:ExclusiveLock"]);
    let result = finalise(pool.as_ref(), &presentation(&reserved, Some(token))).expect("finalise");
    assert_eq!(
        (result.outcome, result.void_reason),
        (
            Finalised::VoidedRetryable,
            Some(CrossrefVoidReason::ExecutionNotPermitted)
        )
    );
    assert_eq!(attempt_row(&mut connection, job), vec!["OPEN|false|false"]);

    // The pause first: the reservation waits on Q holding its prefix, reads false, and writes no permit.
    enable_execution(pool.as_ref()).expect("enable");
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let before = permit_count(&mut connection);
    let (paused, reserved, transcript) = interleave(
        &pool,
        PausePoint::install("t282b", "AFTER UPDATE", "work_upsert_control"),
        pause_execution,
        move |pool| reserve_work_upsert(pool, job, token).map(|r| r.permit_id),
        true,
    );
    assert_eq!(paused, Ok(false));
    assert_eq!(reserved, Err(ThothError::WorkUpsertExecutionNotPermitted));
    assert_eq!(transcript, vec!["advisory:ShareLock"]);
    assert_eq!(permit_count(&mut connection), before);
}

#[test]
fn t283_enable_against_the_claim_and_finalisation() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");

    // Enable first, holding Q: the claim waits behind it, then reads true and claims.
    let (_p, _i, _a, _w, _job) = fx::claimable_job(pool.as_ref(), &mut connection);
    work_upsert_crud::enable_work_upsert_capture(pool.as_ref(), DistributionPlatform::Crossref)
        .expect("capture");
    let (enabled, claimed, transcript) = interleave(
        &pool,
        PausePoint::install("t283a", "AFTER UPDATE", "work_upsert_control"),
        enable_execution,
        |pool| fx::claim(pool).len(),
        true,
    );
    assert_eq!(enabled, Ok(true));
    assert_eq!(claimed, 1);
    assert_eq!(transcript, vec!["advisory:ShareLock"]);

    // Enable first against finalisation: finalisation waits on Q, then reads true and authorises.
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    pause_execution(pool.as_ref()).expect("pause");
    let input = presentation(&reservation, Some(token));
    let (enabled, finalised, transcript) = interleave(
        &pool,
        PausePoint::install("t283b", "AFTER UPDATE", "work_upsert_control"),
        enable_execution,
        move |pool| finalise(pool, &input).map(|r| r.outcome),
        true,
    );
    assert_eq!(enabled, Ok(true));
    assert_eq!(finalised, Ok(Finalised::Authorized));
    assert_eq!(transcript, vec!["advisory:ShareLock"]);

    // Finalisation first under false, holding Q: enable waits until finalisation has voided.
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    pause_execution(pool.as_ref()).expect("pause");
    let input = presentation(&reservation, Some(token));
    let (finalised, enabled, transcript) = interleave(
        &pool,
        PausePoint::install("t283c", "BEFORE UPDATE", "crossref_write_permit"),
        move |pool| finalise(pool, &input).map(|r| (r.outcome, r.void_reason)),
        enable_execution,
        true,
    );
    assert_eq!(
        finalised,
        Ok((
            Finalised::VoidedRetryable,
            Some(CrossrefVoidReason::ExecutionNotPermitted)
        ))
    );
    assert_eq!(enabled, Ok(true));
    assert_eq!(transcript, vec!["advisory:ExclusiveLock"]);
    // A claim after enable claims the retried job once it is failed retryably.
    job_crud::fail_distribution_job(
        pool.as_ref(),
        job,
        token,
        "CROSSREF_PERMIT_VOIDED_RETRYABLE",
        None,
        true,
    )
    .expect("fail retryably");
    fx::execute(&mut connection, &format!("UPDATE distribution_job SET available_at = now() - interval '1 second' WHERE distribution_job_id = '{job}'"));
    assert_eq!(fx::claim(pool.as_ref()).len(), 1);
}

#[test]
fn t284_a_repeated_pause_with_a_claim_queued_behind_it() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (_p, _i, _a, _w, job) = fx::claimable_job(pool.as_ref(), &mut connection);
    fx::enable_execution(pool.as_ref());
    pause_execution(pool.as_ref()).expect("paused profile");
    let control_version = fx::texts(
        &mut connection,
        "SELECT xmin::text AS value FROM work_upsert_control",
    )
    .remove(0);

    // The first pause holds Q EXCLUSIVE, as the implementation's does.
    let first = hold(&format!("SELECT pg_advisory_xact_lock({GATE})"));
    let mut observer = race::dedicated();
    let second_pool = pool.clone();
    let second = std::thread::spawn(move || pause_execution(second_pool.as_ref()));
    fx::wait_until(|| gate_waiters(&mut observer) == 1);
    let claim_pool = pool.clone();
    let claim = std::thread::spawn(move || fx::claim(claim_pool.as_ref()).len());
    fx::wait_until(|| gate_waiters(&mut observer) == 2);
    // pg_blocking_pids of the queued claim names both the holder and the queued pause.
    assert_eq!(
        fx::count(&mut observer, "SELECT count(*) AS count FROM pg_stat_activity WHERE wait_event_type = 'Lock' AND cardinality(pg_blocking_pids(pid)) = 2"),
        1
    );
    commit(first);
    assert_eq!(second.join().expect("second pause"), Ok(false));
    assert_eq!(claim.join().expect("claim"), 0);
    assert_eq!(
        fx::texts(
            &mut connection,
            "SELECT xmin::text AS value FROM work_upsert_control"
        )
        .remove(0),
        control_version,
        "the repeated pause wrote nothing"
    );
    assert_eq!(
        job_state(&mut connection, job).split('|').next(),
        Some("PENDING")
    );
}

#[test]
fn t285_the_pause_does_not_wait_for_the_report_completion_or_t2() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (_p, _i, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    finalise(pool.as_ref(), &presentation(&reservation, Some(token))).expect("finalise");
    let (permit, reservation_token) = (reservation.permit_id, reservation.reservation_token);
    // The report holds X, uncommitted: the pause returns without waiting.
    let (reported, paused, _) = interleave(
        &pool,
        PausePoint::install("t285", "AFTER UPDATE", "crossref_write_permit"),
        move |pool| report(pool, permit, reservation_token, Outcome::Accepted),
        pause_execution,
        false,
    );
    assert_eq!(paused, Ok(false));
    assert_eq!(reported, Ok(CrossrefWritePermitState::Accepted));
    // After the pause: completion succeeds and T2 creates the due successor at generation 2; a claim returns nothing.
    fx::execute(
        &mut connection,
        &format!("UPDATE work SET place = 'Due' WHERE work_id = '{work}'"),
    );
    assert_eq!(
        job_crud::complete_distribution_job(pool.as_ref(), job, token).map(|j| j.status),
        Ok(DistributionJobStatus::Succeeded)
    );
    assert_eq!(
        fx::job_summary(&mut connection, work),
        vec![
            "SUCCEEDED|-|1|1|false|true|CROSSREF",
            "PENDING|-|2|2|true|false|CROSSREF"
        ]
    );
    assert!(fx::claim(pool.as_ref()).is_empty());
}

#[test]
fn t286_capture_and_materialization_proceed_during_the_pause_which_destroys_nothing() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (_p, _i, _a, running_work, _job, _token) =
        claimed_work_upsert(pool.as_ref(), &mut connection);
    let (_p2, _i2, _a2, residue_work) = fx::drainable_work(pool.as_ref(), &mut connection, 1);
    let snapshot = |connection: &mut PgConnection| {
        fx::texts(connection,
            "SELECT coalesce((SELECT string_agg(permit_id::text || xmin::text, ',') FROM crossref_write_permit), '') || '#' \
                 || coalesce((SELECT string_agg(distribution_job_id::text || xmin::text, ',' ORDER BY distribution_job_id) FROM distribution_job), '') || '#' \
                 || coalesce((SELECT string_agg(distribution_job_attempt_id::text || xmin::text, ',') FROM distribution_job_attempt), '') || '#' \
                 || coalesce((SELECT string_agg(work_id::text || xmin::text, ',' ORDER BY work_id) FROM work_upsert_generation), '') || '#' \
                 || coalesce((SELECT string_agg(publisher_id::text || xmin::text, ',') FROM work_upsert_admission), '') || '#' \
                 || coalesce((SELECT string_agg(publisher_id::text || xmin::text, ',') FROM publisher_distribution_platform), '') || '#' \
                 || coalesce((SELECT xmin::text FROM work_crossref_version_floor), '') AS value")
            .remove(0)
    };

    // The pause holds Q with its update uncommitted: a capture and a materialization commit without waiting.
    let mut pause = PausePoint::install("t286", "AFTER UPDATE", "work_upsert_control");
    let mut observer = race::dedicated();
    let pause_pool = pool.clone();
    let pauser = std::thread::spawn(move || pause_execution(pause_pool.as_ref()));
    fx::wait_until(|| race::paused_sessions(&mut observer) == 1);
    fx::execute(
        &mut connection,
        &format!("UPDATE work SET place = 'During' WHERE work_id = '{running_work}'"),
    );
    let unit = fx::materialize(pool.as_ref(), residue_work, false);
    assert_eq!(
        unit.outcome,
        crate::model::work_upsert::WorkUpsertMaterializationOutcome::Created
    );
    assert!(
        race::waits(&mut observer).is_empty(),
        "nothing waited on the pause"
    );
    pause.release();
    assert_eq!(pauser.join().expect("pause"), Ok(false));
    drop(pause);
    assert!(
        fx::claim(pool.as_ref()).is_empty(),
        "nothing is claimable after the pause"
    );
    assert_eq!(
        fx::job_summary(&mut connection, residue_work),
        vec!["PENDING|-|1|1|false|false|CROSSREF"]
    );

    // Sequentially: a repeated pause and an enable change nothing but the control row.
    let before = snapshot(&mut connection);
    assert_eq!(pause_execution(pool.as_ref()), Ok(false));
    assert_eq!(snapshot(&mut connection), before);
    assert_eq!(enable_execution(pool.as_ref()), Ok(true));
    assert_eq!(snapshot(&mut connection), before);
    // The control CHECK refuses (capture = false, execution = true).
    // Capture is monotone, so the row is first brought to (false, false) with triggers bypassed, inside the
    // rolled-back transaction; the CHECK itself is never bypassed.
    let refused = attempt_rolled_back(
        &mut connection,
        "SET LOCAL session_replication_role = replica; \
         UPDATE work_upsert_control SET capture_enabled = false, execution_enabled = false; \
         SET LOCAL session_replication_role = origin; \
         UPDATE work_upsert_control SET execution_enabled = true",
    )
    .expect_err("check");
    assert!(
        refused.contains("work_upsert_control_execution_requires_capture_check"),
        "{refused}"
    );
}

#[test]
fn t287_no_new_deadlock_with_the_coordinator_work_deletion_and_publisher_deletion() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let delete_work = |pool: &crate::db::PgPool, work: Uuid| {
        crate::model::work::Work::from_id(pool, &work)
            .expect("work")
            .delete(pool)
            .map(|w| w.work_id)
    };

    // Finalisation holding Q: the pause waits on Q, the writer on P1 and Work deletion on W; finalisation authorises.
    let (p1, _i, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let mut pause = PausePoint::install("t287a", "BEFORE UPDATE", "crossref_write_permit");
    let mut observer = race::dedicated();
    let input = presentation(&reservation, Some(token));
    let p = pool.clone();
    let finaliser = std::thread::spawn(move || finalise(p.as_ref(), &input).map(|r| r.outcome));
    fx::wait_until(|| race::paused_sessions(&mut observer) == 1);
    let p = pool.clone();
    let pauser = std::thread::spawn(move || pause_execution(p.as_ref()));
    let p = pool.clone();
    let writer = std::thread::spawn(move || disable_crossref(p.as_ref(), p1));
    let p = pool.clone();
    let deleter = std::thread::spawn(move || delete_work(p.as_ref(), work));
    race::wait_for_waits(&mut observer, 3);
    pause.release();
    assert_eq!(
        finaliser.join().expect("finaliser"),
        Ok(Finalised::Authorized)
    );
    assert_eq!(pauser.join().expect("pauser"), Ok(false));
    assert_eq!(writer.join().expect("writer"), Ok(()));
    assert_eq!(
        deleter.join().expect("deleter"),
        Err(ThothError::WorkDeleteBlockedByFencedAttempt)
    );
    drop(pause);

    // The writer first, holding P1: finalisation waits at P1 before Q, and the pause does not wait.
    enable_execution(pool.as_ref()).expect("enable");
    let (p1, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let mut pause =
        PausePoint::install("t287b", "BEFORE UPDATE", "publisher_distribution_platform");
    let p = pool.clone();
    let writer = std::thread::spawn(move || disable_crossref(p.as_ref(), p1));
    fx::wait_until(|| race::paused_sessions(&mut observer) == 1);
    let input = presentation(&reservation, Some(token));
    let p = pool.clone();
    let finaliser = std::thread::spawn(move || {
        finalise(p.as_ref(), &input).map(|r| (r.outcome, r.void_reason))
    });
    race::wait_for_waits(&mut observer, 1);
    assert_eq!(
        pause_execution(pool.as_ref()),
        Ok(false),
        "the pause does not wait"
    );
    pause.release();
    assert_eq!(writer.join().expect("writer"), Ok(()));
    assert_eq!(
        finaliser.join().expect("finaliser"),
        Ok((
            Finalised::VoidedJobRetired,
            Some(CrossrefVoidReason::AssignmentDisabled)
        ))
    );
    drop(pause);

    // Work deletion first, holding W: finalisation waits at W, the pause does not wait; the claim is stale.
    enable_execution(pool.as_ref()).expect("enable");
    let (_p, _i, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let mut pause = PausePoint::install("t287c", "BEFORE DELETE", "work");
    let p = pool.clone();
    let deleter = std::thread::spawn(move || delete_work(p.as_ref(), work));
    fx::wait_until(|| race::paused_sessions(&mut observer) == 1);
    let input = presentation(&reservation, Some(token));
    let p = pool.clone();
    let finaliser = std::thread::spawn(move || finalise(p.as_ref(), &input).map(|r| r.outcome));
    race::wait_for_waits(&mut observer, 1);
    assert_eq!(
        pause_execution(pool.as_ref()),
        Ok(false),
        "the pause does not wait"
    );
    pause.release();
    assert_eq!(deleter.join().expect("deleter"), Ok(work));
    assert_eq!(
        finaliser.join().expect("finaliser"),
        Err(ThothError::CrossrefPermitClaimStale)
    );
    drop(pause);

    // Publisher deletion holding the PENDING job: the claim takes Q, skips the locked job and returns without waiting.
    enable_execution(pool.as_ref()).expect("enable");
    let (publisher, _i, _a, _w, _job) = fx::claimable_job(pool.as_ref(), &mut connection);
    let (deleted, claimed, transcript) = interleave(
        &pool,
        PausePoint::install("t287d", "BEFORE DELETE", "publisher"),
        move |pool| {
            crate::model::publisher::Publisher::from_id(pool, &publisher)
                .expect("publisher")
                .delete(pool)
                .map(|p| p.publisher_id)
        },
        |pool| fx::claim(pool).len(),
        false,
    );
    assert_eq!(deleted, Ok(publisher));
    assert_eq!(claimed, 0);
    assert!(transcript.is_empty());
}

#[test]
fn t288_reservation_keys_floor_permit_row_and_reconciliation_with_the_gate_queued() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let mut observer = race::dedicated();

    // A WORK_UPSERT reservation holding Q and its DOI keys: the pause waits on Q and an overlapping legacy reservation
    // on the shared key, which is then refused.
    let (_p, _i, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let mut pause = PausePoint::install("t288a", "BEFORE INSERT", "crossref_write_permit");
    let p = pool.clone();
    let reserver = std::thread::spawn(move || {
        reserve_work_upsert(p.as_ref(), job, token).map(|r| r.permit_id)
    });
    fx::wait_until(|| race::paused_sessions(&mut observer) == 1);
    let p = pool.clone();
    let pauser = std::thread::spawn(move || pause_execution(p.as_ref()));
    let p = pool.clone();
    let legacy = std::thread::spawn(move || {
        permit_crud::reserve_legacy_scheduled_crossref_write(p.as_ref(), work).map(|r| r.permit_id)
    });
    for _ in 0..400 {
        if race::waits(&mut observer).len() >= 2 || pauser.is_finished() || legacy.is_finished() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(
        !pauser.is_finished() && !legacy.is_finished(),
        "waits {:?}, pause finished {}, legacy finished {}",
        race::waits(&mut observer),
        pauser.is_finished(),
        legacy.is_finished()
    );
    let waits = race::waits(&mut observer);
    assert!(
        waits.contains(&"advisory:ExclusiveLock".to_string()),
        "{waits:?}"
    );
    pause.release();
    let first_permit = reserver
        .join()
        .expect("reserver")
        .expect("the WORK_UPSERT reservation");
    assert_eq!(pauser.join().expect("pauser"), Ok(false));
    assert!(matches!(
        legacy.join().expect("legacy"),
        Err(ThothError::CrossrefPermitBlocked)
    ));
    drop(pause);
    // Resolve that permit, so the floor advance below is not refused NOT_DRAINED before its pause point.
    permit_crud::void_crossref_write_reservation_as_superuser(
        pool.as_ref(),
        first_permit,
        "t288",
        "INC-288",
    )
    .expect("void");

    // G-7 holding F: a reservation holding Q and its keys waits at F, the pause queued behind; G-7 advances, the
    // reservation allocates above the floor, and the pause completes last.
    enable_execution(pool.as_ref()).expect("enable");
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let mut pause = PausePoint::install("t288e", "BEFORE INSERT", "crossref_version_floor_audit");
    let p = pool.clone();
    let advancer = std::thread::spawn(move || {
        permit_crud::advance_crossref_version_floor(
            p.as_ref(),
            &advance_input(Uuid::new_v4()),
            "superuser",
        )
        .map(|a| a.after_value)
    });
    fx::wait_until(|| race::paused_sessions(&mut observer) == 1);
    let p = pool.clone();
    let reserver = std::thread::spawn(move || {
        reserve_work_upsert(p.as_ref(), job, token).map(|r| r.crossref_timestamp)
    });
    race::wait_for_waits(&mut observer, 1);
    let p = pool.clone();
    let pauser = std::thread::spawn(move || pause_execution(p.as_ref()));
    fx::wait_until(|| gate_waiters(&mut observer) == 1);
    pause.release();
    assert_eq!(advancer.join().expect("advancer"), Ok(TARGET));
    assert!(reserver.join().expect("reserver").expect("reserved") > TARGET);
    assert_eq!(pauser.join().expect("pauser"), Ok(false));
    drop(pause);

    // Finalisation holding Q makes reconciliation wait at the attempt row; reconciliation then records ACCEPTED.
    enable_execution(pool.as_ref()).expect("enable");
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let input = presentation(&reservation, Some(token));
    let permit = reservation.permit_id;
    let (finalised, reconciled, transcript) = interleave(
        &pool,
        PausePoint::install("t288c", "BEFORE UPDATE", "crossref_write_permit"),
        move |pool| finalise(pool, &input).map(|r| r.outcome),
        move |pool| reconcile(pool, permit, Outcome::Accepted, "R-288"),
        true,
    );
    assert_eq!(finalised, Ok(Finalised::Authorized));
    assert_eq!(reconciled, Ok(CrossrefWritePermitState::Accepted));
    assert_eq!(transcript, vec!["transactionid:ShareLock"]);

    // A void holding the permit row makes finalisation, holding Q, wait, with the pause queued behind it.
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let reservation = reserve_work_upsert(pool.as_ref(), job, token).expect("reserve");
    let (permit, reservation_token) = (reservation.permit_id, reservation.reservation_token);
    let mut pause = PausePoint::install("t288d", "AFTER UPDATE", "crossref_write_permit");
    let p = pool.clone();
    let voider = std::thread::spawn(move || owner_void(p.as_ref(), permit, reservation_token));
    fx::wait_until(|| race::paused_sessions(&mut observer) == 1);
    let input = presentation(&reservation, Some(token));
    let p = pool.clone();
    let finaliser = std::thread::spawn(move || finalise(p.as_ref(), &input).map(|r| r.outcome));
    race::wait_for_waits(&mut observer, 1);
    let p = pool.clone();
    let pauser = std::thread::spawn(move || pause_execution(p.as_ref()));
    fx::wait_until(|| gate_waiters(&mut observer) == 1);
    pause.release();
    assert_eq!(
        voider.join().expect("voider"),
        Ok(CrossrefWritePermitState::Voided)
    );
    assert_eq!(
        finaliser.join().expect("finaliser"),
        Err(ThothError::CrossrefPermitIllegalTransition)
    );
    assert_eq!(pauser.join().expect("pauser"), Ok(false));

    // Reversed: a legacy reservation holds the shared DOI key; the WORK_UPSERT reservation holds Q and waits for the
    // key; the pause queues on Q behind it. The three commit legacy, refused reservation, pause.
    enable_execution(pool.as_ref()).expect("enable");
    let (_p, imprint, _a, work, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let parent = fx::insert_eligible_work(&mut connection, imprint, Uuid::new_v4());
    fx::execute(
        &mut connection,
        &format!("UPDATE work SET landing_page = NULL WHERE work_id = '{parent}'"),
    );
    fx::relate_child(&mut connection, parent, work, 1);
    let mut pause = PausePoint::install("t288b", "BEFORE INSERT", "crossref_write_permit");
    let p = pool.clone();
    let legacy = std::thread::spawn(move || {
        permit_crud::reserve_legacy_scheduled_crossref_write(p.as_ref(), parent)
            .map(|r| r.permit_id)
    });
    fx::wait_until(|| race::paused_sessions(&mut observer) == 1);
    let p = pool.clone();
    let reserver = std::thread::spawn(move || {
        reserve_work_upsert(p.as_ref(), job, token).map(|r| r.permit_id)
    });
    fx::wait_until(|| race::waits(&mut observer) == vec!["advisory:ExclusiveLock"]);
    let p = pool.clone();
    let pauser = std::thread::spawn(move || pause_execution(p.as_ref()));
    fx::wait_until(|| gate_waiters(&mut observer) == 1);
    assert!(!reserver.is_finished() && !pauser.is_finished());
    pause.release();
    assert!(legacy.join().expect("legacy").is_ok());
    assert_eq!(
        reserver.join().expect("reserver"),
        Err(ThothError::CrossrefPermitBlocked)
    );
    assert_eq!(pauser.join().expect("pauser"), Ok(false));
}

#[test]
fn t289_the_gate_key_collides_with_no_other_advisory_key() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let holder = hold(&format!("SELECT pg_advisory_xact_lock({GATE})"));
    let q = fx::texts(
        &mut connection,
        "SELECT hashtext('be06:work_upsert:execution:CROSSREF')::text AS value",
    )
    .remove(0);
    for (label, lock) in [
        ("the DOI namespace with the same second key", format!("pg_try_advisory_xact_lock(1948572001, {q})")),
        ("the one-argument key (namespace << 32 | q)", format!("pg_try_advisory_xact_lock((1948572002::bigint << 32) | ({q}::bigint & 4294967295))")),
        ("another profile's gate", "pg_try_advisory_xact_lock(1948572002, hashtext('be06:work_upsert:execution:ZENODO'))".to_string()),
        ("the one-argument hashtext form", format!("pg_try_advisory_xact_lock({q})")),
    ] {
        assert_eq!(
            attempt_rolled_back(&mut connection, &format!("DO $$ BEGIN IF NOT {lock} THEN RAISE EXCEPTION 'busy'; END IF; END $$")),
            Ok(()),
            "{label} was granted"
        );
    }
    for lock in [
        format!("pg_try_advisory_xact_lock({GATE})"),
        format!("pg_try_advisory_xact_lock_shared({GATE})"),
    ] {
        assert!(
            attempt_rolled_back(
                &mut connection,
                &format!("DO $$ BEGIN IF NOT {lock} THEN RAISE EXCEPTION 'busy'; END IF; END $$")
            )
            .is_err(),
            "the gate itself is refused: {lock}"
        );
    }
    commit(holder);
    // The 17 DistributionPlatform members give 17 distinct gate keys.
    assert_eq!(
        fx::texts(&mut connection,
            "SELECT count(*)::text || '|' || count(DISTINCT hashtext('be06:work_upsert:execution:' || enumlabel))::text AS value \
             FROM pg_enum WHERE enumtypid = 'distribution_platform'::regtype"),
        vec!["17|17"]
    );
}

#[test]
fn r7_two_reservations_of_one_attempt_yield_one_permit() {
    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    let (_p, _i, _a, _w, job, token) = claimed_work_upsert(pool.as_ref(), &mut connection);
    let (first, second, transcript) = interleave(
        &pool,
        PausePoint::install("r7", "BEFORE INSERT", "crossref_write_permit"),
        move |pool| reserve_work_upsert(pool, job, token).map(|r| r.permit_id),
        move |pool| reserve_work_upsert(pool, job, token).map(|r| r.permit_id),
        true,
    );
    assert!(first.is_ok());
    assert_eq!(
        second,
        Err(ThothError::CrossrefPermitAttemptAlreadyReserved)
    );
    assert_eq!(
        transcript,
        vec!["transactionid:ShareLock"],
        "the second waits on J"
    );
    assert_eq!(permit_count(&mut connection), 1);
}

// ---------------------------------------------------------------------------------------------------------------------
// Static containment: F16 and R52B T167 (no provider or GitHub traffic path), and T203 (no generic permit writer).
// ---------------------------------------------------------------------------------------------------------------------

fn non_test_sources(root: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(root).expect("read") {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            non_test_sources(&path, files);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if name != "tests.rs" && !name.ends_with("_tests.rs") {
                files.push(path);
            }
        }
    }
}

#[test]
fn f16_t167_no_be06_path_carries_a_github_client_or_any_network_call() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    // F16: the four named files.
    for path in [
        "src/model/crossref_write_permit/mod.rs",
        "src/model/crossref_write_permit/crud.rs",
        "src/graphql/work_upsert.rs",
        "src/graphql/mutation.rs",
    ] {
        let text = std::fs::read_to_string(manifest.join(path))
            .expect("read")
            .to_lowercase();
        let (forge, client) = (
            "buhtig".chars().rev().collect::<String>(),
            "tsewqer".chars().rev().collect::<String>(),
        );
        assert!(!text.contains(&forge), "{path} names {forge}");
        assert!(!text.contains(&client), "{path} names {client}");
    }
    // T167: no BE-06 file, source or test, has an HTTP client or a provider endpoint, so no test and no code path can
    // reach Crossref.
    let workspace = manifest.parent().expect("workspace");
    let be06_files = [
        "thoth-api/src/model/work_upsert/mod.rs",
        "thoth-api/src/model/work_upsert/registry.rs",
        "thoth-api/src/model/work_upsert/crud.rs",
        "thoth-api/src/model/work_upsert/policy.rs",
        "thoth-api/src/model/work_upsert/tests.rs",
        "thoth-api/src/model/crossref_write_permit/mod.rs",
        "thoth-api/src/model/crossref_write_permit/crud.rs",
        "thoth-api/src/model/crossref_write_permit/tests.rs",
        "thoth-api/src/graphql/work_upsert.rs",
        "thoth-api/tests/work_upsert_capture.rs",
        "thoth-api/tests/crossref_permit_concurrency.rs",
        "thoth-api/migrations/20260910_v1.10.0/up.sql",
        "thoth-api/migrations/20260911_v1.10.0/up.sql",
    ];
    for path in be06_files {
        let Ok(text) = std::fs::read_to_string(workspace.join(path)) else {
            continue;
        };
        let lower = text.to_lowercase();
        for forbidden in [
            "tsewqer",
            "tneilc::repyh",
            "tneilc::cwa",
            "qeru",
            "chasi",
            "::lruc",
            "maertspct",
            "gro.ferssorc.iod",
            "gro.ferssorc.ipa",
            "gro.ferssorc.tset",
            "knilbd",
            "tseuqer_ptth",
        ]
        .map(|reversed| reversed.chars().rev().collect::<String>())
        {
            assert!(!lower.contains(&forbidden), "{path} contains {forbidden}");
        }
    }
}

#[test]
fn t203_no_generic_path_writes_a_permit_and_the_only_referential_actions_are_set_null() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    non_test_sources(&manifest.join("src"), &mut files);
    let sql_write = regex::Regex::new(
        r"(?i)(insert\s+into|update|delete\s+from|truncate)\s+(public\.)?crossref_write_permit(_doi)?\b",
    )
    .expect("regex");
    let alias = regex::Regex::new(r"crossref_write_permit(_doi)?\s+as\s+(\w+)").expect("regex");
    let mut writers = Vec::new();
    for path in &files {
        let text = std::fs::read_to_string(path).expect("read");
        let display = path.display().to_string();
        let mut names = vec![
            "crossref_write_permit".to_string(),
            "crossref_write_permit_doi".to_string(),
        ];
        names.extend(alias.captures_iter(&text).map(|c| c[2].to_string()));
        let mut writes = sql_write.is_match(&text);
        for name in &names {
            for call in ["insert_into", "update", "delete"] {
                if text.contains(&format!("{call}({name}::table")) {
                    writes = true;
                }
            }
        }
        if writes {
            writers.push(display);
        }
    }
    assert_eq!(writers.len(), 1, "{writers:?}");
    assert!(
        writers[0].ends_with("src/model/crossref_write_permit/crud.rs"),
        "{writers:?}"
    );

    let (_guard, pool) = test_db::setup_test_db();
    let mut connection = pool.get().expect("connection");
    assert_eq!(
        fx::texts(
            &mut connection,
            "SELECT string_agg(conname || ':' || confdeltype::text, ',' ORDER BY conname) AS value FROM pg_constraint \
             WHERE contype = 'f' AND conrelid = 'public.crossref_write_permit'::regclass"
        ),
        vec![
            "crossref_write_permit_distribution_job_attempt_id_fkey:n,crossref_write_permit_distribution_job_id_fkey:n,crossref_write_permit_publisher_id_fkey:n"
        ]
    );
    assert_eq!(
        fx::count(
            &mut connection,
            "SELECT count(*) AS count FROM pg_constraint WHERE contype = 'f' \
             AND confrelid IN ('public.crossref_write_permit'::regclass, 'public.crossref_write_permit_doi'::regclass) \
             AND confdeltype NOT IN ('a', 'r')"
        ),
        0,
        "nothing cascades into or out of the permit tables"
    );
}
