use crate::model::{
    file::{File, FileType},
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
/// - preventing manual update of auto-generated Thoth Hosting URLs
pub struct PublicationPolicy;

fn ensure_no_hosted_file(db: &crate::db::PgPool, publication_id: uuid::Uuid) -> ThothResult<()> {
    let file = File::from_publication_id(db, &publication_id, FileType::A11yReport)?;
    if file.is_some() {
        Err(ThothError::HostedFileUrlEditError)
    } else {
        Ok(())
    }
}

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

        if patch.accessibility_report_url != current.accessibility_report_url
            && !ctx.allow_hosted_file_url_update()
        {
            ensure_no_hosted_file(ctx.db(), current.publication_id)?;
        }

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
