//! `CITATIONS-01` slice C3: the read of stored citation links
//! (`metricCitations`).
//!
//! The caller has already passed `METRICS_READ_SERVICE`. The read then
//! checks, in this order and before any database access where possible:
//!
//! 1. `THOTH_METRIC_CITATIONS_SERVING`: `OFF` is refused as
//!    `METRIC_CITATIONS_NOT_SERVED`, after the role check, so a caller without
//!    the role learns nothing about the switch;
//! 2. the request's shape and bounds (1 to 25 publishers, at most 100 works,
//!    at most 10 source codes, `first` 1 to 100, an ordered date range, a
//!    bounded cursor), none of which touches the database;
//! 3. a concurrent-read slot, shared with `metricDashboard`, and the whole-read
//!    deadline of that read.
//!
//! Everything else runs in one `READ ONLY, REPEATABLE READ` transaction, so
//! the summary and the page are one snapshot:
//!
//! 1. the selected publishers exist and each holds `METRICS_DASHBOARD`
//!    through its package (one unentitled publisher refuses the whole
//!    request; entitlement is re-checked on every page);
//! 2. every listed work belongs to a selected publisher (an unknown work and
//!    another publisher's work are refused with one message);
//! 3. the listed source codes exist, and at most one citing source is in
//!    scope (counts are never summed across sources);
//! 4. the summary, the citing-year histogram and one page.
//!
//! Statements are set-based, so their number does not depend on how many
//! works or links are involved. The page is keyset-paginated on
//! (`citing_sort_date` DESC, `link_id` DESC), a total order; its cursor is
//! opaque and bound to a hash of the selected publishers and filters, so it
//! cannot be replayed with other ones.

use std::collections::BTreeSet;
use std::time::Instant;

use chrono::NaiveDate;
use diesel::sql_types::{
    Array, Bool, Date, Integer, Jsonb, Nullable, SmallInt, Text, Timestamptz, Uuid as SqlUuid,
};
use diesel::{sql_query, RunQueryDsl};
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::MetricCitationsServing;
use crate::db::PgPool;
use crate::model::metric_dashboard::{
    checkout, MetricReadError, ReadDeadline, CUSTOM_PLANS_SQL, METRIC_DASHBOARD_READS,
    METRIC_DASHBOARD_READ_DEADLINE, METRIC_DASHBOARD_SLOT_WAIT, STATEMENT_TIMEOUT_SQL,
    UNKNOWN_PUBLISHER,
};
use crate::model::publisher::{PublisherCapability, ThothPackage};
use crate::model::work::WorkType;
use crate::model::Timestamp;

/// Publishers one read may select.
pub const METRIC_CITATIONS_MAX_PUBLISHERS: usize = 25;
/// Works one read may name.
pub const METRIC_CITATIONS_MAX_WORKS: usize = 100;
/// Source codes one read may name.
pub const METRIC_CITATIONS_MAX_SOURCES: usize = 10;
/// Largest page.
pub const METRIC_CITATIONS_MAX_PAGE: i32 = 100;
/// Page size when `first` is omitted.
pub const METRIC_CITATIONS_DEFAULT_PAGE: i32 = 50;
/// Longest accepted cursor, in characters.
const MAX_CURSOR_CHARS: usize = 512;
/// Longest accepted source code, in characters.
const MAX_SOURCE_CODE_CHARS: usize = 200;

const PUBLISHER_CARDINALITY: &str = "publisherIds must contain between one and 25 publisher IDs.";
const DUPLICATE_PUBLISHER: &str = "publisherIds must not contain duplicate IDs.";
const TOO_MANY_WORKS: &str = "workIds must contain at most 100 work IDs.";
const DUPLICATE_WORK: &str = "workIds must not contain duplicate IDs.";
const UNKNOWN_WORK: &str =
    "workIds contains an unknown work ID or a work of a publisher that is not selected.";
const TOO_MANY_SOURCES: &str = "sourceCodes must contain at most 10 source codes.";
const BAD_SOURCE_CODE: &str =
    "sourceCodes must contain non-blank codes of at most 200 characters, without duplicates.";
const UNKNOWN_SOURCE: &str = "sourceCodes contains an unknown source code.";
const ONE_SOURCE: &str =
    "More than one citation source holds data, so sourceCodes must name exactly one.";
