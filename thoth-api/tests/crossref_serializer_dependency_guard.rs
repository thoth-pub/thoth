#![cfg(feature = "backend")]

//! `BE-06` serializer dependency guard (R52B section 8.3, T38) and the membership rows of section 16.11 that need the
//! database (T244, T245).
//!
//! T38 pairs the section 8.2 capture matrix with the field set the Crossref serializer actually reads, and fails CI on
//! divergence: every client field the serializer reads must be classified, every column the matrix says is captured
//! must be watched by Migration 2's capture triggers, and no field section 8.3 excludes may be read. Nothing here
//! contacts Crossref.

#[allow(dead_code)]
mod support;

use std::collections::{BTreeMap, BTreeSet};

use diesel::connection::SimpleConnection;
use diesel::sql_types::Text;
use diesel::{PgConnection, QueryableByName, RunQueryDsl};
use uuid::Uuid;

const SERIALIZER: &str = "../thoth-export-server/src/xml/doideposit_crossref.rs";
const QUERIES: &str = "../thoth-client/assets/queries.graphql";
const MIGRATION_2: &str = "migrations/20260911_v1.10.0/up.sql";

fn read(path: &str) -> String {
    std::fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// Every field name the client queries select, in snake case.
fn queried_fields(queries: &str) -> BTreeSet<String> {
    let identifier = regex::Regex::new(r"^([a-zA-Z_][a-zA-Z0-9_]*)").expect("regex");
    let snake = regex::Regex::new(r"([a-z0-9])([A-Z])").expect("regex");
    queries
        .lines()
        .map(str::trim)
        .filter(|line| {
            !line.is_empty()
                && !["fragment", "query", "mutation", "}", "...", "#"]
                    .iter()
                    .any(|prefix| line.starts_with(prefix))
        })
        .filter_map(|line| identifier.captures(line).map(|c| c[1].to_string()))
        .map(|name| snake.replace_all(&name, "${1}_${2}").to_lowercase())
        .collect()
}

/// The queried fields the serializer's non-test source reads.
fn serializer_reads(serializer: &str, queried: &BTreeSet<String>) -> BTreeSet<String> {
    let source = serializer
        .split_once("#[cfg(test)]")
        .map_or(serializer, |(body, _)| body);
    let access = regex::Regex::new(r"\.([a-z][a-z0-9_]*)\b").expect("regex");
    access
        .captures_iter(source)
        .map(|c| c[1].to_string())
        .filter(|name| queried.contains(name))
        .collect()
}

/// Section 8.2's matrix, keyed by the client field the serializer reads: the physical `table.column` pairs that carry
/// it, or `IDENTITY` for an immutable key.
fn matrix() -> BTreeMap<&'static str, Vec<&'static str>> {
    BTreeMap::from([
        ("abstract_type", vec!["abstract.abstract_type"]),
        ("abstracts", vec!["abstract.work_id"]),
        ("affiliations", vec!["affiliation.contribution_id"]),
        ("article_title", vec!["reference.article_title"]),
        ("author", vec!["reference.author"]),
        (
            "canonical",
            vec![
                "title.canonical",
                "abstract.canonical",
                "location.canonical",
            ],
        ),
        ("component_number", vec!["reference.component_number"]),
        ("content", vec!["abstract.content"]),
        (
            "contribution_ordinal",
            vec!["contribution.contribution_ordinal"],
        ),
        ("contribution_type", vec!["contribution.contribution_type"]),
        ("contributions", vec!["contribution.work_id"]),
        ("contributor", vec!["contribution.contributor_id"]),
        ("crossmark_doi", vec!["imprint.crossmark_doi"]),
        ("doi", vec!["work.doi", "reference.doi"]),
        ("edition", vec!["work.edition", "reference.edition"]),
        ("first_name", vec!["contribution.first_name"]),
        (
            "first_page",
            vec!["work.first_page", "reference.first_page"],
        ),
        ("full_text_url", vec!["location.full_text_url"]),
        ("fundings", vec!["funding.work_id"]),
        ("grant_number", vec!["funding.grant_number"]),
        ("imprint", vec!["work.imprint_id"]),
        (
            "institution",
            vec!["funding.institution_id", "affiliation.institution_id"],
        ),
        ("institution_doi", vec!["institution.institution_doi"]),
        ("institution_name", vec!["institution.institution_name"]),
        ("isbn", vec!["publication.isbn", "reference.isbn"]),
        ("issn", vec!["reference.issn"]),
        ("issn_digital", vec!["series.issn_digital"]),
        ("issn_print", vec!["series.issn_print"]),
        ("issue", vec!["reference.issue"]),
        ("issue_number", vec!["issue.issue_number"]),
        ("issues", vec!["issue.work_id", "issue.issue_ordinal"]),
        ("journal_title", vec!["reference.journal_title"]),
        ("landing_page", vec!["work.landing_page"]),
        ("last_name", vec!["contribution.last_name"]),
        ("last_page", vec!["work.last_page"]),
        ("license", vec!["work.license"]),
        (
            "locale_code",
            vec!["abstract.locale_code", "title.locale_code"],
        ),
        (
            "locations",
            vec!["location.publication_id", "location.location_platform"],
        ),
        ("orcid", vec!["contributor.orcid"]),
        ("place", vec!["work.place"]),
        (
            "publication_date",
            vec!["work.publication_date", "reference.publication_date"],
        ),
        ("publication_type", vec!["publication.publication_type"]),
        ("publications", vec!["publication.work_id"]),
        ("publisher", vec!["imprint.publisher_id"]),
        ("publisher_name", vec!["publisher.publisher_name"]),
        ("reference_ordinal", vec!["reference.reference_ordinal"]),
        ("references", vec!["reference.work_id"]),
        ("related_work", vec!["work_relation.related_work_id"]),
        ("relation_ordinal", vec!["work_relation.relation_ordinal"]),
        ("relation_type", vec!["work_relation.relation_type"]),
        ("relations", vec!["work_relation.relator_work_id"]),
        ("ror", vec!["institution.ror"]),
        ("series", vec!["issue.series_id"]),
        ("series_name", vec!["series.series_name"]),
        ("series_title", vec!["reference.series_title"]),
        ("standard_designator", vec!["reference.standard_designator"]),
        (
            "standards_body_acronym",
            vec!["reference.standards_body_acronym"],
        ),
        ("standards_body_name", vec!["reference.standards_body_name"]),
        ("subtitle", vec!["title.subtitle"]),
        ("title", vec!["title.title"]),
        ("titles", vec!["title.work_id"]),
        (
            "unstructured_citation",
            vec!["reference.unstructured_citation"],
        ),
        ("volume", vec!["reference.volume"]),
        ("volume_title", vec!["reference.volume_title"]),
        ("withdrawn_date", vec!["work.withdrawn_date"]),
        ("work_id", vec!["IDENTITY"]),
        ("work_status", vec!["work.work_status"]),
        ("work_type", vec!["work.work_type"]),
    ])
}

/// Section 8.3's excluded fields: the serializer must never read them.
const EXCLUDED: [&str; 34] = [
    "position",
    "website",
    "country_code",
    "retrieval_date",
    "copyright_holder",
    "general_note",
    "bibliography_note",
    "page_count",
    "page_breakdown",
    "page_interval",
    "image_count",
    "table_count",
    "audio_count",
    "video_count",
    "toc",
    "lccn",
    "oclc",
    "cover_url",
    "cover_caption",
    "reference",
    "subjects",
    "languages",
    "biographies",
    "prices",
    "width_mm",
    "width_cm",
    "width_in",
    "height_mm",
    "height_cm",
    "height_in",
    "depth_mm",
    "weight_g",
    "weight_oz",
    "accessibility_standard",
];

/// Per physical table, `None` for a whole-table trigger or the `UPDATE OF` column list.
fn capture_triggers(migration: &str) -> BTreeMap<String, Option<BTreeSet<String>>> {
    let trigger = regex::Regex::new(
        r"CREATE TRIGGER work_upsert_capture AFTER (.+?) ON public\.(\w+) FOR EACH ROW",
    )
    .expect("regex");
    trigger
        .captures_iter(migration)
        .map(|c| {
            let events = c[1].to_string();
            let columns = events.split_once("UPDATE OF ").map(|(_, list)| {
                list.split(',')
                    .map(|column| column.trim().to_string())
                    .collect()
            });
            (c[2].to_string(), columns)
        })
        .collect()
}

/// The guard itself: every divergence between the serializer's read set, the matrix and the migration.
fn divergences(serializer: &str, queries: &str, migration: &str) -> Vec<String> {
    let queried = queried_fields(queries);
    let reads = serializer_reads(serializer, &queried);
    let matrix = matrix();
    let triggers = capture_triggers(migration);
    let mut found = Vec::new();
    for field in &reads {
        if !matrix.contains_key(field.as_str()) {
            found.push(format!(
                "the serializer reads `{field}`, which the capture matrix does not classify"
            ));
        }
        if EXCLUDED.contains(&field.as_str()) {
            found.push(format!(
                "the serializer reads `{field}`, which section 8.3 excludes"
            ));
        }
    }
    for (field, columns) in &matrix {
        for column in columns.iter().filter(|c| **c != "IDENTITY") {
            let (table, name) = column.split_once('.').expect("table.column");
            match triggers.get(table) {
                None => found.push(format!("{field}: no capture trigger on {table}")),
                Some(Some(list)) if !list.contains(name) => found.push(format!(
                    "{field}: {table}.{name} is not in the trigger's UPDATE OF list"
                )),
                _ => {}
            }
        }
    }
    found
}

#[test]
fn t38_the_serializer_read_set_equals_the_capture_matrix() {
    let (serializer, queries, migration) = (read(SERIALIZER), read(QUERIES), read(MIGRATION_2));
    assert_eq!(
        capture_triggers(&migration).len(),
        16,
        "the sixteen capture triggers"
    );
    let divergences = divergences(&serializer, &queries, &migration);
    assert!(divergences.is_empty(), "{divergences:#?}");
    // Every matrix row is actually read: the matrix carries no stale row.
    let reads = serializer_reads(&serializer, &queried_fields(&queries));
    for field in matrix().keys() {
        assert!(
            reads.contains(*field),
            "the matrix classifies `{field}`, which the serializer no longer reads"
        );
    }
}

#[test]
fn t38_negative_controls_a_new_serializer_read_or_a_narrowed_trigger_fails_the_guard() {
    let (serializer, queries, migration) = (read(SERIALIZER), read(QUERIES), read(MIGRATION_2));
    let cut = serializer.find("#[cfg(test)]").expect("test module");
    // A field added to the serializer without a matrix row.
    let added = format!(
        "{}fn drift(work: &Work) -> Option<String> {{ work.cover_url.clone() }}\n{}",
        &serializer[..cut],
        &serializer[cut..]
    );
    let found = divergences(&added, &queries, &migration);
    assert!(
        found.iter().any(|d| d.contains("`cover_url`")),
        "{found:#?}"
    );
    // A capture trigger narrowed so that a matrix column is no longer watched.
    let narrowed = migration.replacen(
        "UPDATE OF canonical, full_text_url, location_platform, publication_id ON public.location",
        "UPDATE OF canonical, publication_id ON public.location",
        1,
    );
    assert_ne!(narrowed, migration);
    let found = divergences(&serializer, &queries, &narrowed);
    assert!(
        found.iter().any(|d| d.contains("location.full_text_url")),
        "{found:#?}"
    );
}

// ---------------------------------------------------------------------------------------------------------------------
// T244 and T245: the membership rule of section 14.3 against the serializer's registration sets, and its ordering.
// ---------------------------------------------------------------------------------------------------------------------

#[derive(QueryableByName)]
struct TextRow {
    #[diesel(sql_type = Text)]
    value: String,
}

fn text(connection: &mut PgConnection, sql: &str) -> String {
    diesel::sql_query(sql)
        .get_result::<TextRow>(connection)
        .unwrap_or_else(|error| panic!("{sql}: {error}"))
        .value
}

fn execute(connection: &mut PgConnection, sql: &str) {
    connection
        .batch_execute(sql)
        .unwrap_or_else(|error| panic!("{sql}: {error}"));
}

fn membership(connection: &mut PgConnection, root: Uuid) -> String {
    text(
        connection,
        &format!(
            "SELECT array_to_string(public.crossref_deposit_membership('{root}'), ',') AS value"
        ),
    )
}

#[test]
fn t244_the_membership_rule_equals_the_serializer_registration_set_on_every_shape() {
    let _guard = support::test_lock();
    let pool = support::db_pool();
    support::reset_db(pool.as_ref()).expect("reset");
    let mut connection = pool.get().expect("connection");
    let publisher = Uuid::new_v4();
    let imprint = Uuid::new_v4();
    execute(
        &mut connection,
        &format!(
            "INSERT INTO publisher (publisher_id, publisher_name) VALUES ('{publisher}', 'T244');
             INSERT INTO imprint (imprint_id, publisher_id, imprint_name) VALUES ('{imprint}', '{publisher}', 'T244');"
        ),
    );
    let work = |connection: &mut PgConnection,
                kind: &str,
                doi: Option<&str>,
                landing_page: Option<&str>| {
        let id = Uuid::new_v4();
        let edition = if kind == "book-chapter" { "NULL" } else { "1" };
        let doi = doi.map_or("NULL".to_string(), |d| format!("'{d}'"));
        let landing_page = landing_page.map_or("NULL".to_string(), |l| format!("'{l}'"));
        execute(
            connection,
            &format!(
                "INSERT INTO work (work_id, work_type, work_status, edition, imprint_id, doi, landing_page, publication_date) \
                 VALUES ('{id}', '{kind}', 'active', {edition}, '{imprint}', {doi}, {landing_page}, '2026-01-01')"
            ),
        );
        id
    };
    let relate = |connection: &mut PgConnection, parent: Uuid, child: Uuid| {
        execute(
            connection,
            &format!(
                "INSERT INTO work_relation (relator_work_id, related_work_id, relation_type, relation_ordinal) \
                 VALUES ('{parent}', '{child}', 'has-child', 1), ('{child}', '{parent}', 'is-child-of', 1)"
            ),
        );
    };
    // The shapes of the export server's T243, each with the registration set the extractor returned there.
    let base = work(
        &mut connection,
        "monograph",
        Some("https://doi.org/10.00001/BOOK.0001"),
        Some("https://www.book.com"),
    );
    let with_chapter = work(
        &mut connection,
        "monograph",
        Some("https://doi.org/10.00001/BOOK.0002"),
        Some("https://www.book.com"),
    );
    let chapter_1 = work(
        &mut connection,
        "book-chapter",
        Some("https://doi.org/10.00001/CHAPTER.1"),
        Some("https://www.book.com/ch1"),
    );
    relate(&mut connection, with_chapter, chapter_1);
    let without_landing_page = work(
        &mut connection,
        "monograph",
        Some("https://doi.org/10.00001/BOOK.0003"),
        None,
    );
    let chapter_3 = work(
        &mut connection,
        "book-chapter",
        Some("https://doi.org/10.00001/CHAPTER.3"),
        Some("https://www.book.com/ch3"),
    );
    relate(&mut connection, without_landing_page, chapter_3);
    let without_doi = work(
        &mut connection,
        "monograph",
        None,
        Some("https://www.book.com"),
    );
    let chapter_4 = work(
        &mut connection,
        "book-chapter",
        Some("https://doi.org/10.00001/CHAPTER.4"),
        Some("https://www.book.com/ch4"),
    );
    relate(&mut connection, without_doi, chapter_4);
    let chapterless_undoi = work(
        &mut connection,
        "monograph",
        Some("https://doi.org/10.00001/BOOK.0005"),
        Some("https://www.book.com"),
    );
    let chapter_without_doi = work(
        &mut connection,
        "book-chapter",
        None,
        Some("https://www.book.com/ch5"),
    );
    relate(&mut connection, chapterless_undoi, chapter_without_doi);
    let registers_nothing = work(
        &mut connection,
        "monograph",
        Some("https://doi.org/10.00001/BOOK.0006"),
        None,
    );

    for (shape, root, expected) in [
        ("root", base, "https://doi.org/10.00001/book.0001"),
        (
            "root and a chapter",
            with_chapter,
            "https://doi.org/10.00001/book.0002,https://doi.org/10.00001/chapter.1",
        ),
        (
            "root without a landing page",
            without_landing_page,
            "https://doi.org/10.00001/chapter.3",
        ),
        (
            "root without a DOI",
            without_doi,
            "https://doi.org/10.00001/chapter.4",
        ),
        (
            "a chapter without a DOI",
            chapterless_undoi,
            "https://doi.org/10.00001/book.0005",
        ),
        ("registers nothing", registers_nothing, ""),
    ] {
        assert_eq!(membership(&mut connection, root), expected, "{shape}");
    }
}

#[test]
fn t245_the_digest_feeding_order_is_code_point_order() {
    let _guard = support::test_lock();
    let pool = support::db_pool();
    let mut connection = pool.get().expect("connection");
    let values = ["a_b", "ab", "a.b", "a-b"];
    let mut rust = values.to_vec();
    rust.sort_unstable();
    let c_order = text(
        &mut connection,
        "SELECT string_agg(v, ' ' ORDER BY v COLLATE \"C\") AS value FROM unnest(ARRAY['a_b', 'ab', 'a.b', 'a-b']) v",
    );
    assert_eq!(
        c_order,
        rust.join(" "),
        "C order is code-point order, as Rust sorts"
    );
    assert_eq!(c_order, "a-b a.b a_b ab");
    // The membership function feeds the digest in C order, never in the database's default collation.
    let function = text(
        &mut connection,
        "SELECT pg_get_functiondef('public.crossref_deposit_membership(uuid)'::regprocedure) AS value",
    );
    assert!(function.contains("COLLATE \"C\""), "{function}");
    // An ICU collation orders the same values differently, which is why the order is pinned.
    let icu = text(
        &mut connection,
        "SELECT coalesce((SELECT collname FROM pg_collation WHERE collprovider = 'i' AND collname = 'en-US-x-icu'), '') AS value",
    );
    if !icu.is_empty() {
        let icu_order = text(
            &mut connection,
            "SELECT string_agg(v, ' ' ORDER BY v COLLATE \"en-US-x-icu\") AS value FROM unnest(ARRAY['a_b', 'ab', 'a.b', 'a-b']) v",
        );
        assert_eq!(icu_order, "a_b a-b a.b ab", "ICU orders differently");
    }
}
