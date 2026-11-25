//! Exact ISBN to work lookup for platform usage drivers (`THOTH-ISBN-01`).
//!
//! Platforms such as Project MUSE, Google Books and JSTOR report usage by ISBN
//! and carry no Thoth DOI. The public, read-only `workIsbnMatches` query turns a
//! batch of ISBNs, as a platform wrote them, into the Thoth work that owns the
//! matching publication.
//!
//! - One result per input element, in input order; duplicates give duplicates.
//! - Normalisation is [`normalise_isbn`]: ISBN-10 and ISBN-13 with or without
//!   hyphens, spaces or an `ISBN` prefix, check digit validated.
//! - Matching compares digits only, against publications of every type, so the
//!   stored hyphenation never causes a miss.
//! - Several publications of one work are one match. Publications of different
//!   works are `AMBIGUOUS`. A work with no DOI is `NO_DOI`.
//!
//! The query writes nothing, creates no state and needs no role.

use std::collections::BTreeMap;
use std::str::FromStr;

use diesel::sql_types::{Array, Nullable, Text, Uuid as SqlUuid};
use diesel::{sql_query, QueryableByName, RunQueryDsl};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use crate::db::PgPool;
use crate::model::work::WorkType;
use crate::model::{Doi, Isbn};

/// The most ISBNs one call accepts.
pub const ISBN_LOOKUP_MAX_INPUTS: usize = 500;

#[derive(Debug, Copy, Clone, PartialEq, Eq, juniper::GraphQLEnum)]
#[graphql(description = "Outcome of matching one ISBN to a Thoth work")]
pub enum IsbnMatchStatus {
    #[graphql(
        description = "Exactly one work owns a publication with this ISBN, and it has a DOI"
    )]
    Matched,
    #[graphql(description = "No publication in scope has this ISBN")]
    Unknown,
    #[graphql(description = "Publications of more than one work in scope have this ISBN")]
    Ambiguous,
    #[graphql(description = "Not a valid ISBN-10 or ISBN-13 (shape or check digit)")]
    Invalid,
    #[graphql(description = "Exactly one work owns it, but the work has no DOI")]
    NoDoi,
}

#[derive(Debug, Clone, PartialEq, Eq, juniper::GraphQLObject)]
#[graphql(description = "The work matched by one ISBN of a workIsbnMatches call")]
pub struct IsbnWorkMatch {
    #[graphql(description = "The ISBN exactly as supplied")]
    pub isbn: String,
    #[graphql(
        description = "The ISBN-13, hyphenated as Thoth stores it; null when the input is invalid"
    )]
    pub normalised_isbn: Option<String>,
    pub status: IsbnMatchStatus,
    #[graphql(description = "The owning work, when MATCHED or NO_DOI")]
    pub work_id: Option<Uuid>,
    #[graphql(description = "The work's DOI, when MATCHED")]
    pub work_doi: Option<Doi>,
    #[graphql(description = "The work's type, when MATCHED or NO_DOI")]
    pub work_type: Option<WorkType>,
    #[graphql(description = "The publisher of the work's imprint, when MATCHED or NO_DOI")]
    pub publisher_id: Option<Uuid>,
}

/// A valid ISBN-13: its 13 digits and its hyphenated form where Thoth can
/// hyphenate it (digits otherwise).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalisedIsbn {
    pub digits: String,
    pub display: String,
}

fn is_dash(c: char) -> bool {
    matches!(c, '-' | '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}')
}

fn isbn13_check_ok(digits: &[u32]) -> bool {
    let sum: u32 = digits
        .iter()
        .enumerate()
        .map(|(i, d)| if i % 2 == 0 { *d } else { *d * 3 })
        .sum();
    sum.is_multiple_of(10)
}

fn isbn10_check_ok(values: &[u32]) -> bool {
    let sum: u32 = values
        .iter()
        .enumerate()
        .map(|(i, v)| (10 - i as u32) * *v)
        .sum();
    sum.is_multiple_of(11)
}

/// After the word `ISBN`: skip separators and an optional `10`/`13` label that
/// is followed by a colon (`ISBN-13: 978...`).
fn strip_label(rest: &str) -> &str {
    let rest = rest.trim_start_matches([':', '-', ' ', '\u{a0}']);
    for label in ["10", "13"] {
        if let Some(after) = rest.strip_prefix(label) {
            let after_ws = after.trim_start_matches([' ', '\u{a0}']);
            if let Some(tail) = after_ws.strip_prefix(':') {
                return tail;
            }
        }
    }
    rest
}

