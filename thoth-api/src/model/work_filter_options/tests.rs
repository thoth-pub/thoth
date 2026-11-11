//! `MET-WP10-FILTER-API-01` unit evidence for the `workFilterOptions` read:
//! validation order and messages, the one-per-request claim, the slot guard,
//! the sorting helpers, the error contract, the failure classification and
//! the log decision. None of these tests needs a database, and none touches
//! the process-wide slot counter.
//!
//! The database and schema evidence (T1 to T14) lives in
//! `crate::graphql::work_filter_options_tests`.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};

use diesel::result::{DatabaseErrorKind, Error as DieselError};
use uuid::Uuid;

use super::*;
use crate::model::tests::db::failing_pool;

fn ids(count: usize) -> Vec<Uuid> {
    (0..count).map(|_| Uuid::new_v4()).collect()
}

fn uuid(text: &str) -> Uuid {
    Uuid::parse_str(text).expect("a UUID")
}

// ==========================================================================
// Validation (section 3.4)
// ==========================================================================

#[test]
fn validation_fails_on_the_first_rule_broken_with_its_message() {
    assert_eq!(
        validate(&[]),
        Err(WorkFilterOptionsError::Invalid(EMPTY_MESSAGE))
    );
    let one = Uuid::new_v4();
    assert_eq!(
        validate(&[one, Uuid::new_v4(), one]),
        Err(WorkFilterOptionsError::Invalid(DUPLICATE_MESSAGE))
    );
    assert_eq!(
        validate(&ids(101)),
        Err(WorkFilterOptionsError::Invalid(TOO_MANY_MESSAGE))
    );
    // 101 IDs with a duplicate: the duplicate wins.
    let mut duplicated = ids(100);
    duplicated.push(duplicated[42]);
    assert_eq!(duplicated.len(), 101);
    assert_eq!(
        validate(&duplicated),
        Err(WorkFilterOptionsError::Invalid(DUPLICATE_MESSAGE))
    );
    // The bound itself and a single ID are accepted.
    assert_eq!(validate(&ids(WORK_FILTER_OPTIONS_MAX_PUBLISHERS)), Ok(()));
    assert_eq!(validate(&ids(1)), Ok(()));
}

#[test]
fn the_validation_messages_and_type_are_the_approved_text() {
    for (error, message) in [
        (
            WorkFilterOptionsError::Invalid(EMPTY_MESSAGE),
            "publisherIds must contain at least one publisher.",
        ),
        (
            WorkFilterOptionsError::Invalid(DUPLICATE_MESSAGE),
            "publisherIds must not contain duplicate values.",
        ),
        (
            WorkFilterOptionsError::Invalid(TOO_MANY_MESSAGE),
            "publisherIds may contain at most 100 publishers.",
        ),
    ] {
        assert_eq!(error.code(), "WORK_FILTER_OPTIONS_INVALID");
        assert_eq!(error.message(), message);
    }
}

// ==========================================================================
// The public entry and its claim (section 3.8)
// ==========================================================================

#[test]
fn a_second_call_on_one_claim_counter_is_refused_before_validation() {
    // A pool that cannot connect: nothing here may reach it.
    let pool = failing_pool();
    let claims = AtomicUsize::new(0);
    // The first call claims, then fails validation without a slot or a
    // checkout.
    assert_eq!(
        work_filter_options(&pool, &claims, &[]),
        Err(WorkFilterOptionsError::Invalid(EMPTY_MESSAGE))
    );
    // An empty list on the second call is still one-per-request.
    assert_eq!(
        work_filter_options(&pool, &claims, &[]),
        Err(WorkFilterOptionsError::OnePerRequest)
    );
    // So is a valid list, and a duplicate one.
    let one = Uuid::new_v4();
    assert_eq!(
        work_filter_options(&pool, &claims, &[one]),
        Err(WorkFilterOptionsError::OnePerRequest)
    );
    assert_eq!(
        work_filter_options(&pool, &claims, &[one, one]),
        Err(WorkFilterOptionsError::OnePerRequest)
    );
    assert_eq!(claims.load(Ordering::SeqCst), 4);
}

