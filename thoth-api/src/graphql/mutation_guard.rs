//! Central mutation request guard (`ADR-0006` section 4.12.6).
//!
//! The pinned Juniper executes a mutation operation's *compatible repeated
//! top-level occurrences* of one response key as several resolver invocations,
//! which duplicates a **write**. This guard detects that shape at the GraphQL
//! request boundary and, in `ENFORCE`, declines the request before any resolver
//! runs.
//!
//! # Pinned-Juniper coupling
//!
//! The baseline eligibility gate reproduces pinned juniper 0.16.2's own request
//! pipeline, in the same order, by calling juniper's own helpers. Several of
//! those helpers are exported but marked `#[doc(hidden)]` on 0.16.2:
//!
//! ```text
//! juniper::parser::parse_document_source                       #[doc(hidden)]
//! juniper::executor::get_operation                             #[doc(hidden)]
//! juniper::validation::ValidatorContext                        #[doc(hidden)] ctor
//! juniper::validation::visit_all_rules                         #[doc(hidden)]
//! juniper::validation::visit                                   #[doc(hidden)]
//! juniper::validation::validate_input_values                   #[doc(hidden)]
//! juniper::validation::rules::disable_introspection::factory   #[doc(hidden)]
//! juniper::RootNode::schema                                    #[doc(hidden)] field
//! juniper::RootNode::introspection_disabled                    #[doc(hidden)] field
//! ```
//!
//! They are **public-callable without `unsafe` and without private-field
//! access**, but they are *not* stable public API: they carry no
//! documentation-level or semantic-versioning stability promise. Any juniper
//! version change requires revalidating this whole module before merge and
//! before activation (`ADR-0006` section 4.12.14). Because the gate is
//! expressed as ordinary calls against those items, a surface that is removed
//! or re-typed fails the build rather than silently changing behaviour.
//!
//! `juniper::ast` is a private module, so `Fragment`, `Field`, `Directive` and
//! `ast::Arguments` cannot be named here. Fragment expansion therefore holds
//! `&[Selection]` selection sets, and directive evaluation is written so those
//! types are only ever inferred.

use std::collections::{HashMap, HashSet};

use juniper::{
    executor::get_operation,
    http::{GraphQLRequest, GraphQLResponse},
    parser::parse_document_source,
    validation::{
        rules, validate_input_values, visit as visit_rule, visit_all_rules, MultiVisitorNil,
        RuleError, ValidatorContext,
    },
    GraphQLError, GraphQLType, InputValue, OperationType, ScalarValue, Selection, Variables,
};

/// Whether `@skip`/`@include` definitely exclude this selection.
///
/// Juniper's own `is_excluded` is `pub(super)`, so this is reimplemented on
/// public API and must stay behaviourally identical to
/// `juniper::types::base::is_excluded` for literal values, variables, operation
/// defaults, request overrides, multiple directives, and directives on fields,
/// fragment spreads and inline fragments.
///
/// **This is a macro, not a function, and deliberately so.** `juniper::ast` is
/// private, so `Directive` and `Arguments` cannot be named in a signature. A
/// macro keeps a single definition while letting every type be *inferred* at
/// the three expansion sites (field, fragment spread, inline fragment).
///
/// **One deliberate divergence from juniper, in the safe direction.** Juniper's
/// version `unwrap()`s the resolved condition and would panic on an
/// unresolvable one; on the pinned stack that case is unreachable because
/// validation has already run. Here, a condition that genuinely cannot be
/// resolved *after applying operation defaults* is treated as **not
/// excluding** — the occurrence counts as executable, so the guard rejects
/// conservatively rather than admitting a possible duplicate write
/// (`ADR-0006` section 4.12.6.7). An omitted-but-defaulted variable is
/// **resolved** by `effective_variables` before this runs, so it is never
/// classified as undecidable.
macro_rules! is_excluded {
    ($directives:expr, $variables:expr) => {{
        let mut excluded = false;
        if let Some(directives) = $directives.as_ref() {
            for spanning in directives {
                let directive = &spanning.item;
                let condition = directive
                    .arguments
                    .iter()
                    .flat_map(|arguments| arguments.item.get("if"))
                    .filter_map(|value| value.item.clone().into_const($variables))
                    .find_map(|value| bool_of(&value));
                // Unresolved after defaults: conservatively executable.
                if let Some(condition) = condition {
                    let name = directive.name.item;
                    if (name == "skip" && condition) || (name == "include" && !condition) {
                        excluded = true;
                        break;
                    }
                }
            }
        }
        excluded
    }};
}

