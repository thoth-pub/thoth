use crate::model::{
    publication::{NewPublication, PatchPublication, Publication, PublicationProperties},
    work::{Work, WorkProperties},
    Crud,
};
use crate::policy::{CreatePolicy, DeletePolicy, PolicyContext, UpdatePolicy, UserAccess};
use thoth_errors::{ThothError, ThothResult};

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
        let user = ctx.require_publisher_for(current)?;
        let work = Work::from_id(ctx.db(), &current.work_id)?;
        if work.is_published() && !user.is_superuser() {
            return Err(ThothError::ThothDeletePublicationError);
        }
        Ok(())
    }
}