const DATE_ORDER: &str = "citingFrom must not be after citingTo.";
const BAD_PAGE_SIZE: &str = "first must be between 1 and 100.";
const BAD_CURSOR: &str = "after is not a valid cursor for this request.";

/// The request of `metricCitations`.
#[derive(juniper::GraphQLInputObject, Debug, Clone, PartialEq, Eq)]
#[graphql(description = "A bounded request for the citation links of selected publishers")]
pub struct MetricCitationsInput {
    #[graphql(
        description = "The publishers to read, one to 25. Every one must exist and be entitled to the Metrics dashboard, else the whole request is refused"
    )]
    pub publisher_ids: Vec<Uuid>,
    #[graphql(
        description = "Limit the read to these works, at most 100. Each must belong to a selected publisher; an unknown work and another publisher's work are refused alike"
    )]
    pub work_ids: Option<Vec<Uuid>>,
    #[graphql(
        description = "Only citing works dated on or after this day. A citing work known only by its year is dated 1 January of that year"
    )]
    pub citing_from: Option<NaiveDate>,
    #[graphql(
        description = "Only citing works dated on or before this day. A citing work known only by its year is dated 1 January of that year"
    )]
    pub citing_to: Option<NaiveDate>,
    #[graphql(
        description = "Limit the read to these source codes, at most 10. Counts are never summed across sources, so when more than one source holds data exactly one code is required"
    )]
    pub source_codes: Option<Vec<String>>,
    #[graphql(description = "Page size, 1 to 100. Defaults to 50")]
    pub first: Option<i32>,
    #[graphql(
        description = "The `endCursor` of the previous page. Valid only with the same publishers and filters"
    )]
    pub after: Option<String>,
}

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq, Eq)]
#[graphql(description = "Citing works of one year")]
pub struct MetricCitationYearCount {
    #[graphql(
        description = "The citing work's publication year; null for citing works with no known year"
    )]
    pub year: Option<i32>,
    pub count: i32,
}

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq, Eq)]
#[graphql(
    description = "Counts over every selected work and filter, not only the returned page. Counts are never summed across sources"
)]
pub struct MetricCitationsSummary {
    #[graphql(description = "Active citation links in scope")]
    pub link_count: i32,
    #[graphql(description = "Works in scope with at least one active citation link")]
    pub cited_work_count: i32,
    #[graphql(description = "Works the source matched by DOI on its last resolved attempt")]
    pub matched_work_count: i32,
    #[graphql(
        description = "Works whose last attempt failed, or that the source no longer finds after matching them, or whose enumeration was not confirmed: their links may be out of date"
    )]
    pub unconfirmed_work_count: i32,
    #[graphql(
        description = "Works the source does not know by DOI, excluding unconfirmed ones. Zero citations and not found are different things"
    )]
    pub not_found_work_count: i32,
    #[graphql(description = "Works in scope with a DOI that no attempt has reached yet")]
    pub not_yet_refreshed_work_count: i32,
    #[graphql(
        description = "The source's own reported citation count, summed over matched works. It can differ from the number of links Thoth lists"
    )]
    pub reported_count_total: i32,
    #[graphql(
        description = "The oldest completed refresh across matched works in scope, so the page never claims freshness it lacks; null when any matched work has no completed refresh or none is matched"
    )]
    pub last_refreshed_at: Option<Timestamp>,
    #[graphql(description = "Active links per citing publication year, oldest first")]
    pub by_citing_year: Vec<MetricCitationYearCount>,
}

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq, Eq)]
#[graphql(description = "The citing work's authors as the source lists them")]
pub struct MetricCitationAuthors {
    #[graphql(description = "At most five author names, plain text")]
    pub names: Vec<String>,
    #[graphql(description = "How many authors the citing work has in all, when known")]
    pub total: Option<i32>,
}

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq, Eq)]
#[graphql(description = "The source a citation link comes from")]
pub struct MetricCitationSource {
    pub code: String,
}

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq, Eq)]
#[graphql(description = "The Thoth work a citation link points at")]
pub struct MetricCitedWork {
    pub work_id: Uuid,
    #[graphql(description = "The work's canonical full title, when it has one")]
    pub full_title: Option<String>,
    #[graphql(description = "The work's DOI, as Thoth stores it")]
    pub doi: Option<String>,
    pub work_type: WorkType,
}

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq, Eq)]
#[graphql(
    description = "One citing work. All text is plain text from a third-party source and must be escaped on display"
)]
pub struct MetricCitationLink {
    pub link_id: Uuid,
    #[graphql(description = "The source's own identifier of the citing work")]
    pub citing_external_id: String,
    pub citing_doi: Option<String>,
    pub citing_title: Option<String>,
    pub citing_type: Option<String>,
    pub citing_publication_year: Option<i32>,
    pub citing_publication_date: Option<NaiveDate>,
    #[graphql(
        description = "The ordering date: the publication date, else 1 January of the year, else 0001-01-01"
    )]
    pub citing_sort_date: NaiveDate,
    pub citing_venue: Option<String>,
    pub citing_authors: Option<MetricCitationAuthors>,
    pub citing_url: Option<String>,
    pub source: MetricCitationSource,
    pub cited_work: MetricCitedWork,
    pub first_seen_at: Timestamp,
}

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq, Eq)]
#[graphql(description = "Where the next page starts")]
pub struct MetricCitationsPageInfo {
    pub has_next_page: bool,
    #[graphql(description = "Pass as `after` for the next page; null when there is none")]
    pub end_cursor: Option<String>,
}

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq, Eq)]
#[graphql(description = "Citation links and their summary, read in one snapshot")]
pub struct MetricCitations {
    pub summary: MetricCitationsSummary,
    #[graphql(description = "Active links, newest citing work first")]
    pub links: Vec<MetricCitationLink>,
    pub page_info: MetricCitationsPageInfo,
}

