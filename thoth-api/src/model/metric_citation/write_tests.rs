//! `CITATIONS-01` slice C2: the citation write path against a real database.
//!
//! Every test asserts persisted state. The fixture is one enabled
//! `openalex-citations` DRIVER source with an `EMPTY`-configured account for an
//! `OBELISK` publisher (which holds `METRICS_COLLECT`), six DOI works of that
//! publisher, a withdrawn work, a work without a DOI and one work of another
//! publisher.

use std::sync::Arc;

use chrono::NaiveDate;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::{sql_query, RunQueryDsl};
use uuid::Uuid;

use super::write::*;
use super::MetricCitationRefreshStatus as Status;
use crate::db::PgPool;
use crate::model::metric_ingestion_lifecycle::{
    claim_metric_source_units, ClaimMetricSourceUnitsInput, MetricLifecycleError as E,
    MetricSourceUnitClaim,
};
use crate::model::metric_platform::tests::{scalar_i64, setup_registry_db};
use crate::model::tests::db::{test_db_url, TestDbGuard};

pub(crate) const SOURCE: &str = "openalex-citations";
pub(crate) const ACCOUNT: &str = "openalex-citations-obp";

pub(crate) struct Fixture {
    pub pool: Arc<PgPool>,
    pub source_id: Uuid,
    pub publisher_id: Uuid,
    pub works: Vec<Uuid>,
    pub foreign_work: Uuid,
    pub withdrawn_work: Uuid,
}

pub(crate) fn exec(pool: &PgPool, sql: &str) {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(sql)
        .execute(&mut connection)
        .unwrap_or_else(|error| panic!("{sql}: {error}"));
}

pub(crate) fn setup() -> (TestDbGuard, Fixture) {
    let (guard, _registry_pool) = setup_registry_db();
    let pool = Arc::new(
        diesel::r2d2::Pool::builder()
            .max_size(8)
            .build(ConnectionManager::<PgConnection>::new(test_db_url()))
            .expect("Failed to create a citation test pool"),
    );
    let source_id = Uuid::new_v4();
    let platform_id = Uuid::new_v4();
    let publisher_id = Uuid::new_v4();
    let other_publisher = Uuid::new_v4();
    let imprint_id = Uuid::new_v4();
    let other_imprint = Uuid::new_v4();
    exec(&pool, &format!("INSERT INTO metric_source (source_id, code, acquisition_type, driver_key, enabled) VALUES ('{source_id}', '{SOURCE}', 'DRIVER', 'openalex-citations', TRUE)"));
    exec(&pool, &format!("INSERT INTO metric_platform (platform_id, code, display_name, ownership_class, enabled) VALUES ('{platform_id}', 'openalex', 'OpenAlex', 'THOTH_MANAGED', TRUE)"));
    exec(&pool, &format!("INSERT INTO publisher (publisher_id, publisher_name, subscription_package) VALUES ('{publisher_id}', 'OBP', 'OBELISK')"));
    exec(&pool, &format!("INSERT INTO publisher (publisher_id, publisher_name, subscription_package) VALUES ('{other_publisher}', 'Other', 'OBELISK')"));
    exec(&pool, &format!("INSERT INTO metric_source_account (source_account_id, code, source_id, platform_id, external_key, expected_publisher_id, configuration, enabled) VALUES ('{}', '{ACCOUNT}', '{source_id}', '{platform_id}', 'obp', '{publisher_id}', '{{}}'::jsonb, TRUE)", Uuid::new_v4()));
    exec(&pool, &format!("INSERT INTO imprint (imprint_id, publisher_id, imprint_name) VALUES ('{imprint_id}', '{publisher_id}', 'Imprint')"));
    exec(&pool, &format!("INSERT INTO imprint (imprint_id, publisher_id, imprint_name) VALUES ('{other_imprint}', '{other_publisher}', 'Other imprint')"));
    let work = |imprint: Uuid, status: &str, doi: Option<String>| {
        let id = Uuid::new_v4();
        let doi = doi.map_or("NULL".to_string(), |d| format!("'{d}'"));
        exec(&pool, &format!("INSERT INTO work (work_id, work_type, work_status, imprint_id, edition, doi) VALUES ('{id}', 'monograph', '{status}', '{imprint}', 1, {doi})"));
        id
    };
    let works: Vec<Uuid> = (1..=6)
        .map(|n| {
            work(
                imprint_id,
                "forthcoming",
                Some(format!("https://doi.org/10.5555/OBP.{n}")),
            )
        })
        .collect();
    let withdrawn_work = work(
        imprint_id,
        "cancelled",
        Some("https://doi.org/10.5555/obp.w".into()),
    );
    work(imprint_id, "forthcoming", None);
    let foreign_work = work(
        other_imprint,
        "forthcoming",
        Some("https://doi.org/10.5555/other.1".into()),
    );
    (
        guard,
        Fixture {
            pool,
            source_id,
            publisher_id,
            works,
            foreign_work,
            withdrawn_work,
        },
    )
}

pub(crate) fn claim(f: &Fixture) -> MetricSourceUnitClaim {
    let mut claims = claim_metric_source_units(
        &f.pool,
        &ClaimMetricSourceUnitsInput {
            source_code: SOURCE.into(),
            limit: Some(5),
            lease_seconds: Some(600),
        },
    )
    .expect("claim");
    assert_eq!(claims.len(), 1, "one claimable unit");
    claims.remove(0)
}

