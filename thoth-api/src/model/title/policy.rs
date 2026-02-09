use crate::markup::MarkupFormat;
use crate::model::title::{NewTitle, PatchTitle, Title};
use crate::policy::{CreatePolicy, DeletePolicy, PolicyContext, UpdatePolicy};
use crate::schema::work_title;

use diesel::dsl::{exists, select};
use diesel::prelude::*;
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

/// Write policies for `Title`.
///
/// For now this policy enforces the tenant boundary only:
/// - authentication
/// - publisher membership derived from the entity / input via `PublisherId`
pub struct TitlePolicy;

fn has_canonical_title(db: &crate::db::PgPool, work_id: &Uuid) -> ThothResult<bool> {
    let mut connection = db.get()?;
    let query = work_title::table
        .filter(work_title::work_id.eq(work_id))
        .filter(work_title::canonical.eq(true));

    let result: bool = select(exists(query)).get_result(&mut connection)?;
    Ok(result)
}

impl CreatePolicy<NewTitle, Option<MarkupFormat>> for TitlePolicy {
    fn can_create<C: PolicyContext>(
        ctx: &C,
        data: &NewTitle,
        markup: Option<MarkupFormat>,
    ) -> ThothResult<()> {
        ctx.require_publisher_for(data)?;

        // Title creation requires a markup format.
        markup.ok_or(ThothError::MissingMarkupFormat)?;

        // Canonical titles: only one canonical title is allowed per work.
        if data.canonical && has_canonical_title(ctx.db(), &data.work_id)? {
            return Err(ThothError::CanonicalTitleExistsError);
        }

        Ok(())
    }
}

impl UpdatePolicy<Title, PatchTitle, Option<MarkupFormat>> for TitlePolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &Title,
        patch: &PatchTitle,
        markup: Option<MarkupFormat>,
    ) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ctx.require_publisher_for(patch)?;

        // Title updates require a markup format.
        markup.ok_or(ThothError::MissingMarkupFormat)?;

        Ok(())
    }
}

impl DeletePolicy<Title> for TitlePolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &Title) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        Ok(())
    }
}
