//! Focused `MET-WP1-11` database tests for `metric_reconciliation_issue`: the
//! approved eight-field contract, the repository-standard UUID identity
//! default, the required non-cascading run relationship, the nullable
//! non-cascading canonical-record relationship, opaque required `issue_type`
//! and `severity`, the deliberately unconstrained and deliberately
//! unreferenced `remote_event_id`, the `'{}'` `details` default, nullable
//! `resolved_at`, and the exact column/check/foreign-key/index/default
//! inventory.
//!
//! The reconciliation-run fixture and the raw-SQL column helpers come from
//! `metric_reconciliation_run/tests.rs`, the sibling module this task creates;
//! the canonical record fixture reuses the existing `pub(crate)` helper
//! `fixture_record` from `metric_record_revision/tests.rs`. No other model's
//! test module is widened, because this task's write budget does not permit
//! it: in particular the merged `metric_operas_import/tests.rs` fixtures are
//! not exported, so the inbound-ledger rows the remote-event tests need are
//! inserted here through raw SQL.
//!
//! These tests deliberately assert **schema** behaviour only. This slice
//! creates no reconciliation issue at runtime and implements no reconciliation
//! execution, comparison, issue classification, closed issue-type or severity
//! vocabulary, resolution or reopening workflow, loop prevention, divergence
//! handling or completeness determination, and nothing here pretends
//! otherwise. In particular an accepted `issue_type` or `severity` string is
//! evidence of nonblank-text storage only, never of a recognised issue class
//! or severity level, and an accepted `details` document is evidence of JSONB
//! storage only, never of validated reconciliation evidence.
//!
//! Remote-event boundary (reviewed and load-bearing): `MET-WP1-10` established
//! canonical remote identity as the composite `(remote_instance,
//! remote_event_id)` and deliberately established that a bare
//! `remote_event_id` is not globally unique. The tests below therefore assert
//! that this column has no foreign key, no uniqueness and no companion
//! `remote_instance`, `operas_import_id`, `export_id`, `mapping_id` or
//! `record_revision_id` column, and that a remote event unknown to the inbound
//! ledger stays representable.

use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{sql_query, ExpressionMethods, QueryDsl, RunQueryDsl};
use uuid::Uuid;

use super::MetricReconciliationIssue;
use crate::db::PgPool;
use crate::model::metric_import::tests::check_constraint_names;
use crate::model::metric_platform::tests::{scalar_i64, setup_registry_db};
use crate::model::metric_reconciliation_run::tests::{fixture_run, insert_run_row, Column, RunRow};
use crate::model::metric_record::tests::{delete_row, foreign_keys, index_definition, index_names};
use crate::model::metric_record_revision::tests::fixture_record;
use crate::model::Timestamp;
use crate::schema::metric_reconciliation_issue;

/// Column names that would betray an invented reconciliation relationship, a
/// resolution workflow or a claim/retry protocol having been smuggled into
/// this persistence-only slice. The approved design's issue shorthand names
/// none of them, and the first five would in particular manufacture
/// referential integrity the merged inbound-ledger identity contract does not
/// support.
const DEFERRED_ISSUE_COLUMNS: [&str; 13] = [
    "export_id",
    "mapping_id",
    "operas_import_id",
    "record_revision_id",
    "remote_instance",
    "attempt_count",
    "claimed_at",
    "created_at",
    "reopened_at",
    "resolution",
    "resolved_by",
    "status",
    "updated_at",
];

/// The column values one raw-SQL reconciliation-issue insert supplies.
struct IssueRow<'a> {
    issue_id: Column<'a>,
    run_id: Column<'a>,
    issue_type: Column<'a>,
    severity: Column<'a>,
    record_id: Column<'a>,
    remote_event_id: Column<'a>,
    details: Column<'a>,
    resolved_at: Column<'a>,
}

impl<'a> IssueRow<'a> {
    /// The minimal valid issue: an arbitrary nonblank fixture type and
    /// severity belonging to `run_id`, with the identity, canonical record,
    /// remote event, details and resolution columns left out so their defaults
    /// — and the deliberate nullability of the rest — are exercised rather
    /// than restated. None of these values is approved reconciliation data.
    fn minimal(run_id: &'a str) -> Self {
        Self {
            issue_id: Column::Omitted,
            run_id: Column::Value(run_id),
            issue_type: Column::Value("fixture-issue-type"),
            severity: Column::Value("fixture-severity"),
            record_id: Column::Omitted,
            remote_event_id: Column::Omitted,
            details: Column::Omitted,
            resolved_at: Column::Omitted,
        }
    }
}

