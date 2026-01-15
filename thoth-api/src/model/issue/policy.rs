use diesel::dsl::{exists, select};
use diesel::prelude::*;
use uuid::Uuid;

use crate::model::issue::{Issue, NewIssue, PatchIssue};
use crate::policy::{CreatePolicy, DeletePolicy, MovePolicy, PolicyContext, UpdatePolicy};
use thoth_errors::{ThothError, ThothResult};

/// Write policies for `Issue`.
///
/// These policies are responsible for:
/// - requiring authentication
/// - requiring publisher membership (tenant boundary)
pub struct IssuePolicy;

/// Ensure the work's imprint matches the series imprint for an issue.
fn issue_imprints_match(db: &crate::db::PgPool, work_id: Uuid, series_id: Uuid) -> ThothResult<()> {
    use crate::schema::{series, work};

    let mut conn = db.get()?;

    let query = series::table
        .inner_join(work::table.on(work::imprint_id.eq(series::imprint_id)))
        .filter(series::series_id.eq(series_id))
        .filter(work::work_id.eq(work_id));

    match select(exists(query)).get_result(&mut conn)? {
        true => Ok(()),
        false => Err(ThothError::IssueImprintsError),
    }
}

impl CreatePolicy<NewIssue> for IssuePolicy {
    fn can_create<C: PolicyContext>(ctx: &C, data: &NewIssue, _params: ()) -> ThothResult<()> {
        ctx.require_publisher_for(data)?;

        issue_imprints_match(ctx.db(), data.work_id, data.series_id)
    }
}

impl UpdatePolicy<Issue, PatchIssue> for IssuePolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &Issue,
        patch: &PatchIssue,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ctx.require_publisher_for(patch)?;

        issue_imprints_match(ctx.db(), patch.work_id, patch.series_id)
    }
}

impl DeletePolicy<Issue> for IssuePolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &Issue) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        Ok(())
    }
}

impl MovePolicy<Issue> for IssuePolicy {
    fn can_move<C: PolicyContext>(ctx: &C, current: &Issue) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        Ok(())
    }
}