#[test]
fn the_production_controls_are_the_approved_bounds() {
    let controls = WorkFilterReadControls::production();
    assert_eq!(controls.max_entries, 20_000);
    assert_eq!(
        controls.statement_timeout_sql,
        "SET LOCAL statement_timeout = '10s'"
    );
    assert_eq!(controls.checkout_timeout, Duration::from_secs(1));
    assert!(std::ptr::eq(controls.slots, &WORK_FILTER_OPTIONS_SLOTS));
    assert!(controls.test_statement.is_none());
    assert_eq!(WORK_FILTER_OPTIONS_MAX_PUBLISHERS, 100);
    assert_eq!(WORK_FILTER_OPTIONS_MAX_CONCURRENT_READS, 2);
    assert_eq!(
        WORK_FILTER_OPTIONS_CUSTOM_PLANS_SQL,
        "SET LOCAL plan_cache_mode = force_custom_plan"
    );
}

// ==========================================================================
// The slot guard (section 3.7)
// ==========================================================================

#[test]
fn at_most_two_slots_are_held_and_a_third_fails_at_once() {
    let slots = AtomicUsize::new(0);
    let first = try_take_slot(&slots).expect("a first slot");
    let second = try_take_slot(&slots).expect("a second slot");
    assert_eq!(slots.load(Ordering::SeqCst), 2);
    assert!(try_take_slot(&slots).is_none());
    assert_eq!(slots.load(Ordering::SeqCst), 2, "a refusal takes nothing");
    drop(first);
    assert_eq!(slots.load(Ordering::SeqCst), 1);
    let third = try_take_slot(&slots).expect("a freed slot");
    assert!(try_take_slot(&slots).is_none());
    drop((second, third));
    assert_eq!(slots.load(Ordering::SeqCst), 0);
}

#[test]
fn a_guard_dropped_while_unwinding_releases_its_slot() {
    let slots = AtomicUsize::new(0);
    let kept = try_take_slot(&slots).expect("a slot");
    let panicked = catch_unwind(AssertUnwindSafe(|| {
        let _held = try_take_slot(&slots).expect("the second slot");
        assert_eq!(slots.load(Ordering::SeqCst), 2);
        panic!("injected");
    }));
    assert!(panicked.is_err());
    assert_eq!(slots.load(Ordering::SeqCst), 1);
    drop(kept);
    assert_eq!(slots.load(Ordering::SeqCst), 0);
}

#[test]
fn a_busy_builder_call_takes_no_checkout_and_leaves_the_counter_unchanged() {
    // Both slots of a local counter held: the builder refuses before the
    // checkout (the failing pool would otherwise take its own timeout).
    let slots = AtomicUsize::new(0);
    let held = (try_take_slot(&slots), try_take_slot(&slots));
    let pool = failing_pool();
    let controls = WorkFilterReadControls {
        slots: &slots,
        ..WorkFilterReadControls::production()
    };
    assert_eq!(
        work_filter_options_with(&pool, &[Uuid::new_v4()], &controls),
        Err(WorkFilterOptionsError::Busy)
    );
    assert_eq!(slots.load(Ordering::SeqCst), 2);
    // Validation comes first: an invalid list is invalid even when busy.
    assert_eq!(
        work_filter_options_with(&pool, &[], &controls),
        Err(WorkFilterOptionsError::Invalid(EMPTY_MESSAGE))
    );
    drop(held);
    assert_eq!(slots.load(Ordering::SeqCst), 0);
}

// ==========================================================================
// Sorting (section 3.3)
// ==========================================================================

