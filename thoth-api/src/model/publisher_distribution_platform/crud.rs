//! Assignment lifecycle and focused queries for `publisher_distribution_platform`.
//!
//! `Crud` is deliberately **not** implemented for the assignment entity: BE-02
//! adds no generic CRUD mutation surface. The functions here are the only
//! supported domain writes, and BE-02 exposes none of them through GraphQL.

use diesel::pg::PgConnection;
use diesel::{Connection, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use super::{
    DistributionPlatform, PublisherDistributionPlatform, PublisherDistributionPlatformAssignment,
};
use crate::db::PgPool;
use crate::model::Timestamp;
use crate::schema::{publisher, publisher_distribution_platform};

/// The columns of the public assignment projection, in canonical order.
type AssignmentRow = (Uuid, DistributionPlatform, Timestamp);

/// Whether one connection-scoped lifecycle call wrote persisted assignment
/// state.
///
/// `BE-02`'s pool-level `enable`/`disable` return `ThothResult<()>`, so a caller
/// cannot distinguish an idempotent no-op from a linked-state repair. The
/// connection-scoped primitives report this instead, which is what lets the
/// `BE-03` service-configuration write coordinator decide whether a request was
/// a true no-op (specification section 7.7 item 3).
///
/// It is deliberately the minimum `BE-03` needs — whether persisted state
/// changed — and is not a job-oriented change description.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub(crate) enum AssignmentLifecycleOutcome {
    /// Nothing was written and no timestamp moved.
    Unchanged,
    /// Persisted assignment state was written: an insert, a re-enable, a
    /// disable, or a linked-state repair.
    Changed,
}

impl AssignmentLifecycleOutcome {
    pub(crate) fn changed(self) -> bool {
        self == AssignmentLifecycleOutcome::Changed
    }
}

impl PublisherDistributionPlatform {
    /// Enable `platform` for `publisher_id`, normalizing its linked group.
    ///
    /// A non-assignable destination fails closed **before** any transaction is
    /// opened, so no row is created or modified.
    ///
    /// For a destination with no linked group this is the singleton lifecycle:
    /// an absent row is inserted with a new activation, a disabled row is
    /// re-enabled with a **new** activation, and an already-enabled row is an
    /// idempotent no-op that writes nothing and moves no timestamp.
    ///
    /// For a linked group this is the group normalization of specification
    /// section 7.2: the operation is a complete no-op only when every member
    /// row exists, is enabled, and shares one `activation_id` and one
    /// `enabled_at`. Any other state — including a one-sided pair, a
    /// split-activation pair or a split-timestamp pair — is atomically
    /// repaired to one new shared activation.
    pub fn enable(
        db: &PgPool,
        publisher_id: Uuid,
        platform: DistributionPlatform,
    ) -> ThothResult<()> {
        // Kept exactly as merged: the non-assignable check runs **before** a
        // connection is acquired or a transaction opened.
        if !platform.is_assignable() {
            return Err(ThothError::DistributionPlatformNotAssignable(
                platform.to_string(),
            ));
        }
        let mut connection = db.get()?;
        connection.transaction(|connection| {
            let _outcome = Self::enable_on(connection, publisher_id, platform)?;
            Ok(())
        })
    }

