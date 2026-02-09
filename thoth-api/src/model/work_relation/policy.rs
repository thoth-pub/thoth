use crate::model::work_relation::{NewWorkRelation, PatchWorkRelation, WorkRelation};
use crate::policy::{CreatePolicy, DeletePolicy, MovePolicy, PolicyContext, UpdatePolicy};
use thoth_errors::ThothResult;

/// Write policies for `WorkRelation`.
///
/// `WorkRelation` spans two works and therefore potentially two publisher scopes.
/// This policy enforces:
/// - authentication
/// - membership for *all* publishers involved (via `PublisherIds`)
pub struct WorkRelationPolicy;

impl CreatePolicy<NewWorkRelation> for WorkRelationPolicy {
    fn can_create<C: PolicyContext>(
        ctx: &C,
        data: &NewWorkRelation,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publishers_for(data)?;
        Ok(())
    }
}

impl UpdatePolicy<WorkRelation, PatchWorkRelation> for WorkRelationPolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &WorkRelation,
        patch: &PatchWorkRelation,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publishers_for(current)?;
        ctx.require_publishers_for(patch)?;

        Ok(())
    }
}

impl DeletePolicy<WorkRelation> for WorkRelationPolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &WorkRelation) -> ThothResult<()> {
        ctx.require_publishers_for(current)?;
        Ok(())
    }
}

impl MovePolicy<WorkRelation> for WorkRelationPolicy {
    fn can_move<C: PolicyContext>(ctx: &C, current: &WorkRelation) -> ThothResult<()> {
        ctx.require_publishers_for(current)?;
        Ok(())
    }
}
