use crate::model::publisher::{NewPublisher, PatchPublisher, Publisher};
use crate::policy::{CreatePolicy, DeletePolicy, PolicyContext, UpdatePolicy};
use thoth_errors::ThothResult;

/// Write policies for `Publisher`.
///
/// Publisher records define tenancy boundaries. As such, write access is restricted to superusers.
pub struct PublisherPolicy;

impl CreatePolicy<NewPublisher> for PublisherPolicy {
    fn can_create<C: PolicyContext>(ctx: &C, _data: &NewPublisher, _params: ()) -> ThothResult<()> {
        ctx.require_superuser()?;
        Ok(())
    }
}

impl UpdatePolicy<Publisher, PatchPublisher> for PublisherPolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &Publisher,
        patch: &PatchPublisher,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ctx.require_publisher_for(patch)?;

        Ok(())
    }
}

impl DeletePolicy<Publisher> for PublisherPolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, _current: &Publisher) -> ThothResult<()> {
        ctx.require_superuser()?;
        Ok(())
    }
}