/// Read a `Boolean` out of a resolved const `InputValue`.
fn bool_of<S: ScalarValue>(value: &InputValue<S>) -> Option<bool> {
    match value {
        InputValue::Scalar(scalar) => scalar.as_bool(),
        _ => None,
    }
}

/// Guard mode, and the single value store availability is derived from.
///
/// This is deliberately the *only* switch. `ADR-0006` invariant 30 requires
/// `OFF + store available` and `OBSERVE + store available` to be structurally
/// unrepresentable, so there is no second "enable loaders" flag anywhere in the
/// codebase — [`MutationGuardMode::store_available`] is the sole answer to the
/// question, and it is derived from this enum.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MutationGuardMode {
    /// Evaluate nothing, reject nothing, emit nothing. The default, and the
    /// merged production state.
    #[default]
    Off,
    /// Evaluate exactly as `Enforce` would, but never reject. Emits one
    /// observation event per would-be rejection.
    Observe,
    /// Evaluate and reject a baseline-valid mutation whose executable top-level
    /// response key occurs more than once.
    Enforce,
}

impl MutationGuardMode {
    /// Whether the request-scoped loader store may be used.
    ///
    /// `ADR-0006` invariant 30:
    ///
    /// ```text
    /// loader store available  =>  guard mode == ENFORCE
    /// ```
    ///
    /// The store's mutation isolation guarantee depends on enforcement, so the
    /// store is unavailable in every other mode.
    pub fn store_available(self) -> bool {
        matches!(self, Self::Enforce)
    }

    /// Whether the guard evaluates at all. `OFF` short-circuits ahead of the
    /// eligibility gate, so it imposes no parse or validation cost on any
    /// request of any kind.
    fn evaluates(self) -> bool {
        !matches!(self, Self::Off)
    }

    /// The value recorded on a guard event, so `OBSERVE` and `ENFORCE` evidence
    /// can never be conflated.
    fn as_event_str(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Observe => "OBSERVE",
            Self::Enforce => "ENFORCE",
        }
    }
}

impl std::str::FromStr for MutationGuardMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_uppercase().as_str() {
            "OFF" => Ok(Self::Off),
            "OBSERVE" => Ok(Self::Observe),
            "ENFORCE" => Ok(Self::Enforce),
            other => Err(format!(
                "Unknown mutation guard mode `{other}`; expected OFF, OBSERVE or ENFORCE"
            )),
        }
    }
}

/// The guard's decision for one GraphQL request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GuardOutcome {
    /// The guard did not evaluate, the request was not a mutation, the request
    /// was baseline-invalid, or no executable duplicate was found. Ordinary
    /// juniper execution proceeds untouched.
    Proceed,
    /// `ENFORCE` only: reject before any resolver runs, carrying the colliding
    /// response keys in deterministic order.
    Reject { collisions: Vec<String> },
}

/// A structured guard observation, recorded once per would-be or actual
/// rejection.
///
/// The permitted fields are exhaustive (`ADR-0006` section 8.2): the mode, the
/// colliding response keys and — only when the request supplied one — the
/// operation name. The GraphQL document, the variables and every mutation
/// argument value are deliberately absent, because mutation arguments carry
/// publisher and user data. That absence is a privacy requirement, and it is
/// enforced by this struct having no field able to hold them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GuardEvent {
    pub(crate) mode: &'static str,
    pub(crate) operation_name: Option<String>,
    pub(crate) collisions: Vec<String>,
}

impl GuardEvent {
    fn message(&self) -> String {
        match &self.operation_name {
            Some(name) => format!(
                "GraphQL mutation guard [{}]: duplicate top-level response key(s) [{}] \
                 in operation `{name}`",
                self.mode,
                self.collisions.join(", "),
            ),
            None => format!(
                "GraphQL mutation guard [{}]: duplicate top-level response key(s) [{}]",
                self.mode,
                self.collisions.join(", "),
            ),
        }
    }
}