fn begin(f: &Fixture, token: Uuid) -> Result<CitationRefresh, E> {
    begin_citation_refresh(
        &f.pool,
        &BeginCitationRefreshInput {
            source_account_code: ACCOUNT.into(),
            lease_token: token,
        },
    )
}

fn link(id: &str, doi: Option<&str>) -> CitationLinkInput {
    CitationLinkInput {
        citing_external_id: id.into(),
        citing_doi: doi.map(Into::into),
        citing_title: Some(format!("Title of {id}")),
        citing_type: Some("article".into()),
        citing_publication_year: Some(2024),
        citing_publication_date: NaiveDate::from_ymd_opt(2024, 3, 9),
        citing_venue: Some("A Journal".into()),
        citing_author_names: Some(vec!["Ada Lovelace".into()]),
        citing_authors_total: Some(1),
        citing_url: Some(format!("https://example.org/{id}")),
    }
}

fn matched(
    work: Uuid,
    links: Vec<CitationLinkInput>,
    complete: bool,
    total: i32,
) -> CitationWorkEntryInput {
    CitationWorkEntryInput {
        work_id: work,
        match_status: CitationLookupStatus::Matched,
        external_work_id: Some(format!("W{work}")),
        reported_count: Some(total),
        links,
        enumeration_complete: complete,
        enumerated_total: complete.then_some(total),
    }
}

fn plain(work: Uuid, status: CitationLookupStatus) -> CitationWorkEntryInput {
    CitationWorkEntryInput {
        work_id: work,
        match_status: status,
        external_work_id: None,
        reported_count: None,
        links: Vec::new(),
        enumeration_complete: false,
        enumerated_total: None,
    }
}

fn submit(
    f: &Fixture,
    token: Uuid,
    refresh: Uuid,
    key: &str,
    works: Vec<CitationWorkEntryInput>,
) -> Result<CitationBatchResult, E> {
    submit_citation_batch(
        &f.pool,
        &SubmitCitationBatchInput {
            source_account_code: ACCOUNT.into(),
            lease_token: token,
            refresh_id: refresh,
            batch_key: key.into(),
            works,
        },
    )
}

fn complete(f: &Fixture, token: Uuid, refresh: Uuid) -> Result<CitationRefresh, E> {
    complete_citation_refresh(
        &f.pool,
        &CompleteCitationRefreshInput {
            source_account_code: ACCOUNT.into(),
            lease_token: token,
            refresh_id: refresh,
        },
    )
}

fn queue(
    f: &Fixture,
    token: Uuid,
    refresh: Uuid,
    limit: Option<i32>,
    after: Option<String>,
) -> Result<CitationRefreshWorksPage, E> {
    citation_refresh_works(
        &f.pool,
        &CitationRefreshWorksInput {
            source_account_code: ACCOUNT.into(),
            lease_token: token,
            refresh_id: refresh,
            limit,
            after,
        },
    )
}

fn count(f: &Fixture, sql: &str) -> i64 {
    scalar_i64(&f.pool, &format!("(({sql}))::bigint"))
}

fn links_of(f: &Fixture, work: Uuid, status: &str) -> i64 {
    count(
        f,
        &format!("SELECT count(*) FROM metric_citation_link WHERE cited_work_id = '{work}' AND status = '{status}'"),
    )
}

/// A full refresh of one work: begin, one batch, complete. Returns the result.
fn refresh_once(f: &Fixture, work: Uuid, ids: &[&str]) -> CitationRefresh {
    let claim = claim(f);
    let refresh = begin(f, claim.lease_token).unwrap();
    let links: Vec<CitationLinkInput> = ids.iter().map(|id| link(id, None)).collect();
    submit(
        f,
        claim.lease_token,
        refresh.refresh_id,
        "b1",
        vec![matched(work, links, true, ids.len() as i32)],
    )
    .unwrap();
    complete(f, claim.lease_token, refresh.refresh_id).unwrap()
}

fn expire_lease(f: &Fixture) {
    exec(&f.pool, "UPDATE metric_source_checkpoint SET lease_expires_at = transaction_timestamp() - interval '1 minute'");
}

#[test]
fn begin_is_idempotent_per_claim_and_a_later_claim_fails_the_stale_refresh() {
    let (_guard, f) = setup();
    let first = claim(&f);
    let refresh = begin(&f, first.lease_token).unwrap();
    assert_eq!(refresh.status, Status::Running);
    assert_eq!(
        begin(&f, first.lease_token).unwrap().refresh_id,
        refresh.refresh_id
    );
    assert_eq!(count(&f, "SELECT count(*) FROM metric_citation_refresh"), 1);
    assert!(
        count(
            &f,
            "SELECT count(*) FROM metric_source_checkpoint WHERE last_discovered_at IS NOT NULL"
        ) == 1
    );

    // The lease expires mid-refresh; the next claim supersedes it.
    expire_lease(&f);
    let second = claim(&f);
    assert_ne!(second.lease_token, first.lease_token);
    assert!(matches!(
        begin(&f, first.lease_token),
        Err(E::StaleSourceClaim)
    ));
    let next = begin(&f, second.lease_token).unwrap();
    assert_ne!(next.refresh_id, refresh.refresh_id);
    assert_eq!(
        count(&f, &format!("SELECT count(*) FROM metric_citation_refresh WHERE refresh_id = '{}' AND status = 'FAILED' AND completed_at IS NOT NULL", refresh.refresh_id)),
        1
    );
    assert_eq!(
        count(
            &f,
            "SELECT count(*) FROM metric_citation_refresh WHERE status = 'RUNNING'"
        ),
        1
    );
    // The old refresh and old token are refused by every operation.
    for result in [
        submit(
            &f,
            first.lease_token,
            refresh.refresh_id,
            "x",
            vec![plain(f.works[0], CitationLookupStatus::NotFound)],
        )
        .map(|_| ()),
        complete(&f, first.lease_token, refresh.refresh_id).map(|_| ()),
        queue(&f, first.lease_token, refresh.refresh_id, None, None).map(|_| ()),
    ] {
        assert!(matches!(result, Err(E::StaleSourceClaim)));
    }
    assert!(matches!(
        submit(
            &f,
            second.lease_token,
            refresh.refresh_id,
            "x",
            vec![plain(f.works[0], CitationLookupStatus::NotFound)]
        ),
        Err(E::CitationRefreshNotRunning)
    ));
}

