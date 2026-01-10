use crate::model::contact::{Contact, NewContact, PatchContact};
use crate::policy::{CreatePolicy, DeletePolicy, PolicyContext, UpdatePolicy};
use thoth_errors::ThothResult;

/// Write policies for `Contact`.
///
/// These policies are responsible for:
/// - requiring authentication
/// - requiring publisher membership (tenant boundary)
pub struct ContactPolicy;

impl CreatePolicy<NewContact> for ContactPolicy {
    fn can_create<C: PolicyContext>(ctx: &C, data: &NewContact, _params: ()) -> ThothResult<()> {
        ctx.require_publisher_for(data)?;
        Ok(())
    }
}

impl UpdatePolicy<Contact, PatchContact> for ContactPolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &Contact,
        patch: &PatchContact,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ctx.require_publisher_for(patch)?;
        Ok(())
    }
}

impl DeletePolicy<Contact> for ContactPolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &Contact) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        Ok(())
    }
}
