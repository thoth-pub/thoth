//! `CITATIONS-01` slice C3: the citation read against a real database.
//!
//! The fixture is `write_tests::setup` (one `openalex-citations` source, an
//! OBP-like publisher with six DOI works, a cancelled work and one foreign
//! publisher's work). The publisher is given the `SPHINX` package so it holds
//! `METRICS_DASHBOARD`; links and work states are inserted directly, because
//! the write path has its own tests.

use chrono::NaiveDate;
use uuid::Uuid;

use super::read::*;
use super::write_tests::{exec, setup, Fixture};
use super::MetricCitationsServing as Serving;
use crate::model::metric_dashboard::MetricReadError as E;
use crate::model::tests::db::TestDbGuard;

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn entitled() -> (TestDbGuard, Fixture) {
    let (guard, f) = setup();
    exec(
        &f.pool,
        &format!(
            "UPDATE publisher SET subscription_package = 'SPHINX' WHERE publisher_id = '{}'",
            f.publisher_id
        ),
    );
    (guard, f)
}

fn request(f: &Fixture) -> MetricCitationsInput {
    MetricCitationsInput {
        publisher_ids: vec![f.publisher_id],
        work_ids: None,
        citing_from: None,
        citing_to: None,
        source_codes: None,
        first: None,
        after: None,
    }
}

fn read(f: &Fixture, input: &MetricCitationsInput) -> Result<MetricCitations, E> {
    metric_citations(&f.pool, Serving::On, input)
}

/// One work state row. `last_refreshed` is an SQL expression or `NULL`.
fn state(
    f: &Fixture,
    work: Uuid,
    status: &str,
    reported: Option<i32>,
    unconfirmed: bool,
    last_refreshed: &str,
) {
    let matched = status == "MATCHED";
    exec(
        &f.pool,
        &format!(
            "INSERT INTO metric_citation_work_state \
               (cited_work_id, source_id, match_status, last_matched_at, external_work_id, \
                reported_count, last_attempted_at, last_refreshed_at, unconfirmed) \
             VALUES ('{work}', '{}', '{status}', {}, {}, {}, now(), {last_refreshed}, {unconfirmed})",
            f.source_id,
            if matched { "now()" } else { "NULL" },
            if matched { "'W1'" } else { "NULL" },
            reported.map_or("NULL".to_string(), |n| n.to_string()),
        ),
    );
}

/// One citing link; `published` is `Some((year, date))` with either optional.
fn link(
    f: &Fixture,
    work: Uuid,
    id: &str,
    year: Option<i32>,
    published: Option<NaiveDate>,
    status: &str,
) -> Uuid {
    let link_id = Uuid::new_v4();
    let sort = published
        .or_else(|| year.map(|y| date(y, 1, 1)))
        .unwrap_or_else(|| date(1, 1, 1));
    exec(
        &f.pool,
        &format!(
            "INSERT INTO metric_citation_link \
               (link_id, cited_work_id, source_id, citing_external_id, citing_title, \
                citing_publication_year, citing_publication_date, citing_sort_date, status, \
                withdrawn_at) \
             VALUES ('{link_id}', '{work}', '{}', '{id}', 'Title {id}', {}, {}, '{sort}', \
                     '{status}', {})",
            f.source_id,
            year.map_or("NULL".to_string(), |y| y.to_string()),
            published.map_or("NULL".to_string(), |d| format!("'{d}'")),
            if status == "WITHDRAWN" {
                "now()"
            } else {
                "NULL"
            },
        ),
    );
    link_id
}

