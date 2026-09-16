#![cfg(feature = "backend")]

//! `BE-06` Work deletion and editorial capture on the Alternative A schema: R52B section 25.23 T325 (T303's four
//! schedules and the rolled-back deletion), T326 (K1, K2, K4, K5, K6s, K6c and D2b), T327 (S01-S13) and T328 (the
//! `v1.10.0` deletion units against a held chapter edit), run against this implementation's migration with the
//! released statement shapes (R52B T306) and the implementation's own deletion units.
//!
//! Every schedule holds one transaction at a named pause point inside its open transaction — between the released
//! trigger and capture (an `AFTER` trigger named to sort between them), or after the row lock and before the `BEFORE`
//! triggers, or at commit after the flush — starts the other, reads the wait from `pg_locks`, releases, and requires
//! both to commit with no `40P01`, no orphaned generation row and no queue residue. Nothing here contacts Crossref.

#[allow(dead_code)]
mod support;

use std::sync::Arc;

use diesel::connection::SimpleConnection;
use diesel::sql_types::Text;
use diesel::{Connection, PgConnection, QueryableByName, RunQueryDsl};
use thoth_api::db::PgPool;
use uuid::Uuid;

const PAUSE_NAMESPACE: i64 = 1_948_579_200;

#[derive(QueryableByName)]
struct TextRow {
    #[diesel(sql_type = Text)]
    value: String,
}

