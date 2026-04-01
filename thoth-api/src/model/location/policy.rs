use diesel::dsl::exists;
use diesel::prelude::*;
use diesel::select;
use uuid::Uuid;

use super::{Location, LocationPlatform, NewLocation, PatchLocation};
use crate::policy::{CreatePolicy, DeletePolicy, PolicyContext, UpdatePolicy, UserAccess};
use crate::schema::location;
use thoth_errors::{ThothError, ThothResult};

/// Write policies for `Location`.
///
/// These policies are responsible for:
/// - requiring authentication
/// - requiring publisher membership (tenant boundary)
/// - enforcing any additional business rules (e.g. Thoth platform restrictions)
pub struct LocationPolicy;

fn has_canonical_thoth_location(
    db: &crate::db::PgPool,
    publication_id: &Uuid,
) -> ThothResult<bool> {
    let mut connection = db.get()?;
    let query = location::table
        .filter(location::publication_id.eq(publication_id))
        .filter(location::location_platform.eq(LocationPlatform::Thoth))
        .filter(location::canonical.eq(true));

    let result: bool = select(exists(query)).get_result(&mut connection)?;
    Ok(result)
}

impl CreatePolicy<NewLocation> for LocationPolicy {
    fn can_create<C: PolicyContext>(ctx: &C, data: &NewLocation, _params: ()) -> ThothResult<()> {
        let user = ctx.require_publisher_for(data)?;

        // Only superusers can create new locations where Location Platform is Thoth.
        if !user.is_superuser() && data.location_platform == LocationPlatform::Thoth {
            return Err(ThothError::ThothLocationError);
        }

        // Canonical locations must be complete; non-canonical locations must satisfy rules.
        if data.canonical {
            data.canonical_record_complete(ctx.db())?;
        } else {
            data.can_be_non_canonical(ctx.db())?;
        }

        Ok(())
    }
}

impl UpdatePolicy<Location, PatchLocation> for LocationPolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &Location,
        patch: &PatchLocation,
        _params: (),
    ) -> ThothResult<()> {
        let user = ctx.require_publisher_for(current)?;
        ctx.require_publisher_for(patch)?;

        // Only superusers can edit locations where Location Platform is Thoth.
        if !user.is_superuser() && current.location_platform == LocationPlatform::Thoth {
            return Err(ThothError::ThothLocationError);
        }

        // Only superusers can update the canonical location when a Thoth Location Platform
        // canonical location already exists for the publication.
        if patch.canonical
            && has_canonical_thoth_location(ctx.db(), &patch.publication_id)?
            && !user.is_superuser()
        {
            return Err(ThothError::ThothUpdateCanonicalError);
        }

        // If setting canonical to true, require record completeness.
        if patch.canonical {
            patch.canonical_record_complete(ctx.db())?;
        }

        Ok(())
    }
}

impl DeletePolicy<Location> for LocationPolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, location: &Location) -> ThothResult<()> {
        let user = ctx.require_publisher_for(location)?;

        // Thoth platform locations are superuser-restricted.
        if !user.is_superuser() && location.location_platform == LocationPlatform::Thoth {
            return Err(ThothError::ThothLocationError);
        }

        Ok(())
    }
}