/// Result of evaluating one request, including any event that must be emitted.
pub(crate) struct GuardDecision {
    pub(crate) outcome: GuardOutcome,
    pub(crate) event: Option<GuardEvent>,
}

impl GuardDecision {
    fn proceed() -> Self {
        Self {
            outcome: GuardOutcome::Proceed,
            event: None,
        }
    }
}

/// The client-visible rejection message.
///
/// It names only the GraphQL-level shape it declines. It exposes no loader,
/// store or scope internals, and it does not claim the document is invalid
/// GraphQL — the document may be perfectly valid; this server declines to
/// execute it because the pinned executor would perform the write twice.
fn rejection_message(collisions: &[String]) -> String {
    format!(
        "This mutation is not accepted: the top-level response key(s) [{}] are selected more \
         than once in a single mutation operation, which would execute the same mutation more \
         than once. Give each top-level mutation field a distinct alias.",
        collisions.join(", ")
    )
}

/// Build the rejection response.
///
/// This is a `GraphQLError::ValidationError` carrying `RuleError`s with the
/// colliding source positions, so `GraphQLResponse::is_ok()` is `false` and the
/// **existing** handler branch returns HTTP 400 with the ordinary
/// `{"errors":[{"message","locations"}]}` body and no `data` key. No new
/// handler branch and no one-off HTTP protocol.
pub(crate) fn rejection_response<S>(
    collisions: &[String],
    positions: Vec<juniper::parser::SourcePosition>,
) -> GraphQLResponse<S>
where
    S: ScalarValue,
{
    let message = rejection_message(collisions);
    GraphQLResponse::from_result(Err(GraphQLError::ValidationError(vec![RuleError::new(
        &message, &positions,
    )])))
}

/// Emit the guard's structured warning event.
///
/// Exactly one record per would-be (in `OBSERVE`) or actual (in `ENFORCE`)
/// rejection.
pub(crate) fn emit_event(event: &GuardEvent) {
    log::warn!("{}", event.message());
}

