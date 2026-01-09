use crate::model::series::{NewSeries, PatchSeries, Series};
use crate::policy::{CreatePolicy, DeletePolicy, PolicyContext, UpdatePolicy};
use thoth_errors::ThothResult;

/// Write policies for `Series`.
///
/// For now this policy enforces the tenant boundary only:
/// - authentication
/// - publisher membership derived from the entity / input via `PublisherId`
pub struct SeriesPolicy;

impl CreatePolicy<NewSeries> for SeriesPolicy {
    fn can_create<C: PolicyContext>(ctx: &C, data: &NewSeries, _params: ()) -> ThothResult<()> {
        ctx.require_publisher_for(data)?;
        Ok(())
    }
}

impl UpdatePolicy<Series, PatchSeries> for SeriesPolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &Series,
        patch: &PatchSeries,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ctx.require_publisher_for(patch)?;

        Ok(())
    }
}

impl DeletePolicy<Series> for SeriesPolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &Series) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        Ok(())
    }
}