/// Insert one reconciliation issue through raw SQL.
fn insert_issue_row(pool: &PgPool, row: IssueRow<'_>) -> Result<usize, DieselError> {
    use crate::model::metric_reconciliation_run::tests::push_column;

    let mut columns: Vec<&'static str> = Vec::new();
    let mut values: Vec<String> = Vec::new();
    push_column(
        &mut columns,
        &mut values,
        "issue_id",
        row.issue_id,
        "::uuid",
    );
    push_column(&mut columns, &mut values, "run_id", row.run_id, "::uuid");
    push_column(&mut columns, &mut values, "issue_type", row.issue_type, "");
    push_column(&mut columns, &mut values, "severity", row.severity, "");
    push_column(
        &mut columns,
        &mut values,
        "record_id",
        row.record_id,
        "::uuid",
    );
    push_column(
        &mut columns,
        &mut values,
        "remote_event_id",
        row.remote_event_id,
        "",
    );
    push_column(&mut columns, &mut values, "details", row.details, "::jsonb");
    push_column(
        &mut columns,
        &mut values,
        "resolved_at",
        row.resolved_at,
        "::timestamptz",
    );

    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(format!(
        "INSERT INTO metric_reconciliation_issue ({}) VALUES ({})",
        columns.join(", "),
        values.join(", ")
    ))
    .execute(&mut connection)
}

/// The single stored reconciliation issue.
fn only_issue(pool: &PgPool) -> MetricReconciliationIssue {
    let mut connection = pool.get().expect("Failed to get DB connection");
    metric_reconciliation_issue::table
        .first(&mut connection)
        .expect("Failed to load the stored reconciliation issue")
}

/// Insert one inbound OPERAS ledger row directly.
///
/// `metric_operas_import/tests.rs` exports no fixture and this task's write
/// budget does not permit widening it, so the remote-event tests build their
/// own inbound evidence. `import_id` is deliberately omitted: the merged
/// inbound ledger records remote-event evidence before normalization, so a
/// canonical import is not needed here.
fn insert_operas_import(pool: &PgPool, remote_instance: &str, remote_event_id: &str) {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "INSERT INTO metric_operas_import \
             (remote_instance, remote_event_id, payload_hash, status) \
         VALUES ($1, $2, 'fixture-payload-hash', 'fixture-status')",
    )
    .bind::<diesel::sql_types::Text, _>(remote_instance)
    .bind::<diesel::sql_types::Text, _>(remote_event_id)
    .execute(&mut connection)
    .expect("Failed to insert the inbound OPERAS ledger fixture row");
}

#[test]
fn migration_seeds_no_reconciliation_issue_row() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_reconciliation_issue)"),
        0,
        "MET-WP1-11 must not seed any metric_reconciliation_issue row: this \
         slice classifies nothing and creates no issue at runtime"
    );
}

#[test]
fn a_complete_reconciliation_issue_round_trips_through_diesel() {
    let (_guard, pool) = setup_registry_db();
    let run_id = fixture_run(&pool);
    let (_fixture, record_id) = fixture_record(&pool, "fixture-identity-hash");
    let mut connection = pool.get().expect("Failed to get DB connection");

    // Fixture values only. None of these is a real issue type, severity,
    // remote event identifier or evidence payload.
    let issue_id = Uuid::new_v4();
    let resolved_at = Timestamp::parse_from_rfc3339("2026-03-04T07:08:09.101112Z")
        .expect("Failed to parse the fixture resolution time");
    let details = serde_json::json!({"fixture": "details", "observed": [1, 2]});

    diesel::insert_into(metric_reconciliation_issue::table)
        .values((
            metric_reconciliation_issue::issue_id.eq(issue_id),
            metric_reconciliation_issue::run_id.eq(run_id),
            metric_reconciliation_issue::issue_type.eq("fixture-issue-type"),
            metric_reconciliation_issue::severity.eq("fixture-severity"),
            metric_reconciliation_issue::record_id.eq(Some(record_id)),
            metric_reconciliation_issue::remote_event_id.eq(Some("fixture-remote-event-id")),
            metric_reconciliation_issue::details.eq(&details),
            metric_reconciliation_issue::resolved_at.eq(Some(resolved_at)),
        ))
        .execute(&mut connection)
        .expect("Failed to insert the complete reconciliation issue");

    let loaded: MetricReconciliationIssue = metric_reconciliation_issue::table
        .filter(metric_reconciliation_issue::issue_id.eq(issue_id))
        .first(&mut connection)
        .expect("Failed to load the complete reconciliation issue");
    assert_eq!(
        loaded,
        MetricReconciliationIssue {
            issue_id,
            run_id,
            issue_type: "fixture-issue-type".to_string(),
            severity: "fixture-severity".to_string(),
            record_id: Some(record_id),
            remote_event_id: Some("fixture-remote-event-id".to_string()),
            details,
            resolved_at: Some(resolved_at),
        },
        "every approved reconciliation-issue field must round-trip through the \
         manually maintained Diesel contract unchanged"
    );
}