/// Normalise one platform-supplied ISBN. `None` means invalid.
pub fn normalise_isbn(input: &str) -> Option<NormalisedIsbn> {
    let trimmed = input.trim();
    let rest = match trimmed.get(..4) {
        Some(prefix) if prefix.eq_ignore_ascii_case("isbn") => strip_label(&trimmed[4..]),
        _ => trimmed,
    };
    let cleaned: String = rest
        .chars()
        .filter(|c| !is_dash(*c) && !c.is_whitespace() && *c != '\u{a0}')
        .collect();

    let digits13 = match cleaned.chars().count() {
        13 => {
            if !cleaned.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
            let values: Vec<u32> = cleaned.chars().filter_map(|c| c.to_digit(10)).collect();
            if !isbn13_check_ok(&values)
                || !(cleaned.starts_with("978") || cleaned.starts_with("979"))
            {
                return None;
            }
            cleaned
        }
        10 => {
            let chars: Vec<char> = cleaned.chars().collect();
            let mut values = Vec::with_capacity(10);
            for (i, c) in chars.iter().enumerate() {
                match (i, *c) {
                    (9, 'X') | (9, 'x') => values.push(10),
                    (_, c) if c.is_ascii_digit() => values.push(c.to_digit(10)?),
                    _ => return None,
                }
            }
            if !isbn10_check_ok(&values) {
                return None;
            }
            let body: String = chars[..9].iter().collect();
            let mut core = format!("978{body}");
            let sum: u32 = core
                .chars()
                .filter_map(|c| c.to_digit(10))
                .enumerate()
                .map(|(i, d)| if i % 2 == 0 { d } else { d * 3 })
                .sum();
            core.push(char::from_digit((10 - sum % 10) % 10, 10)?);
            core
        }
        _ => return None,
    };

    let display = Isbn::from_str(&digits13)
        .map(|isbn| isbn.to_string())
        .unwrap_or_else(|_| digits13.clone());
    Some(NormalisedIsbn {
        digits: digits13,
        display,
    })
}

#[derive(QueryableByName)]
struct MatchRow {
    #[diesel(sql_type = Text)]
    digits: String,
    #[diesel(sql_type = SqlUuid)]
    work_id: Uuid,
    #[diesel(sql_type = Nullable<Text>)]
    doi: Option<String>,
    #[diesel(sql_type = crate::schema::sql_types::WorkType)]
    work_type: WorkType,
    #[diesel(sql_type = SqlUuid)]
    publisher_id: Uuid,
}

const MATCH_SQL: &str = "\
SELECT replace(p.isbn, '-', '') AS digits, w.work_id, w.doi, w.work_type, \
       i.publisher_id \
  FROM publication p \
  JOIN work w ON w.work_id = p.work_id \
  JOIN imprint i ON i.imprint_id = w.imprint_id \
 WHERE replace(p.isbn, '-', '') = ANY($1) \
   AND (cardinality($2::uuid[]) = 0 OR i.publisher_id = ANY($2))";

/// Resolve a batch of ISBNs; see the module documentation.
pub fn work_isbn_matches(
    db: &PgPool,
    isbns: &[String],
    publishers: &[Uuid],
) -> ThothResult<Vec<IsbnWorkMatch>> {
    if isbns.len() > ISBN_LOOKUP_MAX_INPUTS {
        return Err(ThothError::InternalError(format!(
            "at most {ISBN_LOOKUP_MAX_INPUTS} ISBNs per call"
        )));
    }
    let normalised: Vec<Option<NormalisedIsbn>> = isbns.iter().map(|i| normalise_isbn(i)).collect();
    let mut wanted: Vec<String> = normalised
        .iter()
        .flatten()
        .map(|n| n.digits.clone())
        .collect();
    wanted.sort();
    wanted.dedup();

    let mut found: BTreeMap<String, Vec<MatchRow>> = BTreeMap::new();
    if !wanted.is_empty() {
        let mut connection = db.get()?;
        let rows: Vec<MatchRow> = sql_query(MATCH_SQL)
            .bind::<Array<Text>, _>(&wanted)
            .bind::<Array<SqlUuid>, _>(publishers)
            .load(&mut connection)?;
        for row in rows {
            found.entry(row.digits.clone()).or_default().push(row);
        }
    }

    Ok(isbns
        .iter()
        .zip(normalised)
        .map(|(raw, norm)| build_match(raw, norm, &found))
        .collect())
}

fn build_match(
    raw: &str,
    norm: Option<NormalisedIsbn>,
    found: &BTreeMap<String, Vec<MatchRow>>,
) -> IsbnWorkMatch {
    let blank = |status, normalised_isbn| IsbnWorkMatch {
        isbn: raw.to_string(),
        normalised_isbn,
        status,
        work_id: None,
        work_doi: None,
        work_type: None,
        publisher_id: None,
    };
    let Some(norm) = norm else {
        return blank(IsbnMatchStatus::Invalid, None);
    };
    let Some(rows) = found.get(&norm.digits) else {
        return blank(IsbnMatchStatus::Unknown, Some(norm.display));
    };
    let mut work_ids: Vec<Uuid> = rows.iter().map(|r| r.work_id).collect();
    work_ids.sort();
    work_ids.dedup();
    if work_ids.len() > 1 {
        return blank(IsbnMatchStatus::Ambiguous, Some(norm.display));
    }
    let row = &rows[0];
    let doi = row
        .doi
        .as_deref()
        .filter(|d| !d.trim().is_empty())
        .and_then(|d| Doi::from_str(d).ok());
    IsbnWorkMatch {
        isbn: raw.to_string(),
        normalised_isbn: Some(norm.display),
        status: if doi.is_some() {
            IsbnMatchStatus::Matched
        } else {
            IsbnMatchStatus::NoDoi
        },
        work_id: Some(row.work_id),
        work_doi: doi,
        work_type: Some(row.work_type),
        publisher_id: Some(row.publisher_id),
    }
}

#[cfg(test)]
mod tests;