fn session() -> PgConnection {
    PgConnection::establish(&support::test_db_url()).expect("session")
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

fn wait_until(mut condition: impl FnMut() -> bool) {
    for _ in 0..600 {
        if condition() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("the expected wait was not observed");
}

/// Where the first transaction pauses.
#[derive(Clone, Copy)]
enum Point {
    /// After the released row trigger has locked its Works and before capture: `AFTER <event> ON <table>`.
    BeforeCapture(&'static str, &'static str),
    /// After the row lock and before the released `BEFORE` triggers: `BEFORE <event> ON <table>`.
    BeforeTriggers(&'static str, &'static str),
}

struct Pause {
    controller: PgConnection,
    trigger: String,
    table: &'static str,
    name: String,
    held: bool,
}

impl Pause {
    fn install(name: &str, point: Point) -> Self {
        let (trigger, timing, table) = match point {
            Point::BeforeCapture(event, table) => (
                format!("t_be06_pause_{name}"),
                format!("AFTER {event}"),
                table,
            ),
            Point::BeforeTriggers(event, table) => (
                format!("a_be06_pause_{name}"),
                format!("BEFORE {event}"),
                table,
            ),
        };
        let mut controller = session();
        execute(
            &mut controller,
            &format!(
                "CREATE SCHEMA IF NOT EXISTS be06_test;
                 CREATE TABLE IF NOT EXISTS be06_test.deletion_pause_arm (point text PRIMARY KEY);
                 CREATE OR REPLACE FUNCTION be06_test.deletion_pause() RETURNS trigger LANGUAGE plpgsql AS $$
                 DECLARE armed boolean;
                 BEGIN
                     SELECT true INTO armed FROM be06_test.deletion_pause_arm WHERE point = TG_ARGV[0] FOR UPDATE SKIP LOCKED;
                     IF armed THEN
                         DELETE FROM be06_test.deletion_pause_arm WHERE point = TG_ARGV[0];
                         PERFORM pg_advisory_xact_lock({PAUSE_NAMESPACE}, hashtext(TG_ARGV[0]));
                     END IF;
                     IF TG_OP = 'DELETE' THEN RETURN OLD; END IF;
                     RETURN NEW;
                 END $$;
                 SELECT pg_advisory_lock({PAUSE_NAMESPACE}, hashtext('{name}'));
                 INSERT INTO be06_test.deletion_pause_arm (point) VALUES ('{name}') ON CONFLICT DO NOTHING;
                 CREATE TRIGGER {trigger} {timing} ON public.{table} FOR EACH ROW EXECUTE FUNCTION be06_test.deletion_pause('{name}');"
            ),
        );
        Pause {
            controller,
            trigger,
            table,
            name: name.to_string(),
            held: true,
        }
    }

    fn release(&mut self) {
        if self.held {
            execute(
                &mut self.controller,
                &format!(
                    "SELECT pg_advisory_unlock({PAUSE_NAMESPACE}, hashtext('{}'))",
                    self.name
                ),
            );
            self.held = false;
        }
    }
}

impl Drop for Pause {
    fn drop(&mut self) {
        self.release();
        let _ = self.controller.batch_execute(&format!(
            "DROP TRIGGER IF EXISTS {} ON public.{}; DELETE FROM be06_test.deletion_pause_arm WHERE point = '{}';",
            self.trigger, self.table, self.name
        ));
    }
}

fn paused(observer: &mut PgConnection) -> bool {
    text(observer, &format!(
        "SELECT count(*)::text AS value FROM pg_locks WHERE locktype = 'advisory' AND NOT granted AND classid = {PAUSE_NAMESPACE}"
    )) != "0"
}

fn lock_waits(observer: &mut PgConnection) -> String {
    text(observer, &format!(
        "SELECT coalesce(string_agg(locktype || ':' || mode, ',' ORDER BY locktype, mode), '') AS value FROM pg_locks \
         WHERE NOT granted AND NOT (locktype = 'advisory' AND classid = {PAUSE_NAMESPACE})"
    ))
}

type Action = Box<dyn FnOnce(&PgPool) -> Result<(), String> + Send>;

/// A released editorial statement shape, as one transaction.
fn sql(statements: impl Into<String>) -> Action {
    let statements = statements.into();
    Box::new(move |_pool| {
        session()
            .batch_execute(&format!("BEGIN; {statements}; COMMIT;"))
            .map_err(|error| error.to_string())
    })
}

/// A transaction that is rolled back after its statements.
fn rolled_back(statements: impl Into<String>) -> Action {
    let statements = statements.into();
    Box::new(move |_pool| {
        session()
            .batch_execute(&format!("BEGIN; {statements}; ROLLBACK;"))
            .map_err(|error| error.to_string())
    })
}

/// The implementation's deletion unit, reached as the API reaches it: the GraphQL delete mutation, whose resolver calls
/// `Crud::delete` and so the `v1.10.0` unit.
fn delete_through_api(mutation: &'static str, argument: &'static str, id: Uuid) -> Action {
    Box::new(move |pool| {
        let pool = Arc::new(pool.clone());
        let response = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime")
            .block_on(support::execute_graphql(
                pool,
                Some(support::superuser("deletion-units")),
                &format!("mutation {{ {mutation}({argument}: \"{id}\") {{ __typename }} }}"),
                None,
            ));
        match support::first_error_message(&response) {
            None => Ok(()),
            Some(message) => Err(message.to_string()),
        }
    })
}

fn delete_work(work: Uuid) -> Action {
    delete_through_api("deleteWork", "workId", work)
}

fn delete_imprint(imprint: Uuid) -> Action {
    delete_through_api("deleteImprint", "imprintId", imprint)
}

fn delete_publisher(publisher: Uuid) -> Action {
    delete_through_api("deletePublisher", "publisherId", publisher)
}

/// Hold `first` at `point`, run `second`, observe whether `second` waits, release, and require both to commit.
fn schedule(
    pool: &Arc<PgPool>,
    name: &str,
    first: Action,
    point: Point,
    second: Action,
    second_waits: bool,
) -> String {
    let mut pause = Pause::install(name, point);
    let mut observer = session();
    let p = pool.clone();
    let first = std::thread::spawn(move || first(p.as_ref()));
    wait_until(|| paused(&mut observer) || first.is_finished());
    assert!(
        !first.is_finished(),
        "{name}: the first transaction reached its pause: {:?}",
        first.join()
    );
    let p = pool.clone();
    let second = std::thread::spawn(move || second(p.as_ref()));
    let transcript = if second_waits {
        wait_until(|| !lock_waits(&mut observer).is_empty() || second.is_finished());
        let transcript = lock_waits(&mut observer);
        assert!(
            !second.is_finished(),
            "{name}: the second transaction waits: {:?}",
            second.join()
        );
        transcript
    } else {
        wait_until(|| second.is_finished() || !lock_waits(&mut observer).is_empty());
        lock_waits(&mut observer)
    };
    pause.release();
    let first = first.join().expect("first");
    let second = second.join().expect("second");
    assert_eq!(first, Ok(()), "{name}: the first commits (no 40P01)");
    assert_eq!(second, Ok(()), "{name}: the second commits (no 40P01)");
    drop(pause);
    transcript
}

fn assert_clean(connection: &mut PgConnection, name: &str) {
    assert_eq!(
        text(connection, "SELECT count(*)::text AS value FROM work_upsert_generation g WHERE NOT EXISTS (SELECT 1 FROM work w WHERE w.work_id = g.work_id)"),
        "0",
        "{name}: no orphaned generation row"
    );
    assert_eq!(
        text(
            connection,
            "SELECT count(*)::text AS value FROM work_upsert_capture_queue"
        ),
        "0",
        "{name}: no queue residue"
    );
}

struct Library {
    publisher: Uuid,
    imprint: Uuid,
    book: Uuid,
    chapters: [Uuid; 2],
    chapter_titles: [Uuid; 2],
    relations: [Uuid; 2],
}

fn library(connection: &mut PgConnection) -> Library {
    let id = Uuid::new_v4;
    let l = Library {
        publisher: id(),
        imprint: id(),
        book: id(),
        chapters: [id(), id()],
        chapter_titles: [id(), id()],
        relations: [id(), id()],
    };
    execute(
        connection,
        &format!(
            "INSERT INTO publisher (publisher_id, publisher_name) VALUES ('{p}', 'Deletion {p}');
             INSERT INTO imprint (imprint_id, publisher_id, imprint_name) VALUES ('{i}', '{p}', 'Deletion {i}');
             INSERT INTO work (work_id, work_type, work_status, edition, imprint_id, doi, publication_date, landing_page)
             VALUES ('{b}', 'monograph', 'active', 1, '{i}', 'https://doi.org/10.12345/del-{bs}', '2026-01-01', 'https://example.org/{bs}'),
                    ('{c0}', 'book-chapter', 'active', NULL, '{i}', 'https://doi.org/10.12345/del-{c0s}', '2026-01-01', 'https://example.org/{c0s}'),
                    ('{c1}', 'book-chapter', 'active', NULL, '{i}', 'https://doi.org/10.12345/del-{c1s}', '2026-01-01', 'https://example.org/{c1s}');
             INSERT INTO title (title_id, work_id, locale_code, full_title, title, canonical)
             VALUES ('{t0}', '{c0}', 'en', 'C0', 'C0', true), ('{t1}', '{c1}', 'en', 'C1', 'C1', true);
             INSERT INTO work_relation (work_relation_id, relator_work_id, related_work_id, relation_type, relation_ordinal)
             VALUES ('{r0}', '{b}', '{c0}', 'has-child', 1), (gen_random_uuid(), '{c0}', '{b}', 'is-child-of', 1),
                    ('{r1}', '{b}', '{c1}', 'has-child', 2), (gen_random_uuid(), '{c1}', '{b}', 'is-child-of', 1);",
            p = l.publisher, i = l.imprint, b = l.book, bs = l.book.simple(),
            c0 = l.chapters[0], c0s = l.chapters[0].simple(), c1 = l.chapters[1], c1s = l.chapters[1].simple(),
            t0 = l.chapter_titles[0], t1 = l.chapter_titles[1], r0 = l.relations[0], r1 = l.relations[1],
        ),
    );
    l
}

fn setup() -> (support::TestDbGuard, Arc<PgPool>) {
    let guard = support::test_lock();
    let pool = support::db_pool();
    support::reset_db(pool.as_ref()).expect("reset");
    (guard, pool)
}

fn title_edit(title: Uuid) -> Action {
    sql(format!(
        "UPDATE title SET title = 'Edited', full_title = 'Edited' WHERE title_id = '{title}'"
    ))
}

/// The released 30-column Work `UPDATE` shape, reduced to its effect: every column rewritten.
fn work_edit(work: Uuid) -> Action {
    sql(format!(
        "UPDATE work SET work_type = work_type, work_status = work_status, edition = edition, doi = doi, \
         publication_date = publication_date, place = 'Edited', page_count = page_count, first_page = first_page, \
         last_page = last_page, landing_page = landing_page, imprint_id = imprint_id WHERE work_id = '{work}'"
    ))
}

#[test]
fn t325_t303_schedules_and_the_rolled_back_deletion_commit() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");

    // D1: a chapter title edit, held between the released trigger and capture, against the parent's deletion.
    let l = library(&mut connection);
    let waits = schedule(
        &pool,
        "d1",
        title_edit(l.chapter_titles[0]),
        Point::BeforeCapture("UPDATE", "title"),
        sql(format!("DELETE FROM work WHERE work_id = '{}'", l.book)),
        true,
    );
    assert!(
        waits.contains("transactionid"),
        "d1: the deletion waited on the edit: {waits}"
    );
    assert_clean(&mut connection, "d1");

    // D2: a chapter Work edit against the parent's deletion.
    let l = library(&mut connection);
    schedule(
        &pool,
        "d2",
        work_edit(l.chapters[0]),
        Point::BeforeCapture("UPDATE", "work"),
        sql(format!("DELETE FROM work WHERE work_id = '{}'", l.book)),
        true,
    );
    assert_clean(&mut connection, "d2");

    // D3: a chapter title edit against the Imprint's deletion.
    let l = library(&mut connection);
    schedule(
        &pool,
        "d3",
        title_edit(l.chapter_titles[0]),
        Point::BeforeCapture("UPDATE", "title"),
        sql(format!(
            "DELETE FROM imprint WHERE imprint_id = '{}'",
            l.imprint
        )),
        true,
    );
    assert_clean(&mut connection, "d3");

    // D4: a chapter title edit against the Publisher's deletion.
    let l = library(&mut connection);
    schedule(
        &pool,
        "d4",
        title_edit(l.chapter_titles[0]),
        Point::BeforeCapture("UPDATE", "title"),
        sql(format!(
            "DELETE FROM publisher WHERE publisher_id = '{}'",
            l.publisher
        )),
        true,
    );
    assert_clean(&mut connection, "d4");

    // The rolled-back deletion: the parent's advance from the edit survives.
    let l = library(&mut connection);
    let before: i64 = text(&mut connection, &format!("SELECT source_generation::text AS value FROM work_upsert_generation WHERE work_id = '{}'", l.book)).parse().expect("n");
    schedule(
        &pool,
        "d5",
        title_edit(l.chapter_titles[0]),
        Point::BeforeCapture("UPDATE", "title"),
        rolled_back(format!("DELETE FROM work WHERE work_id = '{}'", l.book)),
        true,
    );
    let after: i64 = text(&mut connection, &format!("SELECT source_generation::text AS value FROM work_upsert_generation WHERE work_id = '{}'", l.book)).parse().expect("n");
    assert_eq!(
        after,
        before + 1,
        "the parent's advance survives the rolled-back deletion"
    );
    assert_clean(&mut connection, "d5");
}

#[test]
fn t326_k1_k2_k4_k5_k6s_k6c_and_d2b_commit() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");

    // K1: a sibling reorder (one UPDATE per sibling, one transaction) against an edit of the chapter its next UPDATE
    // moves; the edit is held before capture.
    let l = library(&mut connection);
    let reorder = format!(
        "UPDATE work_relation SET relation_ordinal = 3 WHERE work_relation_id = '{r0}'; \
         UPDATE work_relation SET relation_ordinal = 1 WHERE work_relation_id = '{r1}'; \
         UPDATE work_relation SET relation_ordinal = 2 WHERE work_relation_id = '{r0}'",
        r0 = l.relations[0],
        r1 = l.relations[1]
    );
    schedule(
        &pool,
        "k1",
        title_edit(l.chapter_titles[1]),
        Point::BeforeCapture("UPDATE", "title"),
        sql(reorder.clone()),
        true,
    );
    assert_clean(&mut connection, "k1");
    // K2: the other order — the reorder held after its first statement's released trigger, the edit waits for it.
    let l = library(&mut connection);
    let reorder = format!(
        "UPDATE work_relation SET relation_ordinal = 3 WHERE work_relation_id = '{r1}'; \
         UPDATE work_relation SET relation_ordinal = 2 WHERE work_relation_id = '{r0}'; \
         UPDATE work_relation SET relation_ordinal = 1 WHERE work_relation_id = '{r1}'",
        r0 = l.relations[0],
        r1 = l.relations[1]
    );
    schedule(
        &pool,
        "k2",
        sql(reorder),
        Point::BeforeCapture("UPDATE", "work_relation"),
        title_edit(l.chapter_titles[1]),
        true,
    );
    assert_clean(&mut connection, "k2");

    // K4: deleting a Work related to both chapters of a surviving book, against an edit of one of those chapters.
    let l = library(&mut connection);
    let related = Uuid::new_v4();
    execute(&mut connection, &format!(
        "INSERT INTO work (work_id, work_type, work_status, edition, imprint_id) VALUES ('{related}', 'monograph', 'forthcoming', 1, '{i}');
         INSERT INTO work_relation (relator_work_id, related_work_id, relation_type, relation_ordinal)
         VALUES ('{related}', '{c0}', 'has-part', 1), ('{c0}', '{related}', 'is-part-of', 1),
                ('{related}', '{c1}', 'has-part', 2), ('{c1}', '{related}', 'is-part-of', 1);",
        i = l.imprint, c0 = l.chapters[0], c1 = l.chapters[1]
    ));
    schedule(
        &pool,
        "k4",
        title_edit(l.chapter_titles[1]),
        Point::BeforeCapture("UPDATE", "title"),
        sql(format!("DELETE FROM work WHERE work_id = '{related}'")),
        true,
    );
    assert_clean(&mut connection, "k4");

    // K5: a relation re-pointed from one chapter to another, held before capture, against the creation of a relation
    // on the old chapter.
    let l = library(&mut connection);
    let other = Uuid::new_v4();
    execute(&mut connection, &format!(
        "INSERT INTO work (work_id, work_type, work_status, edition, imprint_id) VALUES ('{other}', 'monograph', 'forthcoming', 1, '{}')",
        l.imprint
    ));
    let pointer = Uuid::new_v4();
    execute(&mut connection, &format!(
        "INSERT INTO work_relation (work_relation_id, relator_work_id, related_work_id, relation_type, relation_ordinal)
         VALUES ('{pointer}', '{other}', '{c0}', 'has-part', 1), (gen_random_uuid(), '{c0}', '{other}', 'is-part-of', 1)",
        c0 = l.chapters[0]
    ));
    let third = Uuid::new_v4();
    execute(&mut connection, &format!(
        "INSERT INTO work (work_id, work_type, work_status, edition, imprint_id) VALUES ('{third}', 'monograph', 'forthcoming', 1, '{}')",
        l.imprint
    ));
    schedule(
        &pool,
        "k5",
        sql(format!(
            "UPDATE work_relation SET related_work_id = '{c1}' WHERE work_relation_id = '{pointer}'; \
             UPDATE work_relation SET relator_work_id = '{c1}' \
              WHERE relator_work_id = '{c0}' AND related_work_id = '{other}' AND relation_type = 'is-part-of'",
            c0 = l.chapters[0],
            c1 = l.chapters[1]
        )),
        Point::BeforeCapture("UPDATE", "work_relation"),
        sql(format!(
            "INSERT INTO work_relation (relator_work_id, related_work_id, relation_type, relation_ordinal)
             VALUES ('{third}', '{c0}', 'has-part', 1), ('{c0}', '{third}', 'is-part-of', 2)",
            c0 = l.chapters[0]
        )),
        false,
    );
    assert_clean(&mut connection, "k5");

    // K6s: two deletions whose cascades reach chapters of two books in opposite orders, sharing no Work row; the first
    // is held mid-cascade.
    let a = library(&mut connection);
    let b = library(&mut connection);
    let (x, y) = (Uuid::new_v4(), Uuid::new_v4());
    execute(&mut connection, &format!(
        "INSERT INTO work (work_id, work_type, work_status, edition, imprint_id) VALUES ('{x}', 'monograph', 'forthcoming', 1, '{ia}'), ('{y}', 'monograph', 'forthcoming', 1, '{ib}');
         INSERT INTO work_relation (relator_work_id, related_work_id, relation_type, relation_ordinal)
         VALUES ('{x}', '{a0}', 'has-part', 1), ('{a0}', '{x}', 'is-part-of', 1), ('{x}', '{b0}', 'has-part', 2), ('{b0}', '{x}', 'is-part-of', 1),
                ('{y}', '{b1}', 'has-part', 1), ('{b1}', '{y}', 'is-part-of', 1), ('{y}', '{a1}', 'has-part', 2), ('{a1}', '{y}', 'is-part-of', 1);",
        ia = a.imprint, ib = b.imprint, a0 = a.chapters[0], b0 = b.chapters[0], a1 = a.chapters[1], b1 = b.chapters[1]
    ));
    schedule(
        &pool,
        "k6s",
        sql(format!("DELETE FROM work WHERE work_id = '{x}'")),
        Point::BeforeCapture("DELETE", "work_relation"),
        sql(format!("DELETE FROM work WHERE work_id = '{y}'")),
        false,
    );
    assert_clean(&mut connection, "k6s");

    // K6c: the same shape, the first deletion held at commit after its flush.
    let a = library(&mut connection);
    let b = library(&mut connection);
    let (x, y) = (Uuid::new_v4(), Uuid::new_v4());
    execute(&mut connection, &format!(
        "INSERT INTO work (work_id, work_type, work_status, edition, imprint_id) VALUES ('{x}', 'monograph', 'forthcoming', 1, '{ia}'), ('{y}', 'monograph', 'forthcoming', 1, '{ib}');
         INSERT INTO work_relation (relator_work_id, related_work_id, relation_type, relation_ordinal)
         VALUES ('{x}', '{a0}', 'has-part', 1), ('{a0}', '{x}', 'is-part-of', 1), ('{x}', '{b0}', 'has-part', 2), ('{b0}', '{x}', 'is-part-of', 1),
                ('{y}', '{b1}', 'has-part', 1), ('{b1}', '{y}', 'is-part-of', 1), ('{y}', '{a1}', 'has-part', 2), ('{a1}', '{y}', 'is-part-of', 1);",
        ia = a.imprint, ib = b.imprint, a0 = a.chapters[0], b0 = b.chapters[0], a1 = a.chapters[1], b1 = b.chapters[1]
    ));
    let mut control = session();
    let key = 1_948_579_299_i64;
    execute(&mut control, &format!("SELECT pg_advisory_lock({key})"));
    let x_thread = std::thread::spawn(move || {
        let mut deleter = session();
        deleter
            .batch_execute(&format!(
                "CREATE TEMP TABLE k6c_pause (x int);
                 CREATE FUNCTION pg_temp.k6c_pause() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_advisory_xact_lock({key}); RETURN NULL; END $$;
                 CREATE CONSTRAINT TRIGGER k6c_pause AFTER INSERT ON k6c_pause DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION pg_temp.k6c_pause();
                 BEGIN; DELETE FROM work WHERE work_id = '{x}'; INSERT INTO k6c_pause VALUES (1); COMMIT;"
            ))
            .map_err(|e| e.to_string())
    });
    let mut observer = session();
    wait_until(|| {
        text(&mut observer, &format!("SELECT count(*)::text AS value FROM pg_locks WHERE locktype = 'advisory' AND NOT granted AND objid = {key}")) == "1"
    });
    let y_thread = std::thread::spawn(move || {
        session()
            .batch_execute(&format!(
                "BEGIN; DELETE FROM work WHERE work_id = '{y}'; COMMIT;"
            ))
            .map_err(|e| e.to_string())
    });
    wait_until(|| y_thread.is_finished() || !lock_waits(&mut observer).is_empty());
    execute(&mut control, &format!("SELECT pg_advisory_unlock({key})"));
    assert_eq!(x_thread.join().expect("x"), Ok(()), "k6c: no 40P01");
    assert_eq!(y_thread.join().expect("y"), Ok(()), "k6c: no 40P01");
    assert_clean(&mut connection, "k6c");

    // D2b: a chapter Work edit held after its row lock and before its BEFORE triggers, against the parent's deletion.
    let l = library(&mut connection);
    schedule(
        &pool,
        "d2b",
        work_edit(l.chapters[0]),
        Point::BeforeTriggers("UPDATE", "work"),
        sql(format!("DELETE FROM work WHERE work_id = '{}'", l.book)),
        true,
    );
    assert_clean(&mut connection, "d2b");
}