/// A request after validation.
struct Validated {
    publishers: Vec<Uuid>,
    works: Vec<Uuid>,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    sources: Vec<String>,
    first: i32,
    cursor: Option<Cursor>,
}

/// The keyset position, bound to the request it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Cursor {
    sort_date: NaiveDate,
    link_id: Uuid,
}

/// Answer one `metricCitations` request.
///
/// The caller has already passed `METRICS_READ_SERVICE`.
pub(crate) fn metric_citations(
    db: &PgPool,
    serving: MetricCitationsServing,
    input: &MetricCitationsInput,
) -> Result<MetricCitations, MetricReadError> {
    if serving == MetricCitationsServing::Off {
        return Err(MetricReadError::CitationsNotServed);
    }
    let request = validate(input)?;
    let _permit = METRIC_DASHBOARD_READS.acquire(METRIC_DASHBOARD_SLOT_WAIT)?;
    let deadline = ReadDeadline::new(Instant::now() + METRIC_DASHBOARD_READ_DEADLINE);
    let mut connection = checkout(db, &deadline)?;
    deadline.check()?;
    connection
        .build_transaction()
        .read_only()
        .repeatable_read()
        .run(|connection| evaluate(connection, &request, &deadline))
}

fn validate(input: &MetricCitationsInput) -> Result<Validated, MetricReadError> {
    let publishers = &input.publisher_ids;
    if publishers.is_empty() {
        return Err(MetricReadError::QueryInvalid(PUBLISHER_CARDINALITY));
    }
    if publishers.iter().collect::<BTreeSet<_>>().len() != publishers.len() {
        return Err(MetricReadError::QueryInvalid(DUPLICATE_PUBLISHER));
    }
    if publishers.len() > METRIC_CITATIONS_MAX_PUBLISHERS {
        return Err(MetricReadError::QueryLimitExceeded(PUBLISHER_CARDINALITY));
    }
    let works = input.work_ids.clone().unwrap_or_default();
    if works.len() > METRIC_CITATIONS_MAX_WORKS {
        return Err(MetricReadError::QueryLimitExceeded(TOO_MANY_WORKS));
    }
    if works.iter().collect::<BTreeSet<_>>().len() != works.len() {
        return Err(MetricReadError::QueryInvalid(DUPLICATE_WORK));
    }
    let sources = input.source_codes.clone().unwrap_or_default();
    if sources.len() > METRIC_CITATIONS_MAX_SOURCES {
        return Err(MetricReadError::QueryLimitExceeded(TOO_MANY_SOURCES));
    }
    if sources
        .iter()
        .any(|code| code.trim().is_empty() || code.chars().count() > MAX_SOURCE_CODE_CHARS)
        || sources.iter().collect::<BTreeSet<_>>().len() != sources.len()
    {
        return Err(MetricReadError::QueryInvalid(BAD_SOURCE_CODE));
    }
    if let (Some(from), Some(to)) = (input.citing_from, input.citing_to) {
        if from > to {
            return Err(MetricReadError::QueryInvalid(DATE_ORDER));
        }
    }
    let first = input.first.unwrap_or(METRIC_CITATIONS_DEFAULT_PAGE);
    if first < 1 {
        return Err(MetricReadError::QueryInvalid(BAD_PAGE_SIZE));
    }
    if first > METRIC_CITATIONS_MAX_PAGE {
        return Err(MetricReadError::QueryLimitExceeded(BAD_PAGE_SIZE));
    }
    let mut request = Validated {
        publishers: publishers.clone(),
        works,
        from: input.citing_from,
        to: input.citing_to,
        sources,
        first,
        cursor: None,
    };
    request.publishers.sort();
    request.works.sort();
    request.sources.sort();
    if let Some(after) = &input.after {
        request.cursor = Some(decode_cursor(after, &filter_hash(&request))?);
    }
    Ok(request)
}