#[test]
fn only_the_citation_driver_with_the_collect_capability_is_served() {
    let (_guard, f) = setup();
    let claim = claim(&f);
    exec(
        &f.pool,
        &format!(
            "UPDATE metric_source SET driver_key = 'cloudfront' WHERE source_id = '{}'",
            f.source_id
        ),
    );
    assert!(matches!(
        begin(&f, claim.lease_token),
        Err(E::SourceNotEligible)
    ));
    exec(
        &f.pool,
        &format!(
            "UPDATE metric_source SET driver_key = 'openalex-citations' WHERE source_id = '{}'",
            f.source_id
        ),
    );
    exec(
        &f.pool,
        &format!(
            "UPDATE publisher SET subscription_package = 'OASIS' WHERE publisher_id = '{}'",
            f.publisher_id
        ),
    );
    assert!(matches!(
        begin(&f, claim.lease_token),
        Err(E::MetricsCollectNotEntitled)
    ));
    assert_eq!(count(&f, "SELECT count(*) FROM metric_citation_refresh"), 0);
    assert!(matches!(
        begin_citation_refresh(
            &f.pool,
            &BeginCitationRefreshInput {
                source_account_code: "nope".into(),
                lease_token: claim.lease_token
            }
        ),
        Err(E::SourceAccountNotFound)
    ));
}

#[test]
fn the_queue_is_scoped_ordered_bounded_and_always_terminates() {
    let (_guard, f) = setup();
    let claim = claim(&f);
    let refresh = begin(&f, claim.lease_token).unwrap();
    let t = claim.lease_token;
    // Only the publisher's six DOI works that are not withdrawn.
    let page = queue(&f, t, refresh.refresh_id, Some(4), None).unwrap();
    assert_eq!(page.works.len(), 4);
    assert!(!page.done && page.next_after.is_some());
    let rest = queue(&f, t, refresh.refresh_id, Some(4), page.next_after.clone()).unwrap();
    assert_eq!(rest.works.len(), 2);
    assert!(rest.done && rest.next_after.is_none());
    let all: Vec<Uuid> = page
        .works
        .iter()
        .chain(&rest.works)
        .map(|w| w.work_id)
        .collect();
    assert!(!all.contains(&f.foreign_work) && !all.contains(&f.withdrawn_work));
    for w in &f.works {
        assert!(all.contains(w));
    }
    assert!(
        page.works.iter().all(|w| w.doi.starts_with("10.5555/obp.")),
        "{:?}",
        page.works
    );
    assert!(matches!(
        queue(&f, t, refresh.refresh_id, Some(1), Some("garbage".into())),
        Err(E::LifecycleLimitExceeded)
    ));

    // A submitted work, of any outcome, is never returned again in this
    // refresh or the next seven days: a poison DOI cannot starve the queue.
    submit(
        &f,
        t,
        refresh.refresh_id,
        "b1",
        vec![
            plain(f.works[0], CitationLookupStatus::Failed),
            plain(f.works[1], CitationLookupStatus::NotFound),
        ],
    )
    .unwrap();
    let again = queue(&f, t, refresh.refresh_id, None, None).unwrap();
    assert_eq!(again.works.len(), 4);
    assert!(again
        .works
        .iter()
        .all(|w| w.work_id != f.works[0] && w.work_id != f.works[1]));
    // Older than seven days qualifies again, oldest first.
    exec(&f.pool, &format!("UPDATE metric_citation_work_state SET last_attempted_at = transaction_timestamp() - interval '8 days' WHERE cited_work_id = '{}'", f.works[0]));
    let aged = queue(&f, t, refresh.refresh_id, None, None).unwrap();
    assert_eq!(aged.works.len(), 5);
}

#[test]
fn a_claim_reports_done_once_four_hundred_works_were_submitted() {
    let (_guard, f) = setup();
    let claim = claim(&f);
    let refresh = begin(&f, claim.lease_token).unwrap();
    exec(
        &f.pool,
        &format!(
            "UPDATE metric_citation_refresh SET works_seen = 400 WHERE refresh_id = '{}'",
            refresh.refresh_id
        ),
    );
    let page = queue(&f, claim.lease_token, refresh.refresh_id, None, None).unwrap();
    assert!(page.done && page.works.is_empty());
}

