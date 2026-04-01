use crate::model::affiliation::{Affiliation, NewAffiliation, PatchAffiliation};
use crate::policy::{CreatePolicy, DeletePolicy, MovePolicy, PolicyContext, UpdatePolicy};
use thoth_errors::ThothResult;

/// Write policies for `Affiliation`.
///
/// These policies are responsible for:
/// - requiring authentication
/// - requiring publisher membership (tenant boundary)
pub struct AffiliationPolicy;

impl CreatePolicy<NewAffiliation> for AffiliationPolicy {
    fn can_create<C: PolicyContext>(
        ctx: &C,
        data: &NewAffiliation,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publisher_for(data)?;
        Ok(())
    }
}

impl UpdatePolicy<Affiliation, PatchAffiliation> for AffiliationPolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &Affiliation,
        patch: &PatchAffiliation,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ctx.require_publisher_for(patch)?;

        Ok(())
    }
}

impl DeletePolicy<Affiliation> for AffiliationPolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &Affiliation) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        Ok(())
    }
}

impl MovePolicy<Affiliation> for AffiliationPolicy {
    fn can_move<C: PolicyContext>(ctx: &C, current: &Affiliation) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        Ok(())
    }
}
