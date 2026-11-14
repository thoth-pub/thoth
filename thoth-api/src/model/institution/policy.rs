use crate::model::institution::crud::may_write_ror_state;
use crate::model::institution::{Institution, NewInstitution, PatchInstitution};
use crate::policy::{CreatePolicy, DeletePolicy, PolicyContext, UpdatePolicy};
use thoth_errors::{ThothError, ThothResult};

/// Write policies for `Institution`.
///
/// These policies are responsible for:
/// - requiring authentication
/// - requiring publisher membership (tenant boundary) for deletes
/// - reserving `rorStatus` and `rorSuccessor` to `SUPERUSER` and
///   `ROR_SYNC_SERVICE` (ROR-SYNC-01C, thoth#1024)
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
    /// Any authenticated caller may update an institution. A caller without
    /// `SUPERUSER` or `ROR_SYNC_SERVICE` that sends a `rorStatus` or
    /// `rorSuccessor` differing from `current` (the row the resolver loaded)
    /// is denied the whole update. An equal value passes here, and the model's
    /// update drops it, so it is never written.
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &Institution,
        patch: &PatchInstitution,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_authentication()?;
        if may_write_ror_state(ctx) {
            return Ok(());
        }

        if patch
            .ror_status
            .is_some_and(|status| status != current.ror_status)
        {
            return Err(ThothError::Unauthorised);
        }

        let successor_differs = match &patch.ror_successor {
            juniper::Nullable::ImplicitNull => false,
            juniper::Nullable::ExplicitNull => current.ror_successor.is_some(),
            juniper::Nullable::Some(successor) => current.ror_successor.as_ref() != Some(successor),
        };
        if successor_differs {
            return Err(ThothError::Unauthorised);
        }

        Ok(())
    }
}

impl DeletePolicy<Institution> for InstitutionPolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &Institution) -> ThothResult<()> {
        ctx.require_publishers_for(current)?;
        Ok(())
    }
}