#[test]
fn a_batch_stores_sanitised_links_counts_refusals_and_replays_exactly() {
    let (_guard, f) = setup();
    let claim = claim(&f);
    let t = claim.lease_token;
    let refresh = begin(&f, t).unwrap();
    let work = f.works[0];
    let mut odd = link("W-odd", Some("HTTPS://DOI.ORG/10.1234/ODD"));
    odd.citing_title = Some(format!("A\u{0}\u{202E}  \n b{}", "x".repeat(1_200)));
    odd.citing_type = Some("t".repeat(100));
    odd.citing_url = Some("http://insecure.example".into());
    odd.citing_author_names = Some((0..9).map(|n| format!("Author {n}")).collect());
    odd.citing_authors_total = Some(9);
    odd.citing_publication_date = None;
    let entry = matched(
        work,
        vec![
            odd,
            link("  ", None),                                      // blank identity
            link(&"i".repeat(201), None),                          // over-long identity
            link("W-self", Some("https://doi.org/10.5555/obp.1")), // self-citation
            link("W-bad-doi", Some("not a doi")),
        ],
        false,
        5,
    );
    let first = submit(&f, t, refresh.refresh_id, "b1", vec![entry.clone()]).unwrap();
    assert!(!first.replayed);
    assert_eq!(
        (
            first.links_upserted,
            first.links_refused,
            first.links_merged
        ),
        (2, 3, 0)
    );
    assert_eq!(links_of(&f, work, "ACTIVE"), 2);
    assert_eq!(count(&f, &format!("SELECT count(*) FROM metric_citation_link WHERE cited_work_id = '{work}' AND citing_external_id = 'W-odd' AND citing_doi = '10.1234/odd' AND citing_type = repeat('t', 64) AND citing_url IS NULL AND char_length(citing_title) = 1000 AND citing_title LIKE 'A b%' AND citing_sort_date = '2024-01-01' AND (citing_authors -> 'names') @> '[\"Author 0\"]'::jsonb AND jsonb_array_length(citing_authors -> 'names') = 5 AND (citing_authors ->> 'total')::int = 9")), 1);
    assert_eq!(count(&f, &format!("SELECT count(*) FROM metric_citation_link WHERE cited_work_id = '{work}' AND citing_external_id = 'W-bad-doi' AND citing_doi IS NULL")), 1);
    // The self-citation is seen but never stored; blank and over-long are only counted.
    assert_eq!(count(&f, &format!("SELECT count(*) FROM metric_citation_refresh_seen WHERE refresh_id = '{}' AND citing_external_id = 'W-self'", refresh.refresh_id)), 1);
    assert_eq!(
        count(
            &f,
            "SELECT count(*) FROM metric_citation_link WHERE citing_external_id = 'W-self'"
        ),
        0
    );
    assert_eq!(count(&f, &format!("SELECT count(*) FROM metric_citation_work_state WHERE cited_work_id = '{work}' AND match_status = 'MATCHED' AND external_work_id = 'W{work}' AND last_attempted_at IS NOT NULL AND enumerated_count IS NULL")), 1);

    // Replay returns the committed outcome and writes nothing; another payload
    // under the same key is refused.
    let before = count(&f, "SELECT count(*) FROM metric_citation_link");
    let replay = submit(&f, t, refresh.refresh_id, "b1", vec![entry.clone()]).unwrap();
    assert!(replay.replayed);
    assert_eq!((replay.links_upserted, replay.links_refused), (2, 3));
    assert_eq!(
        count(&f, "SELECT count(*) FROM metric_citation_link"),
        before
    );
    let mut other = entry;
    other.links.truncate(1);
    assert!(matches!(
        submit(&f, t, refresh.refresh_id, "b1", vec![other]),
        Err(E::CitationBatchIdempotencyMismatch)
    ));
    let counters = count(&f, &format!("SELECT works_seen::bigint * 1000 + links_upserted * 100 + links_refused FROM metric_citation_refresh WHERE refresh_id = '{}'", refresh.refresh_id));
    assert_eq!(counters, 1000 + 200 + 3, "replay counted nothing twice");
}

#[test]
fn a_batch_refuses_foreign_works_and_every_bound_atomically() {
    let (_guard, f) = setup();
    let claim = claim(&f);
    let t = claim.lease_token;
    let refresh = begin(&f, t).unwrap();
    let r = refresh.refresh_id;
    assert!(matches!(
        submit(
            &f,
            t,
            r,
            "b1",
            vec![
                matched(f.works[0], vec![link("A", None)], false, 1),
                matched(f.foreign_work, vec![link("B", None)], false, 1)
            ]
        ),
        Err(E::CitationWorkNotInPublisher)
    ));
    assert!(matches!(
        submit(
            &f,
            t,
            r,
            "b1",
            vec![plain(Uuid::new_v4(), CitationLookupStatus::NotFound)]
        ),
        Err(E::CitationWorkNotInPublisher)
    ));
    assert_eq!(count(&f, "SELECT count(*) FROM metric_citation_link"), 0);
    assert_eq!(count(&f, "SELECT count(*) FROM metric_citation_batch"), 0);
    assert_eq!(
        count(&f, "SELECT count(*) FROM metric_citation_work_state"),
        0
    );

    let big: Vec<CitationLinkInput> = (0..2_001).map(|n| link(&format!("L{n}"), None)).collect();
    let fifty_one: Vec<CitationWorkEntryInput> = (0..51)
        .map(|_| plain(f.works[0], CitationLookupStatus::NotFound))
        .collect();
    let mut heavy = link("H", None);
    heavy.citing_title = Some("t".repeat(1_100_000));
    for bad in [
        vec![],
        fifty_one,
        vec![matched(f.works[0], big, false, 2_001)],
        vec![matched(f.works[0], vec![heavy], false, 1)],
        vec![{
            let mut e = matched(f.works[0], vec![], true, 0);
            e.enumerated_total = None;
            e
        }],
        vec![{
            let mut e = plain(f.works[0], CitationLookupStatus::NotFound);
            e.links = vec![link("X", None)];
            e
        }],
        vec![{
            let mut e = matched(f.works[0], vec![], false, 0);
            e.external_work_id = Some("  ".into());
            e
        }],
        vec![{
            let mut e = matched(f.works[0], vec![], false, 0);
            e.reported_count = Some(-1);
            e
        }],
    ] {
        assert!(matches!(
            submit(&f, t, r, "bx", bad),
            Err(E::LifecycleLimitExceeded)
        ));
    }
    assert!(matches!(
        submit(
            &f,
            t,
            r,
            " ",
            vec![plain(f.works[0], CitationLookupStatus::NotFound)]
        ),
        Err(E::LifecycleLimitExceeded)
    ));
    assert_eq!(count(&f, "SELECT count(*) FROM metric_citation_batch"), 0);
}