#[test]
fn the_repository_standard_uuid_default_supplies_an_issue_identity() {
    let (_guard, pool) = setup_registry_db();
    let run_id = fixture_run(&pool).to_string();

    for _ in 0..2 {
        insert_issue_row(&pool, IssueRow::minimal(&run_id))
            .expect("an issue inserted without an explicit identity must be accepted");
    }

    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(DISTINCT issue_id) FROM metric_reconciliation_issue)"
        ),
        2,
        "the repository-standard UUID default must generate a distinct identity \
         per issue when the insert omits issue_id"
    );

    // No inferred issue identity: two issues of one type, on one run, with the
    // same absent record and remote event must both be storable, because
    // runtime deduplication and reopening semantics are not design-fixed.
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_reconciliation_issue)"),
        2,
        "no uniqueness over (run_id, issue_type, record_id, remote_event_id) \
         may be invented: WP9 owns deduplication and reopening"
    );
}

#[test]
fn an_issue_must_belong_to_an_existing_run() {
    let (_guard, pool) = setup_registry_db();
    let run_id = fixture_run(&pool).to_string();

    insert_issue_row(&pool, IssueRow::minimal(&run_id))
        .expect("an issue naming an existing run must be accepted");

    let orphan = Uuid::new_v4().to_string();
    let result = insert_issue_row(&pool, IssueRow::minimal(&orphan));
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "a run_id naming no reconciliation run must be rejected, got {result:?}"
    );

    let result = insert_issue_row(
        &pool,
        IssueRow {
            run_id: Column::Null,
            ..IssueRow::minimal(&run_id)
        },
    );
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::NotNullViolation,
                _
            ))
        ),
        "a NULL run_id must be rejected: every issue belongs to exactly one \
         run, got {result:?}"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_reconciliation_issue)"),
        1
    );
}

#[test]
fn deleting_a_referenced_run_is_restricted_and_does_not_cascade() {
    let (_guard, pool) = setup_registry_db();
    let run_id = fixture_run(&pool);
    insert_issue_row(&pool, IssueRow::minimal(&run_id.to_string()))
        .expect("Failed to insert the referencing issue");

    let result = delete_row(&pool, "metric_reconciliation_run", "run_id", run_id);
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "deleting a reconciliation run while its durable issue evidence exists \
         must fail rather than cascade, got {result:?}"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_reconciliation_issue)"),
        1,
        "the durable issue evidence must survive the refused deletion"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_reconciliation_run)"),
        1,
        "the referenced run must survive the refused deletion"
    );

    // Several issues may belong to one run: no uniqueness machinery exists on
    // the referencing side.
    insert_issue_row(
        &pool,
        IssueRow {
            issue_type: Column::Value("a-second-fixture-issue-type"),
            ..IssueRow::minimal(&run_id.to_string())
        },
    )
    .expect("a run must be able to carry several issues");
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_index i \
              JOIN pg_class c ON c.oid = i.indrelid \
              WHERE c.relname = 'metric_reconciliation_issue' AND i.indisunique \
                AND 'run_id' = ANY ( \
                    SELECT a.attname FROM pg_attribute a \
                    WHERE a.attrelid = c.oid AND a.attnum = ANY (i.indkey)))",
        ),
        0,
        "no unique index may cover run_id: one run carries many issues"
    );
}

#[test]
fn a_canonical_record_is_optional_referentially_enforced_and_non_cascading() {
    let (_guard, pool) = setup_registry_db();
    let run_id = fixture_run(&pool).to_string();
    let (_fixture, record_id) = fixture_record(&pool, "fixture-identity-hash");

    // A named canonical record must exist.
    insert_issue_row(
        &pool,
        IssueRow {
            record_id: Column::Value(&record_id.to_string()),
            ..IssueRow::minimal(&run_id)
        },
    )
    .expect("an issue naming an existing canonical record must be accepted");

    let orphan = Uuid::new_v4().to_string();
    let result = insert_issue_row(
        &pool,
        IssueRow {
            record_id: Column::Value(&orphan),
            ..IssueRow::minimal(&run_id)
        },
    );
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "a record_id naming no canonical record must be rejected, got {result:?}"
    );

    // Deleting a referenced canonical record must fail rather than erase the
    // evidence.
    let result = delete_row(&pool, "metric_record", "record_id", record_id);
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        ),
        "deleting a canonical record while durable reconciliation evidence \
         references it must fail rather than cascade, got {result:?}"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_reconciliation_issue)"),
        1,
        "the durable issue evidence must survive the refused deletion"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_record)"),
        1,
        "the referenced canonical record must survive the refused deletion"
    );
}

