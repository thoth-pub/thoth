use crate::model::contributor::{Contributor, NewContributor, PatchContributor};
use crate::policy::{CreatePolicy, DeletePolicy, PolicyContext, UpdatePolicy};
use thoth_errors::ThothResult;

/// Write policies for `Contributor`.
///
/// These policies are responsible for:
/// - requiring authentication
/// - requiring publisher membership (tenant boundary)
pub struct ContributorPolicy;

impl CreatePolicy<NewContributor> for ContributorPolicy {
    fn can_create<C: PolicyContext>(
        ctx: &C,
        _data: &NewContributor,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_authentication()?;
        Ok(())
    }
}

impl UpdatePolicy<Contributor, PatchContributor> for ContributorPolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        _current: &Contributor,
        _patch: &PatchContributor,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_authentication()?;

        Ok(())
    }
}

impl DeletePolicy<Contributor> for ContributorPolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &Contributor) -> ThothResult<()> {
        ctx.require_publishers_for(current)?;
        Ok(())
    }
}
