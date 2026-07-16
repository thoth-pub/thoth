use crate::model::additional_resource::{
    AdditionalResource, NewAdditionalResource, PatchAdditionalResource,
};
use crate::model::file::File;
use crate::model::work::{Work, WorkType};
use crate::model::Crud;
use crate::policy::{CreatePolicy, DeletePolicy, MovePolicy, PolicyContext, UpdatePolicy};
use thoth_errors::{ThothError, ThothResult};

/// Write policies for `AdditionalResource`.
///
/// These policies are responsible for:
/// - enforcing publisher scoping
/// - preventing attachment to chapter records
/// - preventing manual update of auto-generated Thoth Hosting URLs
pub struct AdditionalResourcePolicy;

fn ensure_work_is_book(db: &crate::db::PgPool, work_id: uuid::Uuid) -> ThothResult<()> {
    let work = Work::from_id(db, &work_id)?;
    if work.work_type == WorkType::BookChapter {
        Err(ThothError::ChapterBookMetadataError)
    } else {
        Ok(())
    }
}

fn ensure_no_hosted_file(
    db: &crate::db::PgPool,
    additional_resource_id: uuid::Uuid,
) -> ThothResult<()> {
    let file = File::from_additional_resource_id(db, &additional_resource_id)?;
    if file.is_some() {
        Err(ThothError::HostedFileUrlEditError)
    } else {
        Ok(())
    }
}

impl CreatePolicy<NewAdditionalResource> for AdditionalResourcePolicy {
    fn can_create<C: PolicyContext>(
        ctx: &C,
        data: &NewAdditionalResource,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publisher_for(data)?;
        ensure_work_is_book(ctx.db(), data.work_id)
    }
}

impl UpdatePolicy<AdditionalResource, PatchAdditionalResource> for AdditionalResourcePolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &AdditionalResource,
        patch: &PatchAdditionalResource,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ctx.require_publisher_for(patch)?;
        ensure_work_is_book(ctx.db(), current.work_id)?;
        ensure_work_is_book(ctx.db(), patch.work_id)?;

        if patch.url != current.url && !ctx.allow_hosted_file_url_update() {
            ensure_no_hosted_file(ctx.db(), current.additional_resource_id)?;
        }
        Ok(())
    }
}

impl DeletePolicy<AdditionalResource> for AdditionalResourcePolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &AdditionalResource) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ensure_work_is_book(ctx.db(), current.work_id)
    }
}

impl MovePolicy<AdditionalResource> for AdditionalResourcePolicy {
    fn can_move<C: PolicyContext>(ctx: &C, current: &AdditionalResource) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ensure_work_is_book(ctx.db(), current.work_id)
    }
}
