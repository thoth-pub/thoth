#![cfg(feature = "backend")]

//! `BE-06` source-generation capture (R52B section 8, section 25.3) and the
//! commit-time flush's properties against this implementation's own migration
//! (section 25.23 as required by section 29): T1-T37 and T39, T317-T322 and
//! T324 with the negative controls of T318, T319 and T321, and the two drift
//! guards of section 10.5.
//!
//! Every test edits through ordinary SQL on the released schema with both BE-06
//! migrations applied, as the released editorial statements do, on real
//! PostgreSQL connections. Nothing here contacts Crossref.

#[allow(dead_code)]
mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use diesel::connection::SimpleConnection;
use diesel::sql_types::{BigInt, Text, Uuid as SqlUuid};
use diesel::{Connection, PgConnection, QueryableByName, RunQueryDsl};
use thoth_api::db::PgPool;
use uuid::Uuid;

#[derive(QueryableByName)]
struct GenerationRow {
    #[diesel(sql_type = SqlUuid)]
    work_id: Uuid,
    #[diesel(sql_type = BigInt)]
    source_generation: i64,
}

#[derive(QueryableByName)]
struct TextRow {
    #[diesel(sql_type = Text)]
    value: String,
}

fn execute(connection: &mut PgConnection, sql: &str) {
    connection
        .batch_execute(sql)
        .unwrap_or_else(|error| panic!("{sql}: {error}"));
}

fn text(connection: &mut PgConnection, sql: &str) -> String {
    diesel::sql_query(sql)
        .get_result::<TextRow>(connection)
        .unwrap_or_else(|error| panic!("{sql}: {error}"))
        .value
}

fn dedicated() -> PgConnection {
    PgConnection::establish(&support::test_db_url()).expect("dedicated connection")
}

fn generations(connection: &mut PgConnection) -> BTreeMap<Uuid, i64> {
    diesel::sql_query(
        "SELECT work_id, source_generation FROM work_upsert_generation WHERE execution_profile = 'CROSSREF'",
    )
    .load::<GenerationRow>(connection)
    .expect("generations")
    .into_iter()
    .map(|row| (row.work_id, row.source_generation))
    .collect()
}

fn changed(before: &BTreeMap<Uuid, i64>, after: &BTreeMap<Uuid, i64>) -> BTreeMap<Uuid, i64> {
    let mut changed = BTreeMap::new();
    for (work, value) in after {
        let delta = value - before.get(work).copied().unwrap_or(0);
        if delta != 0 {
            changed.insert(*work, delta);
        }
    }
    changed
}

/// Run `sql` as one committed transaction and return every Work whose
/// generation changed, with its delta. Also asserts the transaction left no
/// queue row and no orphaned generation row.
fn deltas(connection: &mut PgConnection, sql: &str) -> BTreeMap<Uuid, i64> {
    let before = generations(connection);
    execute(connection, &format!("BEGIN; {sql}; COMMIT;"));
    let after = generations(connection);
    assert_clean(connection);
    changed(&before, &after)
}

fn once(works: &[Uuid]) -> BTreeMap<Uuid, i64> {
    works.iter().map(|work| (*work, 1)).collect()
}

fn queue_rows(connection: &mut PgConnection) -> String {
    text(
        connection,
        "SELECT count(*)::text AS value FROM work_upsert_capture_queue",
    )
}

fn orphans(connection: &mut PgConnection) -> String {
    text(
        connection,
        "SELECT count(*)::text AS value FROM work_upsert_generation g \
         WHERE NOT EXISTS (SELECT 1 FROM work w WHERE w.work_id = g.work_id)",
    )
}

fn assert_clean(connection: &mut PgConnection) {
    assert_eq!(queue_rows(connection), "0", "no queue residue");
    assert_eq!(orphans(connection), "0", "no orphaned generation row");
}

/// True when another session holds a lock on the Work row that conflicts with
/// `FOR UPDATE`.
fn work_row_locked(connection: &mut PgConnection, work: Uuid) -> bool {
    match connection.batch_execute(&format!(
        "BEGIN; SELECT 1 FROM work WHERE work_id = '{work}' FOR UPDATE NOWAIT; ROLLBACK;"
    )) {
        Ok(()) => false,
        Err(error) => {
            connection.batch_execute("ROLLBACK").ok();
            assert!(
                error.to_string().contains("could not obtain lock"),
                "{error}"
            );
            true
        }
    }
}

fn generation_row_locked(connection: &mut PgConnection, work: Uuid) -> bool {
    match connection.batch_execute(&format!(
        "BEGIN; SELECT 1 FROM work_upsert_generation WHERE work_id = '{work}' FOR UPDATE NOWAIT; ROLLBACK;"
    )) {
        Ok(()) => false,
        Err(error) => {
            connection.batch_execute("ROLLBACK").ok();
            assert!(error.to_string().contains("could not obtain lock"), "{error}");
            true
        }
    }
}

fn backend_pid(connection: &mut PgConnection) -> String {
    text(connection, "SELECT pg_backend_pid()::text AS value")
}

