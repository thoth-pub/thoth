use crate::model::funding::{Funding, NewFunding, PatchFunding};
use crate::policy::{CreatePolicy, DeletePolicy, PolicyContext, UpdatePolicy};
use thoth_errors::ThothResult;

/// Write policies for `Funding`.
///
/// These policies are responsible for:
/// - requiring authentication
/// - requiring publisher membership (tenant boundary)
pub struct FundingPolicy;

impl CreatePolicy<NewFunding> for FundingPolicy {
    fn can_create<C: PolicyContext>(ctx: &C, data: &NewFunding, _params: ()) -> ThothResult<()> {
        ctx.require_publisher_for(data)?;
        Ok(())
    }
}

impl UpdatePolicy<Funding, PatchFunding> for FundingPolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &Funding,
        patch: &PatchFunding,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ctx.require_publisher_for(patch)?;

        Ok(())
    }
}

impl DeletePolicy<Funding> for FundingPolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &Funding) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        Ok(())
    }
}