/// Evaluate one GraphQL request.
///
/// Placement is the request boundary, before `GraphQLRequest::execute`. This
/// function does **not** replace `execute`, makes **no** authorization
/// decision, and never returns, rewrites or suppresses a juniper validation
/// error of its own.
pub(crate) fn evaluate<QueryT, MutationT, SubscriptionT, S>(
    mode: MutationGuardMode,
    request: &GraphQLRequest<S>,
    root_node: &juniper::RootNode<'_, QueryT, MutationT, SubscriptionT, S>,
) -> GuardDecision
where
    S: ScalarValue,
    QueryT: GraphQLType<S>,
    MutationT: GraphQLType<S, Context = QueryT::Context>,
    SubscriptionT: GraphQLType<S, Context = QueryT::Context>,
{
    // Mode is checked FIRST. In `OFF` we return before any parsing, so `OFF`
    // imposes no duplicate-work cost on production traffic of any kind
    // (`ADR-0006` section 4.12.6.6). Nothing below this line runs in `OFF`.
    if !mode.evaluates() {
        return GuardDecision::proceed();
    }

    // ---- Baseline eligibility gate (`ADR-0006` section 4.12.6.5.3) ----------
    //
    // Reproduces pinned juniper's own pipeline using juniper's own helpers. Any
    // stage reporting an error makes the request baseline-invalid: no duplicate
    // analysis, no event, no guard error. Ordinary `GraphQLRequest::execute()`
    // then produces the canonical client-visible error.

    // 1. parse
    let Ok(document) = parse_document_source(&request.query, &root_node.schema) else {
        return GuardDecision::proceed();
    };

    // 2. operation selection
    let Ok(operation) = get_operation(&document, request.operation_name.as_deref()) else {
        return GuardDecision::proceed();
    };

    // 3. non-mutation fast path.
    //
    // Selection deliberately precedes validation (`ADR-0006` section
    // 4.12.6.5.4): juniper validates then selects, the gate selects then
    // validates, so a non-mutation exits before the expensive stages. This is
    // safe because exiting here makes NO decision — no rejection, no event —
    // which is indistinguishable from "no collision", and because
    // `operation_type` is a parser-level token obtained by exactly the call
    // juniper itself makes. Queries and subscriptions leave here.
    if operation.item.operation_type != OperationType::Mutation {
        return GuardDecision::proceed();
    }

    // 4. ordinary document/schema validation
    {
        let mut ctx = ValidatorContext::new(&root_node.schema, &document);
        visit_all_rules(&mut ctx, &document);

        // 5. disabled-introspection rule, only when configured, mirroring
        //    juniper's own conditional.
        if root_node.introspection_disabled {
            visit_rule(
                &mut MultiVisitorNil.with(rules::disable_introspection::factory()),
                &mut ctx,
                &document,
            );
        }

        if !ctx.into_errors().is_empty() {
            return GuardDecision::proceed();
        }
    }

    // 6. input-variable validation
    let request_variables = request.variables();
    if !validate_input_values(&request_variables, operation, &root_node.schema).is_empty() {
        return GuardDecision::proceed();
    }

    // ---- The request is baseline-valid and is a mutation. --------------------

    // Effective variables, constructed exactly as the pinned executor does
    // (`ADR-0006` section 4.12.6.5.1): start from the request variables, then
    // insert each operation-level default only where the request supplied no
    // value. This mirrors juniper's `all_vars.entry(name).or_insert(default)`
    // in `execute_validated_query{,_async}`. Evaluating directives against raw
    // request variables is prohibited and demonstrably over-rejects.
    let effective_variables = effective_variables(&request_variables, operation);

    // Named fragment selection sets, held as `&[Selection]` because
    // `juniper::ast::Fragment` is not publicly nameable.
    let mut fragments: HashMap<&str, &[Selection<'_, S>]> = HashMap::new();
    for definition in document.iter() {
        if let juniper::Definition::Fragment(fragment) = definition {
            fragments.insert(fragment.item.name.item, &fragment.item.selection_set);
        }
    }

    let mut occurrences: Vec<(String, juniper::parser::SourcePosition)> = Vec::new();
    let mut visiting: Vec<&str> = Vec::new();
    collect_top_level_occurrences(
        &operation.item.selection_set,
        &fragments,
        &effective_variables,
        &mut visiting,
        &mut occurrences,
    );

    // A response key colliding means it has more than one *executable*
    // top-level occurrence.
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for (key, _) in &occurrences {
        *counts.entry(key.as_str()).or_insert(0) += 1;
    }

    // Deterministic collision ordering: sorted by response key.
    let mut collisions: Vec<String> = counts
        .iter()
        .filter(|(_, count)| **count > 1)
        .map(|(key, _)| (*key).to_string())
        .collect();
    collisions.sort();

    if collisions.is_empty() {
        return GuardDecision::proceed();
    }

    let event = GuardEvent {
        mode: mode.as_event_str(),
        operation_name: request.operation_name.clone(),
        collisions: collisions.clone(),
    };

    match mode {
        // Unreachable — `evaluates()` returned above — but written exhaustively
        // rather than with a catch-all so a future mode must be considered.
        MutationGuardMode::Off => GuardDecision::proceed(),
        // Detect, record, reject nothing. The request then continues through
        // existing juniper execution completely unchanged.
        MutationGuardMode::Observe => GuardDecision {
            outcome: GuardOutcome::Proceed,
            event: Some(event),
        },
        MutationGuardMode::Enforce => GuardDecision {
            outcome: GuardOutcome::Reject { collisions },
            event: Some(event),
        },
    }
}

/// Collect the source positions of every occurrence of the colliding keys, so
/// the rejection's `RuleError` carries real locations.
pub(crate) fn collision_positions<QueryT, MutationT, SubscriptionT, S>(
    request: &GraphQLRequest<S>,
    root_node: &juniper::RootNode<'_, QueryT, MutationT, SubscriptionT, S>,
    collisions: &[String],
) -> Vec<juniper::parser::SourcePosition>
where
    S: ScalarValue,
    QueryT: GraphQLType<S>,
    MutationT: GraphQLType<S, Context = QueryT::Context>,
    SubscriptionT: GraphQLType<S, Context = QueryT::Context>,
{
    let Ok(document) = parse_document_source(&request.query, &root_node.schema) else {
        return Vec::new();
    };
    let Ok(operation) = get_operation(&document, request.operation_name.as_deref()) else {
        return Vec::new();
    };

    let request_variables = request.variables();
    let effective = effective_variables(&request_variables, operation);

    let mut fragments: HashMap<&str, &[Selection<'_, S>]> = HashMap::new();
    for definition in document.iter() {
        if let juniper::Definition::Fragment(fragment) = definition {
            fragments.insert(fragment.item.name.item, &fragment.item.selection_set);
        }
    }

    let mut occurrences = Vec::new();
    let mut visiting = Vec::new();
    collect_top_level_occurrences(
        &operation.item.selection_set,
        &fragments,
        &effective,
        &mut visiting,
        &mut occurrences,
    );

    let wanted: HashSet<&str> = collisions.iter().map(String::as_str).collect();
    let mut positions: Vec<_> = occurrences
        .into_iter()
        .filter(|(key, _)| wanted.contains(key.as_str()))
        .map(|(_, position)| position)
        .collect();
    positions.sort();
    positions.dedup();
    positions
}

/// Build the effective variable map: operation defaults overridden by request
/// variables.
///
/// `VariableDefinitions` and `VariableDefinition` are not publicly nameable, so
/// they are reached by field access on the `Operation` value without naming
/// their types.
fn effective_variables<S>(
    request_variables: &Variables<S>,
    operation: &juniper::Spanning<juniper::Operation<'_, S>>,
) -> Variables<S>
where
    S: ScalarValue,
{
    let mut effective = request_variables.clone();
    if let Some(definitions) = operation.item.variable_definitions.as_ref() {
        for (name, definition) in definitions.item.items.iter() {
            if let Some(default) = definition.default_value.as_ref() {
                // `or_insert`: the request value wins where one was supplied.
                effective
                    .entry(name.item.into())
                    .or_insert_with(|| default.item.clone());
            }
        }
    }
    effective
}

/// Expand a top-level selection set into executable response-key occurrences.
///
/// Direct fields, named fragment spreads and inline fragments are all expanded.
/// The response key is the alias when present, otherwise the field name.
///
/// **Cycle safety.** `visiting` is a stack of the fragment names currently
/// being expanded on *this* path. A spread is skipped only when it would
/// re-enter a fragment already on that path. It is deliberately not a
/// global "seen" set: two legitimate sibling spreads of one fragment are two
/// distinct occurrences and must both be counted, which a global set would
/// wrongly suppress.
fn collect_top_level_occurrences<'a, S>(
    selection_set: &'a [Selection<'a, S>],
    fragments: &HashMap<&'a str, &'a [Selection<'a, S>]>,
    variables: &Variables<S>,
    visiting: &mut Vec<&'a str>,
    occurrences: &mut Vec<(String, juniper::parser::SourcePosition)>,
) where
    S: ScalarValue,
{
    for selection in selection_set {
        match selection {
            Selection::Field(field) => {
                if is_excluded!(&field.item.directives, variables) {
                    continue;
                }
                let response_key = field
                    .item
                    .alias
                    .as_ref()
                    .map(|alias| alias.item)
                    .unwrap_or(field.item.name.item);
                occurrences.push((response_key.to_string(), field.item.name.span.start));
            }
            Selection::FragmentSpread(spread) => {
                // The directive on the spread itself gates the whole expansion.
                if is_excluded!(&spread.item.directives, variables) {
                    continue;
                }
                let name = spread.item.name.item;
                if visiting.contains(&name) {
                    // Cycle: this fragment is already being expanded on this
                    // path. A cyclic document is rejected by juniper's own
                    // `no_fragment_cycles` rule at the eligibility gate, so
                    // this is belt-and-braces against unbounded recursion.
                    continue;
                }
                let Some(inner) = fragments.get(name) else {
                    // Unknown fragment: juniper's `known_fragment_names` rule
                    // already rejected this at the gate.
                    continue;
                };
                visiting.push(name);
                collect_top_level_occurrences(inner, fragments, variables, visiting, occurrences);
                visiting.pop();
            }
            Selection::InlineFragment(inline) => {
                if is_excluded!(&inline.item.directives, variables) {
                    continue;
                }
                collect_top_level_occurrences(
                    &inline.item.selection_set,
                    fragments,
                    variables,
                    visiting,
                    occurrences,
                );
            }
        }
    }
}