/// Poll until `pid` waits on a heavyweight lock. Panics after ten seconds.
fn wait_for_lock_wait(observer: &mut PgConnection, pid: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let waiting = text(
            observer,
            &format!("SELECT count(*)::text AS value FROM pg_stat_activity WHERE pid = {pid} AND wait_event_type = 'Lock'"),
        );
        if waiting == "1" {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "session {pid} never waited on a lock"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The waiting session's `pg_locks` transcript: every ungranted lock it requests.
fn ungranted(observer: &mut PgConnection, pid: &str) -> String {
    text(
        observer,
        &format!(
            "SELECT coalesce(string_agg(locktype || ':' || mode, ',' ORDER BY locktype, mode), '') AS value \
             FROM pg_locks WHERE pid = {pid} AND NOT granted"
        ),
    )
}

/// A negative control: replace the flush with a variant missing exactly one
/// element, and restore the migration's own definition when dropped.
struct FlushVariant {
    original: String,
}

impl FlushVariant {
    fn without(pattern: &str, replacement: &str) -> Self {
        let mut connection = dedicated();
        let original = text(
            &mut connection,
            "SELECT pg_get_functiondef('public.work_upsert_capture_flush()'::regprocedure) AS value",
        );
        assert_eq!(
            original.matches(pattern).count(),
            1,
            "the element removed: {pattern}"
        );
        execute(&mut connection, &original.replacen(pattern, replacement, 1));
        FlushVariant { original }
    }
}

impl Drop for FlushVariant {
    fn drop(&mut self) {
        let mut connection = dedicated();
        execute(&mut connection, &self.original);
    }
}

const IMMEDIACY_DEFENCE: &str =
    "IF coalesce(current_setting('be06.capture_probe', true), '') = 'fired' THEN";
const VANISHED_ROOT_CLEANUP: &str =
    "AND NOT EXISTS (SELECT 1 FROM public.work w WHERE w.work_id = g.work_id)";

struct Fixture {
    publisher: Uuid,
    imprint: Uuid,
    other_imprint: Uuid,
    other_publisher: Uuid,
    /// A root Work with every child row.
    a: Uuid,
    /// A second root Work under the same publisher.
    b: Uuid,
    /// Two chapters of `a`.
    chapter: Uuid,
    second_chapter: Uuid,
    contributor: Uuid,
    contribution: Uuid,
    chapter_contribution: Uuid,
    affiliation: Uuid,
    institution: Uuid,
    other_institution: Uuid,
    publication: Uuid,
    location: Uuid,
    funding: Uuid,
    series: Uuid,
    other_series: Uuid,
    issue: Uuid,
    title: Uuid,
    chapter_title: Uuid,
    second_chapter_title: Uuid,
    abstract_id: Uuid,
    reference: Uuid,
    relation: Uuid,
}

fn setup() -> (support::TestDbGuard, Arc<PgPool>) {
    let guard = support::test_lock();
    let pool = support::db_pool();
    support::reset_db(pool.as_ref()).expect("reset");
    (guard, pool)
}

fn fixture(connection: &mut PgConnection) -> Fixture {
    let id = Uuid::new_v4;
    let f = Fixture {
        publisher: id(),
        imprint: id(),
        other_imprint: id(),
        other_publisher: id(),
        a: id(),
        b: id(),
        chapter: id(),
        second_chapter: id(),
        contributor: id(),
        contribution: id(),
        chapter_contribution: id(),
        affiliation: id(),
        institution: id(),
        other_institution: id(),
        publication: id(),
        location: id(),
        funding: id(),
        series: id(),
        other_series: id(),
        issue: id(),
        title: id(),
        chapter_title: id(),
        second_chapter_title: id(),
        abstract_id: id(),
        reference: id(),
        relation: id(),
    };
    let tag = f.a.simple();
    execute(
        connection,
        &format!(
            "INSERT INTO publisher (publisher_id, publisher_name) VALUES ('{p}', 'Capture {tag}'), ('{op}', 'Other {tag}');
             INSERT INTO imprint (imprint_id, publisher_id, imprint_name) VALUES ('{i}', '{p}', 'Imprint {tag}'), ('{oi}', '{op}', 'Other imprint {tag}');
             INSERT INTO work (work_id, work_type, work_status, edition, imprint_id, doi, publication_date, landing_page)
             VALUES ('{a}', 'monograph', 'active', 1, '{i}', 'https://doi.org/10.12345/a-{tag}', '2026-01-01', 'https://example.org/a'),
                    ('{b}', 'monograph', 'active', 1, '{i}', 'https://doi.org/10.12345/b-{tag}', '2026-01-01', 'https://example.org/b'),
                    ('{c}', 'book-chapter', 'active', NULL, '{i}', 'https://doi.org/10.12345/c-{tag}', '2026-01-01', 'https://example.org/c'),
                    ('{c2}', 'book-chapter', 'active', NULL, '{i}', 'https://doi.org/10.12345/c2-{tag}', '2026-01-01', 'https://example.org/c2');
             INSERT INTO work_relation (work_relation_id, relator_work_id, related_work_id, relation_type, relation_ordinal)
             VALUES ('{rel}', '{a}', '{c}', 'has-child', 1), (gen_random_uuid(), '{c}', '{a}', 'is-child-of', 1),
                    (gen_random_uuid(), '{a}', '{c2}', 'has-child', 2), (gen_random_uuid(), '{c2}', '{a}', 'is-child-of', 2);
             INSERT INTO contributor (contributor_id, last_name, full_name, orcid, website)
             VALUES ('{k}', 'Author', 'An Author', 'https://orcid.org/0000-0002-1825-0097', 'https://example.org/k');
             INSERT INTO contribution (contribution_id, work_id, contributor_id, contribution_type, last_name, full_name, contribution_ordinal)
             VALUES ('{ka}', '{a}', '{k}', 'author', 'Author', 'An Author', 1),
                    ('{kc}', '{c}', '{k}', 'author', 'Author', 'An Author', 1);
             INSERT INTO institution (institution_id, institution_name, country_code) VALUES ('{n}', 'Institution', 'gbr'), ('{on}', 'Other institution', 'gbr');
             INSERT INTO affiliation (affiliation_id, contribution_id, institution_id, affiliation_ordinal, position)
             VALUES ('{af}', '{ka}', '{n}', 1, 'Lecturer');
             INSERT INTO publication (publication_id, publication_type, work_id, isbn) VALUES ('{pub}', 'Paperback', '{a}', '978-3-16-148410-0');
             INSERT INTO location (location_id, publication_id, full_text_url, canonical) VALUES ('{loc}', '{pub}', 'https://example.org/ft', true);
             INSERT INTO funding (funding_id, work_id, institution_id, grant_number) VALUES ('{fu}', '{a}', '{n}', 'G-1');
             INSERT INTO series (series_id, series_type, series_name, imprint_id, issn_print) VALUES ('{s}', 'book-series', 'Series', '{i}', '1234-5678'), ('{os}', 'book-series', 'Other series', '{i}', '8765-4321');
             INSERT INTO issue (issue_id, series_id, work_id, issue_ordinal, issue_number) VALUES ('{is}', '{s}', '{a}', 1, 1);
             INSERT INTO title (title_id, work_id, locale_code, full_title, title, canonical)
             VALUES ('{t}', '{a}', 'en', 'Title', 'Title', true), ('{ct}', '{c}', 'en', 'Chapter', 'Chapter', true), ('{ct2}', '{c2}', 'en', 'Chapter 2', 'Chapter 2', true);
             INSERT INTO abstract (abstract_id, work_id, content, locale_code, abstract_type, canonical) VALUES ('{ab}', '{a}', '<p>Abstract</p>', 'en', 'long', true);
             INSERT INTO reference (reference_id, work_id, reference_ordinal, doi, publication_date, retrieval_date)
             VALUES ('{r}', '{a}', 1, 'https://doi.org/10.12345/ref-{tag}', '2020-01-01', '2020-02-02');",
            p = f.publisher, op = f.other_publisher, i = f.imprint, oi = f.other_imprint, a = f.a, b = f.b, c = f.chapter,
            c2 = f.second_chapter, rel = f.relation, k = f.contributor, ka = f.contribution, kc = f.chapter_contribution,
            n = f.institution, on = f.other_institution, af = f.affiliation, pub = f.publication, loc = f.location,
            fu = f.funding, s = f.series, os = f.other_series, is = f.issue, t = f.title, ct = f.chapter_title,
            ct2 = f.second_chapter_title, ab = f.abstract_id, r = f.reference,
        ),
    );
    assert_clean(connection);
    f
}

fn assert_update(
    connection: &mut PgConnection,
    table: &str,
    key: &str,
    id: Uuid,
    sets: &[&str],
    expected: &BTreeMap<Uuid, i64>,
) {
    for set in sets {
        assert_eq!(
            &deltas(
                connection,
                &format!("UPDATE {table} SET {set} WHERE {key} = '{id}'")
            ),
            expected,
            "{table}: {set}"
        );
    }
}

fn assert_excluded(connection: &mut PgConnection, table: &str, key: &str, id: Uuid, sets: &[&str]) {
    for set in sets {
        assert!(
            deltas(
                connection,
                &format!("UPDATE {table} SET {set} WHERE {key} = '{id}'")
            )
            .is_empty(),
            "{table}: {set} is excluded"
        );
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// T1-T16: one test per section 8.2 table; T17-T34 inside the table they exercise.
// ---------------------------------------------------------------------------------------------------------------------

#[test]
fn t1_t21_t33_work_columns_ownership_and_chapter_propagation() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    assert_update(
        &mut connection,
        "work",
        "work_id",
        f.a,
        &[
            "work_type = 'edited-book'",
            "work_status = 'forthcoming'",
            "doi = 'https://doi.org/10.12345/changed'",
            "edition = 2",
            "publication_date = '2026-02-02'",
            "place = 'Cambridge'",
            "license = 'https://creativecommons.org/licenses/by/4.0/'",
            "landing_page = 'https://example.org/changed'",
        ],
        &once(&[f.a]),
    );
    execute(
        &mut connection,
        &format!(
            "UPDATE work SET work_status = 'active' WHERE work_id = '{}'",
            f.a
        ),
    );
    assert_update(
        &mut connection,
        "work",
        "work_id",
        f.a,
        &["work_status = 'withdrawn', withdrawn_date = '2026-06-01'"],
        &once(&[f.a]),
    );
    // T33: a chapter edit propagates to every HAS_CHILD parent and to no other Work.
    assert_update(
        &mut connection,
        "work",
        "work_id",
        f.chapter,
        &["first_page = '1'", "last_page = '20'"],
        &once(&[f.a, f.chapter]),
    );
    // T21: an ownership move.
    assert_update(
        &mut connection,
        "work",
        "work_id",
        f.b,
        &[&format!("imprint_id = '{}'", f.other_imprint)],
        &once(&[f.b]),
    );
    // Insert and delete of a Work.
    let new_work = Uuid::new_v4();
    assert_eq!(
        deltas(&mut connection, &format!(
            "INSERT INTO work (work_id, work_type, work_status, edition, imprint_id) VALUES ('{new_work}', 'monograph', 'forthcoming', 1, '{}')",
            f.imprint
        )),
        once(&[new_work])
    );
    // Excluded work columns (T37 counterpart for work): checked after the watched ones above.
    assert_excluded(
        &mut connection,
        "work",
        "work_id",
        f.a,
        &[
            "copyright_holder = 'Holder'",
            "general_note = 'note'",
            "bibliography_note = 'note'",
            "page_count = 10",
            "page_breakdown = 'x'",
            "image_count = 1",
            "table_count = 1",
            "audio_count = 1",
            "video_count = 1",
            "toc = 'toc'",
            "lccn = '1'",
            "oclc = '1'",
            "cover_url = 'https://example.org/cover'",
            "cover_caption = 'caption'",
            "reference = 'internal'",
        ],
    );
}

#[test]
fn t2_t17_t18_title_values_canonical_flip_and_move() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    assert_update(
        &mut connection,
        "title",
        "title_id",
        f.title,
        &["title = 'New'", "subtitle = 'Sub'", "full_title = 'Full'"],
        &once(&[f.a]),
    );
    // T17
    assert_update(
        &mut connection,
        "title",
        "title_id",
        f.title,
        &["canonical = false"],
        &once(&[f.a]),
    );
    // T18: both roots in one transaction.
    assert_update(
        &mut connection,
        "title",
        "title_id",
        f.title,
        &[&format!("work_id = '{}'", f.b)],
        &once(&[f.a, f.b]),
    );
    assert_eq!(
        deltas(
            &mut connection,
            &format!("DELETE FROM title WHERE title_id = '{}'", f.title)
        ),
        once(&[f.b])
    );
}

#[test]
fn t3_t19_t20_abstract_values_locale_and_move() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    assert_update(
        &mut connection,
        "abstract",
        "abstract_id",
        f.abstract_id,
        &[
            "content = '<p>New</p>'",
            "abstract_type = 'short'",
            "canonical = false",
        ],
        &once(&[f.a]),
    );
    // T20
    assert_update(
        &mut connection,
        "abstract",
        "abstract_id",
        f.abstract_id,
        &["locale_code = 'fr'"],
        &once(&[f.a]),
    );
    // T19
    assert_update(
        &mut connection,
        "abstract",
        "abstract_id",
        f.abstract_id,
        &[&format!("work_id = '{}'", f.b)],
        &once(&[f.a, f.b]),
    );
}

#[test]
fn t4_t22_t23_contribution_values_move_and_contributor_change() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    assert_update(
        &mut connection,
        "contribution",
        "contribution_id",
        f.contribution,
        &[
            "contribution_type = 'editor'",
            "first_name = 'First'",
            "last_name = 'Last'",
            "full_name = 'Full'",
            "contribution_ordinal = 2",
        ],
        &once(&[f.a]),
    );
    // T22
    assert_update(
        &mut connection,
        "contribution",
        "contribution_id",
        f.contribution,
        &[&format!("work_id = '{}'", f.b)],
        &once(&[f.a, f.b]),
    );
    // T23
    let other = Uuid::new_v4();
    execute(&mut connection, &format!("INSERT INTO contributor (contributor_id, last_name, full_name, orcid) VALUES ('{other}', 'Other', 'Other', 'https://orcid.org/0000-0001-5109-3700')"));
    assert_update(
        &mut connection,
        "contribution",
        "contribution_id",
        f.contribution,
        &[&format!("contributor_id = '{other}'")],
        &once(&[f.b]),
    );
}