#[test]
fn a_recordless_issue_is_representable() {
    let (_guard, pool) = setup_registry_db();
    let run_id = fixture_run(&pool).to_string();

    // Several design-named issue categories exist before or without a canonical
    // record — unexpected remote records, unmapped measures and unresolved
    // works — so a NULL record_id is a first-class state, not a degraded one.
    insert_issue_row(&pool, IssueRow::minimal(&run_id))
        .expect("an issue omitting record_id must be accepted");
    assert_eq!(
        only_issue(&pool).record_id,
        None,
        "record_id must remain NULL until a later WP9 path names a canonical record"
    );

    insert_issue_row(
        &pool,
        IssueRow {
            record_id: Column::Null,
            issue_type: Column::Value("fixture-unmapped-measure"),
            ..IssueRow::minimal(&run_id)
        },
    )
    .expect("an explicit NULL record_id must be accepted");

    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_reconciliation_issue WHERE record_id IS NULL)"
        ),
        2
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_record)"),
        0,
        "no canonical record may be created automatically by storing recordless \
         reconciliation evidence"
    );
}

#[test]
fn arbitrary_nonblank_text_round_trips_in_issue_type_and_severity() {
    let (_guard, pool) = setup_registry_db();
    let run_id = fixture_run(&pool).to_string();
    let mut connection = pool.get().expect("Failed to get DB connection");

    // Deliberately mixed and deliberately not a vocabulary: accepting all of
    // these is the point. The approved design's prose examples are illustrative
    // rather than an exhaustive enum, and its operational mention of
    // high-severity alerts defines no closed severity domain, so neither column
    // may be turned into a vocabulary. The padded values must round-trip
    // byte-for-byte, proving nothing trims, normalizes or case-folds them.
    let values = [
        "fixture-value",
        "MISSING_EXPORT",
        "value divergence",
        "a",
        "  padded fixture value  ",
        "ünïcödé välüé",
        "CRITICAL",
        "0123456789abcdef",
    ];

    for (index, value) in values.into_iter().enumerate() {
        let type_issue_id = Uuid::new_v4();
        insert_issue_row(
            &pool,
            IssueRow {
                issue_id: Column::Value(&type_issue_id.to_string()),
                issue_type: Column::Value(value),
                ..IssueRow::minimal(&run_id)
            },
        )
        .unwrap_or_else(|error| {
            panic!("arbitrary nonblank issue_type {value:?} must be accepted: {error:?}")
        });

        let severity_issue_id = Uuid::new_v4();
        insert_issue_row(
            &pool,
            IssueRow {
                issue_id: Column::Value(&severity_issue_id.to_string()),
                severity: Column::Value(value),
                ..IssueRow::minimal(&run_id)
            },
        )
        .unwrap_or_else(|error| {
            panic!("arbitrary nonblank severity {value:?} must be accepted: {error:?}")
        });

        let stored_type: String = metric_reconciliation_issue::table
            .filter(metric_reconciliation_issue::issue_id.eq(type_issue_id))
            .select(metric_reconciliation_issue::issue_type)
            .first(&mut connection)
            .expect("Failed to load the stored issue_type");
        let stored_severity: String = metric_reconciliation_issue::table
            .filter(metric_reconciliation_issue::issue_id.eq(severity_issue_id))
            .select(metric_reconciliation_issue::severity)
            .first(&mut connection)
            .expect("Failed to load the stored severity");
        assert_eq!(
            (stored_type.as_str(), stored_severity.as_str()),
            (value, value),
            "variant {index} {value:?} must round-trip unchanged in both opaque \
             columns; storing it is not a claim that it is a recognised issue \
             class or severity level"
        );
    }
}

#[test]
fn blank_and_whitespace_only_text_is_rejected_in_every_required_column() {
    let (_guard, pool) = setup_registry_db();
    let run_id = fixture_run(&pool).to_string();

    for (index, blank) in ["", " ", "   ", "\t", "\n", " \t\n "]
        .into_iter()
        .enumerate()
    {
        let cases = [
            (
                "issue_type",
                IssueRow {
                    issue_type: Column::Value(blank),
                    ..IssueRow::minimal(&run_id)
                },
            ),
            (
                "severity",
                IssueRow {
                    severity: Column::Value(blank),
                    ..IssueRow::minimal(&run_id)
                },
            ),
            (
                "remote_event_id",
                IssueRow {
                    remote_event_id: Column::Value(blank),
                    ..IssueRow::minimal(&run_id)
                },
            ),
        ];
        for (column, row) in cases {
            let result = insert_issue_row(&pool, row);
            assert!(
                matches!(
                    result,
                    Err(DieselError::DatabaseError(
                        DatabaseErrorKind::CheckViolation,
                        _
                    ))
                ),
                "blank/whitespace-only {column} variant {index} ({blank:?}) must \
                 be rejected by the required-text CHECK, got {result:?}"
            );
        }
    }

    for column in ["issue_type", "severity"] {
        let row = match column {
            "issue_type" => IssueRow {
                issue_type: Column::Null,
                ..IssueRow::minimal(&run_id)
            },
            _ => IssueRow {
                severity: Column::Null,
                ..IssueRow::minimal(&run_id)
            },
        };
        let result = insert_issue_row(&pool, row);
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::NotNullViolation,
                    _
                ))
            ),
            "a NULL {column} must be rejected, got {result:?}"
        );
    }

    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_reconciliation_issue)"),
        0,
        "no rejected reconciliation issue may have been stored"
    );
}