fn publisher(name: &str, id: Uuid) -> WorkFilterPublisher {
    WorkFilterPublisher {
        publisher_id: id,
        publisher_name: name.to_string(),
    }
}

#[test]
fn names_sort_in_byte_order_then_by_id() {
    let low = uuid("0f000000-0000-4000-8000-000000000000");
    let high = uuid("10000000-0000-4000-8000-000000000000");
    let mut items = vec![
        publisher("aa", Uuid::new_v4()),
        publisher("same", high),
        publisher("a-b", Uuid::new_v4()),
        publisher("\u{e9}t\u{e9}", Uuid::new_v4()),
        publisher("Ab", Uuid::new_v4()),
        publisher("same", low),
        publisher("Z", Uuid::new_v4()),
    ];
    sort_by_name_then_id(&mut items, |p| (p.publisher_name.as_str(), p.publisher_id));
    let names: Vec<&str> = items.iter().map(|p| p.publisher_name.as_str()).collect();
    // Byte order: upper case before lower case, '-' (0x2d) before 'a'
    // (0x61), and a non-ASCII letter after every ASCII one.
    assert_eq!(
        names,
        ["Ab", "Z", "a-b", "aa", "same", "same", "\u{e9}t\u{e9}"]
    );
    assert_eq!(items[4].publisher_id, low);
    assert_eq!(items[5].publisher_id, high);
}

#[test]
fn uuid_order_is_canonical_lower_case_text_order() {
    let mut ids: Vec<Uuid> = (0..64).map(|_| Uuid::new_v4()).collect();
    ids.push(uuid("ffffffff-0000-4000-8000-000000000000"));
    ids.push(uuid("00000000-ffff-4000-8000-000000000000"));
    let mut by_value = ids.clone();
    by_value.sort();
    let mut by_text = ids;
    by_text.sort_by_key(|id| id.hyphenated().to_string());
    assert_eq!(by_value, by_text);
    assert!(by_text
        .iter()
        .all(|id| id.to_string() == id.to_string().to_lowercase()));
}

#[test]
fn enum_values_sort_by_label_not_declaration_order() {
    // Declaration order is BookChapter, Monograph, EditedBook, Textbook,
    // JournalIssue, BookSet.
    let sorted = sort_by_label(vec![
        ("monograph".to_string(), WorkType::Monograph),
        ("edited-book".to_string(), WorkType::EditedBook),
        ("book-set".to_string(), WorkType::BookSet),
        ("journal-issue".to_string(), WorkType::JournalIssue),
        ("book-chapter".to_string(), WorkType::BookChapter),
        ("textbook".to_string(), WorkType::Textbook),
    ]);
    assert_eq!(
        sorted,
        [
            WorkType::BookChapter,
            WorkType::BookSet,
            WorkType::EditedBook,
            WorkType::JournalIssue,
            WorkType::Monograph,
            WorkType::Textbook,
        ]
    );
    let languages = sort_by_label(vec![
        ("fre".to_string(), LanguageCode::Fre),
        ("eng".to_string(), LanguageCode::Eng),
        ("ger".to_string(), LanguageCode::Ger),
    ]);
    assert_eq!(
        languages,
        [LanguageCode::Eng, LanguageCode::Fre, LanguageCode::Ger]
    );
}

// ==========================================================================
// Assembly: unknown IDs, bounds, malformed rows (sections 3.6 and 3.5)
// ==========================================================================

fn row(kind: &str) -> OptionRow {
    OptionRow {
        kind: kind.to_string(),
        id: None,
        name: None,
        series_type: None,
        issn_print: None,
        issn_digital: None,
        ror: None,
        work_type: None,
        language_code: None,
        label: None,
    }
}

fn named(kind: &str, name: &str) -> OptionRow {
    OptionRow {
        id: Some(Uuid::new_v4()),
        name: Some(name.to_string()),
        ..row(kind)
    }
}