#[test]
fn t5_contributor_orcid_fans_out_and_website_is_excluded() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    assert_update(
        &mut connection,
        "contributor",
        "contributor_id",
        f.contributor,
        &[
            "orcid = 'https://orcid.org/0000-0002-9079-593X'",
            "orcid = NULL",
        ],
        &once(&[f.a, f.chapter]),
    );
    assert_excluded(
        &mut connection,
        "contributor",
        "contributor_id",
        f.contributor,
        &[
            "website = 'https://example.org/new'",
            "first_name = 'Changed'",
        ],
    );
}

#[test]
fn t6_t24_t25_t26_affiliation() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    // T26
    assert_update(
        &mut connection,
        "affiliation",
        "affiliation_id",
        f.affiliation,
        &["affiliation_ordinal = 2"],
        &once(&[f.a]),
    );
    // T24
    assert_update(
        &mut connection,
        "affiliation",
        "affiliation_id",
        f.affiliation,
        &[&format!("institution_id = '{}'", f.other_institution)],
        &once(&[f.a]),
    );
    // T25: the chapter root propagates to its parent, which is also the old owner.
    assert_update(
        &mut connection,
        "affiliation",
        "affiliation_id",
        f.affiliation,
        &[&format!("contribution_id = '{}'", f.chapter_contribution)],
        &once(&[f.a, f.chapter]),
    );
}

