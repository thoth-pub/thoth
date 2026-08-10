//! Look-ahead-driven synchronous set-based prefetch (`ADR-0006` sections 4.15,
//! 4.19).
//!
//! A prefetch site sits on a resolver that has just resolved a **list of
//! already-authorized parent items**. It inspects the request's selection set
//! through juniper's look-ahead, discovers whether a loader-backed field is
//! selected — either as a **direct** child of those items or as a
//! **descendant** beneath intermediate object fields — projects the terminal
//! loader keys from the already-resolved items, de-duplicates them, and issues
//! one set-based statement per distinct normalized terminal shape.
//!
//! # The four things a site settles (`ADR-0006` section 4.19.1)
//!
//! ```text
//! selection path                 ordered schema field names from the list item's
//!                                selection set down to the terminal field
//! terminal loader identity       which loader backs the terminal field
//! terminal load-shape ctor       the loader-owned constructor of section 4.4.2
//! key projector                  resolved list item -> terminal loader key
//! ```
//!
//! A direct-child site is the degenerate case: a selection path of length one
//! and an identity key projector.
//!
//! # Alias safety
//!
//! Matching is on [`LookAheadSelection::field_original_name`] at **every**
//! segment, never on `field_name()` (which returns the alias when present).
//! `LookAheadChildren::select(name)` and `has_child(name)` are never used at
//! any segment: both match `field_name()`, and both return only the *first*
//! match, so both would miss an aliased terminal field such as `a: imprints`,
//! and would miss a second
//! matching branch (`ADR-0006` sections 4.15.1, 4.19.3).
//!
//! Traversal is recursive over [`LookAheadSelection::children`] and collects
//! **every** matching terminal selection across **every** matching intermediate
//! branch. It stops at no level's first match.
//!
//! Unlike the mutation guard's eligibility-gate surfaces, `look_ahead()`,
//! `children()` and `field_original_name()` are ordinary documented public
//! juniper API — not `#[doc(hidden)]` — so this traversal carries the weaker of
//! the two coupling risks.
//!
//! # Where results go
//!
//! Into the **ordinary** terminal identity
//! `(scope, loader, shape, terminal key)`. There is no separate namespace for
//! indirectly prefetched entries: within one scope an ancestor-prefetched entry
//! and a parent-list-prefetched entry are the same entry and satisfy each
//! other's lookups (`ADR-0006` section 4.19.2).
//!
//! # Fail closed
//!
//! A site that cannot derive its scope performs **no** prefetch, does not fail
//! the parent list field, and leaves every affected terminal lookup to read
//! `NotLoaded` and take the ordinary direct-query fallback
//! (`ADR-0006` section 4.12.9).

// No production field installs a prefetch site in this task, so in a non-test
// build these are legitimately unreferenced. See `super::batching` for the same
// note; the `#[cfg(test)]` proof fixture installs two direct sites and one
// descendant site and exercises all of this.
#![cfg_attr(not(test), allow(dead_code))]

use juniper::{Executor, LookAheadSelection, ScalarValue};

use super::{
    batching::{BatchLoader, DispatchResult},
    scope::top_level_response_key,
};

/// Describes how to reach a loader-backed terminal field from a resolved list
/// item, and how to build the terminal loader's key and shape.
///
/// `path` is the ordered list of **schema** field names (never aliases) from
/// the list item's selection set down to the terminal loader-backed field. A
/// single-element path is a direct child.
pub(crate) struct PrefetchTarget<'a, Item, L: BatchLoader, S: ScalarValue> {
    /// Ordered schema field names, terminal segment last.
    pub(crate) path: &'a [&'a str],
    /// Builds the terminal loader's normalized shape from the **terminal**
    /// selection — never from an ancestor selection. An intermediate field's
    /// own arguments are not part of the terminal loader's shape.
    ///
    /// This must call the loader's single shape constructor, so the prefetch
    /// site and the child lookup cannot drift.
    pub(crate) terminal_shape: fn(&LookAheadSelection<'_, S>) -> L::Shape,
    /// Projects a terminal loader key from an already-resolved list item.
    ///
    /// **Security rule** (`ADR-0006` section 4.19.4): this may read only data
    /// already present on the already-resolved, already-authorized item — a
    /// foreign key on the row, not a value re-derived from user input — and
    /// must never bypass an intermediate authorization decision. It returns
    /// [`None`] where no key can be projected for that item.
    pub(crate) project_key: fn(&Item) -> Option<L::Key>,
}

