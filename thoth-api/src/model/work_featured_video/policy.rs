use crate::model::work::{Work, WorkType};
use crate::model::work_featured_video::{
    NewWorkFeaturedVideo, PatchWorkFeaturedVideo, WorkFeaturedVideo,
};
use crate::model::Crud;
use crate::policy::{CreatePolicy, DeletePolicy, PolicyContext, UpdatePolicy};
use thoth_errors::{ThothError, ThothResult};

/// Write policies for `WorkFeaturedVideo`.
///
/// These policies enforce publisher scoping and prevent attachment to chapter records.
pub struct WorkFeaturedVideoPolicy;

fn ensure_work_is_book(db: &crate::db::PgPool, work_id: uuid::Uuid) -> ThothResult<()> {
    let work = Work::from_id(db, &work_id)?;
    if work.work_type == WorkType::BookChapter {
        Err(ThothError::ChapterBookMetadataError)
    } else {
        Ok(())
    }
}

impl CreatePolicy<NewWorkFeaturedVideo> for WorkFeaturedVideoPolicy {
    fn can_create<C: PolicyContext>(
        ctx: &C,
        data: &NewWorkFeaturedVideo,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publisher_for(data)?;
        ensure_work_is_book(ctx.db(), data.work_id)
    }
}

impl UpdatePolicy<WorkFeaturedVideo, PatchWorkFeaturedVideo> for WorkFeaturedVideoPolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &WorkFeaturedVideo,
        patch: &PatchWorkFeaturedVideo,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ctx.require_publisher_for(patch)?;
        ensure_work_is_book(ctx.db(), current.work_id)?;
        ensure_work_is_book(ctx.db(), patch.work_id)
    }
}

impl DeletePolicy<WorkFeaturedVideo> for WorkFeaturedVideoPolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &WorkFeaturedVideo) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ensure_work_is_book(ctx.db(), current.work_id)
    }
}