#[test]
fn a_remote_event_is_optional_and_unconstrained_beyond_nonblank_text() {
    let (_guard, pool) = setup_registry_db();
    let run_id = fixture_run(&pool).to_string();

    // The row exists before, and often entirely without, remote evidence.
    insert_issue_row(&pool, IssueRow::minimal(&run_id))
        .expect("an issue omitting remote_event_id must be accepted");
    assert_eq!(
        only_issue(&pool).remote_event_id,
        None,
        "an omitted remote_event_id must remain NULL rather than acquire a default"
    );

    insert_issue_row(
        &pool,
        IssueRow {
            remote_event_id: Column::Null,
            ..IssueRow::minimal(&run_id)
        },
    )
    .expect("an explicit NULL remote_event_id must be accepted");

    // No syntax, length, UUID/URI or uniqueness rule constrains a supplied
    // value: the approved design fixes none.
    for value in [
        "fixture-remote-event-id",
        "https://example.invalid/events/1",
        "not a uri at all",
        "a",
        "  padded fixture value  ",
        "ünïcödé välüé",
    ] {
        insert_issue_row(
            &pool,
            IssueRow {
                remote_event_id: Column::Value(value),
                ..IssueRow::minimal(&run_id)
            },
        )
        .unwrap_or_else(|error| {
            panic!("arbitrary nonblank remote_event_id {value:?} must be accepted: {error:?}")
        });
    }

    // The same bare remote event identifier may repeat across issues: this
    // column carries no uniqueness at all.
    insert_issue_row(
        &pool,
        IssueRow {
            remote_event_id: Column::Value("fixture-remote-event-id"),
            ..IssueRow::minimal(&run_id)
        },
    )
    .expect("a repeated remote_event_id must be accepted on the issue ledger");
}

#[test]
fn a_remote_event_absent_from_the_inbound_ledger_remains_representable() {
    let (_guard, pool) = setup_registry_db();
    let run_id = fixture_run(&pool).to_string();

    // "Unexpected remote record" is a design-named issue category, and
    // reconciliation may also refer to outbound or legacy remote evidence, so
    // an issue must be storable for a remote event the inbound ledger has never
    // seen. A foreign key here would make that unrepresentable.
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_import)"),
        0,
        "the inbound ledger must be empty for this test to mean anything"
    );
    insert_issue_row(
        &pool,
        IssueRow {
            remote_event_id: Column::Value("an-event-no-inbound-ledger-row-mentions"),
            issue_type: Column::Value("fixture-unexpected-remote-record"),
            ..IssueRow::minimal(&run_id)
        },
    )
    .expect(
        "a remote event absent from metric_operas_import must remain \
         representable: no foreign key to the inbound ledger is authorized",
    );

    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_reconciliation_issue)"),
        1
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_operas_import)"),
        0,
        "no inbound-ledger row may be created automatically by storing remote \
         reconciliation evidence"
    );
}