fn work_type(label: &str, value: WorkType) -> OptionRow {
    OptionRow {
        work_type: Some(value),
        label: Some(label.to_string()),
        ..row("workTypes")
    }
}

#[test]
fn unknown_ids_win_over_a_list_bound_and_are_sorted() {
    let high = uuid("f0000000-0000-4000-8000-000000000000");
    let low = uuid("0a000000-0000-4000-8000-000000000000");
    let rows = vec![
        OptionRow {
            id: Some(high),
            ..row("unknown")
        },
        named("publishers", "P"),
        named("publishers", "Q"),
        OptionRow {
            id: Some(low),
            ..row("unknown")
        },
    ];
    let failure = assemble(rows, 1).expect_err("unknown");
    assert_eq!(
        failure.error,
        WorkFilterOptionsError::UnknownPublisher(vec![low, high])
    );
    assert_eq!(failure.cause, None);
    assert_eq!(
        failure.error.message(),
        "publisherIds contains unknown publisher(s): \
         0a000000-0000-4000-8000-000000000000, f0000000-0000-4000-8000-000000000000."
    );
}

#[test]
fn the_first_list_over_the_bound_in_field_order_is_named() {
    // workTypes and fundingInstitutions over, series at the bound.
    let rows = vec![
        named("fundingInstitutions", "F1"),
        named("fundingInstitutions", "F2"),
        named("fundingInstitutions", "F3"),
        work_type("monograph", WorkType::Monograph),
        work_type("textbook", WorkType::Textbook),
        work_type("book-set", WorkType::BookSet),
        OptionRow {
            series_type: Some(SeriesType::Journal),
            ..named("series", "S1")
        },
        OptionRow {
            series_type: Some(SeriesType::Journal),
            ..named("series", "S2")
        },
    ];
    let failure = assemble(rows, 2).expect_err("over");
    assert_eq!(
        failure.error,
        WorkFilterOptionsError::LimitExceeded {
            field: "workTypes",
            bound: 2
        }
    );
    assert_eq!(
        failure.error.message(),
        "workFilterOptions.workTypes exceeds 2 entries."
    );
    assert_eq!(failure.error.code(), "WORK_FILTER_OPTIONS_LIMIT_EXCEEDED");
}

#[test]
fn a_row_this_module_cannot_interpret_is_unavailable_never_dropped() {
    for rows in [
        vec![row("series")],
        vec![named("publishers", "P"), row("somethingElse")],
        vec![OptionRow {
            label: Some("eng".to_string()),
            ..row("languageCodes")
        }],
        vec![row("unknown")],
    ] {
        let failure = assemble(rows, 10).expect_err("unexpected");
        assert_eq!(failure.error, WorkFilterOptionsError::Unavailable);
        assert_eq!(failure.cause, Some(Cause::Deserialize));
    }
}

// ==========================================================================
// The error contract and failure classification (section 7.2)
// ==========================================================================

#[test]
fn every_error_has_its_fixed_type_and_message() {
    let unknown = uuid("0a000000-0000-4000-8000-00000000000b");
    for (error, code, message) in [
        (
            WorkFilterOptionsError::OnePerRequest,
            "WORK_FILTER_OPTIONS_ONE_PER_REQUEST",
            "Only one workFilterOptions field may be requested at a time.".to_string(),
        ),
        (
            WorkFilterOptionsError::UnknownPublisher(vec![unknown]),
            "WORK_FILTER_OPTIONS_UNKNOWN_PUBLISHER",
            format!("publisherIds contains unknown publisher(s): {unknown}."),
        ),
        (
            WorkFilterOptionsError::LimitExceeded {
                field: "series",
                bound: 20_000,
            },
            "WORK_FILTER_OPTIONS_LIMIT_EXCEEDED",
            "workFilterOptions.series exceeds 20000 entries.".to_string(),
        ),
        (
            WorkFilterOptionsError::Busy,
            "WORK_FILTER_OPTIONS_BUSY",
            "workFilterOptions is busy; try again shortly.".to_string(),
        ),
        (
            WorkFilterOptionsError::Timeout,
            "WORK_FILTER_OPTIONS_TIMEOUT",
            "workFilterOptions timed out; try again shortly.".to_string(),
        ),
        (
            WorkFilterOptionsError::Unavailable,
            "WORK_FILTER_OPTIONS_UNAVAILABLE",
            "workFilterOptions could not be completed; try again shortly.".to_string(),
        ),
    ] {
        assert_eq!(error.code(), code);
        assert_eq!(error.message(), message);
        let field_error = error.into_field_error();
        assert_eq!(field_error.message(), message);
        assert_eq!(
            field_error.extensions(),
            &graphql_value!({ "type": code }),
            "{code}"
        );
    }
}