#[test]
fn t327_s01_to_s13_commit() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let title = Point::BeforeCapture("UPDATE", "title");
    let work = Point::BeforeCapture("UPDATE", "work");
    let relation_update = Point::BeforeCapture("UPDATE", "work_relation");
    let mut transcripts: Vec<(&str, String)> = Vec::new();

    // S01, S02: two chapter edits under one parent, in both orders.
    let l = library(&mut connection);
    transcripts.push((
        "s01",
        schedule(
            &pool,
            "s01",
            title_edit(l.chapter_titles[0]),
            title,
            title_edit(l.chapter_titles[1]),
            false,
        ),
    ));
    transcripts.push((
        "s02",
        schedule(
            &pool,
            "s02",
            title_edit(l.chapter_titles[1]),
            title,
            title_edit(l.chapter_titles[0]),
            false,
        ),
    ));
    // S03, S04: a parent edit against a chapter edit, in both orders.
    transcripts.push((
        "s03",
        schedule(
            &pool,
            "s03",
            work_edit(l.book),
            work,
            title_edit(l.chapter_titles[0]),
            false,
        ),
    ));
    transcripts.push((
        "s04",
        schedule(
            &pool,
            "s04",
            title_edit(l.chapter_titles[0]),
            title,
            work_edit(l.book),
            false,
        ),
    ));
    assert_clean(&mut connection, "s01-s04");

    // S05, S06: a relation create against a chapter edit, in both orders.
    let (other, another) = (Uuid::new_v4(), Uuid::new_v4());
    execute(&mut connection, &format!(
        "INSERT INTO work (work_id, work_type, work_status, edition, imprint_id) \
         VALUES ('{other}', 'monograph', 'forthcoming', 1, '{i}'), ('{another}', 'monograph', 'forthcoming', 1, '{i}')",
        i = l.imprint
    ));
    let create = |relator: Uuid, ordinal: i32| {
        sql(format!(
            "INSERT INTO work_relation (relator_work_id, related_work_id, relation_type, relation_ordinal) \
             VALUES ('{relator}', '{c0}', 'has-part', {ordinal}), ('{c0}', '{relator}', 'is-part-of', {ordinal})",
            c0 = l.chapters[0]
        ))
    };
    transcripts.push((
        "s05",
        schedule(
            &pool,
            "s05",
            create(other, 1),
            Point::BeforeCapture("INSERT", "work_relation"),
            title_edit(l.chapter_titles[0]),
            false,
        ),
    ));
    transcripts.push((
        "s06",
        schedule(
            &pool,
            "s06",
            title_edit(l.chapter_titles[0]),
            title,
            create(another, 2),
            false,
        ),
    ));
    assert_clean(&mut connection, "s05-s06");

    // S07, S08: a chapter deletion against a parent edit, in both orders.
    let l = library(&mut connection);
    transcripts.push((
        "s07",
        schedule(
            &pool,
            "s07",
            sql(format!(
                "DELETE FROM work WHERE work_id = '{}'",
                l.chapters[0]
            )),
            Point::BeforeCapture("DELETE", "work_relation"),
            work_edit(l.book),
            false,
        ),
    ));
    let l = library(&mut connection);
    transcripts.push((
        "s08",
        schedule(
            &pool,
            "s08",
            work_edit(l.book),
            work,
            sql(format!(
                "DELETE FROM work WHERE work_id = '{}'",
                l.chapters[0]
            )),
            false,
        ),
    ));
    assert_clean(&mut connection, "s07-s08");

    // S09: a relation edit against the parent's deletion.
    let l = library(&mut connection);
    transcripts.push((
        "s09",
        schedule(
            &pool,
            "s09",
            sql(format!(
                "UPDATE work_relation SET relation_ordinal = 7 WHERE work_relation_id = '{}'",
                l.relations[0]
            )),
            relation_update,
            sql(format!("DELETE FROM work WHERE work_id = '{}'", l.book)),
            false,
        ),
    ));
    // S10: a chapter edit against another chapter's deletion.
    let l = library(&mut connection);
    transcripts.push((
        "s10",
        schedule(
            &pool,
            "s10",
            title_edit(l.chapter_titles[0]),
            title,
            sql(format!(
                "DELETE FROM work WHERE work_id = '{}'",
                l.chapters[1]
            )),
            false,
        ),
    ));
    assert_clean(&mut connection, "s09-s10");

    // S11: a chapter edit against the BE-03 coordinator's publisher UPDATE and its cascade.
    let l = library(&mut connection);
    transcripts.push(("s11", schedule(&pool, "s11", title_edit(l.chapter_titles[0]), title, sql(format!("UPDATE publisher SET publisher_name = publisher_name || ' (coordinator)' WHERE publisher_id = '{}'", l.publisher)), false)));
    // S12: a chapter Work edit against an Imprint deletion.
    let l = library(&mut connection);
    transcripts.push((
        "s12",
        schedule(
            &pool,
            "s12",
            work_edit(l.chapters[0]),
            work,
            sql(format!(
                "DELETE FROM imprint WHERE imprint_id = '{}'",
                l.imprint
            )),
            false,
        ),
    ));
    // S13: a chapter edit against a deletion that is rolled back.
    let l = library(&mut connection);
    transcripts.push((
        "s13",
        schedule(
            &pool,
            "s13",
            title_edit(l.chapter_titles[0]),
            title,
            rolled_back(format!("DELETE FROM work WHERE work_id = '{}'", l.book)),
            false,
        ),
    ));
    assert_clean(&mut connection, "s11-s13");

    assert_eq!(transcripts.len(), 13);
    // Both sides committing is the requirement; the waits observed are recorded for the implementation report.
    for (name, transcript) in &transcripts {
        eprintln!("T327 {name}: waits observed while the first was held = [{transcript}]");
    }
}

