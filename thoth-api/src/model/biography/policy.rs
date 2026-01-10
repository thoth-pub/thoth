use diesel::dsl::{exists, select};
use diesel::prelude::*;
use uuid::Uuid;

use crate::markup::MarkupFormat;
use crate::model::biography::{Biography, NewBiography, PatchBiography};
use crate::policy::{CreatePolicy, DeletePolicy, PolicyContext, UpdatePolicy};
use crate::schema::biography;
use thoth_errors::{ThothError, ThothResult};

/// Write policies for `Biography`.
///
/// These policies are responsible for:
/// - requiring authentication
/// - requiring publisher membership (tenant boundary)
/// - requiring a markup format for biography writes
pub struct BiographyPolicy;

fn has_canonical_biography(db: &crate::db::PgPool, contribution_id: &Uuid) -> ThothResult<bool> {
    let mut connection = db.get()?;
    let query = biography::table
        .filter(biography::contribution_id.eq(contribution_id))
        .filter(biography::canonical.eq(true));

    let result: bool = select(exists(query)).get_result(&mut connection)?;
    Ok(result)
}

impl CreatePolicy<NewBiography, Option<MarkupFormat>> for BiographyPolicy {
    fn can_create<C: PolicyContext>(
        ctx: &C,
        data: &NewBiography,
        markup: Option<MarkupFormat>,
    ) -> ThothResult<()> {
        ctx.require_publisher_for(data)?;

        // Biography creation requires a markup format.
        markup.ok_or(ThothError::MissingMarkupFormat)?;

        if data.canonical && has_canonical_biography(ctx.db(), &data.contribution_id)? {
            return Err(ThothError::CanonicalBiographyExistsError);
        }

        Ok(())
    }
}

impl UpdatePolicy<Biography, PatchBiography, Option<MarkupFormat>> for BiographyPolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &Biography,
        patch: &PatchBiography,
        markup: Option<MarkupFormat>,
    ) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ctx.require_publisher_for(patch)?;

        // Biography updates require a markup format.
        markup.ok_or(ThothError::MissingMarkupFormat)?;

        Ok(())
    }
}

impl DeletePolicy<Biography> for BiographyPolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &Biography) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        Ok(())
    }
}
