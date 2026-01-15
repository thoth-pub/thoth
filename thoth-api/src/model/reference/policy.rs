use crate::model::reference::{NewReference, PatchReference, Reference};
use crate::policy::{CreatePolicy, DeletePolicy, MovePolicy, PolicyContext, UpdatePolicy};
use thoth_errors::ThothResult;

/// Write policies for `Reference`.
///
/// For now this policy enforces the tenant boundary only:
/// - authentication
/// - publisher membership derived from the entity / input via `PublisherId`
pub struct ReferencePolicy;

impl CreatePolicy<NewReference> for ReferencePolicy {
    fn can_create<C: PolicyContext>(ctx: &C, data: &NewReference, _params: ()) -> ThothResult<()> {
        ctx.require_publisher_for(data)?;
        Ok(())
    }
}

impl UpdatePolicy<Reference, PatchReference> for ReferencePolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &Reference,
        patch: &PatchReference,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ctx.require_publisher_for(patch)?;

        Ok(())
    }
}

impl DeletePolicy<Reference> for ReferencePolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &Reference) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        Ok(())
    }
}

impl MovePolicy<Reference> for ReferencePolicy {
    fn can_move<C: PolicyContext>(ctx: &C, current: &Reference) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        Ok(())
    }
}