/// The dated fixture: three links of work 0 (2023-06-01, 2021 only, no date),
/// one of work 1, one WITHDRAWN link that must never appear.
fn seed_links(f: &Fixture) -> Vec<Uuid> {
    state(
        f,
        f.works[0],
        "MATCHED",
        Some(5),
        false,
        "now() - interval '1 day'",
    );
    state(
        f,
        f.works[1],
        "MATCHED",
        Some(1),
        false,
        "now() - interval '3 days'",
    );
    vec![
        link(
            f,
            f.works[0],
            "A",
            Some(2023),
            Some(date(2023, 6, 1)),
            "ACTIVE",
        ),
        link(f, f.works[0], "B", Some(2021), None, "ACTIVE"),
        link(f, f.works[0], "C", None, None, "ACTIVE"),
        link(
            f,
            f.works[1],
            "D",
            Some(2023),
            Some(date(2023, 6, 1)),
            "ACTIVE",
        ),
        link(
            f,
            f.works[1],
            "GONE",
            Some(2024),
            Some(date(2024, 1, 1)),
            "WITHDRAWN",
        ),
    ]
}

#[test]
fn the_switch_off_refuses_and_touches_no_data() {
    let (_guard, f) = entitled();
    seed_links(&f);
    assert_eq!(
        metric_citations(&f.pool, Serving::Off, &request(&f)),
        Err(E::CitationsNotServed)
    );
    assert_eq!(E::CitationsNotServed.code(), "METRIC_CITATIONS_NOT_SERVED");
    // Even a request that would otherwise be refused is answered by the switch.
    let mut empty = request(&f);
    empty.publisher_ids.clear();
    assert_eq!(
        metric_citations(&f.pool, Serving::Off, &empty),
        Err(E::CitationsNotServed)
    );
}

#[test]
fn the_summary_counts_every_state_and_never_counts_withdrawn_links() {
    let (_guard, f) = entitled();
    seed_links(&f);
    // works[2] not found, works[3] never matched and failed (stored NOT_FOUND +
    // unconfirmed), works[4] matched and unconfirmed, works[5] never reached.
    state(&f, f.works[2], "NOT_FOUND", None, false, "NULL");
    state(&f, f.works[3], "NOT_FOUND", None, true, "NULL");
    state(
        &f,
        f.works[4],
        "MATCHED",
        Some(0),
        true,
        "now() - interval '9 days'",
    );

    let result = read(&f, &request(&f)).unwrap();
    let s = &result.summary;
    assert_eq!(s.link_count, 4);
    assert_eq!(s.cited_work_count, 2);
    assert_eq!(s.matched_work_count, 3);
    assert_eq!(s.unconfirmed_work_count, 2);
    assert_eq!(s.not_found_work_count, 1);
    // works[5] has a DOI and no state. The cancelled work has a DOI and no
    // state either, but the write path never asks for it, so it is not
    // counted as waiting.
    assert_eq!(s.not_yet_refreshed_work_count, 1);
    assert_eq!(s.reported_count_total, 6);
    assert!(s.last_refreshed_at.is_some());
    let years: Vec<(Option<i32>, i32)> =
        s.by_citing_year.iter().map(|y| (y.year, y.count)).collect();
    assert_eq!(years, vec![(Some(2021), 1), (Some(2023), 2), (None, 1)]);
    assert_eq!(result.links.len(), 4);
    assert!(result.links.iter().all(|l| l.citing_external_id != "GONE"));
    assert!(!result.page_info.has_next_page);
    assert_eq!(result.page_info.end_cursor, None);
}

#[test]
fn freshness_is_withheld_while_a_matched_work_has_no_completed_refresh() {
    let (_guard, f) = entitled();
    state(
        &f,
        f.works[0],
        "MATCHED",
        Some(1),
        false,
        "now() - interval '2 days'",
    );
    assert!(read(&f, &request(&f))
        .unwrap()
        .summary
        .last_refreshed_at
        .is_some());
    state(&f, f.works[1], "MATCHED", Some(1), true, "NULL");
    assert_eq!(
        read(&f, &request(&f)).unwrap().summary.last_refreshed_at,
        None
    );
}