#[test]
fn withdrawal_happens_only_at_completion_and_only_for_a_complete_work() {
    let (_guard, f) = setup();
    let work = f.works[0];
    let first = refresh_once(&f, work, &["A", "B", "C"]);
    assert_eq!(first.status, Status::Completed);
    assert_eq!((first.links_upserted, first.links_withdrawn), (3, 0));
    assert_eq!(links_of(&f, work, "ACTIVE"), 3);
    assert_eq!(count(&f, &format!("SELECT enumerated_count FROM metric_citation_work_state WHERE cited_work_id = '{work}'")), 3);
    assert_eq!(
        count(&f, "SELECT count(*) FROM metric_citation_refresh_seen"),
        0,
        "seen ids are removed at completion"
    );

    // Second refresh sees two of the three: nothing is withdrawn mid-refresh.
    let claim = claim(&f);
    let t = claim.lease_token;
    let refresh = begin(&f, t).unwrap();
    submit(
        &f,
        t,
        refresh.refresh_id,
        "b1",
        vec![matched(
            work,
            vec![link("A", None), link("B", None)],
            true,
            2,
        )],
    )
    .unwrap();
    assert_eq!(links_of(&f, work, "ACTIVE"), 3, "a batch never withdraws");
    let done = complete(&f, t, refresh.refresh_id).unwrap();
    assert_eq!((done.status, done.links_withdrawn), (Status::Completed, 1));
    assert_eq!(links_of(&f, work, "ACTIVE"), 2);
    assert_eq!(count(&f, &format!("SELECT count(*) FROM metric_citation_link WHERE cited_work_id = '{work}' AND citing_external_id = 'C' AND status = 'WITHDRAWN' AND withdrawn_at IS NOT NULL")), 1);
    // Completing again returns nothing new; the lease was released.
    assert!(matches!(
        complete(&f, t, refresh.refresh_id),
        Err(E::StaleSourceClaim)
    ));
    assert_eq!(count(&f, "SELECT count(*) FROM metric_source_checkpoint WHERE lease_owner IS NULL AND last_completed_at IS NOT NULL AND cursor IS NULL"), 1);
}

#[test]
fn a_withdrawn_link_that_returns_is_reactivated_and_keeps_its_first_sighting() {
    let (_guard, f) = setup();
    let work = f.works[0];
    refresh_once(&f, work, &["A", "B"]);
    let first_seen = count(&f, &format!("SELECT extract(epoch FROM first_seen_at)::bigint FROM metric_citation_link WHERE cited_work_id = '{work}' AND citing_external_id = 'B'"));
    refresh_once(&f, work, &["A"]);
    assert_eq!(links_of(&f, work, "WITHDRAWN"), 1);
    refresh_once(&f, work, &["A", "B"]);
    assert_eq!(
        (
            links_of(&f, work, "ACTIVE"),
            links_of(&f, work, "WITHDRAWN")
        ),
        (2, 0)
    );
    assert_eq!(count(&f, &format!("SELECT count(*) FROM metric_citation_link WHERE cited_work_id = '{work}' AND citing_external_id = 'B' AND withdrawn_at IS NULL AND extract(epoch FROM first_seen_at)::bigint = {first_seen}")), 1);
}

#[test]
fn the_server_guard_holds_back_a_collapse_and_accepts_the_same_lower_count_twice() {
    let (_guard, f) = setup();
    let work = f.works[0];
    let ten: Vec<String> = (0..10).map(|n| format!("L{n}")).collect();
    let ten: Vec<&str> = ten.iter().map(String::as_str).collect();
    refresh_once(&f, work, &ten);
    assert_eq!(links_of(&f, work, "ACTIVE"), 10);

    // 2 of 10 is below half: nothing is withdrawn, the work is unconfirmed.
    let held = refresh_once(&f, work, &["L0", "L1"]);
    assert_eq!(
        (held.status, held.links_withdrawn),
        (Status::CompletedWithErrors, 0)
    );
    assert_eq!(links_of(&f, work, "ACTIVE"), 10);
    assert_eq!(count(&f, &format!("SELECT count(*) FROM metric_citation_work_state WHERE cited_work_id = '{work}' AND unconfirmed AND enumerated_count = 10 AND guard_held_count = 2")), 1);

    // The same lower count on the next complete refresh is accepted.
    let accepted = refresh_once(&f, work, &["L0", "L1"]);
    assert_eq!(
        (accepted.status, accepted.links_withdrawn),
        (Status::Completed, 8)
    );
    assert_eq!(links_of(&f, work, "ACTIVE"), 2);
    assert_eq!(count(&f, &format!("SELECT count(*) FROM metric_citation_work_state WHERE cited_work_id = '{work}' AND NOT unconfirmed AND enumerated_count = 2 AND guard_held_count IS NULL")), 1);

    // A previous count of two or fewer is never guarded.
    let tiny = refresh_once(&f, work, &[]);
    assert_eq!((tiny.status, tiny.links_withdrawn), (Status::Completed, 2));
}