#[test]
fn t7_institution_values_fan_out_through_affiliation_and_funding() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    // The institution is reached by A's affiliation and A's funding.
    assert_update(
        &mut connection,
        "institution",
        "institution_id",
        f.institution,
        &[
            "institution_name = 'Renamed'",
            "ror = 'https://ror.org/0abcdef12'",
            "institution_doi = 'https://doi.org/10.12345/inst'",
        ],
        &once(&[f.a]),
    );
    execute(
        &mut connection,
        &format!(
            "UPDATE funding SET institution_id = '{}' WHERE funding_id = '{}'",
            f.other_institution, f.funding
        ),
    );
    execute(
        &mut connection,
        &format!(
            "UPDATE affiliation SET contribution_id = '{}' WHERE affiliation_id = '{}'",
            f.chapter_contribution, f.affiliation
        ),
    );
    assert_update(
        &mut connection,
        "institution",
        "institution_id",
        f.institution,
        &["institution_name = 'Chapter'"],
        &once(&[f.a, f.chapter]),
    );
    assert_excluded(
        &mut connection,
        "institution",
        "institution_id",
        f.institution,
        &["country_code = 'fra'"],
    );
}

#[test]
fn t8_t27_publication_values_move_and_dimensions_excluded() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    assert_update(
        &mut connection,
        "publication",
        "publication_id",
        f.publication,
        &[
            "publication_type = 'Hardback'",
            "isbn = '978-0-306-40615-7'",
        ],
        &once(&[f.a]),
    );
    assert_excluded(
        &mut connection,
        "publication",
        "publication_id",
        f.publication,
        &[
            "width_mm = 100, width_in = 3.94",
            "height_mm = 200, height_in = 7.87",
            "depth_mm = 20, depth_in = 0.79",
            "weight_g = 300, weight_oz = 10.58",
        ],
    );
    // T27
    assert_update(
        &mut connection,
        "publication",
        "publication_id",
        f.publication,
        &[&format!("work_id = '{}'", f.b)],
        &once(&[f.a, f.b]),
    );
}

#[test]
fn t9_t28_t29_location_values_platform_and_move() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    assert_update(
        &mut connection,
        "location",
        "location_id",
        f.location,
        &[
            "full_text_url = 'https://example.org/other'",
            "canonical = false",
        ],
        &once(&[f.a]),
    );
    // T29
    assert_update(
        &mut connection,
        "location",
        "location_id",
        f.location,
        &["location_platform = 'OAPEN'"],
        &once(&[f.a]),
    );
    assert_excluded(
        &mut connection,
        "location",
        "location_id",
        f.location,
        &["landing_page = 'https://example.org/landing'"],
    );
    // T28
    let other = Uuid::new_v4();
    execute(&mut connection, &format!("INSERT INTO publication (publication_id, publication_type, work_id) VALUES ('{other}', 'Hardback', '{}')", f.b));
    assert_update(
        &mut connection,
        "location",
        "location_id",
        f.location,
        &[&format!("publication_id = '{other}'")],
        &once(&[f.a, f.b]),
    );
}

#[test]
fn t10_t30_funding_values_and_move() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    assert_update(
        &mut connection,
        "funding",
        "funding_id",
        f.funding,
        &[
            "grant_number = 'G-2'",
            &format!("institution_id = '{}'", f.other_institution),
        ],
        &once(&[f.a]),
    );
    assert_excluded(
        &mut connection,
        "funding",
        "funding_id",
        f.funding,
        &["program = 'Programme'", "project_name = 'Project'"],
    );
    // T30
    assert_update(
        &mut connection,
        "funding",
        "funding_id",
        f.funding,
        &[&format!("work_id = '{}'", f.b)],
        &once(&[f.a, f.b]),
    );
}

#[test]
fn t11_t31_issue_values_and_work_and_series_moves() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    assert_update(
        &mut connection,
        "issue",
        "issue_id",
        f.issue,
        &["issue_number = 2", "issue_ordinal = 2"],
        &once(&[f.a]),
    );
    // T31
    assert_update(
        &mut connection,
        "issue",
        "issue_id",
        f.issue,
        &[&format!("series_id = '{}'", f.other_series)],
        &once(&[f.a]),
    );
    assert_update(
        &mut connection,
        "issue",
        "issue_id",
        f.issue,
        &[&format!("work_id = '{}'", f.b)],
        &once(&[f.a, f.b]),
    );
}

#[test]
fn t12_series_values_and_imprint_excluded() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    assert_update(
        &mut connection,
        "series",
        "series_id",
        f.series,
        &[
            "series_name = 'Renamed'",
            "issn_print = '1111-2222'",
            "issn_digital = '3333-4444'",
        ],
        &once(&[f.a]),
    );
    assert_excluded(
        &mut connection,
        "series",
        "series_id",
        f.series,
        &[
            &format!("imprint_id = '{}'", f.other_imprint),
            "series_url = 'https://example.org/s'",
        ],
    );
    assert_eq!(
        deltas(
            &mut connection,
            &format!(
                "UPDATE series SET series_name = 'Unused' WHERE series_id = '{}'",
                f.other_series
            )
        ),
        BTreeMap::new()
    );
}

#[test]
fn t13_imprint_values_and_publisher_move_fan_out() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    let all = once(&[f.a, f.b, f.chapter, f.second_chapter]);
    assert_update(
        &mut connection,
        "imprint",
        "imprint_id",
        f.imprint,
        &[
            "crossmark_doi = 'https://doi.org/10.12345/crossmark'",
            &format!("publisher_id = '{}'", f.other_publisher),
        ],
        &all,
    );
    assert_excluded(
        &mut connection,
        "imprint",
        "imprint_id",
        f.imprint,
        &[
            "imprint_url = 'https://example.org/i'",
            "imprint_name = 'Renamed'",
        ],
    );
}

#[test]
fn t14_publisher_name_fans_out_and_other_columns_are_excluded() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    assert_update(
        &mut connection,
        "publisher",
        "publisher_id",
        f.publisher,
        &["publisher_name = 'Renamed'"],
        &once(&[f.a, f.b, f.chapter, f.second_chapter]),
    );
    assert_excluded(
        &mut connection,
        "publisher",
        "publisher_id",
        f.publisher,
        &[
            "publisher_url = 'https://example.org/p'",
            "publisher_shortname = 'P'",
        ],
    );
}

#[test]
fn t15_t32_reference_values_publication_date_and_move() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    assert_update(
        &mut connection,
        "reference",
        "reference_id",
        f.reference,
        &[
            "doi = 'https://doi.org/10.12345/other-ref'",
            "unstructured_citation = 'Citation'",
            "journal_title = 'Journal'",
            "article_title = 'Article'",
            "author = 'Author'",
            "reference_ordinal = 2",
        ],
        &once(&[f.a]),
    );
    // T32
    assert_update(
        &mut connection,
        "reference",
        "reference_id",
        f.reference,
        &["publication_date = '2021-01-01'"],
        &once(&[f.a]),
    );
    assert_update(
        &mut connection,
        "reference",
        "reference_id",
        f.reference,
        &[&format!("work_id = '{}'", f.b)],
        &once(&[f.a, f.b]),
    );
}