/// A hash of everything a cursor must not outlive: the selected publishers,
/// works, sources and date range, in canonical order.
fn filter_hash(request: &Validated) -> String {
    let mut text = String::from("metric-citations-cursor-v1");
    let join = |ids: &[Uuid]| {
        ids.iter()
            .map(Uuid::to_string)
            .collect::<Vec<_>>()
            .join(",")
    };
    text.push_str(&format!("|p={}", join(&request.publishers)));
    text.push_str(&format!("|w={}", join(&request.works)));
    text.push_str(&format!("|s={}", request.sources.join("\u{1f}")));
    text.push_str(&format!(
        "|f={}|t={}",
        request.from.map(|d| d.to_string()).unwrap_or_default(),
        request.to.map(|d| d.to_string()).unwrap_or_default()
    ));
    hex::encode(Sha256::digest(text.as_bytes()))
}

fn encode_cursor(hash: &str, cursor: &Cursor) -> String {
    let body = serde_json::json!({
        "h": hash,
        "d": cursor.sort_date.to_string(),
        "l": cursor.link_id.to_string(),
    });
    hex::encode(body.to_string())
}

fn decode_cursor(after: &str, expected_hash: &str) -> Result<Cursor, MetricReadError> {
    let invalid = || MetricReadError::QueryInvalid(BAD_CURSOR);
    if after.is_empty() || after.len() > MAX_CURSOR_CHARS {
        return Err(invalid());
    }
    let bytes = hex::decode(after).map_err(|_| invalid())?;
    let body: JsonValue = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    let field = |name: &str| body.get(name).and_then(JsonValue::as_str);
    if field("h") != Some(expected_hash) {
        return Err(invalid());
    }
    let sort_date = field("d")
        .and_then(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
        .ok_or_else(invalid)?;
    let link_id = field("l")
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or_else(invalid)?;
    Ok(Cursor { sort_date, link_id })
}

#[derive(diesel::QueryableByName)]
struct PublisherRow {
    #[diesel(sql_type = SqlUuid)]
    publisher_id: Uuid,
    #[diesel(sql_type = crate::schema::sql_types::ThothPackage)]
    subscription_package: ThothPackage,
}

#[derive(diesel::QueryableByName)]
struct CountRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    n: i64,
}

#[derive(diesel::QueryableByName)]
struct SourceRow {
    #[diesel(sql_type = SqlUuid)]
    source_id: Uuid,
    #[diesel(sql_type = Bool)]
    has_data: bool,
}

#[derive(diesel::QueryableByName)]
struct SummaryRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    link_count: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    cited_work_count: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    matched: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    unconfirmed: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    not_found: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    not_yet: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    reported_total: i64,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    last_refreshed: Option<Timestamp>,
}

#[derive(diesel::QueryableByName)]
struct YearRow {
    #[diesel(sql_type = Nullable<Integer>)]
    year: Option<i32>,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    n: i64,
}

