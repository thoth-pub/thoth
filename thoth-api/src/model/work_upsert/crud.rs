//! The work-level substrate operations of `BE-06`: control, seed, admission,
//! materialization, the drain and the work-level reports.
//!
//! Every function here is a model function behind a new BE-06 entry point and
//! opens every transaction it runs through
//! [`work_upsert_transaction`](super::work_upsert_transaction) (Amendment 3
//! section 10.3, EB1). Request authorization is the resolver's, and always
//! precedes the call.

use diesel::prelude::*;
use thoth_errors::{ThothError, ThothResult};

use super::registry::{self, WorkLevelExecutionProfile};
use super::{take_execution_gate, work_upsert_transaction, WorkUpsertControl, WorkUpsertTxError};
use crate::db::PgPool;
use crate::model::publisher_distribution_platform::DistributionPlatform;
use crate::schema::work_upsert_control;

/// The registered profile of `platform`, or `WORK_UPSERT_PROFILE_NOT_IMPLEMENTED`
/// before any database access.
fn registered(platform: DistributionPlatform) -> ThothResult<&'static WorkLevelExecutionProfile> {
    registry::execution_profile(platform).ok_or(ThothError::WorkUpsertProfileNotImplemented)
}

/// Read one control row `FOR UPDATE`; its absence is the released
/// `EntityNotFound`.
fn lock_control(
    connection: &mut PgConnection,
    profile: DistributionPlatform,
) -> Result<WorkUpsertControl, WorkUpsertTxError> {
    work_upsert_control::table
        .filter(work_upsert_control::execution_profile.eq(profile))
        .select((
            work_upsert_control::execution_profile,
            work_upsert_control::capture_enabled,
            work_upsert_control::execution_enabled,
        ))
        .for_update()
        .first::<WorkUpsertControl>(connection)
        .optional()?
        .ok_or(WorkUpsertTxError::Thoth(ThothError::EntityNotFound))
}

/// `enableWorkUpsertCapture` (Amendment 3 section 9.1): `(false, false)` to
/// `(true, false)`; idempotent; takes no execution gate.
pub fn enable_work_upsert_capture(
    db: &PgPool,
    platform: DistributionPlatform,
) -> ThothResult<WorkUpsertControl> {
    let profile = registered(platform)?;
    work_upsert_transaction(db, |connection| {
        let control = lock_control(connection, profile.key)?;
        if control.capture_enabled {
            return Ok(control);
        }
        diesel::update(work_upsert_control::table)
            .filter(work_upsert_control::execution_profile.eq(profile.key))
            .set((
                work_upsert_control::capture_enabled.eq(true),
                work_upsert_control::updated_at.eq(diesel::dsl::now),
            ))
            .execute(connection)?;
        Ok(WorkUpsertControl {
            capture_enabled: true,
            ..control
        })
    })
}

/// `setWorkUpsertExecution` (Amendment 3 section 9.1): takes the profile's
/// execution gate `Q` EXCLUSIVE first and holds it to commit, so it returns
/// only after every consumer that read the previous value has committed.
pub fn set_work_upsert_execution(
    db: &PgPool,
    platform: DistributionPlatform,
    enabled: bool,
) -> ThothResult<WorkUpsertControl> {
    let profile = registered(platform)?;
    work_upsert_transaction(db, |connection| {
        take_execution_gate(connection, profile.key, true)?;
        let control = lock_control(connection, profile.key)?;
        if enabled && !control.capture_enabled {
            return Err(ThothError::WorkUpsertCaptureNotEnabled.into());
        }
        if control.execution_enabled == enabled {
            return Ok(control);
        }
        diesel::update(work_upsert_control::table)
            .filter(work_upsert_control::execution_profile.eq(profile.key))
            .set((
                work_upsert_control::execution_enabled.eq(enabled),
                work_upsert_control::updated_at.eq(diesel::dsl::now),
            ))
            .execute(connection)?;
        Ok(WorkUpsertControl {
            execution_enabled: enabled,
            ..control
        })
    })
}

/// Report 10, `workUpsertControl`: the control row of every profile, in
/// canonical platform order.
pub fn work_upsert_controls(db: &PgPool) -> ThothResult<Vec<WorkUpsertControl>> {
    let mut rows = work_upsert_transaction(db, |connection| {
        Ok(work_upsert_control::table
            .select((
                work_upsert_control::execution_profile,
                work_upsert_control::capture_enabled,
                work_upsert_control::execution_enabled,
            ))
            .load::<WorkUpsertControl>(connection)?)
    })?;
    rows.sort_by_key(|row| {
        DistributionPlatform::ALL
            .iter()
            .position(|platform| *platform == row.execution_profile)
    });
    Ok(rows)
}
