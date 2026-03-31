use diesel::dsl::{exists, select};
use diesel::prelude::*;
use uuid::Uuid;

use super::{Abstract, AbstractType, NewAbstract, PatchAbstract};
use crate::markup::MarkupFormat;
use crate::policy::{CreatePolicy, DeletePolicy, PolicyContext, UpdatePolicy};
use crate::schema::work_abstract;
use thoth_errors::{ThothError, ThothResult};

pub const MAX_SHORT_ABSTRACT_CHAR_LIMIT: u16 = 350;

/// Write policies for `Abstract`.
///
/// `Abstract` spans two works and therefore potentially two publisher scopes.
/// This policy enforces:
/// - authentication
/// - membership for *all* publishers involved (via `PublisherIds`)
pub struct AbstractPolicy;

fn has_canonical_abstract(
    db: &crate::db::PgPool,
    work_id: &Uuid,
    abstract_type: AbstractType,
    exclude_abstract_id: Option<&Uuid>,
) -> ThothResult<bool> {
    let mut connection = db.get()?;
    let mut query = work_abstract::table
        .filter(work_abstract::work_id.eq(work_id))
        .filter(work_abstract::abstract_type.eq(abstract_type))
        .filter(work_abstract::canonical.eq(true))
        .into_boxed();

    if let Some(abstract_id) = exclude_abstract_id {
        query = query.filter(work_abstract::abstract_id.ne(abstract_id));
    }

    let result: bool = select(exists(query)).get_result(&mut connection)?;
    Ok(result)
}

impl CreatePolicy<NewAbstract, Option<MarkupFormat>> for AbstractPolicy {
    fn can_create<C: PolicyContext>(
        ctx: &C,
        data: &NewAbstract,
        markup: Option<MarkupFormat>,
    ) -> ThothResult<()> {
        ctx.require_publisher_for(data)?;

        // Abstract creation requires a markup format.
        markup.ok_or(ThothError::MissingMarkupFormat)?;

        // Canonical abstracts: only one canonical abstract is allowed per work and type.
        if data.canonical
            && has_canonical_abstract(ctx.db(), &data.work_id, data.abstract_type, None)?
        {
            return Err(ThothError::CanonicalAbstractExistsError);
        }

        if data.abstract_type == AbstractType::Short
            && data.content.len() > MAX_SHORT_ABSTRACT_CHAR_LIMIT as usize
        {
            return Err(ThothError::ShortAbstractLimitExceedError);
        };

        Ok(())
    }
}

impl UpdatePolicy<Abstract, PatchAbstract, Option<MarkupFormat>> for AbstractPolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &Abstract,
        patch: &PatchAbstract,
        markup: Option<MarkupFormat>,
    ) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ctx.require_publisher_for(patch)?;

        // Abstract creation requires a markup format.
        markup.ok_or(ThothError::MissingMarkupFormat)?;

        if patch.canonical
            && has_canonical_abstract(
                ctx.db(),
                &patch.work_id,
                patch.abstract_type,
                Some(&current.abstract_id),
            )?
        {
            return Err(ThothError::CanonicalAbstractExistsError);
        }

        if patch.abstract_type == AbstractType::Short
            && patch.content.len() > MAX_SHORT_ABSTRACT_CHAR_LIMIT as usize
        {
            return Err(ThothError::ShortAbstractLimitExceedError);
        };

        Ok(())
    }
}

impl DeletePolicy<Abstract> for AbstractPolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &Abstract) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        Ok(())
    }
}