#[test]
fn one_bare_remote_event_id_stays_representable_across_distinct_remote_instances() {
    let (_guard, pool) = setup_registry_db();
    let run_id = fixture_run(&pool).to_string();

    // MET-WP1-10 established canonical remote identity as the composite
    // (remote_instance, remote_event_id): the same bare event ID legitimately
    // exists on two distinct remote instances. The reconciliation shorthand
    // carries no remote_instance, so a single-column foreign key or a global
    // uniqueness rule would contradict that merged identity contract.
    insert_operas_import(
        &pool,
        "fixture-remote-instance-a",
        "a-shared-remote-event-id",
    );
    insert_operas_import(
        &pool,
        "fixture-remote-instance-b",
        "a-shared-remote-event-id",
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(DISTINCT remote_instance) FROM metric_operas_import \
              WHERE remote_event_id = 'a-shared-remote-event-id')"
        ),
        2,
        "the inbound ledger must genuinely hold one event ID on two instances"
    );

    for issue_type in [
        "fixture-issue-for-instance-a",
        "fixture-issue-for-instance-b",
    ] {
        insert_issue_row(
            &pool,
            IssueRow {
                remote_event_id: Column::Value("a-shared-remote-event-id"),
                issue_type: Column::Value(issue_type),
                ..IssueRow::minimal(&run_id)
            },
        )
        .expect(
            "one bare remote event ID must stay representable across distinct \
             remote instances without an invented reconciliation foreign key or \
             global uniqueness rule",
        );
    }

    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_reconciliation_issue \
              WHERE remote_event_id = 'a-shared-remote-event-id')"
        ),
        2
    );

    // Neither a foreign key nor a uniqueness rule may cover the column, and no
    // companion remote_instance column may exist to make one possible.
    for (name, definition) in foreign_keys(&pool, "metric_reconciliation_issue") {
        assert!(
            !definition.contains("remote_event_id"),
            "{name} must not reference remote_event_id: merged inbound identity \
             is the composite (remote_instance, remote_event_id) and a bare \
             event ID is deliberately not globally unique: {definition}"
        );
    }
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_index i \
              JOIN pg_class c ON c.oid = i.indrelid \
              WHERE c.relname = 'metric_reconciliation_issue' AND i.indisunique \
                AND 'remote_event_id' = ANY ( \
                    SELECT a.attname FROM pg_attribute a \
                    WHERE a.attrelid = c.oid AND a.attnum = ANY (i.indkey)))",
        ),
        0,
        "no unique index may cover remote_event_id"
    );
}

#[test]
fn details_defaults_to_an_empty_object_and_arbitrary_structure_round_trips() {
    let (_guard, pool) = setup_registry_db();
    let run_id_uuid = fixture_run(&pool);
    let run_id = run_id_uuid.to_string();
    let mut connection = pool.get().expect("Failed to get DB connection");

    insert_issue_row(&pool, IssueRow::minimal(&run_id))
        .expect("an issue omitting details must be accepted");
    assert_eq!(
        only_issue(&pool).details,
        serde_json::json!({}),
        "an omitted details must default to an empty object, so an issue exists \
         before its machine-readable evidence is attached"
    );

    // No required keys and no semantic interpretation of an empty object: the
    // approved design requires issues to be machine-readable but defines no
    // universal evidence schema.
    let payloads = [
        serde_json::json!({}),
        serde_json::json!({"expected": 12, "observed": 9}),
        serde_json::json!({"nested": {"remote": {"instances": ["a", "b"]}}}),
        serde_json::json!([{"not": "an object at all"}]),
        serde_json::json!("a bare JSON string"),
        serde_json::json!(null),
    ];
    for (index, details) in payloads.iter().enumerate() {
        let issue_id = Uuid::new_v4();
        diesel::insert_into(metric_reconciliation_issue::table)
            .values((
                metric_reconciliation_issue::issue_id.eq(issue_id),
                metric_reconciliation_issue::run_id.eq(run_id_uuid),
                metric_reconciliation_issue::issue_type.eq(format!("fixture-issue-type-{index}")),
                metric_reconciliation_issue::severity.eq("fixture-severity"),
                metric_reconciliation_issue::details.eq(details),
            ))
            .execute(&mut connection)
            .unwrap_or_else(|error| {
                panic!("arbitrary structured details {details} must be accepted: {error:?}")
            });

        let stored: serde_json::Value = metric_reconciliation_issue::table
            .filter(metric_reconciliation_issue::issue_id.eq(issue_id))
            .select(metric_reconciliation_issue::details)
            .first(&mut connection)
            .expect("Failed to load the stored details");
        assert_eq!(
            &stored, details,
            "{details} must round-trip unchanged; storing it is not a claim that \
             it is validated reconciliation evidence"
        );
    }

    let result = insert_issue_row(
        &pool,
        IssueRow {
            details: Column::Null,
            ..IssueRow::minimal(&run_id)
        },
    );
    assert!(
        matches!(
            result,
            Err(DieselError::DatabaseError(
                DatabaseErrorKind::NotNullViolation,
                _
            ))
        ),
        "an explicit NULL details must be rejected, got {result:?}"
    );
}

