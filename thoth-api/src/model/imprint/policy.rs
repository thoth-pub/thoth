use crate::model::imprint::{Imprint, NewImprint, PatchImprint};
use crate::policy::{CreatePolicy, DeletePolicy, PolicyContext, UpdatePolicy};
use thoth_errors::ThothResult;

/// Write policies for `Imprint`.
///
/// These policies are responsible for:
/// - requiring authentication
/// - requiring publisher membership (tenant boundary)
pub struct ImprintPolicy;

impl CreatePolicy<NewImprint> for ImprintPolicy {
    fn can_create<C: PolicyContext>(ctx: &C, data: &NewImprint, _params: ()) -> ThothResult<()> {
        ctx.require_publisher_for(data)?;
        Ok(())
    }
}

impl UpdatePolicy<Imprint, PatchImprint> for ImprintPolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &Imprint,
        patch: &PatchImprint,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ctx.require_publisher_for(patch)?;
        Ok(())
    }
}

impl DeletePolicy<Imprint> for ImprintPolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &Imprint) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        Ok(())
    }
}