#[test]
fn t328_the_v1_10_0_deletion_units_commit_against_a_held_chapter_edit() {
    let (_guard, pool) = setup();
    let mut connection = pool.get().expect("connection");
    let before_capture_title = Point::BeforeCapture("UPDATE", "title");
    // A Work deletion of the parent.
    let l = library(&mut connection);
    schedule(
        &pool,
        "u1",
        title_edit(l.chapter_titles[0]),
        before_capture_title,
        delete_work(l.book),
        true,
    );
    assert_clean(&mut connection, "u1");
    // An Imprint deletion.
    let l = library(&mut connection);
    schedule(
        &pool,
        "u2",
        title_edit(l.chapter_titles[0]),
        before_capture_title,
        delete_imprint(l.imprint),
        true,
    );
    assert_clean(&mut connection, "u2");
    // A chapter deletion.
    let l = library(&mut connection);
    schedule(
        &pool,
        "u3",
        title_edit(l.chapter_titles[0]),
        before_capture_title,
        delete_work(l.chapters[0]),
        true,
    );
    assert_clean(&mut connection, "u3");
    // A Publisher deletion.
    let l = library(&mut connection);
    schedule(
        &pool,
        "u4",
        title_edit(l.chapter_titles[0]),
        before_capture_title,
        delete_publisher(l.publisher),
        true,
    );
    assert_clean(&mut connection, "u4");
}
