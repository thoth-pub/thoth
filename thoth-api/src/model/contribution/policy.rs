use crate::model::contribution::{Contribution, NewContribution, PatchContribution};
use crate::policy::{CreatePolicy, DeletePolicy, MovePolicy, PolicyContext, UpdatePolicy};
use thoth_errors::ThothResult;

/// Write policies for `Contribution`.
///
/// These policies are responsible for:
/// - requiring authentication
/// - requiring publisher membership (tenant boundary)
///
/// `Contribution` is scoped to a parent `Work`, and publisher membership is derived from the
/// `PublisherId` implementation (via `work_id`).
pub struct ContributionPolicy;

impl CreatePolicy<NewContribution> for ContributionPolicy {
    fn can_create<C: PolicyContext>(
        ctx: &C,
        data: &NewContribution,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publisher_for(data)?;
        Ok(())
    }
}

impl UpdatePolicy<Contribution, PatchContribution> for ContributionPolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &Contribution,
        patch: &PatchContribution,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ctx.require_publisher_for(patch)?;

        Ok(())
    }
}

impl DeletePolicy<Contribution> for ContributionPolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &Contribution) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        Ok(())
    }
}

impl MovePolicy<Contribution> for ContributionPolicy {
    fn can_move<C: PolicyContext>(ctx: &C, current: &Contribution) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        Ok(())
    }
}