#[derive(diesel::QueryableByName)]
struct LinkRow {
    #[diesel(sql_type = SqlUuid)]
    link_id: Uuid,
    #[diesel(sql_type = Text)]
    citing_external_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    citing_doi: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    citing_title: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    citing_type: Option<String>,
    #[diesel(sql_type = Nullable<SmallInt>)]
    citing_publication_year: Option<i16>,
    #[diesel(sql_type = Nullable<Date>)]
    citing_publication_date: Option<NaiveDate>,
    #[diesel(sql_type = Date)]
    citing_sort_date: NaiveDate,
    #[diesel(sql_type = Nullable<Text>)]
    citing_venue: Option<String>,
    #[diesel(sql_type = Nullable<Jsonb>)]
    citing_authors: Option<JsonValue>,
    #[diesel(sql_type = Nullable<Text>)]
    citing_url: Option<String>,
    #[diesel(sql_type = Timestamptz)]
    first_seen_at: Timestamp,
    #[diesel(sql_type = Text)]
    source_code: String,
    #[diesel(sql_type = SqlUuid)]
    work_id: Uuid,
    #[diesel(sql_type = Nullable<Text>)]
    full_title: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    work_doi: Option<String>,
    #[diesel(sql_type = crate::schema::sql_types::WorkType)]
    work_type: WorkType,
}

/// The works of the selected publishers, narrowed by `$2` (an empty array
/// selects them all).
const SCOPE_WORKS: &str = "SELECT w.work_id, w.doi, w.work_status \
       FROM public.work w \
       JOIN public.imprint i ON i.imprint_id = w.imprint_id \
      WHERE i.publisher_id = ANY($1) \
        AND (cardinality($2::uuid[]) = 0 OR w.work_id = ANY($2))";

const PUBLISHERS_SQL: &str = "SELECT publisher_id, subscription_package \
       FROM public.publisher WHERE publisher_id = ANY($1)";

const WORKS_CHECK_SQL: &str = "SELECT count(*) AS n \
       FROM public.work w \
       JOIN public.imprint i ON i.imprint_id = w.imprint_id \
      WHERE w.work_id = ANY($1) AND i.publisher_id = ANY($2)";

const SOURCES_SQL: &str = "SELECT m.source_id, \
              EXISTS (SELECT 1 FROM public.metric_citation_work_state s \
                       WHERE s.source_id = m.source_id) AS has_data \
         FROM public.metric_source m \
        WHERE cardinality($1::text[]) = 0 OR m.code = ANY($1)";

/// `$1` publishers, `$2` works, `$3` from, `$4` to, `$5` source ids.
fn summary_sql() -> String {
    format!(
        "WITH sw AS ({SCOPE_WORKS}), \
              fl AS ( \
                SELECT l.cited_work_id \
                  FROM public.metric_citation_link l \
                  JOIN sw ON sw.work_id = l.cited_work_id \
                 WHERE l.source_id = ANY($5) AND l.status = 'ACTIVE' \
                   AND ($3::date IS NULL OR l.citing_sort_date >= $3) \
                   AND ($4::date IS NULL OR l.citing_sort_date <= $4)), \
              st AS ( \
                SELECT s.* FROM public.metric_citation_work_state s \
                  JOIN sw ON sw.work_id = s.cited_work_id \
                 WHERE s.source_id = ANY($5)) \
         SELECT (SELECT count(*) FROM fl) AS link_count, \
                (SELECT count(DISTINCT cited_work_id) FROM fl) AS cited_work_count, \
                (SELECT count(*) FROM st WHERE match_status = 'MATCHED') AS matched, \
                (SELECT count(*) FROM st WHERE unconfirmed) AS unconfirmed, \
                (SELECT count(*) FROM st \
                  WHERE match_status = 'NOT_FOUND' AND NOT unconfirmed) AS not_found, \
                (SELECT count(*) FROM sw \
                  WHERE sw.doi IS NOT NULL \
                    AND sw.work_status NOT IN ('withdrawn', 'cancelled') \
                    AND NOT EXISTS (SELECT 1 FROM st WHERE st.cited_work_id = sw.work_id)) \
                  AS not_yet, \
                (SELECT COALESCE(sum(reported_count), 0)::bigint FROM st \
                  WHERE match_status = 'MATCHED') AS reported_total, \
                (SELECT CASE WHEN bool_or(last_refreshed_at IS NULL) THEN NULL \
                             ELSE min(last_refreshed_at) END \
                   FROM st WHERE match_status = 'MATCHED') AS last_refreshed"
    )
}

