use crate::model::endorsement::{Endorsement, NewEndorsement, PatchEndorsement};
use crate::model::work::{Work, WorkType};
use crate::model::Crud;
use crate::policy::{CreatePolicy, DeletePolicy, MovePolicy, PolicyContext, UpdatePolicy};
use thoth_errors::{ThothError, ThothResult};

/// Write policies for `Endorsement`.
///
/// These policies enforce publisher scoping and prevent attachment to chapter records.
pub struct EndorsementPolicy;

fn ensure_work_is_book(db: &crate::db::PgPool, work_id: uuid::Uuid) -> ThothResult<()> {
    let work = Work::from_id(db, &work_id)?;
    if work.work_type == WorkType::BookChapter {
        Err(ThothError::ChapterBookMetadataError)
    } else {
        Ok(())
    }
}

impl CreatePolicy<NewEndorsement> for EndorsementPolicy {
    fn can_create<C: PolicyContext>(
        ctx: &C,
        data: &NewEndorsement,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publisher_for(data)?;
        ensure_work_is_book(ctx.db(), data.work_id)
    }
}

impl UpdatePolicy<Endorsement, PatchEndorsement> for EndorsementPolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &Endorsement,
        patch: &PatchEndorsement,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ctx.require_publisher_for(patch)?;
        ensure_work_is_book(ctx.db(), current.work_id)?;
        ensure_work_is_book(ctx.db(), patch.work_id)
    }
}

impl DeletePolicy<Endorsement> for EndorsementPolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &Endorsement) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ensure_work_is_book(ctx.db(), current.work_id)
    }
}

impl MovePolicy<Endorsement> for EndorsementPolicy {
    fn can_move<C: PolicyContext>(ctx: &C, current: &Endorsement) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ensure_work_is_book(ctx.db(), current.work_id)
    }
}