#[test]
fn an_incomplete_or_unfinished_enumeration_never_withdraws() {
    let (_guard, f) = setup();
    let work = f.works[0];
    refresh_once(&f, work, &["A", "B", "C"]);

    // Driver reports it could not finish: enumerationComplete = false.
    let claim1 = claim(&f);
    let r1 = begin(&f, claim1.lease_token).unwrap();
    submit(
        &f,
        claim1.lease_token,
        r1.refresh_id,
        "b1",
        vec![matched(work, vec![link("A", None)], false, 1)],
    )
    .unwrap();
    let done = complete(&f, claim1.lease_token, r1.refresh_id).unwrap();
    assert_eq!(
        (done.status, done.links_withdrawn),
        (Status::CompletedWithErrors, 0)
    );
    assert_eq!(links_of(&f, work, "ACTIVE"), 3);
    assert_eq!(count(&f, &format!("SELECT count(*) FROM metric_citation_work_state WHERE cited_work_id = '{work}' AND unconfirmed AND enumerated_count = 3")), 1);

    // A lost batch: the final entry claims two ids but only one was ever stored.
    let claim2 = claim(&f);
    let r2 = begin(&f, claim2.lease_token).unwrap();
    submit(
        &f,
        claim2.lease_token,
        r2.refresh_id,
        "b2",
        vec![matched(work, vec![link("A", None)], true, 2)],
    )
    .unwrap();
    let lost = complete(&f, claim2.lease_token, r2.refresh_id).unwrap();
    assert_eq!(
        (lost.status, lost.links_withdrawn),
        (Status::CompletedWithErrors, 0)
    );
    assert_eq!(links_of(&f, work, "ACTIVE"), 3);

    // A refused identity still counts toward the declared total, so a
    // complete enumeration with one blank id withdraws the genuinely absent link.
    let claim3 = claim(&f);
    let r3 = begin(&f, claim3.lease_token).unwrap();
    submit(
        &f,
        claim3.lease_token,
        r3.refresh_id,
        "b3",
        vec![matched(
            work,
            vec![link("A", None), link("B", None), link("", None)],
            true,
            3,
        )],
    )
    .unwrap();
    let ok = complete(&f, claim3.lease_token, r3.refresh_id).unwrap();
    assert_eq!(
        (ok.status, ok.links_withdrawn, ok.links_refused),
        (Status::Completed, 1, 1)
    );
    assert_eq!(links_of(&f, work, "ACTIVE"), 2);
}

#[test]
fn not_found_and_failed_lookups_keep_links_and_the_last_matched_state() {
    let (_guard, f) = setup();
    let work = f.works[0];
    refresh_once(&f, work, &["A", "B"]);
    let claim = claim(&f);
    let t = claim.lease_token;
    let refresh = begin(&f, t).unwrap();
    submit(
        &f,
        t,
        refresh.refresh_id,
        "b1",
        vec![
            plain(work, CitationLookupStatus::NotFound),
            plain(f.works[1], CitationLookupStatus::Failed),
            plain(f.works[2], CitationLookupStatus::NotFound),
        ],
    )
    .unwrap();
    let done = complete(&f, t, refresh.refresh_id).unwrap();
    assert_eq!(
        (done.status, done.links_withdrawn),
        (Status::CompletedWithErrors, 0)
    );
    assert_eq!(links_of(&f, work, "ACTIVE"), 2);
    assert_eq!(count(&f, &format!("SELECT count(*) FROM metric_citation_work_state WHERE cited_work_id = '{work}' AND match_status = 'MATCHED' AND unconfirmed AND external_work_id = 'W{work}' AND last_matched_at IS NOT NULL AND enumerated_count = 2 AND last_error_at IS NOT NULL")), 1);
    // A never-matched work: failed is unconfirmed, not found is a clean answer.
    assert_eq!(count(&f, &format!("SELECT count(*) FROM metric_citation_work_state WHERE cited_work_id = '{}' AND unconfirmed AND last_error_at IS NOT NULL AND external_work_id IS NULL", f.works[1])), 1);
    assert_eq!(count(&f, &format!("SELECT count(*) FROM metric_citation_work_state WHERE cited_work_id = '{}' AND match_status = 'NOT_FOUND' AND NOT unconfirmed AND last_refreshed_at IS NOT NULL", f.works[2])), 1);
    assert_eq!(done.works_not_found, 2);
}