    /// The connection-scoped enable primitive, assuming a caller-owned
    /// transaction.
    ///
    /// This is the single implementation of [`Self::enable`]'s lifecycle: the
    /// pool-level function is a wrapper that acquires a connection, opens a
    /// transaction, delegates here and discards the outcome. There is exactly
    /// one linked-platform algorithm in the repository and it is this one.
    ///
    /// The non-assignable check is performed **here as well**, independently of
    /// anything the caller did and before any write, so `BE-02`'s fail-closed
    /// semantics hold for every internal caller that composes this primitive
    /// into its own transaction (specification section 7.7 item 2). It calls the
    /// same [`DistributionPlatform::is_assignable`] predicate and returns the
    /// same [`ThothError::DistributionPlatformNotAssignable`] as the wrapper, so
    /// this is one rule called from two places, not a second rule.
    ///
    /// Re-taking the publisher row lock is a harmless no-op when the caller's
    /// transaction already holds it, so the lock statement is unconditional.
    pub(crate) fn enable_on(
        connection: &mut PgConnection,
        publisher_id: Uuid,
        platform: DistributionPlatform,
    ) -> ThothResult<AssignmentLifecycleOutcome> {
        if !platform.is_assignable() {
            return Err(ThothError::DistributionPlatformNotAssignable(
                platform.to_string(),
            ));
        }
        let members = platform.linked_members();
        lock_publisher(connection, publisher_id)?;
        let existing = member_rows(connection, publisher_id, &members)?;
        if is_normalized_fully_enabled(&existing, &members) {
            return Ok(AssignmentLifecycleOutcome::Unchanged);
        }
        let activation_id = Uuid::new_v4();
        let transition_at = transaction_timestamp(connection)?;
        for member in &members {
            diesel::insert_into(publisher_distribution_platform::table)
                .values((
                    publisher_distribution_platform::publisher_id.eq(publisher_id),
                    publisher_distribution_platform::platform.eq(*member),
                    publisher_distribution_platform::enabled.eq(true),
                    publisher_distribution_platform::activation_id.eq(activation_id),
                    publisher_distribution_platform::enabled_at.eq(transition_at),
                    publisher_distribution_platform::disabled_at.eq(None::<Timestamp>),
                ))
                .on_conflict((
                    publisher_distribution_platform::publisher_id,
                    publisher_distribution_platform::platform,
                ))
                .do_update()
                .set((
                    publisher_distribution_platform::enabled.eq(true),
                    publisher_distribution_platform::activation_id.eq(activation_id),
                    publisher_distribution_platform::enabled_at.eq(transition_at),
                    publisher_distribution_platform::disabled_at.eq(None::<Timestamp>),
                ))
                .execute(connection)?;
        }
        Ok(AssignmentLifecycleOutcome::Changed)
    }

    /// Disable `platform` for `publisher_id`, and every member of its linked
    /// group.
    ///
    /// Disabled rows are retained, never deleted: the row keeps its
    /// `activation_id` and `enabled_at` and records `disabled_at`. When no
    /// member row is currently enabled — including when no row exists at all —
    /// this is an idempotent no-op that writes nothing, moves no timestamp and
    /// never creates a never-activated row.
    pub fn disable(
        db: &PgPool,
        publisher_id: Uuid,
        platform: DistributionPlatform,
    ) -> ThothResult<()> {
        let mut connection = db.get()?;
        connection.transaction(|connection| {
            let _outcome = Self::disable_on(connection, publisher_id, platform)?;
            Ok(())
        })
    }

    /// The connection-scoped disable primitive, assuming a caller-owned
    /// transaction.
    ///
    /// This is the single implementation of [`Self::disable`]'s lifecycle, and
    /// reports whether it wrote a disable transition. When no member of the
    /// linked group is currently enabled — including when no row exists at all —
    /// it writes nothing and returns
    /// [`AssignmentLifecycleOutcome::Unchanged`].
    pub(crate) fn disable_on(
        connection: &mut PgConnection,
        publisher_id: Uuid,
        platform: DistributionPlatform,
    ) -> ThothResult<AssignmentLifecycleOutcome> {
        let members = platform.linked_members();
        lock_publisher(connection, publisher_id)?;
        let existing = member_rows(connection, publisher_id, &members)?;
        if !existing.iter().any(|row| row.enabled) {
            return Ok(AssignmentLifecycleOutcome::Unchanged);
        }
        let transition_at = transaction_timestamp(connection)?;
        let enabled_members: Vec<DistributionPlatform> = existing
            .iter()
            .filter(|row| row.enabled)
            .map(|row| row.platform)
            .collect();
        diesel::update(
            publisher_distribution_platform::table
                .filter(publisher_distribution_platform::publisher_id.eq(publisher_id))
                .filter(publisher_distribution_platform::platform.eq_any(&enabled_members)),
        )
        .set((
            publisher_distribution_platform::enabled.eq(false),
            publisher_distribution_platform::disabled_at.eq(Some(transition_at)),
        ))
        .execute(connection)?;
        Ok(AssignmentLifecycleOutcome::Changed)
    }

    /// Every persisted assignment row for one publisher, in canonical
    /// destination order, including retained disabled rows.
    ///
    /// This is a domain/test read: the public GraphQL contract exposes only
    /// enabled assignments through [`Self::enabled_assignments`].
    pub fn all_for_publisher(
        db: &PgPool,
        publisher_id: Uuid,
    ) -> ThothResult<Vec<PublisherDistributionPlatform>> {
        let mut connection = db.get()?;
        publisher_distribution_platform::table
            .filter(publisher_distribution_platform::publisher_id.eq(publisher_id))
            .order((
                publisher_distribution_platform::publisher_id.asc(),
                publisher_distribution_platform::platform.asc(),
            ))
            .load::<PublisherDistributionPlatform>(&mut connection)
            .map_err(Into::into)
    }

