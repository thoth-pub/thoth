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

        if data.s3_bucket.is_some()
            || data.cdn_domain.is_some()
            || data.cloudfront_dist_id.is_some()
        {
            ctx.require_superuser()?;
        }

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
        ctx.require_publisher_admin_for(current)?;
        ctx.require_publisher_admin_for(patch)?;

        if patch.s3_bucket != current.s3_bucket
            || patch.cdn_domain != current.cdn_domain
            || patch.cloudfront_dist_id != current.cloudfront_dist_id
        {
            ctx.require_superuser()?;
        }

        Ok(())
    }
}

impl DeletePolicy<Imprint> for ImprintPolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &Imprint) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        Ok(())
    }
}