fn years_sql() -> String {
    format!(
        "WITH sw AS ({SCOPE_WORKS}) \
         SELECT l.citing_publication_year::int AS year, count(*) AS n \
           FROM public.metric_citation_link l \
           JOIN sw ON sw.work_id = l.cited_work_id \
          WHERE l.source_id = ANY($5) AND l.status = 'ACTIVE' \
            AND ($3::date IS NULL OR l.citing_sort_date >= $3) \
            AND ($4::date IS NULL OR l.citing_sort_date <= $4) \
          GROUP BY 1 ORDER BY 1 ASC NULLS LAST"
    )
}

/// `$1` to `$5` as above, `$6` cursor date, `$7` cursor link, `$8` limit.
fn page_sql() -> String {
    format!(
        "WITH sw AS ({SCOPE_WORKS}) \
         SELECT l.link_id, l.citing_external_id, l.citing_doi, l.citing_title, l.citing_type, \
                l.citing_publication_year, l.citing_publication_date, l.citing_sort_date, \
                l.citing_venue, l.citing_authors, l.citing_url, l.first_seen_at, \
                m.code AS source_code, w.work_id, t.full_title, w.doi AS work_doi, w.work_type \
           FROM public.metric_citation_link l \
           JOIN sw ON sw.work_id = l.cited_work_id \
           JOIN public.work w ON w.work_id = l.cited_work_id \
           LEFT JOIN public.title t ON t.work_id = w.work_id AND t.canonical \
           JOIN public.metric_source m ON m.source_id = l.source_id \
          WHERE l.source_id = ANY($5) AND l.status = 'ACTIVE' \
            AND ($3::date IS NULL OR l.citing_sort_date >= $3) \
            AND ($4::date IS NULL OR l.citing_sort_date <= $4) \
            AND ($6::date IS NULL OR (l.citing_sort_date, l.link_id) < ($6::date, $7::uuid)) \
          ORDER BY l.citing_sort_date DESC, l.link_id DESC \
          LIMIT $8"
    )
}