#[test]
fn t16_t34_work_relation_ordinal_and_re_parenting_dirty_every_endpoint() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    assert_update(
        &mut connection,
        "work_relation",
        "work_relation_id",
        f.relation,
        &["relation_ordinal = 5"],
        &once(&[f.a, f.chapter]),
    );
    // T34: re-parenting the pair from A to B dirties all four endpoints (old A, old C, new B, new C) once.
    assert_eq!(
        deltas(&mut connection, &format!(
            "UPDATE work_relation SET relator_work_id = '{b}' WHERE work_relation_id = '{r}';
             UPDATE work_relation SET related_work_id = '{b}' WHERE relator_work_id = '{c}' AND relation_type = 'is-child-of'",
            b = f.b, r = f.relation, c = f.chapter
        )),
        once(&[f.a, f.b, f.chapter])
    );
    // A later chapter edit now propagates to B and not to A.
    assert_update(
        &mut connection,
        "title",
        "title_id",
        f.chapter_title,
        &["title = 'Moved'"],
        &once(&[f.b, f.chapter]),
    );
}

// ---------------------------------------------------------------------------------------------------------------------
// T35-T37: capture negatives, each after the table's watched effects above.
// ---------------------------------------------------------------------------------------------------------------------

#[test]
fn t35_t36_t37_uncaptured_tables_and_excluded_columns() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    let biography = Uuid::new_v4();
    for statement in [
        format!("INSERT INTO subject (work_id, subject_type, subject_code, subject_ordinal) VALUES ('{}', 'bic', 'AAA', 1)", f.a),
        format!("INSERT INTO language (work_id, language_code, language_relation) VALUES ('{}', 'eng', 'original')", f.a),
        format!("INSERT INTO price (publication_id, currency_code, unit_price) VALUES ('{}', 'gbp', 10)", f.publication),
        format!("INSERT INTO award (work_id, title) VALUES ('{}', 'Award')", f.a),
        format!("INSERT INTO endorsement (work_id, author_name) VALUES ('{}', 'Endorser')", f.a),
        format!("INSERT INTO biography (biography_id, contribution_id, content, locale_code) VALUES ('{biography}', '{}', 'Bio', 'en')", f.contribution),
        format!("UPDATE biography SET content = 'Changed' WHERE biography_id = '{biography}'"),
        format!("INSERT INTO book_review (work_id) VALUES ('{}')", f.a),
        format!("INSERT INTO additional_resource (work_id, title, resource_type) VALUES ('{}', 'Resource', 'AUDIO')", f.a),
        format!("INSERT INTO contact (publisher_id, email) VALUES ('{}', 'x@example.org')", f.publisher),
        format!("INSERT INTO work_featured_video (work_id, title) VALUES ('{}', 'Video')", f.a),
        format!("UPDATE subject SET subject_code = 'BBB' WHERE work_id = '{}'", f.a),
        format!("DELETE FROM language WHERE work_id = '{}'", f.a),
    ] {
        assert!(deltas(&mut connection, &statement).is_empty(), "{statement}");
    }
    assert_excluded(
        &mut connection,
        "contributor",
        "contributor_id",
        f.contributor,
        &["website = 'https://example.org/x'"],
    );
    assert_excluded(
        &mut connection,
        "institution",
        "institution_id",
        f.institution,
        &["country_code = 'fra'"],
    );
    assert_excluded(
        &mut connection,
        "series",
        "series_id",
        f.series,
        &[&format!("imprint_id = '{}'", f.other_imprint)],
    );
}

#[test]
fn t35_whole_table_triggers_over_capture_position_and_retrieval_date() {
    // affiliation.position and reference.retrieval_date are not payload dependencies, but their tables' triggers are
    // whole-table (section 8.2 lists no column list for affiliation and reference), so an update of either column
    // advances the owner. Over-capture is permitted by section 8.1; recorded here so a change is visible.
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    assert_update(
        &mut connection,
        "affiliation",
        "affiliation_id",
        f.affiliation,
        &["position = 'Professor'"],
        &once(&[f.a]),
    );
    assert_update(
        &mut connection,
        "reference",
        "reference_id",
        f.reference,
        &["retrieval_date = '2022-01-01'"],
        &once(&[f.a]),
    );
}

#[test]
fn t39_capture_continues_while_both_flags_are_off() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    assert_eq!(
        text(&mut connection, "SELECT string_agg(capture_enabled::text || '|' || execution_enabled::text, ',') AS value FROM work_upsert_control"),
        "false|false"
    );
    assert_update(
        &mut connection,
        "work",
        "work_id",
        f.a,
        &["place = 'York'"],
        &once(&[f.a]),
    );
    assert_update(
        &mut connection,
        "title",
        "title_id",
        f.chapter_title,
        &["title = 'Off'"],
        &once(&[f.a, f.chapter]),
    );
}

// ---------------------------------------------------------------------------------------------------------------------
// The flush (section 8.6), T317-T324 against this implementation's migration.
// ---------------------------------------------------------------------------------------------------------------------

#[test]
fn t317_savepoints_and_subtransactions() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    let (a, b, c) = (f.a, f.b, f.chapter);
    let edit = |work: Uuid, place: &str| {
        format!("UPDATE work SET place = '{place}' WHERE work_id = '{work}'")
    };

    // 1. an edit, then a rolled-back savepoint holding a second edit
    assert_eq!(
        deltas(
            &mut connection,
            &format!(
                "{}; SAVEPOINT s; {}; ROLLBACK TO SAVEPOINT s",
                edit(a, "1"),
                edit(b, "1")
            )
        ),
        once(&[a])
    );
    // 2. a rolled-back savepoint holding the transaction's first edit, then a second: the re-arming case
    assert_eq!(
        deltas(
            &mut connection,
            &format!(
                "SAVEPOINT s; {}; ROLLBACK TO SAVEPOINT s; {}",
                edit(a, "2"),
                edit(b, "2")
            )
        ),
        once(&[b])
    );
    // 3. a released savepoint
    assert_eq!(
        deltas(
            &mut connection,
            &format!("SAVEPOINT s; {}; RELEASE SAVEPOINT s", edit(a, "3"))
        ),
        once(&[a])
    );
    // 4. nested savepoints with the inner one rolled back
    assert_eq!(
        deltas(&mut connection, &format!(
            "SAVEPOINT outer_s; {}; SAVEPOINT inner_s; {}; ROLLBACK TO SAVEPOINT inner_s; RELEASE SAVEPOINT outer_s",
            edit(a, "4"), edit(b, "4")
        )),
        once(&[a])
    );
    // 5. a caught plpgsql exception whose block edited, followed by an edit outside it
    assert_eq!(
        deltas(&mut connection, &format!(
            "DO $$ BEGIN BEGIN {}; RAISE EXCEPTION 'undo'; EXCEPTION WHEN raise_exception THEN NULL; END; {}; END $$",
            edit(b, "5"), edit(c, "5")
        )),
        once(&[a, c])
    );
    // 5'. a caught exception whose block held the only edit leaves nothing
    assert!(deltas(&mut connection, &format!(
        "DO $$ BEGIN BEGIN {}; RAISE EXCEPTION 'undo'; EXCEPTION WHEN raise_exception THEN NULL; END; END $$",
        edit(b, "6")
    )).is_empty());
}

fn t318_probe(connection: &mut PgConnection, a: Uuid, before: i64) -> (i64, String) {
    let now: i64 = text(connection, &format!("SELECT source_generation::text AS value FROM work_upsert_generation WHERE work_id = '{a}'"))
        .parse()
        .expect("generation");
    let arming = text(
        connection,
        "SELECT count(*)::text AS value FROM work_upsert_capture_queue WHERE entry_kind = 'ARM'",
    );
    (now - before, arming)
}