#[test]
fn resolved_at_is_nullable_and_defines_no_resolution_workflow() {
    let (_guard, pool) = setup_registry_db();
    let run_id = fixture_run(&pool).to_string();

    insert_issue_row(&pool, IssueRow::minimal(&run_id))
        .expect("an issue omitting resolved_at must be accepted");
    assert_eq!(
        only_issue(&pool).resolved_at,
        None,
        "an omitted resolved_at must remain NULL rather than acquire a default"
    );

    insert_issue_row(
        &pool,
        IssueRow {
            resolved_at: Column::Null,
            issue_type: Column::Value("fixture-explicit-null"),
            ..IssueRow::minimal(&run_id)
        },
    )
    .expect("an explicit NULL resolved_at must be accepted");

    // A resolution timestamp carries no companion state: this slice defines
    // neither what "resolved" means nor whether an issue may reopen, so nothing
    // constrains when it may be set.
    insert_issue_row(
        &pool,
        IssueRow {
            resolved_at: Column::Value("2026-03-04T07:08:09Z"),
            issue_type: Column::Value("fixture-resolved"),
            ..IssueRow::minimal(&run_id)
        },
    )
    .expect("a resolved_at with no companion resolution state must be accepted");

    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_reconciliation_issue WHERE resolved_at IS NULL)"
        ),
        2
    );
}

#[test]
fn metric_reconciliation_issue_has_exactly_the_approved_columns() {
    let (_guard, pool) = setup_registry_db();
    #[derive(diesel::QueryableByName)]
    struct ColumnRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        column_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        data_type: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        is_nullable: String,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    let columns: Vec<(String, String, String)> = sql_query(
        "SELECT column_name::text, data_type::text, is_nullable::text \
         FROM information_schema.columns \
         WHERE table_schema = 'public' AND table_name = 'metric_reconciliation_issue' \
         ORDER BY ordinal_position",
    )
    .load::<ColumnRow>(&mut connection)
    .expect("Failed to read the metric_reconciliation_issue columns")
    .into_iter()
    .map(|column| (column.column_name, column.data_type, column.is_nullable))
    .collect();
    let observed: Vec<(&str, &str, &str)> = columns
        .iter()
        .map(|(name, data_type, nullable)| (name.as_str(), data_type.as_str(), nullable.as_str()))
        .collect();
    assert_eq!(
        observed,
        vec![
            ("issue_id", "uuid", "NO"),
            ("run_id", "uuid", "NO"),
            ("issue_type", "text", "NO"),
            ("severity", "text", "NO"),
            ("record_id", "uuid", "YES"),
            ("remote_event_id", "text", "YES"),
            ("details", "jsonb", "NO"),
            ("resolved_at", "timestamp with time zone", "YES"),
        ],
        "metric_reconciliation_issue must carry exactly the eight approved \
         design fields, in the approved order and nullability, with no \
         remote_instance, operas_import_id, export_id, mapping_id or \
         record_revision_id column and no resolution-workflow column"
    );

    // Exactly two defaults: the repository-standard UUID identity and the empty
    // details object. In particular severity has none, because this slice
    // defines no default severity, and resolved_at has none, because the row
    // exists unresolved.
    #[derive(diesel::QueryableByName)]
    struct DefaultRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        column_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        column_default: String,
    }
    let defaults: Vec<(String, String)> = sql_query(
        "SELECT column_name::text, column_default::text \
         FROM information_schema.columns \
         WHERE table_schema = 'public' AND table_name = 'metric_reconciliation_issue' \
           AND column_default IS NOT NULL \
         ORDER BY ordinal_position",
    )
    .load::<DefaultRow>(&mut connection)
    .expect("Failed to read the metric_reconciliation_issue defaults")
    .into_iter()
    .map(|row| (row.column_name, row.column_default))
    .collect();
    assert_eq!(
        defaults,
        vec![
            ("issue_id".to_string(), "uuid_generate_v4()".to_string()),
            ("details".to_string(), "'{}'::jsonb".to_string()),
        ],
        "only issue_id and details may carry a default: issue_type and severity \
         must have none, because this slice defines no vocabulary, and \
         resolved_at must have none, because an issue exists unresolved"
    );

    // No invented relationship or workflow column was smuggled onto the issue.
    for column in DEFERRED_ISSUE_COLUMNS {
        assert_eq!(
            scalar_i64(
                &pool,
                &format!(
                    "(SELECT COUNT(*) FROM information_schema.columns \
                      WHERE table_schema = 'public' \
                        AND table_name = 'metric_reconciliation_issue' \
                        AND column_name = '{column}')"
                ),
            ),
            0,
            "MET-WP1-11 must not add the deferred reconciliation-issue column {column}"
        );
    }
}