#[test]
fn a_reided_citing_work_keeps_one_link_and_a_merged_id_never_fails_the_batch() {
    let (_guard, f) = setup();
    let work = f.works[0];
    refresh_once(&f, work, &[]);
    // Seed one ACTIVE link with a DOI under the old id.
    let claim1 = claim(&f);
    let r1 = begin(&f, claim1.lease_token).unwrap();
    submit(
        &f,
        claim1.lease_token,
        r1.refresh_id,
        "b1",
        vec![matched(work, vec![link("OLD", Some("10.1/x"))], true, 1)],
    )
    .unwrap();
    complete(&f, claim1.lease_token, r1.refresh_id).unwrap();

    // The source now reports the same DOI under a new id and no longer lists the old one:
    // the existing row takes the new id.
    let claim2 = claim(&f);
    let r2 = begin(&f, claim2.lease_token).unwrap();
    let out = submit(
        &f,
        claim2.lease_token,
        r2.refresh_id,
        "b1",
        vec![matched(work, vec![link("NEW", Some("10.1/x"))], true, 1)],
    )
    .unwrap();
    assert_eq!((out.links_upserted, out.links_merged), (1, 0));
    complete(&f, claim2.lease_token, r2.refresh_id).unwrap();
    assert_eq!(count(&f, &format!("SELECT count(*) FROM metric_citation_link WHERE cited_work_id = '{work}' AND citing_external_id = 'NEW' AND status = 'ACTIVE'")), 1);
    assert_eq!(
        links_of(&f, work, "ACTIVE") + links_of(&f, work, "WITHDRAWN"),
        1
    );

    // Both ids listed in one refresh: one row, the other id merged, and it
    // lists as seen so nothing is withdrawn for it.
    let claim3 = claim(&f);
    let r3 = begin(&f, claim3.lease_token).unwrap();
    let out = submit(
        &f,
        claim3.lease_token,
        r3.refresh_id,
        "b1",
        vec![matched(
            work,
            vec![link("NEW", Some("10.1/x")), link("ALT", Some("10.1/x"))],
            true,
            2,
        )],
    )
    .unwrap();
    assert_eq!((out.links_upserted, out.links_merged), (1, 1));
    let done = complete(&f, claim3.lease_token, r3.refresh_id).unwrap();
    assert_eq!((done.status, done.links_withdrawn), (Status::Completed, 0));
    assert_eq!(links_of(&f, work, "ACTIVE"), 1);

    // Two brand-new ids on one DOI in one batch: the lowest id wins, deterministically.
    let work2 = f.works[1];
    let claim4 = claim(&f);
    let r4 = begin(&f, claim4.lease_token).unwrap();
    submit(
        &f,
        claim4.lease_token,
        r4.refresh_id,
        "b1",
        vec![matched(
            work2,
            vec![link("Z", Some("10.2/y")), link("M", Some("10.2/y"))],
            false,
            2,
        )],
    )
    .unwrap();
    assert_eq!(count(&f, &format!("SELECT count(*) FROM metric_citation_link WHERE cited_work_id = '{work2}' AND citing_external_id = 'M'")), 1);
    assert_eq!(
        count(
            &f,
            &format!("SELECT count(*) FROM metric_citation_link WHERE cited_work_id = '{work2}'")
        ),
        1
    );
}

#[test]
fn a_flap_back_onto_a_withdrawn_id_reactivates_it_only_without_a_doi_collision() {
    let (_guard, f) = setup();
    let work = f.works[0];
    let seed = |ids: Vec<CitationLinkInput>, total: i32| {
        let claim = claim(&f);
        let r = begin(&f, claim.lease_token).unwrap();
        let out = submit(
            &f,
            claim.lease_token,
            r.refresh_id,
            "b1",
            vec![matched(work, ids, true, total)],
        )
        .unwrap();
        complete(&f, claim.lease_token, r.refresh_id).unwrap();
        out
    };
    seed(vec![link("A", Some("10.3/a")), link("B", None)], 2);
    seed(vec![link("B", None), link("C", Some("10.3/a"))], 2); // A re-id'd to C
    assert_eq!(count(&f, &format!("SELECT count(*) FROM metric_citation_link WHERE cited_work_id = '{work}' AND citing_external_id = 'C' AND citing_doi = '10.3/a' AND status = 'ACTIVE'")), 1);
    // The source flaps back to A: C's row is the DOI owner and A is absent,
    // so the row is re-id'd back; no duplicate exists.
    seed(vec![link("B", None), link("A", Some("10.3/a"))], 2);
    assert_eq!(count(&f, &format!("SELECT count(*) FROM metric_citation_link WHERE cited_work_id = '{work}' AND citing_doi = '10.3/a'")), 1);
    assert_eq!(links_of(&f, work, "ACTIVE"), 2);
}

#[test]
fn a_withdrawn_id_returning_without_its_old_doi_reactivates_without_failing_the_batch() {
    let (_guard, f) = setup();
    let work = f.works[0];
    let seed = |ids: Vec<CitationLinkInput>, total: i32| {
        let claim = claim(&f);
        let r = begin(&f, claim.lease_token).unwrap();
        submit(
            &f,
            claim.lease_token,
            r.refresh_id,
            "b1",
            vec![matched(work, ids, true, total)],
        )
        .unwrap();
        complete(&f, claim.lease_token, r.refresh_id).unwrap();
    };
    seed(vec![link("A", Some("10.3/a"))], 1);
    seed(vec![link("B", None)], 1); // A withdrawn
    seed(vec![link("B", None), link("X", Some("10.3/a"))], 2); // X takes the DOI
    assert_eq!(links_of(&f, work, "WITHDRAWN"), 1);
    // A returns with no DOI while X owns it: A must reactivate, not clash.
    seed(
        vec![link("A", None), link("B", None), link("X", Some("10.3/a"))],
        3,
    );
    assert_eq!(links_of(&f, work, "ACTIVE"), 3);
    assert_eq!(count(&f, &format!("SELECT count(*) FROM metric_citation_link WHERE cited_work_id = '{work}' AND citing_external_id = 'A' AND citing_doi IS NULL AND status = 'ACTIVE'")), 1);
}