const T318_SHAPES: [(&str, &str, &str); 4] = [
    ("between two edits", "", "SET CONSTRAINTS ALL IMMEDIATE"),
    ("before any edit", "SET CONSTRAINTS ALL IMMEDIATE", ""),
    (
        "twice",
        "SET CONSTRAINTS ALL IMMEDIATE",
        "SET CONSTRAINTS ALL IMMEDIATE",
    ),
    (
        "named on the flush",
        "",
        "SET CONSTRAINTS public.work_upsert_capture_flush IMMEDIATE",
    ),
];

/// A shape's name, its in-transaction probe (written generations, queued arming rows) and its committed deltas.
type T318Result = (&'static str, (i64, String), BTreeMap<Uuid, i64>);

/// Returns, per shape, the in-transaction probe (written generations, queued arming rows) and the committed deltas.
fn t318_run(connection: &mut PgConnection, f: &Fixture) -> Vec<T318Result> {
    let mut results = Vec::new();
    for (name, first, second) in T318_SHAPES {
        let before = generations(connection);
        let a_before = before[&f.a];
        execute(connection, "BEGIN");
        if !first.is_empty() {
            execute(connection, first);
        }
        execute(
            connection,
            &format!("UPDATE work SET place = '{name}' WHERE work_id = '{}'", f.a),
        );
        if !second.is_empty() {
            execute(connection, second);
        }
        let probe = t318_probe(connection, f.a, a_before);
        execute(
            connection,
            &format!(
                "UPDATE work SET place = '{name}' WHERE work_id = '{}'; COMMIT",
                f.b
            ),
        );
        let after = generations(connection);
        results.push((name, probe, changed(&before, &after)));
    }
    results
}

#[test]
fn t318_forced_immediacy_writes_no_generation_before_completion() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    for (name, probe, committed) in t318_run(&mut connection, &f) {
        assert_eq!(
            probe,
            (0, "1".to_string()),
            "{name}: 0 written, 1 arming row still queued"
        );
        assert_eq!(
            committed,
            once(&[f.a, f.b]),
            "{name}: both edits applied at completion"
        );
        assert_eq!(queue_rows(&mut connection), "0", "{name}: no queue residue");
    }
    assert_clean(&mut connection);
}

#[test]
fn t318_negative_control_without_the_immediacy_defence_flushes_early() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    let variant = FlushVariant::without(IMMEDIACY_DEFENCE, "IF false THEN");
    let results = t318_run(&mut connection, &f);
    drop(variant);
    // "between two edits": the forcing fires the queued arming row at once.
    let (name, probe, _) = &results[0];
    assert_eq!(
        probe,
        &(1, "0".to_string()),
        "{name}: 1 written, 0 arming rows queued"
    );
    // "twice" and "named on the flush" force after the edit as well.
    for (name, probe, _) in &results[2..] {
        assert_eq!(probe, &(1, "0".to_string()), "{name}: flushed early");
    }
}

/// T319's schedule. Returns (first commits, second commits, generation rows locked while the first waited, the
/// second session's ungranted locks when it was seen waiting).
fn t319_schedule(
    pool: &PgPool,
    f: &Fixture,
) -> (Result<(), String>, Result<(), String>, bool, String) {
    let mut first = dedicated();
    let mut observer = pool.get().expect("observer");
    execute(&mut first, &format!(
        "BEGIN; UPDATE title SET title = 'First' WHERE title_id = '{}'; SET CONSTRAINTS ALL IMMEDIATE;",
        f.chapter_title
    ));
    // The first waits here: is any generation row locked?
    let locked = [f.a, f.chapter, f.second_chapter]
        .iter()
        .any(|work| generation_row_locked(&mut observer, *work));
    let mut second = dedicated();
    let second_pid = backend_pid(&mut second);
    let second_title = f.second_chapter_title;
    let second_thread = std::thread::spawn(move || {
        second
            .batch_execute(&format!("BEGIN; UPDATE title SET title = 'Second' WHERE title_id = '{second_title}'; COMMIT;"))
            .map_err(|error| error.to_string())
    });
    // Synchronisation point: the second has either committed or is waiting on a lock.
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut transcript = String::new();
    while !second_thread.is_finished() {
        let waiting = text(&mut observer, &format!(
            "SELECT count(*)::text AS value FROM pg_stat_activity WHERE pid = {second_pid} AND wait_event_type = 'Lock'"
        ));
        if waiting == "1" {
            transcript = ungranted(&mut observer, &second_pid);
            break;
        }
        assert!(
            Instant::now() < deadline,
            "the second session neither committed nor waited"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    let first_result = first
        .batch_execute(&format!(
            "UPDATE title SET title = 'First again' WHERE title_id = '{}'; COMMIT;",
            f.second_chapter_title
        ))
        .map_err(|error| error.to_string());
    if first_result.is_err() {
        first.batch_execute("ROLLBACK").ok();
    }
    let second_result = second_thread.join().expect("second thread");
    (first_result, second_result, locked, transcript)
}

#[test]
fn t319_concurrent_forced_immediacy_commits_both() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    let before = generations(&mut connection);
    let (first, second, locked, transcript) = t319_schedule(pool.as_ref(), &f);
    assert_eq!(first, Ok(()), "the first commits");
    assert_eq!(second, Ok(()), "the second commits");
    assert!(!locked, "no generation row is locked while the first waits");
    assert_eq!(transcript, "", "the second never waited");
    let after = generations(&mut connection);
    let delta = changed(&before, &after);
    assert_eq!(
        delta.get(&f.a),
        Some(&2),
        "the parent is advanced once per transaction"
    );
    assert_eq!(delta.get(&f.chapter), Some(&1));
    assert_eq!(delta.get(&f.second_chapter), Some(&2));
    assert_clean(&mut connection);
}

#[test]
fn t319_negative_control_without_the_defence_deadlocks() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    let variant = FlushVariant::without(IMMEDIACY_DEFENCE, "IF false THEN");
    let (first, second, locked, transcript) = t319_schedule(pool.as_ref(), &f);
    drop(variant);
    assert!(
        locked,
        "the early flush holds the generation rows while the first waits"
    );
    assert!(
        transcript.contains("tuple") || transcript.contains("transactionid"),
        "the second waited: {transcript}"
    );
    let aborted: Vec<&String> = [&first, &second]
        .into_iter()
        .filter_map(|r| r.as_ref().err())
        .collect();
    assert_eq!(
        aborted.len(),
        1,
        "exactly one aborted: {first:?} {second:?}"
    );
    assert!(aborted[0].contains("deadlock detected"), "{}", aborted[0]);
    assert_clean(&mut connection);
}

