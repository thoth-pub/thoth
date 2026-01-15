use crate::model::publication::{
    NewPublication, PatchPublication, Publication, PublicationProperties,
};
use crate::policy::{CreatePolicy, DeletePolicy, PolicyContext, UpdatePolicy};
use thoth_errors::ThothResult;

/// Write policies for `Publication`.
///
/// These policies are responsible for:
/// - requiring authentication
/// - requiring publisher membership (tenant boundary)
pub struct PublicationPolicy;

impl CreatePolicy<NewPublication> for PublicationPolicy {
    fn can_create<C: PolicyContext>(
        ctx: &C,
        data: &NewPublication,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publisher_for(data)?;
        data.validate(ctx.db())
    }
}

impl UpdatePolicy<Publication, PatchPublication> for PublicationPolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &Publication,
        patch: &PatchPublication,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ctx.require_publisher_for(patch)?;

        patch.validate(ctx.db())
    }
}

impl DeletePolicy<Publication> for PublicationPolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &Publication) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        Ok(())
    }
}