#[test]
fn links_are_ordered_by_date_then_id_with_undated_ones_last_and_carry_their_context() {
    let (_guard, f) = entitled();
    seed_links(&f);
    let result = read(&f, &request(&f)).unwrap();
    let dates: Vec<NaiveDate> = result.links.iter().map(|l| l.citing_sort_date).collect();
    assert_eq!(
        dates,
        vec![
            date(2023, 6, 1),
            date(2023, 6, 1),
            date(2021, 1, 1),
            date(1, 1, 1)
        ]
    );
    // The two links of one date follow link_id DESC.
    assert!(result.links[0].link_id > result.links[1].link_id);
    let first = &result.links[0];
    assert_eq!(first.source.code, "openalex-citations");
    assert!(f.works[0..2].contains(&first.cited_work.work_id));
    assert!(first.cited_work.doi.as_deref().unwrap().contains("OBP"));
    assert_eq!(
        first.citing_title.as_deref().map(|t| &t[..6]),
        Some("Title ")
    );
    // A link known only by year reports the year and no date.
    let year_only = result
        .links
        .iter()
        .find(|l| l.citing_external_id == "B")
        .unwrap();
    assert_eq!(year_only.citing_publication_year, Some(2021));
    assert_eq!(year_only.citing_publication_date, None);
}

#[test]
fn pages_follow_the_cursor_without_gaps_or_repeats_even_as_links_are_withdrawn() {
    let (_guard, f) = entitled();
    state(&f, f.works[0], "MATCHED", Some(7), false, "now()");
    let ids: Vec<Uuid> = (0..7)
        .map(|n| {
            link(
                &f,
                f.works[0],
                &format!("L{n}"),
                Some(2020 + n),
                Some(date(2020 + n, 3, 3)),
                "ACTIVE",
            )
        })
        .collect();
    let mut input = request(&f);
    input.first = Some(3);
    let one = read(&f, &input).unwrap();
    assert_eq!(one.links.len(), 3);
    assert!(one.page_info.has_next_page);
    assert_eq!(one.summary.link_count, 7);

    // A link already served and one not yet served are withdrawn between pages.
    let served: Vec<Uuid> = one.links.iter().map(|l| l.link_id).collect();
    let later = *ids.iter().find(|id| !served.contains(id)).unwrap();
    exec(
        &f.pool,
        &format!(
            "UPDATE metric_citation_link SET status = 'WITHDRAWN', withdrawn_at = now() \
              WHERE link_id IN ('{}', '{later}')",
            served[0]
        ),
    );
    input.after = one.page_info.end_cursor.clone();
    let two = read(&f, &input).unwrap();
    let rest: Vec<Uuid> = two.links.iter().map(|l| l.link_id).collect();
    assert!(rest.iter().all(|id| !served.contains(id)), "no repeats");
    assert!(!rest.contains(&later), "withdrawn link is not served");
    assert_eq!(rest.len(), 3);
    assert_eq!(two.summary.link_count, 5);
    assert!(!two.page_info.has_next_page);
    assert_eq!(two.page_info.end_cursor, None);
}

#[test]
fn a_cursor_cannot_be_replayed_with_other_publishers_works_dates_or_sources() {
    let (_guard, f) = entitled();
    seed_links(&f);
    let mut input = request(&f);
    input.first = Some(1);
    let cursor = read(&f, &input).unwrap().page_info.end_cursor.unwrap();

    let mut replay = request(&f);
    replay.first = Some(1);
    replay.after = Some(cursor.clone());
    assert!(read(&f, &replay).is_ok());
    replay.citing_from = Some(date(2000, 1, 1));
    assert!(matches!(read(&f, &replay), Err(E::QueryInvalid(_))));
    replay.citing_from = None;
    replay.work_ids = Some(vec![f.works[0]]);
    assert!(matches!(read(&f, &replay), Err(E::QueryInvalid(_))));
    replay.work_ids = None;
    replay.source_codes = Some(vec!["openalex-citations".into()]);
    assert!(matches!(read(&f, &replay), Err(E::QueryInvalid(_))));
    // A tampered cursor is refused the same way.
    replay.source_codes = None;
    replay.after = Some(format!("{cursor}00"));
    assert!(matches!(read(&f, &replay), Err(E::QueryInvalid(_))));
}

