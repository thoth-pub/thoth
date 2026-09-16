//! Code-owned input validation for the BE-06 operations (Amendment 3 section 7).
//!
//! Every rule here is evaluated in `thoth-api` on the decoded value, after
//! request authorization and before any database access. None trims, normalises
//! or depends on the database's collation provider; the database CHECKs remain
//! defence in depth only.

use thoth_errors::{ThothError, ThothResult};

use crate::model::publisher_distribution_platform::DistributionPlatform;
use crate::model::work_upsert::registry::{self, WorkLevelExecutionProfile};

/// Whether `value` is blank (Amendment 3 section 7.1): empty, or made only of
/// Unicode scalar values that are `char::is_whitespace` or `char::is_control`.
pub(crate) fn is_blank(value: &str) -> bool {
    value.chars().all(|c| c.is_whitespace() || c.is_control())
}

/// Whether `value` is exactly 64 bytes, each `0`-`9` or `a`-`f`: the lower-case
/// hexadecimal SHA-256 shape (Amendment 3 section 7.2).
pub(crate) fn is_sha256_lower_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// The registered profiles an `executionProfiles` list names, deduplicated in
/// canonical platform order (Amendment 3 section 7.3). An empty list and an
/// unregistered member are refused before any database access.
pub(crate) fn registered_profiles(
    profiles: &[DistributionPlatform],
) -> ThothResult<Vec<&'static WorkLevelExecutionProfile>> {
    if profiles.is_empty() {
        return Err(ThothError::WorkUpsertExecutionProfilesRequired);
    }
    let mut registered = Vec::new();
    for platform in DistributionPlatform::ALL {
        if !profiles.contains(&platform) {
            continue;
        }
        registered.push(
            registry::execution_profile(platform)
                .ok_or(ThothError::WorkUpsertProfileNotImplemented)?,
        );
    }
    Ok(registered)
}

/// The released clamp convention: an absent limit is `default`; `<= 0` runs
/// nothing; values above `max` are clamped. No limit is ever refused.
pub(crate) fn clamp_limit(limit: Option<i32>, default: i32, max: i32) -> i64 {
    i64::from(limit.unwrap_or(default).clamp(0, max))
}

/// E1 of R52B section 14.2 for Crossref: the destination's released descriptor
/// declares the automatic-push route. Static, so a manual, pull or feed route
/// can never be eligible.
pub(crate) fn crossref_route_is_automatic_push() -> bool {
    use crate::model::publisher_distribution_platform::BackCatalogueBehaviour;
    DistributionPlatform::Crossref
        .descriptor()
        .back_catalogue_behaviour
        == BackCatalogueBehaviour::AutomaticPush
}

/// E2 and E3 of R52B section 14.2 for the publisher expression `{publisher}`:
/// an enabled `CROSSREF` assignment, whose activation is by definition the
/// publisher's current one.
pub(crate) const CROSSREF_PUBLISHER_COVERAGE_SQL: &str =
    "EXISTS (SELECT 1 FROM public.publisher_distribution_platform a \
      WHERE a.publisher_id = {publisher} AND a.platform = 'CROSSREF' AND a.enabled)";

/// The SQL-evaluable Crossref eligibility clauses of R52B section 14.2 over the
/// Work alias `w`: E4, E5 and every row-level clause of E6.
///
/// This is the one SQL definition the seed selector, the census population,
/// the drain selector and the materialization unit's step 10 all use
/// (Amendment 3 sections 9.2-9.4, M15). The only E6 clause it cannot evaluate,
/// abstract normalisation, is [`crossref_abstracts_normalise`] over
/// [`CROSSREF_EVALUATED_ABSTRACTS_SQL`].
pub(crate) const CROSSREF_SQL_ELIGIBILITY: &str = "(\
    (w.doi IS NOT NULL OR EXISTS (SELECT 1 FROM public.work_relation r \
        JOIN public.work c ON c.work_id = r.related_work_id \
        WHERE r.relator_work_id = w.work_id AND r.relation_type = 'has-child' AND c.doi IS NOT NULL)) \
    AND (w.work_status IN ('active', 'withdrawn') \
         OR (w.work_status = 'forthcoming' AND w.publication_date IS NOT NULL)) \
    AND w.publication_date IS NOT NULL \
    AND EXISTS (SELECT 1 FROM public.publication p WHERE p.work_id = w.work_id AND p.isbn IS NOT NULL) \
    AND (w.doi IS NULL OR w.landing_page IS NOT NULL) \
    AND EXISTS (SELECT 1 FROM public.title t WHERE t.work_id = w.work_id) \
    AND NOT EXISTS (SELECT 1 FROM public.work_relation r \
        JOIN public.work c ON c.work_id = r.related_work_id \
        WHERE r.relator_work_id = w.work_id AND r.relation_type = 'has-child' AND c.doi IS NOT NULL \
          AND (c.landing_page IS NULL OR c.edition IS NOT NULL \
               OR NOT EXISTS (SELECT 1 FROM public.title ct WHERE ct.work_id = c.work_id))))";

/// The stored content of every abstract the released Crossref serializer can
/// emit for the Work alias `w`, as a `text[]`.
///
/// The released query fetches a Work's abstracts ordered canonical-first with a
/// limit of two and emits all of them, and emits each DOI-bearing child's
/// canonical long and short abstracts. Which non-canonical abstract fills the
/// second place is not ordered, so the clause evaluates every one that can be
/// emitted — all non-canonical abstracts when fewer than two are canonical —
/// and so fails closed.
pub(crate) const CROSSREF_EVALUATED_ABSTRACTS_SQL: &str =
    "(SELECT coalesce(array_agg(e.content), ARRAY[]::text[]) FROM (\
    SELECT ab.content FROM public.abstract ab WHERE ab.work_id = w.work_id \
       AND (ab.canonical OR (SELECT count(*) FROM public.abstract cc \
                              WHERE cc.work_id = w.work_id AND cc.canonical) < 2) \
    UNION ALL \
    SELECT ab.content FROM public.work_relation r \
      JOIN public.work c ON c.work_id = r.related_work_id \
      JOIN public.abstract ab ON ab.work_id = c.work_id \
     WHERE r.relator_work_id = w.work_id AND r.relation_type = 'has-child' AND c.doi IS NOT NULL \
       AND ab.canonical AND ab.abstract_type IN ('long', 'short')) e)";

/// The abstract-normalisation clause of E6: every evaluated abstract normalises
/// under the serializer's own `normalise_crossref_abstract_jats`.
pub(crate) fn crossref_abstracts_normalise(contents: &[String]) -> bool {
    contents
        .iter()
        .all(|content| crate::markup::normalise_crossref_abstract_jats(content).is_ok())
}