fn evaluate(
    connection: &mut diesel::PgConnection,
    request: &Validated,
    deadline: &ReadDeadline,
) -> Result<MetricCitations, MetricReadError> {
    deadline.check()?;
    sql_query(CUSTOM_PLANS_SQL).execute(connection)?;
    deadline.check()?;
    sql_query(STATEMENT_TIMEOUT_SQL).execute(connection)?;

    // 1. Publishers and entitlement, on every page.
    deadline.check()?;
    let rows: Vec<PublisherRow> = sql_query(PUBLISHERS_SQL)
        .bind::<Array<SqlUuid>, _>(&request.publishers)
        .load(connection)?;
    let found: BTreeSet<Uuid> = rows.iter().map(|row| row.publisher_id).collect();
    if request
        .publishers
        .iter()
        .any(|publisher| !found.contains(publisher))
    {
        return Err(MetricReadError::QueryInvalid(UNKNOWN_PUBLISHER));
    }
    // The capability comes from each publisher's package alone: holding
    // METRICS_READ_SERVICE contributes nothing here.
    if rows.iter().any(|row| {
        !row.subscription_package
            .has_capability(PublisherCapability::MetricsDashboard)
    }) {
        return Err(MetricReadError::Unauthorised);
    }

    // 2. Listed works must be the selected publishers' own.
    if !request.works.is_empty() {
        deadline.check()?;
        let owned = sql_query(WORKS_CHECK_SQL)
            .bind::<Array<SqlUuid>, _>(&request.works)
            .bind::<Array<SqlUuid>, _>(&request.publishers)
            .get_result::<CountRow>(connection)?
            .n;
        if owned != request.works.len() as i64 {
            return Err(MetricReadError::QueryInvalid(UNKNOWN_WORK));
        }
    }

    // 3. Sources.
    deadline.check()?;
    let sources: Vec<SourceRow> = sql_query(SOURCES_SQL)
        .bind::<Array<Text>, _>(&request.sources)
        .load(connection)?;
    if !request.sources.is_empty() && sources.len() != request.sources.len() {
        return Err(MetricReadError::QueryInvalid(UNKNOWN_SOURCE));
    }
    let citing_sources: Vec<Uuid> = sources
        .iter()
        .filter(|source| source.has_data)
        .map(|source| source.source_id)
        .collect();
    if citing_sources.len() > 1 {
        return Err(MetricReadError::QueryInvalid(ONE_SOURCE));
    }

    // 4. Summary, histogram and one page, in this one snapshot.
    deadline.check()?;
    let summary: SummaryRow = sql_query(summary_sql())
        .bind::<Array<SqlUuid>, _>(&request.publishers)
        .bind::<Array<SqlUuid>, _>(&request.works)
        .bind::<Nullable<Date>, _>(request.from)
        .bind::<Nullable<Date>, _>(request.to)
        .bind::<Array<SqlUuid>, _>(&citing_sources)
        .get_result(connection)?;
    deadline.check()?;
    let years: Vec<YearRow> = sql_query(years_sql())
        .bind::<Array<SqlUuid>, _>(&request.publishers)
        .bind::<Array<SqlUuid>, _>(&request.works)
        .bind::<Nullable<Date>, _>(request.from)
        .bind::<Nullable<Date>, _>(request.to)
        .bind::<Array<SqlUuid>, _>(&citing_sources)
        .load(connection)?;
    deadline.check()?;
    let mut page: Vec<LinkRow> = sql_query(page_sql())
        .bind::<Array<SqlUuid>, _>(&request.publishers)
        .bind::<Array<SqlUuid>, _>(&request.works)
        .bind::<Nullable<Date>, _>(request.from)
        .bind::<Nullable<Date>, _>(request.to)
        .bind::<Array<SqlUuid>, _>(&citing_sources)
        .bind::<Nullable<Date>, _>(request.cursor.as_ref().map(|c| c.sort_date))
        .bind::<Nullable<SqlUuid>, _>(request.cursor.as_ref().map(|c| c.link_id))
        .bind::<Integer, _>(request.first + 1)
        .load(connection)?;

    let has_next_page = page.len() > request.first as usize;
    page.truncate(request.first as usize);
    let end_cursor = if has_next_page {
        page.last().map(|row| {
            encode_cursor(
                &filter_hash(request),
                &Cursor {
                    sort_date: row.citing_sort_date,
                    link_id: row.link_id,
                },
            )
        })
    } else {
        None
    };

    let count = |value: i64| i32::try_from(value).unwrap_or(i32::MAX);
    Ok(MetricCitations {
        summary: MetricCitationsSummary {
            link_count: count(summary.link_count),
            cited_work_count: count(summary.cited_work_count),
            matched_work_count: count(summary.matched),
            unconfirmed_work_count: count(summary.unconfirmed),
            not_found_work_count: count(summary.not_found),
            not_yet_refreshed_work_count: count(summary.not_yet),
            reported_count_total: count(summary.reported_total),
            last_refreshed_at: summary.last_refreshed,
            by_citing_year: years
                .into_iter()
                .map(|row| MetricCitationYearCount {
                    year: row.year,
                    count: count(row.n),
                })
                .collect(),
        },
        links: page.into_iter().map(link_from_row).collect(),
        page_info: MetricCitationsPageInfo {
            has_next_page,
            end_cursor,
        },
    })
}

fn link_from_row(row: LinkRow) -> MetricCitationLink {
    MetricCitationLink {
        link_id: row.link_id,
        citing_external_id: row.citing_external_id,
        citing_doi: row.citing_doi,
        citing_title: row.citing_title,
        citing_type: row.citing_type,
        citing_publication_year: row.citing_publication_year.map(i32::from),
        citing_publication_date: row.citing_publication_date,
        citing_sort_date: row.citing_sort_date,
        citing_venue: row.citing_venue,
        citing_authors: row.citing_authors.as_ref().and_then(authors_from_json),
        citing_url: row.citing_url,
        source: MetricCitationSource {
            code: row.source_code,
        },
        cited_work: MetricCitedWork {
            work_id: row.work_id,
            full_title: row.full_title,
            doi: row.work_doi,
            work_type: row.work_type,
        },
        first_seen_at: row.first_seen_at,
    }
}