#[test]
fn t320_one_transaction_that_edits_and_deletes_in_both_orders() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    // A chapter title edit, then the parent's deletion.
    let delta = deltas(&mut connection, &format!(
        "UPDATE title SET title = 'Edited' WHERE title_id = '{}'; DELETE FROM work WHERE work_id = '{}'",
        f.chapter_title, f.a
    ));
    assert!(
        !generations(&mut connection).contains_key(&f.a),
        "the deleted Work keeps no generation row"
    );
    assert_eq!(
        delta,
        once(&[f.chapter, f.second_chapter]),
        "the surviving chapters are advanced once"
    );

    // The deletion, then a chapter Work edit.
    support::reset_db(pool.as_ref()).expect("reset");
    let g = fixture(&mut connection);
    let delta = deltas(&mut connection, &format!(
        "DELETE FROM work WHERE work_id = '{}'; UPDATE work SET first_page = '9' WHERE work_id = '{}'",
        g.a, g.chapter
    ));
    assert!(!generations(&mut connection).contains_key(&g.a));
    assert_eq!(delta, once(&[g.chapter, g.second_chapter]));
}

/// T321's schedule: the deleter pauses inside its own completion, after its flush, on an advisory lock this test
/// holds; the writer, holding no Work row, reaches its own flush for the same root. Returns the writer's ungranted
/// locks while it waited.
fn t321_schedule(pool: &PgPool, f: &Fixture) -> String {
    let mut control = dedicated();
    let mut observer = pool.get().expect("observer");
    let key = 1_948_579_321_i64;
    execute(&mut control, &format!("SELECT pg_advisory_lock({key})"));

    let mut deleter = dedicated();
    let deleter_pid = backend_pid(&mut deleter);
    execute(&mut deleter, &format!(
        "CREATE TEMP TABLE be06_pause (x int);
         CREATE FUNCTION pg_temp.be06_pause() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_advisory_xact_lock({key}); RETURN NULL; END $$;
         CREATE CONSTRAINT TRIGGER be06_pause AFTER INSERT ON be06_pause DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION pg_temp.be06_pause();"
    ));
    let b = f.b;
    let deleter_thread = std::thread::spawn(move || {
        // The deletion arms the flush first, so the flush fires before the pause at completion.
        deleter
            .batch_execute(&format!("BEGIN; DELETE FROM work WHERE work_id = '{b}'; INSERT INTO be06_pause VALUES (1); COMMIT;"))
            .map_err(|error| error.to_string())
    });
    wait_for_lock_wait(&mut observer, &deleter_pid);
    assert_eq!(
        ungranted(&mut observer, &deleter_pid),
        "advisory:ExclusiveLock",
        "the deleter is paused after its flush"
    );

    // The writer holds no Work row. Every released editorial statement locks its capture owners before capture fires
    // (drift guard 2), so such a statement waits at that Work row, before its flush, and then resolves no owner. This
    // writer records B as an owner directly, as a capture does, and so reaches its own flush for B.
    let mut writer = dedicated();
    let writer_pid = backend_pid(&mut writer);
    let writer_thread = std::thread::spawn(move || {
        writer
            .batch_execute(&format!(
                "BEGIN; INSERT INTO work_upsert_capture_queue (entry_kind, work_ids) VALUES ('OWNERS', ARRAY['{b}'::uuid]);
                 INSERT INTO work_upsert_capture_queue (entry_kind) VALUES ('ARM'); COMMIT;"
            ))
            .map_err(|error| error.to_string())
    });
    wait_for_lock_wait(&mut observer, &writer_pid);
    let transcript = ungranted(&mut observer, &writer_pid);
    execute(&mut control, &format!("SELECT pg_advisory_unlock({key})"));
    assert_eq!(deleter_thread.join().expect("deleter"), Ok(()));
    assert_eq!(writer_thread.join().expect("writer"), Ok(()));
    transcript
}

fn t321_fixture(connection: &mut PgConnection) -> Fixture {
    let f = fixture(connection);
    assert!(generations(connection).contains_key(&f.b));
    f
}

#[test]
fn t321_the_vanished_root_race_leaves_no_orphan() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = t321_fixture(&mut connection);
    let transcript = t321_schedule(pool.as_ref(), &f);
    assert_eq!(
        transcript, "transactionid:ShareLock",
        "the writer waits on the deleter"
    );
    assert!(!generations(&mut connection).contains_key(&f.b));
    assert_clean(&mut connection);

    // Recorded in the same row: a direct INSERT for a Work that does not exist is accepted (section 8.7)...
    let ghost = Uuid::new_v4();
    execute(&mut connection, &format!("INSERT INTO work_upsert_generation (work_id, execution_profile, source_generation) VALUES ('{ghost}', 'CROSSREF', 1)"));
    assert_eq!(orphans(&mut connection), "1");
    execute(
        &mut connection,
        &format!("DELETE FROM work_upsert_generation WHERE work_id = '{ghost}'"),
    );
    // ...while a Work deleted through a supported path still loses its row.
    execute(
        &mut connection,
        &format!("DELETE FROM work WHERE work_id = '{}'", f.a),
    );
    assert!(!generations(&mut connection).contains_key(&f.a));
    assert_clean(&mut connection);
}

#[test]
fn t321_negative_control_without_the_vanished_root_cleanup_leaves_one_orphan() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = t321_fixture(&mut connection);
    let variant = FlushVariant::without(VANISHED_ROOT_CLEANUP, "AND false");
    let transcript = t321_schedule(pool.as_ref(), &f);
    drop(variant);
    assert_eq!(transcript, "transactionid:ShareLock");
    assert_eq!(orphans(&mut connection), "1", "exactly one orphan");
    assert!(generations(&mut connection).contains_key(&f.b));
    execute(
        &mut connection,
        &format!(
            "DELETE FROM work_upsert_generation WHERE work_id = '{}'",
            f.b
        ),
    );
}

#[test]
fn t322_no_queue_residue_after_rollback_statement_error_or_commit() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    let before = generations(&mut connection);
    execute(
        &mut connection,
        &format!(
            "BEGIN; UPDATE work SET place = 'Rolled back' WHERE work_id = '{}'; ROLLBACK;",
            f.a
        ),
    );
    assert_eq!(generations(&mut connection), before);
    assert_clean(&mut connection);

    execute(
        &mut connection,
        &format!(
            "BEGIN; UPDATE work SET place = 'Errored' WHERE work_id = '{}'",
            f.a
        ),
    );
    assert!(connection.batch_execute("SELECT 1 / 0").is_err());
    execute(&mut connection, "ROLLBACK");
    assert_eq!(generations(&mut connection), before);
    assert_clean(&mut connection);

    // A failed autocommit statement after capture fired inside it.
    assert!(connection
        .batch_execute(&format!(
            "UPDATE work SET place = 'x', edition = 0 WHERE work_id = '{}'",
            f.a
        ))
        .is_err());
    assert_eq!(generations(&mut connection), before);
    assert_clean(&mut connection);

    assert_eq!(
        deltas(
            &mut connection,
            &format!(
                "UPDATE work SET place = 'Committed' WHERE work_id = '{}'",
                f.a
            )
        ),
        once(&[f.a])
    );
}