#[test]
fn a_work_with_more_than_two_thousand_links_spans_batches_and_is_swept_once() {
    let (_guard, f) = setup();
    let work = f.works[0];
    refresh_once(&f, work, &["GONE"]);
    let claim = claim(&f);
    let t = claim.lease_token;
    let refresh = begin(&f, t).unwrap();
    let batch = |from: i32, to: i32, complete_it: bool| {
        let links: Vec<CitationLinkInput> = (from..to)
            .map(|n| link(&format!("N{n:05}"), None))
            .collect();
        let mut entry = matched(work, links, complete_it, 2_500);
        entry.enumerated_total = complete_it.then_some(2_500);
        entry
    };
    submit(
        &f,
        t,
        refresh.refresh_id,
        "b1",
        vec![batch(0, 1_250, false)],
    )
    .unwrap();
    submit(
        &f,
        t,
        refresh.refresh_id,
        "b2",
        vec![batch(1_250, 2_500, true)],
    )
    .unwrap();
    assert_eq!(
        links_of(&f, work, "ACTIVE"),
        2_501,
        "nothing is withdrawn before completion"
    );
    let done = complete(&f, t, refresh.refresh_id).unwrap();
    // Any incomplete entry of a work wins: a work that spans batches must
    // declare `enumerationComplete` and the same total in every entry.
    assert_eq!(done.status, Status::CompletedWithErrors);
    assert_eq!(links_of(&f, work, "ACTIVE"), 2_501);
}

#[test]
fn a_spanning_work_is_swept_when_every_entry_is_complete() {
    let (_guard, f) = setup();
    let work = f.works[0];
    refresh_once(&f, work, &["GONE"]);
    let claim = claim(&f);
    let t = claim.lease_token;
    let refresh = begin(&f, t).unwrap();
    let batch = |from: i32, to: i32| {
        let links: Vec<CitationLinkInput> = (from..to)
            .map(|n| link(&format!("N{n:05}"), None))
            .collect();
        let mut entry = matched(work, links, true, 2_500);
        entry.enumerated_total = Some(2_500);
        entry
    };
    submit(&f, t, refresh.refresh_id, "b1", vec![batch(0, 1_250)]).unwrap();
    submit(&f, t, refresh.refresh_id, "b2", vec![batch(1_250, 2_500)]).unwrap();
    assert_eq!(links_of(&f, work, "ACTIVE"), 2_501);
    let done = complete(&f, t, refresh.refresh_id).unwrap();
    assert_eq!((done.status, done.links_withdrawn), (Status::Completed, 1));
    assert_eq!(links_of(&f, work, "ACTIVE"), 2_500);
    assert_eq!(links_of(&f, work, "WITHDRAWN"), 1);
}

#[test]
fn the_checkpoint_cursor_stays_null_and_the_next_claim_succeeds() {
    let (_guard, f) = setup();
    refresh_once(&f, f.works[0], &["A"]);
    assert_eq!(count(&f, "SELECT count(*) FROM metric_source_checkpoint WHERE cursor IS NULL AND last_successful_period_end IS NULL AND last_completed_at IS NOT NULL AND last_discovered_at IS NOT NULL AND lease_owner IS NULL"), 1);
    let again = claim(&f);
    assert!(again.period_manifest_cursor.is_none());
    refresh_once_with(&f, again, f.works[0], &["A"]);
}

fn refresh_once_with(f: &Fixture, claim: MetricSourceUnitClaim, work: Uuid, ids: &[&str]) {
    let refresh = begin(f, claim.lease_token).unwrap();
    submit(
        f,
        claim.lease_token,
        refresh.refresh_id,
        "b1",
        vec![matched(
            work,
            ids.iter().map(|i| link(i, None)).collect(),
            true,
            ids.len() as i32,
        )],
    )
    .unwrap();
    complete(f, claim.lease_token, refresh.refresh_id).unwrap();
}

#[test]
fn an_expired_lease_commits_nothing_more_but_keeps_committed_batches() {
    let (_guard, f) = setup();
    let claim = claim(&f);
    let t = claim.lease_token;
    let refresh = begin(&f, t).unwrap();
    submit(
        &f,
        t,
        refresh.refresh_id,
        "b1",
        vec![matched(f.works[0], vec![link("A", None)], false, 1)],
    )
    .unwrap();
    expire_lease(&f);
    assert!(matches!(
        submit(
            &f,
            t,
            refresh.refresh_id,
            "b2",
            vec![matched(f.works[0], vec![link("B", None)], false, 2)]
        ),
        Err(E::StaleSourceClaim)
    ));
    assert!(matches!(
        complete(&f, t, refresh.refresh_id),
        Err(E::StaleSourceClaim)
    ));
    assert_eq!(
        links_of(&f, f.works[0], "ACTIVE"),
        1,
        "the committed batch stays committed"
    );
    assert_eq!(count(&f, "SELECT count(*) FROM metric_citation_batch"), 1);
}

#[test]
fn nothing_but_the_citation_tables_and_checkpoint_progress_is_written() {
    let (_guard, f) = setup();
    refresh_once(&f, f.works[0], &["A", "B"]);
    for table in [
        "metric_import",
        "metric_import_batch",
        "metric_record",
        "metric_coverage",
    ] {
        assert_eq!(
            count(&f, &format!("SELECT count(*) FROM {table}")),
            0,
            "{table}"
        );
    }
}