#[test]
fn filters_narrow_the_summary_and_the_page_together() {
    let (_guard, f) = entitled();
    seed_links(&f);
    let mut input = request(&f);
    input.work_ids = Some(vec![f.works[1]]);
    let one_work = read(&f, &input).unwrap();
    assert_eq!(one_work.summary.link_count, 1);
    assert_eq!(one_work.summary.matched_work_count, 1);
    assert_eq!(one_work.links.len(), 1);

    let mut dated = request(&f);
    dated.citing_from = Some(date(2022, 1, 1));
    dated.citing_to = Some(date(2023, 12, 31));
    let result = read(&f, &dated).unwrap();
    assert_eq!(result.summary.link_count, 2);
    assert!(result
        .links
        .iter()
        .all(|l| l.citing_sort_date.to_string().starts_with("2023")));

    // The undated link sorts at 0001-01-01 and is reachable by a range that
    // starts there.
    let mut undated = request(&f);
    undated.citing_to = Some(date(1, 12, 31));
    let result = read(&f, &undated).unwrap();
    assert_eq!(result.links.len(), 1);
    assert_eq!(result.links[0].citing_external_id, "C");

    let mut named = request(&f);
    named.source_codes = Some(vec!["openalex-citations".into()]);
    assert_eq!(read(&f, &named).unwrap().summary.link_count, 4);
    named.source_codes = Some(vec!["no-such-source".into()]);
    assert!(matches!(read(&f, &named), Err(E::QueryInvalid(_))));
}

#[test]
fn works_outside_the_selected_publishers_are_refused_like_unknown_ones() {
    let (_guard, f) = entitled();
    seed_links(&f);
    let mut foreign = request(&f);
    foreign.work_ids = Some(vec![f.foreign_work]);
    let mut unknown = request(&f);
    unknown.work_ids = Some(vec![Uuid::new_v4()]);
    let a = read(&f, &foreign);
    let b = read(&f, &unknown);
    assert!(matches!(a, Err(E::QueryInvalid(_))));
    assert_eq!(a, b, "the two refusals are indistinguishable");
    // A listed own work next to a foreign one is refused whole.
    foreign.work_ids = Some(vec![f.works[0], f.foreign_work]);
    assert!(matches!(read(&f, &foreign), Err(E::QueryInvalid(_))));
}

#[test]
fn links_of_another_publisher_never_appear_and_an_unentitled_publisher_refuses_the_request() {
    let (_guard, f) = entitled();
    seed_links(&f);
    // Another publisher's work with a link, also entitled.
    let other: Uuid = {
        let mut connection = f.pool.get().unwrap();
        use diesel::{sql_query, RunQueryDsl};
        #[derive(diesel::QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            publisher_id: Uuid,
        }
        sql_query(format!(
            "SELECT i.publisher_id FROM work w JOIN imprint i ON i.imprint_id = w.imprint_id \
              WHERE w.work_id = '{}'",
            f.foreign_work
        ))
        .get_result::<Row>(&mut connection)
        .unwrap()
        .publisher_id
    };
    state(&f, f.foreign_work, "MATCHED", Some(9), false, "now()");
    link(
        &f,
        f.foreign_work,
        "X",
        Some(2022),
        Some(date(2022, 2, 2)),
        "ACTIVE",
    );

    // Alone, the first publisher sees only its own links.
    let own = read(&f, &request(&f)).unwrap();
    assert_eq!(own.summary.link_count, 4);
    assert!(own
        .links
        .iter()
        .all(|l| l.cited_work.work_id != f.foreign_work));

    // The other publisher holds OBELISK, without METRICS_DASHBOARD: one
    // unentitled publisher among several refuses the whole request.
    let mut both = request(&f);
    both.publisher_ids = vec![f.publisher_id, other];
    assert_eq!(read(&f, &both), Err(E::Unauthorised));
    let mut alone = request(&f);
    alone.publisher_ids = vec![other];
    assert_eq!(read(&f, &alone), Err(E::Unauthorised));

    // Entitled, both are served together.
    exec(
        &f.pool,
        &format!(
            "UPDATE publisher SET subscription_package = 'PYRAMID' WHERE publisher_id = '{other}'"
        ),
    );
    let together = read(&f, &both).unwrap();
    assert_eq!(together.summary.link_count, 5);

    // An unknown publisher is refused, not skipped.
    let mut missing = request(&f);
    missing.publisher_ids = vec![f.publisher_id, Uuid::new_v4()];
    assert!(matches!(read(&f, &missing), Err(E::QueryInvalid(_))));
}