#[test]
fn t324_truncate_work_cascade_replica_mode_and_the_harness_reset() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    fixture(&mut connection);
    // With triggers enabled: refused by the released permit guard the cascade reaches.
    let refused = connection
        .batch_execute("TRUNCATE work CASCADE")
        .expect_err("refused");
    assert!(
        refused
            .to_string()
            .contains("CROSSREF_PERMIT_DELETE_REFUSED"),
        "{refused}"
    );
    // Under replica no trigger fires, and generation rows survive a truncation of work (section 8.7).
    let mut replica = dedicated();
    let rows = text(
        &mut replica,
        "SELECT count(*)::text AS value FROM work_upsert_generation",
    );
    assert_ne!(rows, "0");
    execute(
        &mut replica,
        "BEGIN; SET LOCAL session_replication_role = replica; TRUNCATE work CASCADE; COMMIT;",
    );
    assert_eq!(
        text(
            &mut replica,
            "SELECT count(*)::text AS value FROM work_upsert_generation"
        ),
        rows
    );
    assert_eq!(orphans(&mut replica), rows);
    // The released harness's reset truncates every public table, so it leaves none.
    support::reset_db(pool.as_ref()).expect("reset");
    assert_eq!(
        text(
            &mut connection,
            "SELECT count(*)::text AS value FROM work_upsert_generation"
        ),
        "0"
    );
    assert_clean(&mut connection);
}

// ---------------------------------------------------------------------------------------------------------------------
// Section 10.5: the two released facts the flush's acyclicity rests on, as drift guards.
// ---------------------------------------------------------------------------------------------------------------------

#[test]
fn drift_guard_1_the_released_relation_trigger_locks_both_endpoints() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    let definition = text(
        &mut connection,
        "SELECT pg_get_functiondef('public.work_relation_work_updated_at_with_relations()'::regprocedure) AS value",
    );
    assert!(
        definition.contains("LEAST") && definition.contains("GREATEST"),
        "{definition}"
    );
    let mut holder = dedicated();
    execute(
        &mut holder,
        &format!(
            "BEGIN; UPDATE work_relation SET relation_ordinal = 7 WHERE work_relation_id = '{}'",
            f.relation
        ),
    );
    assert!(
        work_row_locked(&mut connection, f.a),
        "the relator endpoint is locked"
    );
    assert!(
        work_row_locked(&mut connection, f.chapter),
        "the related endpoint is locked"
    );
    assert!(!work_row_locked(&mut connection, f.b));
    execute(&mut holder, "ROLLBACK");
}

#[test]
fn drift_guard_2_every_capture_owner_is_locked_by_the_statement_that_fired_it() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let f = fixture(&mut connection);
    let other_publication = Uuid::new_v4();
    execute(&mut connection, &format!("INSERT INTO publication (publication_id, publication_type, work_id) VALUES ('{other_publication}', 'Hardback', '{}')", f.b));
    // (statement, the owners the capture resolves)
    let cases: Vec<(String, Vec<Uuid>)> = vec![
        (format!("UPDATE work SET place = 'x' WHERE work_id = '{}'", f.a), vec![f.a]),
        (format!("UPDATE title SET work_id = '{}' WHERE title_id = '{}'", f.b, f.title), vec![f.a, f.b]),
        (format!("UPDATE abstract SET work_id = '{}' WHERE abstract_id = '{}'", f.b, f.abstract_id), vec![f.a, f.b]),
        (format!("UPDATE contribution SET work_id = '{}' WHERE contribution_id = '{}'", f.b, f.contribution), vec![f.a, f.b]),
        (format!("UPDATE contributor SET orcid = NULL WHERE contributor_id = '{}'", f.contributor), vec![f.a, f.chapter]),
        (format!("UPDATE affiliation SET contribution_id = '{}' WHERE affiliation_id = '{}'", f.chapter_contribution, f.affiliation), vec![f.a, f.chapter]),
        (format!("UPDATE institution SET institution_name = 'x' WHERE institution_id = '{}'", f.institution), vec![f.a]),
        (format!("UPDATE publication SET work_id = '{}' WHERE publication_id = '{}'", f.b, f.publication), vec![f.a, f.b]),
        (format!("UPDATE location SET publication_id = '{other_publication}' WHERE location_id = '{}'", f.location), vec![f.a, f.b]),
        (format!("UPDATE funding SET work_id = '{}' WHERE funding_id = '{}'", f.b, f.funding), vec![f.a, f.b]),
        (format!("UPDATE issue SET work_id = '{}' WHERE issue_id = '{}'", f.b, f.issue), vec![f.a, f.b]),
        (format!("UPDATE series SET series_name = 'x' WHERE series_id = '{}'", f.series), vec![f.a]),
        (format!("UPDATE imprint SET crossmark_doi = 'https://doi.org/10.12345/x' WHERE imprint_id = '{}'", f.imprint), vec![f.a, f.b, f.chapter, f.second_chapter]),
        (format!("UPDATE publisher SET publisher_name = 'x' WHERE publisher_id = '{}'", f.publisher), vec![f.a, f.b, f.chapter, f.second_chapter]),
        (format!("UPDATE reference SET work_id = '{}' WHERE reference_id = '{}'", f.b, f.reference), vec![f.a, f.b]),
        (format!("UPDATE work_relation SET relation_ordinal = 9 WHERE work_relation_id = '{}'", f.relation), vec![f.a, f.chapter]),
        (format!("INSERT INTO title (work_id, locale_code, full_title, title, canonical) VALUES ('{}', 'fr', 'T', 'T', false)", f.b), vec![f.b]),
        (format!("DELETE FROM reference WHERE reference_id = '{}'", f.reference), vec![f.a]),
        (format!("DELETE FROM work WHERE work_id = '{}'", f.b), vec![f.b]),
    ];
    let mut tables = BTreeSet::new();
    for (statement, owners) in cases {
        let mut holder = dedicated();
        execute(&mut holder, &format!("BEGIN; {statement}"));
        let queued = text(
            &mut holder,
            "SELECT coalesce(string_agg(DISTINCT u::text, ',' ORDER BY u::text), '') AS value \
             FROM work_upsert_capture_queue q, unnest(q.work_ids) u WHERE q.entry_kind IN ('OWNERS', 'DELETED')",
        );
        let mut expected: Vec<String> = owners.iter().map(Uuid::to_string).collect();
        expected.sort();
        expected.dedup();
        assert_eq!(
            queued,
            expected.join(","),
            "{statement}: the capture's owners"
        );
        for owner in &owners {
            assert!(
                work_row_locked(&mut connection, *owner),
                "{statement}: owner {owner} is not locked by the released statement"
            );
        }
        execute(&mut holder, "ROLLBACK");
        tables.insert(
            statement
                .split_whitespace()
                .nth(if statement.starts_with("UPDATE") {
                    1
                } else {
                    2
                })
                .unwrap_or_default()
                .to_string(),
        );
    }
    // Every one of the sixteen captured tables is covered.
    let captured: BTreeSet<String> = text(
        &mut connection,
        "SELECT string_agg(DISTINCT tgrelid::regclass::text, ',') AS value FROM pg_trigger WHERE tgname = 'work_upsert_capture'",
    )
    .split(',')
    .map(str::to_string)
    .collect();
    assert_eq!(captured.len(), 16);
    assert!(
        captured.is_subset(&tables),
        "uncovered: {:?}",
        captured.difference(&tables).collect::<Vec<_>>()
    );
}
