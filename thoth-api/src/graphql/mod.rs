pub mod model;
pub mod types;

mod batching;
#[cfg(test)]
mod batching_fixture;
#[cfg(test)]
mod batching_tests;
mod mutation;
mod mutation_guard;
mod prefetch;
mod query;
mod scope;

pub use juniper::http::GraphQLRequest;

pub use model::Context;
pub use mutation::MutationRoot;
pub use mutation_guard::MutationGuardMode;
pub use query::QueryRoot;

use juniper::{http::GraphQLResponse, EmptySubscription, RootNode};

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<Context>>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {}, EmptySubscription::new())
}

/// Run the central mutation request guard for one GraphQL request
/// (`ADR-0006` section 4.12.6).
///
/// Call this at the GraphQL request boundary, **before** ordinary
/// `GraphQLRequest::execute`. It does not replace `execute`, and it makes no
/// authorization decision.
///
/// Returns:
///
/// - [`None`] — proceed with ordinary Juniper execution, unchanged. This is the
///   result in `OFF` always, for every non-mutation, for every baseline-invalid
///   request in any mode, for every mutation with no executable duplicate, and
///   for **every** request in `OBSERVE` (which records but never rejects);
/// - [`Some`] — `ENFORCE` only: a validation-style [`GraphQLResponse`] whose
///   `is_ok()` is `false`, to be returned instead of executing. No resolver
///   runs and no write occurs.
///
/// Any observation or rejection event is emitted here, exactly once.
pub fn run_mutation_guard(
    mode: MutationGuardMode,
    request: &GraphQLRequest,
    schema: &Schema,
) -> Option<GraphQLResponse<juniper::DefaultScalarValue>> {
    let decision = mutation_guard::evaluate(mode, request, schema);

    // Exactly one structured warning record per would-be or actual rejection.
    if let Some(event) = &decision.event {
        mutation_guard::emit_event(event);
    }

    match decision.outcome {
        mutation_guard::GuardOutcome::Proceed => None,
        mutation_guard::GuardOutcome::Reject { collisions } => {
            let positions = mutation_guard::collision_positions(request, schema, &collisions);
            Some(mutation_guard::rejection_response(&collisions, positions))
        }
    }
}

#[cfg(test)]
mod tests;
