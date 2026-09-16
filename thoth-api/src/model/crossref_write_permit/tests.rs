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

use diesel::connection::SimpleConnection;
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