    /// The enabled assignments of one publisher, in canonical destination
    /// order.
    ///
    /// This is the direct synchronous baseline of the loader-backed
    /// `Publisher.distributionPlatforms` field and must stay equivalent to it
    /// in membership, filtering and ordering.
    pub fn enabled_assignments(
        db: &PgPool,
        publisher_id: Uuid,
    ) -> ThothResult<Vec<PublisherDistributionPlatformAssignment>> {
        let mut connection = db.get()?;
        let rows: Vec<AssignmentRow> = enabled_assignment_rows(&mut connection, &[publisher_id])?;
        Ok(rows
            .into_iter()
            .map(
                |(_, platform, enabled_at)| PublisherDistributionPlatformAssignment {
                    platform,
                    enabled_at,
                },
            )
            .collect())
    }
}

/// One set-based query returning the enabled assignments of every requested
/// publisher, ordered by `(publisher_id, platform)`.
///
/// The `platform` column is a PostgreSQL enum, so ordering by it yields the
/// canonical declaration order required by the public contract. This is the
/// only statement the DataLoader batch function issues for a dispatch chunk:
/// there is deliberately no per-parent loop, fallback or retry.
pub(crate) fn enabled_assignment_rows(
    connection: &mut PgConnection,
    publisher_ids: &[Uuid],
) -> ThothResult<Vec<AssignmentRow>> {
    publisher_distribution_platform::table
        .filter(publisher_distribution_platform::publisher_id.eq_any(publisher_ids))
        .filter(publisher_distribution_platform::enabled.eq(true))
        .order((
            publisher_distribution_platform::publisher_id.asc(),
            publisher_distribution_platform::platform.asc(),
        ))
        .select((
            publisher_distribution_platform::publisher_id,
            publisher_distribution_platform::platform,
            publisher_distribution_platform::enabled_at,
        ))
        .load::<AssignmentRow>(connection)
        .map_err(Into::into)
}

/// Take the publisher row lock that serializes every assignment transition for
/// one publisher.
///
/// Reads used to decide a transition run after this lock and inside the same
/// transaction, so concurrent transitions serialize rather than racing.
/// Different publishers never contend on the same lock.
///
/// `BE-03`'s service-configuration write coordinator takes the **same** lock, on
/// the same row, as the first statement of its own transaction, so the
/// application-level lock order is `publisher` row first, everything else after
/// (specification section 7.8).
pub(crate) fn lock_publisher(connection: &mut PgConnection, publisher_id: Uuid) -> ThothResult<()> {
    publisher::table
        .filter(publisher::publisher_id.eq(publisher_id))
        .select(publisher::publisher_id)
        .for_update()
        .get_result::<Uuid>(connection)
        .optional()?
        .map(|_| ())
        .ok_or(ThothError::EntityNotFound)
}

/// The transaction timestamp, so every row written by one logical transition
/// carries an identical value.
///
/// `CURRENT_TIMESTAMP` is transaction-scoped, unlike `clock_timestamp()`.
fn transaction_timestamp(connection: &mut PgConnection) -> ThothResult<Timestamp> {
    diesel::select(diesel::dsl::sql::<diesel::sql_types::Timestamptz>(
        "CURRENT_TIMESTAMP",
    ))
    .get_result::<Timestamp>(connection)
    .map_err(Into::into)
}

fn member_rows(
    connection: &mut PgConnection,
    publisher_id: Uuid,
    members: &[DistributionPlatform],
) -> ThothResult<Vec<PublisherDistributionPlatform>> {
    publisher_distribution_platform::table
        .filter(publisher_distribution_platform::publisher_id.eq(publisher_id))
        .filter(publisher_distribution_platform::platform.eq_any(members))
        .order(publisher_distribution_platform::platform.asc())
        .load::<PublisherDistributionPlatform>(connection)
        .map_err(Into::into)
}

/// Whether the member set is already normalized fully enabled, which is the
/// only state in which enabling is a complete no-op.
///
/// Every member row must exist, be enabled with `disabled_at IS NULL`, and
/// share one `activation_id` and one `enabled_at`. A one-sided, split-activation
/// or split-timestamp pair fails this test and is therefore repaired rather
/// than treated as idempotent.
fn is_normalized_fully_enabled(
    existing: &[PublisherDistributionPlatform],
    members: &[DistributionPlatform],
) -> bool {
    if existing.len() != members.len() {
        return false;
    }
    let Some(first) = existing.first() else {
        return false;
    };
    existing.iter().all(|row| {
        row.enabled
            && row.disabled_at.is_none()
            && row.activation_id == first.activation_id
            && row.enabled_at == first.enabled_at
    })
}