fn database_error(text: &str) -> DieselError {
    DieselError::DatabaseError(DatabaseErrorKind::Unknown, Box::new(text.to_string()))
}

#[test]
fn only_the_exact_timeout_text_is_a_timeout() {
    assert_eq!(
        Failure::from(database_error(
            "canceling statement due to statement timeout"
        )),
        Failure::retryable(WorkFilterOptionsError::Timeout, Cause::Timeout)
    );
    for other in [
        "division by zero",
        "canceling statement due to user request",
        "canceling statement due to lock timeout",
        "Canceling statement due to statement timeout",
        "",
    ] {
        assert_eq!(
            Failure::from(database_error(other)),
            Failure::retryable(WorkFilterOptionsError::Unavailable, Cause::Database),
            "{other}"
        );
    }
    assert_eq!(
        Failure::from(DieselError::DeserializationError("bad label".into())),
        Failure::retryable(WorkFilterOptionsError::Unavailable, Cause::Deserialize)
    );
    assert_eq!(
        Failure::from(DieselError::NotFound),
        Failure::retryable(WorkFilterOptionsError::Unavailable, Cause::Database)
    );
}

#[test]
fn each_retryable_failure_logs_one_line_at_its_level_and_nothing_else_logs() {
    for (failure, level, line) in [
        (
            Failure::retryable(WorkFilterOptionsError::Busy, Cause::NoSlot),
            log::Level::Debug,
            "workFilterOptions failed: type=WORK_FILTER_OPTIONS_BUSY cause=no-slot",
        ),
        (
            Failure::retryable(WorkFilterOptionsError::Busy, Cause::Checkout),
            log::Level::Warn,
            "workFilterOptions failed: type=WORK_FILTER_OPTIONS_BUSY cause=checkout",
        ),
        (
            Failure::retryable(WorkFilterOptionsError::Timeout, Cause::Timeout),
            log::Level::Warn,
            "workFilterOptions failed: type=WORK_FILTER_OPTIONS_TIMEOUT cause=timeout",
        ),
        (
            Failure::retryable(WorkFilterOptionsError::Unavailable, Cause::Deserialize),
            log::Level::Error,
            "workFilterOptions failed: type=WORK_FILTER_OPTIONS_UNAVAILABLE cause=deserialize",
        ),
        (
            Failure::retryable(WorkFilterOptionsError::Unavailable, Cause::Database),
            log::Level::Error,
            "workFilterOptions failed: type=WORK_FILTER_OPTIONS_UNAVAILABLE cause=database",
        ),
    ] {
        assert_eq!(log_line(&failure), Some((level, line.to_string())));
    }
    for error in [
        WorkFilterOptionsError::OnePerRequest,
        WorkFilterOptionsError::Invalid(EMPTY_MESSAGE),
        WorkFilterOptionsError::UnknownPublisher(vec![Uuid::new_v4()]),
        WorkFilterOptionsError::LimitExceeded {
            field: "series",
            bound: 1,
        },
    ] {
        assert_eq!(log_line(&Failure::input(error)), None);
    }
}