/// Run a prefetch for `items` at this executor's position.
///
/// Returns the dispatch outcome per distinct normalized terminal shape, in
/// discovery order, for evidence and test purposes. An empty result means
/// nothing was prefetched — because the terminal field was not selected,
/// because no key could be projected, or because the scope could not be
/// derived.
///
/// This never fails the calling parent list resolver: a dispatch failure is
/// recorded in the store as `LoadFailed` and surfaces at the covered **child**
/// field, not here (`ADR-0006` section 4.9).
pub(crate) fn prefetch<Item, L, S, C>(
    executor: &Executor<'_, '_, C, S>,
    store: &super::batching::GraphqlBatchStore,
    db: &crate::db::PgPool,
    items: &[Item],
    target: &PrefetchTarget<'_, Item, L, S>,
) -> Vec<(L::Shape, DispatchResult)>
where
    L: BatchLoader,
    S: ScalarValue,
{
    if !store.is_available() || items.is_empty() || target.path.is_empty() {
        return Vec::new();
    }

    // Fail closed: no scope, no prefetch. Never substitute a shared or
    // request-global namespace.
    let Some(scope) = top_level_response_key(executor) else {
        return Vec::new();
    };

    // Collect EVERY matching terminal selection, across EVERY matching
    // intermediate branch.
    let look_ahead = executor.look_ahead();
    let mut terminals = Vec::new();
    collect_terminal_selections(&look_ahead, target.path, &mut terminals);
    if terminals.is_empty() {
        return Vec::new();
    }

    // Project and de-duplicate the terminal keys from the already-resolved
    // items. De-duplication happens before dispatch, so `n` references to one
    // key yield one key in the statement.
    let mut keys: Vec<L::Key> = Vec::with_capacity(items.len());
    for item in items {
        if let Some(key) = (target.project_key)(item) {
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
    }
    if keys.is_empty() {
        return Vec::new();
    }

    // One dispatch per distinct normalized terminal shape. Identical normalized
    // shapes de-duplicate to one dispatch; different shapes remain separate
    // dispatches (`ADR-0006` section 4.4.4).
    let mut dispatched: Vec<(L::Shape, DispatchResult)> = Vec::new();
    for terminal in &terminals {
        let shape = (target.terminal_shape)(terminal);
        if dispatched.iter().any(|(seen, _)| seen == &shape) {
            continue;
        }
        // A dispatch error is retained in the store; the parent list field
        // still resolves successfully.
        let outcome = store
            .dispatch::<L>(db, &scope, &shape, &keys)
            .unwrap_or(DispatchResult::Failed);
        dispatched.push((shape, outcome));
    }
    dispatched
}

/// Recursively collect every look-ahead selection reachable by following
/// `path`'s **schema** field names, matching on `field_original_name()` at
/// every segment.
///
/// Traversal never stops at the first match at any level, so
///
/// ```graphql
/// imprints {
///   first:  publisher { one: imprints { imprintName } }
///   second: publisher { two: imprints { imprintName } }
/// }
/// ```
///
/// yields **both** terminal selections, not one.
fn collect_terminal_selections<'a, S: ScalarValue>(
    selection: &LookAheadSelection<'a, S>,
    path: &[&str],
    out: &mut Vec<LookAheadSelection<'a, S>>,
) {
    let Some((segment, rest)) = path.split_first() else {
        return;
    };
    for child in selection.children().iter() {
        // `field_original_name()` is the SCHEMA field name; `field_name()`
        // would be the alias.
        if child.field_original_name() != *segment {
            continue;
        }
        if rest.is_empty() {
            out.push(*child);
        } else {
            collect_terminal_selections(child, rest, out);
        }
    }
}