/// The stored `{"names": [...], "total": n}` object; a value of another shape
/// (which the table's CHECK keeps out) reads as absent rather than failing the
/// page.
fn authors_from_json(value: &JsonValue) -> Option<MetricCitationAuthors> {
    let names = value
        .get("names")?
        .as_array()?
        .iter()
        .filter_map(|name| name.as_str().map(str::to_owned))
        .collect();
    let total = value
        .get("total")
        .and_then(JsonValue::as_i64)
        .and_then(|total| i32::try_from(total).ok());
    Some(MetricCitationAuthors { names, total })
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    fn input() -> MetricCitationsInput {
        MetricCitationsInput {
            publisher_ids: vec![Uuid::new_v4()],
            work_ids: None,
            citing_from: None,
            citing_to: None,
            source_codes: None,
            first: None,
            after: None,
        }
    }

    fn invalid(input: &MetricCitationsInput) -> String {
        match validate(input) {
            Err(error) => error.code().to_owned(),
            Ok(_) => "OK".to_owned(),
        }
    }

    #[test]
    fn the_request_bounds_are_enforced_without_a_database() {
        assert_eq!(invalid(&input()), "OK");
        let mut empty = input();
        empty.publisher_ids.clear();
        assert_eq!(invalid(&empty), "METRIC_QUERY_INVALID");
        let mut many = input();
        many.publisher_ids = (0..26).map(|_| Uuid::new_v4()).collect();
        assert_eq!(invalid(&many), "METRIC_QUERY_LIMIT_EXCEEDED");
        let mut dup = input();
        dup.publisher_ids = vec![dup.publisher_ids[0]; 2];
        assert_eq!(invalid(&dup), "METRIC_QUERY_INVALID");
        let mut works = input();
        works.work_ids = Some((0..101).map(|_| Uuid::new_v4()).collect());
        assert_eq!(invalid(&works), "METRIC_QUERY_LIMIT_EXCEEDED");
        let mut first = input();
        first.first = Some(101);
        assert_eq!(invalid(&first), "METRIC_QUERY_LIMIT_EXCEEDED");
        first.first = Some(0);
        assert_eq!(invalid(&first), "METRIC_QUERY_INVALID");
        first.first = Some(100);
        assert_eq!(invalid(&first), "OK");
        let mut dates = input();
        dates.citing_from = NaiveDate::from_ymd_opt(2024, 2, 1);
        dates.citing_to = NaiveDate::from_ymd_opt(2024, 1, 1);
        assert_eq!(invalid(&dates), "METRIC_QUERY_INVALID");
        let mut sources = input();
        sources.source_codes = Some(vec![" ".to_owned()]);
        assert_eq!(invalid(&sources), "METRIC_QUERY_INVALID");
        let mut cursor = input();
        cursor.after = Some("not-hex".to_owned());
        assert_eq!(invalid(&cursor), "METRIC_QUERY_INVALID");
    }

    #[test]
    fn a_cursor_round_trips_and_is_bound_to_its_request() {
        let one = validate(&input()).unwrap();
        let mut other_input = input();
        other_input.publisher_ids = vec![Uuid::new_v4()];
        let other = validate(&other_input).unwrap();
        let position = Cursor {
            sort_date: NaiveDate::from_ymd_opt(2020, 5, 17).unwrap(),
            link_id: Uuid::new_v4(),
        };
        let encoded = encode_cursor(&filter_hash(&one), &position);
        assert_eq!(decode_cursor(&encoded, &filter_hash(&one)), Ok(position));
        assert!(decode_cursor(&encoded, &filter_hash(&other)).is_err());
        assert!(decode_cursor(&"a".repeat(MAX_CURSOR_CHARS + 2), &filter_hash(&one)).is_err());
    }

    #[test]
    fn the_filter_hash_ignores_input_order_but_not_content() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let mut first = input();
        first.publisher_ids = vec![a, b];
        let mut second = input();
        second.publisher_ids = vec![b, a];
        assert_eq!(
            filter_hash(&validate(&first).unwrap()),
            filter_hash(&validate(&second).unwrap())
        );
        second.citing_from = NaiveDate::from_ymd_opt(2020, 1, 1);
        assert_ne!(
            filter_hash(&validate(&first).unwrap()),
            filter_hash(&validate(&second).unwrap())
        );
    }

    #[test]
    fn stored_authors_read_back_and_a_foreign_shape_reads_as_absent() {
        let stored = serde_json::json!({"names": ["A. B.", "C. D."], "total": 7});
        let authors = authors_from_json(&stored).unwrap();
        assert_eq!(authors.names, vec!["A. B.", "C. D."]);
        assert_eq!(authors.total, Some(7));
        assert!(authors_from_json(&serde_json::json!({"x": 1})).is_none());
    }
}
