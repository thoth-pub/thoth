use crate::model::institution::{Institution, NewInstitution, PatchInstitution};
use crate::policy::{CreatePolicy, DeletePolicy, PolicyContext, UpdatePolicy};
use thoth_errors::ThothResult;

/// Write policies for `Institution`.
///
/// These policies are responsible for:
/// - requiring authentication
/// - requiring publisher membership (tenant boundary)
pub struct InstitutionPolicy;

impl CreatePolicy<NewInstitution> for InstitutionPolicy {
    fn can_create<C: PolicyContext>(
        ctx: &C,
        _data: &NewInstitution,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_authentication()?;
        Ok(())
    }
}

impl UpdatePolicy<Institution, PatchInstitution> for InstitutionPolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        _current: &Institution,
        _patch: &PatchInstitution,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_authentication()?;
        Ok(())
    }
}

impl DeletePolicy<Institution> for InstitutionPolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &Institution) -> ThothResult<()> {
        ctx.require_publishers_for(current)?;
        Ok(())
    }
}