#[test]
fn counts_are_never_summed_across_sources() {
    let (_guard, f) = entitled();
    seed_links(&f);
    let second = Uuid::new_v4();
    exec(
        &f.pool,
        &format!(
            "INSERT INTO metric_source (source_id, code, acquisition_type, driver_key, enabled) \
             VALUES ('{second}', 'other-citations', 'DRIVER', 'other-citations', TRUE)"
        ),
    );
    exec(
        &f.pool,
        &format!(
            "INSERT INTO metric_citation_work_state (cited_work_id, source_id, match_status, \
               last_matched_at, external_work_id, reported_count, last_attempted_at) \
             VALUES ('{}', '{second}', 'MATCHED', now(), 'W9', 2, now())",
            f.works[0]
        ),
    );
    // Two sources hold data: the request must name exactly one.
    assert!(matches!(read(&f, &request(&f)), Err(E::QueryInvalid(_))));
    let mut named = request(&f);
    named.source_codes = Some(vec!["openalex-citations".into()]);
    assert_eq!(read(&f, &named).unwrap().summary.link_count, 4);
    named.source_codes = Some(vec!["other-citations".into()]);
    let other = read(&f, &named).unwrap();
    assert_eq!(other.summary.link_count, 0);
    assert_eq!(other.summary.reported_count_total, 2);
    named.source_codes = Some(vec!["openalex-citations".into(), "other-citations".into()]);
    assert!(matches!(read(&f, &named), Err(E::QueryInvalid(_))));
}

#[test]
fn an_empty_catalogue_reads_as_zero_links_and_every_work_not_yet_reached() {
    let (_guard, f) = entitled();
    let result = read(&f, &request(&f)).unwrap();
    assert_eq!(result.summary.link_count, 0);
    assert_eq!(result.summary.matched_work_count, 0);
    assert_eq!(result.summary.not_yet_refreshed_work_count, 6);
    assert_eq!(result.summary.last_refreshed_at, None);
    assert!(result.links.is_empty());
    assert!(result.summary.by_citing_year.is_empty());
}

#[test]
fn the_openalex_platform_never_enters_the_dashboard_scope() {
    use crate::model::metric_dashboard::{
        metric_dashboard, MetricDashboardInput, MetricSelectorInput,
    };
    let (_guard, f) = entitled();
    seed_links(&f);
    // `setup` created the enabled `openalex` platform with no measures.
    let platform_id = {
        use diesel::{sql_query, RunQueryDsl};
        #[derive(diesel::QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            platform_id: Uuid,
        }
        let mut connection = f.pool.get().unwrap();
        sql_query("SELECT platform_id FROM metric_platform WHERE code = 'openalex'")
            .get_result::<Row>(&mut connection)
            .unwrap()
            .platform_id
    };
    let input = MetricDashboardInput {
        selector: MetricSelectorInput {
            publisher_ids: Some(vec![f.publisher_id]),
            ..MetricSelectorInput::default()
        },
        start_date: Some(date(2026, 3, 1)),
        end_date: date(2026, 3, 5),
        measures: None,
        platforms: None,
        timeline_grain: None,
        include_countries: None,
        include_institutions: None,
    };
    let dashboard = metric_dashboard(&f.pool, &input).expect("dashboard read");
    assert!(dashboard
        .totals
        .iter()
        .all(|t| t.platform_id != platform_id));
    assert!(dashboard
        .timeline
        .iter()
        .all(|t| t.platform_id != platform_id));
    assert!(dashboard
        .coverage
        .items
        .iter()
        .all(|item| item.platform_id != platform_id));
    // Naming it explicitly does not make it servable either.
    let mut named = input.clone();
    named.platforms = Some(vec![platform_id]);
    if let Ok(dashboard) = metric_dashboard(&f.pool, &named) {
        assert!(dashboard.totals.is_empty());
    }
}
