use crate::model::work::{NewWork, PatchWork, Work, WorkProperties, WorkType};
use crate::policy::{CreatePolicy, DeletePolicy, PolicyContext, UpdatePolicy, UserAccess};
use thoth_errors::{ThothError, ThothResult};

/// Write policies for `Work`.
///
/// This policy layer enforces:
/// - authentication
/// - publisher membership derived from the entity / input via `PublisherId`
pub struct WorkPolicy;

impl CreatePolicy<NewWork> for WorkPolicy {
    fn can_create<C: PolicyContext>(ctx: &C, data: &NewWork, _params: ()) -> ThothResult<()> {
        ctx.require_publisher_for(data)?;
        data.validate()
    }
}

impl UpdatePolicy<Work, PatchWork> for WorkPolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &Work,
        patch: &PatchWork,
        _params: (),
    ) -> ThothResult<()> {
        let user = ctx.require_publisher_for(current)?;
        ctx.require_publisher_for(patch)?;
        current.can_update_imprint(ctx.db())?;

        if patch.work_type == WorkType::BookChapter {
            current.can_be_chapter(ctx.db())?;
        }

        patch.validate()?;

        if current.is_published() && !patch.is_published() && !user.is_superuser() {
            return Err(ThothError::ThothSetWorkStatusError);
        }
        Ok(())
    }
}

impl DeletePolicy<Work> for WorkPolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &Work) -> ThothResult<()> {
        let user = ctx.require_publisher_for(current)?;
        if current.is_published() && !user.is_superuser() {
            return Err(ThothError::ThothDeleteWorkError);
        }
        Ok(())
    }
}
