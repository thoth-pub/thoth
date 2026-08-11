pub mod model;
pub mod types;

// Temporary compile bridge while Context is migrated off the superseded A2
// store in this same bounded implementation. This module is removed before the
// implementation is presented for review.
mod batching;
mod dataloader;
mod mutation;
mod mutation_guard;
#[cfg(test)]
mod mutation_guard_tests;
mod query;

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

/// Run the central mutation request guard for one GraphQL request.
///
/// The guard is an independent request-acceptance concern. DataLoader
/// availability is request-local and does not depend on `OFF`, `OBSERVE`, or
/// `ENFORCE` (`ADR-0007` invariant 13).
///
/// Call this at the GraphQL request boundary, **before** ordinary
/// `GraphQLRequest::execute`. It does not replace `execute`, and it makes no
/// authorization decision.
pub fn run_mutation_guard(
    mode: MutationGuardMode,
    request: &GraphQLRequest,
    schema: &Schema,
) -> Option<GraphQLResponse<juniper::DefaultScalarValue>> {
    let decision = mutation_guard::evaluate(mode, request, schema);

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