#[test]
fn metric_reconciliation_issue_has_exactly_the_approved_checks() {
    let (_guard, pool) = setup_registry_db();
    // The set is exact and closed: two nonblank required-text rules and the
    // nullable form for the optional remote event. In particular there is no
    // CHECK enumerating issue types or severities and no cross-column rule
    // tying resolved_at, record_id or remote_event_id to anything.
    assert_eq!(
        check_constraint_names(&pool, "metric_reconciliation_issue"),
        vec![
            "metric_reconciliation_issue_issue_type_check",
            "metric_reconciliation_issue_remote_event_id_check",
            "metric_reconciliation_issue_severity_check",
        ],
        "metric_reconciliation_issue must carry exactly the three approved CHECKs"
    );

    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)]
        conname: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        definition: String,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    let definitions: Vec<(String, String)> = sql_query(
        "SELECT c.conname::text AS conname, pg_get_constraintdef(c.oid) AS definition \
         FROM pg_constraint c \
         WHERE c.conrelid = 'public.metric_reconciliation_issue'::regclass AND c.contype = 'c' \
         ORDER BY c.conname",
    )
    .load::<Row>(&mut connection)
    .expect("Failed to read the CHECK definitions")
    .into_iter()
    .map(|row| (row.conname, row.definition))
    .collect();
    for (name, definition) in &definitions {
        assert!(
            definition.contains("[^[:space:]]"),
            "{name} must be the existing Metrics required-text idiom and \
             nothing stronger — no vocabulary, syntax, length or uniqueness \
             rule: {definition}"
        );
        let nullable_escape = definition.contains("IS NULL");
        assert_eq!(
            nullable_escape,
            name.contains("remote_event_id"),
            "only the optional remote_event_id rule may carry a nullable \
             escape; the required-column rules need none: {definition}"
        );
    }
}

#[test]
fn metric_reconciliation_issue_has_exactly_the_authorized_non_cascading_foreign_keys() {
    let (_guard, pool) = setup_registry_db();
    let keys = foreign_keys(&pool, "metric_reconciliation_issue");
    assert_eq!(
        keys.iter().map(|key| key.0.as_str()).collect::<Vec<_>>(),
        vec![
            "metric_reconciliation_issue_record_id_fkey",
            "metric_reconciliation_issue_run_id_fkey",
        ],
        "metric_reconciliation_issue must carry exactly two foreign keys: no \
         key to metric_operas_import, metric_operas_export, \
         metric_operas_mapping or metric_record_revision may exist, because a \
         bare remote_event_id is not the merged composite inbound identity and \
         the approved shorthand names record_id rather than record_revision_id"
    );

    for (name, definition) in &keys {
        assert!(
            !definition.contains("ON DELETE"),
            "{name} must stay non-cascading and use the default restricting \
             behaviour, so durable reconciliation evidence cannot be erased \
             through parent deletion: {definition}"
        );
    }
    let record_key = &keys[0].1;
    assert!(
        record_key.contains("(record_id)") && record_key.contains("metric_record(record_id)"),
        "the record key must reference the MET-WP1-04 canonical record \
         identity: {record_key}"
    );
    let run_key = &keys[1].1;
    assert!(
        run_key.contains("(run_id)") && run_key.contains("metric_reconciliation_run(run_id)"),
        "the run key must reference the reconciliation run identity: {run_key}"
    );
}

#[test]
fn metric_reconciliation_issue_has_exactly_the_required_indexes() {
    let (_guard, pool) = setup_registry_db();
    assert_eq!(
        index_names(&pool, "metric_reconciliation_issue"),
        vec!["metric_reconciliation_issue_pkey"],
        "metric_reconciliation_issue must carry exactly its primary-key index; \
         no run_id, record_id, remote_event_id, issue_type, severity or \
         resolved_at index may exist. PostgreSQL needs no child-side \
         referencing index to enforce these foreign keys, the merged \
         metric_import_error (import_id) precedent likewise carries none, and \
         WP9 may add one only from an actual access pattern with query-plan \
         evidence"
    );
    let primary_key = index_definition(
        &pool,
        "metric_reconciliation_issue",
        "metric_reconciliation_issue_pkey",
    );
    assert!(
        primary_key.contains("UNIQUE") && primary_key.contains("(issue_id)"),
        "the primary key must be the surrogate issue identity: {primary_key}"
    );
}

#[test]
fn no_reconciliation_issue_or_run_is_created_automatically() {
    let (_guard, pool) = setup_registry_db();

    // Storing predecessor Metrics evidence must not manufacture reconciliation
    // state: this slice has no runtime, so nothing observes these writes.
    let (_fixture, _record_id) = fixture_record(&pool, "fixture-identity-hash");
    insert_operas_import(&pool, "fixture-remote-instance", "fixture-remote-event-id");
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_reconciliation_run)"),
        0,
        "no reconciliation run may be created automatically"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_reconciliation_issue)"),
        0,
        "no reconciliation issue may be created automatically"
    );

    // Nor does creating a run manufacture issues for it.
    insert_run_row(&pool, RunRow::minimal()).expect("Failed to insert the fixture run");
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_reconciliation_issue)"),
        0,
        "creating a reconciliation run must not create issues: classification \
         is WP9 runtime behaviour"
    );
}
