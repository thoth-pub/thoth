//! The canonical service-configuration write coordinator and the superuser
//! staff-report queries (`BE-03`).
//!
//! `Crud` is deliberately **not** implemented for service configuration: there
//! is no generic create/update/delete surface for it. The one supported
//! production write is [`replace_publisher_service_configuration`], and it is
//! the only place in the repository that commits a change to a publisher's
//! desired package, desired enabled platform state, canonical configuration
//! version token or configuration audit history.

use std::collections::{HashMap, HashSet};

use diesel::pg::PgConnection;
use diesel::{Connection, ExpressionMethods, NullableExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use super::{
    CanonicalServiceConfigurationState, NewPublisherServiceConfigurationHistory,
    PublisherServiceConfiguration, PublisherServiceConfigurationChange,
    PublisherServiceConfigurationSource, PublisherServiceConfigurationSummary,
    ReplacePublisherServiceConfigurationInput, ServiceConfigurationWriteContext,
};
use crate::db::PgPool;
use crate::model::distribution_job::crud::{
    cancel_pending_jobs_for_disabled_group_on, create_back_catalogue_job_on,
};
use crate::model::distribution_job::{DistributionJobKind, DistributionJobStatus};
use crate::model::publisher::{Publisher, PublisherField, PublisherOrderBy, ThothPackage};
use crate::model::publisher_distribution_platform::crud::{
    enabled_assignment_rows, lock_publisher, AssignmentLifecycleOutcome,
};
use crate::model::publisher_distribution_platform::{
    BackCatalogueBehaviour, DistributionPlatform, PublisherDistributionPlatform,
};
use crate::model::Timestamp;
use crate::schema::{
    distribution_job, publisher, publisher_distribution_platform,
    publisher_service_configuration_history,
};

/// The latest-change columns the staff report needs, in canonical order.
type LatestChangeRow = (Uuid, Timestamp, String, PublisherServiceConfigurationSource);

/// Replace one publisher's desired service configuration atomically.
///
/// **This is the single authoritative production write path for desired service
/// configuration** (specification section 7.6). It owns every committed write
/// across all four of: `publisher.subscription_package`, the publisher's enabled
/// distribution-platform desired state, the canonical version token
/// `publisher.service_configuration_updated_at`, and
/// `publisher_service_configuration_history`.
///
/// It executes exactly **one** transaction on **one** connection and performs
/// the whole of `BE-03` specification section 7.3 steps 2 to 12 inside it.
/// `BE-04` extends that same transaction in place, between steps 9 and 10, to
/// create durable job rows atomically with the desired-state change and to
/// cancel the pending jobs of a withdrawn assignment. It adds **no** second
/// transaction, nested transaction, savepoint, hook, callback, event or
/// after-the-fact best-effort path, so a job cannot exist without the
/// desired-state change that justified it and that change cannot commit without
/// the job it qualified for.
///
/// It makes **no authorization decision of its own**. Authorization is the
/// caller's responsibility (specification sections 7.2 and 11.1), and the caller
/// supplies the audit provenance explicitly as a
/// [`ServiceConfigurationWriteContext`].
///
/// All platform-assignment writes go through `BE-02`'s connection-scoped
/// lifecycle primitives; this function never writes
/// `publisher_distribution_platform` directly and never re-implements the
/// linked-group normalization, the normalized-state predicate, the activation
/// and timestamp semantics or the non-assignable rule.
///
/// It creates no distribution job and triggers no dissemination.
pub(crate) fn replace_publisher_service_configuration(
    db: &PgPool,
    write_context: &ServiceConfigurationWriteContext<'_>,
    data: &ReplacePublisherServiceConfigurationInput,
) -> ThothResult<PublisherServiceConfiguration> {
    let mut connection = db.get()?;
    connection.transaction(|connection| replace_in_transaction(connection, write_context, data))
}

/// Specification section 7.3 steps 2 to 12, inside the coordinator's single
/// transaction.
///
/// This is deliberately private to the coordinator's module: it is the
/// coordinator's transaction body, not a second write entry point.
fn replace_in_transaction(
    connection: &mut PgConnection,
    write_context: &ServiceConfigurationWriteContext<'_>,
    data: &ReplacePublisherServiceConfigurationInput,
) -> ThothResult<PublisherServiceConfiguration> {
    // Step 2. The publisher row lock is the first statement of the transaction,
    // so every read and write below happens under it. An absent row ends the
    // transaction with no write.
    lock_publisher(connection, data.publisher_id)?;

    // Step 3. The canonical current configuration, read under that lock.
    let current = publisher_row(connection, data.publisher_id)?;
    let current_enabled = enabled_platforms(connection, data.publisher_id)?;
    let previous_token = current.service_configuration_updated_at;

    // Steps 4 and 5. Staleness precedes validation and every lifecycle call, so
    // a stale request writes nothing even when it would otherwise have been a
    // true no-op or would otherwise have repaired a split linked group.
    if previous_token != data.expected_updated_at {
        return Err(ThothError::StalePublisherServiceConfiguration);
    }

    // Step 6. Deduplicate the requested set and close it under linked
    // membership, so naming either OAPEN or DOAB enables both and naming
    // neither disables both.
    let desired = normalize_requested_platforms(&data.enabled_distribution_platforms);

    // Step 7. Validate the **whole** normalized desired set before any write, so
    // a rejected request never depends on rollback and fails before the first
    // lifecycle call.
    for platform in &desired {
        if !platform.is_assignable() {
            return Err(ThothError::DistributionPlatformNotAssignable(
                platform.to_string(),
            ));
        }
    }

    // Step 8. Compare only: the package write is deferred to the single
    // publisher UPDATE of step 10. Writing it here as well would update the
    // publisher row twice for a combined package-and-platform change, and the
    // publisher row carries the shared AFTER UPDATE work-freshness trigger, so
    // the second write would re-run that trigger's set-based cascade over the
    // same N work rows for no additional effect.
    let package_changed = current.subscription_package != data.subscription_package;

    // Step 9. Desired state is applied only through BE-02's connection-scoped
    // primitives, one canonical representative per linked group.
    //
    // Every desired group is enabled **unconditionally**: the call is not gated
    // on a membership diff, because membership equality does not imply the group
    // is normalized. The primitive alone decides no-op versus repair.
    //
    // BE-04: each outcome is retained with its group representative, because the
    // qualifying-job determination of step 9a needs both the representative (to
    // reach the group's members and their code-owned descriptors) and the
    // activation identity the lifecycle call minted.
    let mut lifecycle_changed = false;
    let mut activated: Vec<(DistributionPlatform, Uuid)> = Vec::new();
    let mut disabled: Vec<DistributionPlatform> = Vec::new();
    for representative in group_representatives(&desired) {
        let outcome = PublisherDistributionPlatform::enable_on(
            connection,
            data.publisher_id,
            representative,
        )?;
        lifecycle_changed |= outcome.changed();
        // A `Repaired` group is deliberately absent from this list. A repair is
        // not a new zero-enabled-to-enabled activation, and that — and nothing
        // about observed delivery — is why it creates no automatic job.
        if let AssignmentLifecycleOutcome::Activated { activation_id } = outcome {
            activated.push((representative, activation_id));
        }
    }

    // Closure under linked membership guarantees a group is either wholly
    // desired or wholly undesired, so no group receives both calls.
    let undesired_enabled: Vec<DistributionPlatform> = current_enabled
        .iter()
        .copied()
        .filter(|platform| !desired.contains(platform))
        .collect();
    for representative in group_representatives(&undesired_enabled) {
        let outcome = PublisherDistributionPlatform::disable_on(
            connection,
            data.publisher_id,
            representative,
        )?;
        lifecycle_changed |= outcome.changed();
        if outcome == AssignmentLifecycleOutcome::Disabled {
            disabled.push(representative);
        }
    }

    // Step 9a. Qualifying-job determination: pure computation, no I/O. A group
    // qualifies when it was newly `Activated` **and** its member set contains at
    // least one `AutomaticPush` destination. The behaviour is read from
    // code-owned descriptors and never inferred from a destination's name, and
    // an empty target set is what makes the `PullFeed` and `Manual` cases
    // exhaustive and future-proof for a mixed group.
    let qualifying: Vec<(Uuid, Vec<DistributionPlatform>)> = activated
        .iter()
        .filter_map(|(representative, activation_id)| {
            let targets: Vec<DistributionPlatform> = representative
                .linked_members()
                .into_iter()
                .filter(|platform| {
                    platform.descriptor().back_catalogue_behaviour
                        == BackCatalogueBehaviour::AutomaticPush
                })
                .collect();
            (!targets.is_empty()).then_some((*activation_id, targets))
        })
        .collect();

    // The source/switch rule is **one expression at one site**, so no caller —
    // present or future — can bypass either half of it.
    let superuser_api = write_context.source == PublisherServiceConfigurationSource::SuperuserApi;
    let create_jobs = write_context.job_creation.is_on() && superuser_api;

    // Step 9a'. `OFF` fails closed. Treating it as "commit the activation, skip
    // the job" would leave that activation without an onboarding job for ever:
    // nothing afterwards repairs it, because a later replacement naming the same
    // platforms yields `Unchanged`, and section 9.4.4 deliberately runs no sweep
    // when the switch is turned on.
    //
    // Returning the error here discards every lifecycle write step 9 made,
    // because the coordinator owns one transaction on one connection. That is
    // what delivers the required rollback without a savepoint, a compensating
    // write or a second transaction.
    //
    // MIGRATION_BACKFILL is deliberately not subject to this rule: it is
    // job-free *by design* rather than because a feature is disabled, so it
    // commits normally under both switch positions.
    if !qualifying.is_empty() && superuser_api && !write_context.job_creation.is_on() {
        return Err(ThothError::DistributionJobCreationDisabled);
    }

    // Step 9b. Deduplicated job and target writes, in canonical
    // group-representative order. Reached only when `create_jobs` holds.
    //
    // These precede the publisher UPDATE of step 10 deliberately: that statement
    // fires the AFTER UPDATE work-freshness trigger, whose single set-based
    // statement takes row locks on all N of the publisher's work rows and holds
    // them until commit. Writing the jobs first costs nothing and shortens the
    // widest part of the lock footprint.
    if create_jobs {
        for (activation_id, targets) in &qualifying {
            create_back_catalogue_job_on(connection, data.publisher_id, *activation_id, targets)?;
        }
    }

    // Step 9c. Assignment-withdrawal cancellation, in the same transaction as
    // the disable that caused it. `PENDING` jobs for the withdrawn group are
    // cancelled with `ASSIGNMENT_DISABLED`; `RUNNING` jobs are left alone,
    // because external work may be in flight and cancelling cannot undo an
    // upload.
    for representative in &disabled {
        cancel_pending_jobs_for_disabled_group_on(
            connection,
            data.publisher_id,
            &representative.linked_members(),
        )?;
    }

    // A true no-op: the package was unchanged and every lifecycle call reported
    // `Unchanged`. No publisher UPDATE, no token movement, no audit row, and
    // therefore neither publisher trigger fires.
    if !package_changed && !lifecycle_changed {
        return Ok(PublisherServiceConfiguration::new(current));
    }

    // Step 10. **Exactly one** publisher UPDATE for the whole committed change.
    // It carries the package only when the package changed, and always carries
    // the token. Because the publisher row carries the shared AFTER UPDATE
    // work-freshness trigger, this single statement means the cascade runs once
    // per committed change — the same cost for a combined package-and-platform
    // change as for a platform-only change or a linked repair.
    //
    // `GREATEST` makes the token strictly increasing per publisher.
    // `CURRENT_TIMESTAMP` is `transaction_timestamp()`, so a transaction that
    // started earlier but blocked on the row lock would otherwise be able to
    // store a value equal to a token some client still holds.
    // The two branches differ only in whether the package travels with the
    // token. `publisher::table` is repeated rather than hoisted into a local so
    // that both writes remain visible to the specification's write-path
    // containment search for `diesel::update` against `publisher::table`.
    let next_token = || {
        diesel::dsl::sql::<diesel::sql_types::Timestamptz>(
            "GREATEST(CURRENT_TIMESTAMP, service_configuration_updated_at + interval '1 microsecond')",
        )
    };
    let updated: Publisher = if package_changed {
        diesel::update(publisher::table.filter(publisher::publisher_id.eq(data.publisher_id)))
            .set((
                publisher::subscription_package.eq(data.subscription_package),
                publisher::service_configuration_updated_at.eq(next_token()),
            ))
            .returning(publisher::all_columns)
            .get_result(connection)?
    } else {
        diesel::update(publisher::table.filter(publisher::publisher_id.eq(data.publisher_id)))
            .set(publisher::service_configuration_updated_at.eq(next_token()))
            .returning(publisher::all_columns)
            .get_result(connection)?
    };

    // Step 11. Exactly one audit row for the whole committed change, with the
    // caller-supplied source and actor. The after state is read back from the
    // database rather than assumed from the request, so the row records what was
    // actually persisted.
    let after_enabled = enabled_platforms(connection, data.publisher_id)?;
    let before_state = CanonicalServiceConfigurationState {
        subscription_package: current.subscription_package,
        enabled_distribution_platforms: current_enabled,
        configuration_version: previous_token,
    };
    let after_state = CanonicalServiceConfigurationState {
        subscription_package: updated.subscription_package,
        enabled_distribution_platforms: after_enabled,
        configuration_version: updated.service_configuration_updated_at,
    };
    let audit = NewPublisherServiceConfigurationHistory {
        publisher_id: data.publisher_id,
        actor: write_context.actor.to_string(),
        source: write_context.source,
        before_state: serde_json::to_value(&before_state)?,
        after_state: serde_json::to_value(&after_state)?,
    };
    diesel::insert_into(publisher_service_configuration_history::table)
        .values(&audit)
        .execute(connection)?;

    // Step 12.
    Ok(PublisherServiceConfiguration::new(updated))
}

/// The requested platform set, deduplicated and closed under linked membership,
/// in canonical [`DistributionPlatform::ALL`] order.
///
/// Duplicates are deduplicated rather than rejected: the argument is a set.
pub(crate) fn normalize_requested_platforms(
    requested: &[DistributionPlatform],
) -> Vec<DistributionPlatform> {
    let mut closed: HashSet<DistributionPlatform> = HashSet::new();
    for platform in requested {
        for member in platform.linked_members() {
            closed.insert(member);
        }
    }
    DistributionPlatform::ALL
        .into_iter()
        .filter(|platform| closed.contains(platform))
        .collect()
}

/// One canonical representative per linked group: the earliest member of the
/// group present in `platforms`, which must already be in canonical order.
fn group_representatives(platforms: &[DistributionPlatform]) -> Vec<DistributionPlatform> {
    let mut covered: HashSet<DistributionPlatform> = HashSet::new();
    let mut representatives: Vec<DistributionPlatform> = Vec::new();
    for platform in platforms {
        if covered.contains(platform) {
            continue;
        }
        for member in platform.linked_members() {
            covered.insert(member);
        }
        representatives.push(*platform);
    }
    representatives
}

/// One publisher row, read on the caller's connection.
fn publisher_row(connection: &mut PgConnection, publisher_id: Uuid) -> ThothResult<Publisher> {
    publisher::table
        .filter(publisher::publisher_id.eq(publisher_id))
        .first::<Publisher>(connection)
        .map_err(Into::into)
}

/// The publisher's currently enabled platforms, in canonical order.
///
/// This reuses `BE-02`'s existing set-based assignment statement rather than
/// introducing a second one.
fn enabled_platforms(
    connection: &mut PgConnection,
    publisher_id: Uuid,
) -> ThothResult<Vec<DistributionPlatform>> {
    Ok(enabled_assignment_rows(connection, &[publisher_id])?
        .into_iter()
        .map(|(_, platform, _)| platform)
        .collect())
}

impl PublisherServiceConfiguration {
    /// The superuser staff report: one page of publisher configurations with
    /// their latest change metadata.
    ///
    /// Two set-based statements for a page of N publishers — one for the
    /// filtered, ordered, paginated publisher page and one for the latest change
    /// per publisher in that page — and no per-publisher loop. The protected
    /// `enabledDistributionPlatforms` field resolves separately through `BE-02`'s
    /// existing request-local assignment DataLoader.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn all_summaries(
        db: &PgPool,
        limit: i32,
        offset: i32,
        order: PublisherOrderBy,
        publishers: Vec<Uuid>,
        packages: Vec<ThothPackage>,
        enabled_platforms: Vec<DistributionPlatform>,
        job_statuses: Vec<DistributionJobStatus>,
        without_back_catalogue_job: Option<bool>,
    ) -> ThothResult<Vec<PublisherServiceConfigurationSummary>> {
        let mut connection = db.get()?;
        let query = filtered_publishers(
            publishers,
            packages,
            enabled_platforms,
            job_statuses,
            without_back_catalogue_job,
        );
        // The requested order plus a mandatory `publisher_id ASC` tie-breaker,
        // exactly as `publishersByDistributionPlatform` does, so offset
        // pagination is deterministic.
        let query = match order.field {
            PublisherField::PublisherId => {
                apply_directional_order!(
                    query,
                    order.direction,
                    order,
                    publisher::publisher_id,
                    publisher::publisher_id
                )
            }
            PublisherField::PublisherName => {
                apply_directional_order!(
                    query,
                    order.direction,
                    order,
                    publisher::publisher_name,
                    publisher::publisher_id
                )
            }
            PublisherField::PublisherShortname => {
                apply_directional_order!(
                    query,
                    order.direction,
                    order,
                    publisher::publisher_shortname,
                    publisher::publisher_id
                )
            }
            PublisherField::PublisherUrl => {
                apply_directional_order!(
                    query,
                    order.direction,
                    order,
                    publisher::publisher_url,
                    publisher::publisher_id
                )
            }
            PublisherField::ZitadelId => {
                apply_directional_order!(
                    query,
                    order.direction,
                    order,
                    publisher::zitadel_id,
                    publisher::publisher_id
                )
            }
            PublisherField::AccessibilityStatement => {
                apply_directional_order!(
                    query,
                    order.direction,
                    order,
                    publisher::accessibility_statement,
                    publisher::publisher_id
                )
            }
            PublisherField::AccessibilityReportUrl => {
                apply_directional_order!(
                    query,
                    order.direction,
                    order,
                    publisher::accessibility_report_url,
                    publisher::publisher_id
                )
            }
            PublisherField::CreatedAt => {
                apply_directional_order!(
                    query,
                    order.direction,
                    order,
                    publisher::created_at,
                    publisher::publisher_id
                )
            }
            PublisherField::UpdatedAt => {
                apply_directional_order!(
                    query,
                    order.direction,
                    order,
                    publisher::updated_at,
                    publisher::publisher_id
                )
            }
        };

        let page: Vec<Publisher> = query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Publisher>(&mut connection)?;

        let publisher_ids: Vec<Uuid> = page.iter().map(|row| row.publisher_id).collect();
        let mut changes = latest_changes(&mut connection, &publisher_ids)?;

        Ok(page
            .into_iter()
            .map(|row| PublisherServiceConfigurationSummary {
                last_change: changes.remove(&row.publisher_id),
                configuration: PublisherServiceConfiguration::new(row),
            })
            .collect())
    }

    /// The number of publishers the staff report matches before pagination.
    ///
    /// This applies exactly the same filter predicates as
    /// [`Self::all_summaries`].
    pub(crate) fn count(
        db: &PgPool,
        publishers: Vec<Uuid>,
        packages: Vec<ThothPackage>,
        enabled_platforms: Vec<DistributionPlatform>,
        job_statuses: Vec<DistributionJobStatus>,
        without_back_catalogue_job: Option<bool>,
    ) -> ThothResult<i32> {
        let mut connection = db.get()?;
        // See the `Crud::count` note on the i64 -> i32 conversion.
        filtered_publishers(
            publishers,
            packages,
            enabled_platforms,
            job_statuses,
            without_back_catalogue_job,
        )
        .count()
        .get_result::<i64>(&mut connection)
        .map(|total| total.to_string().parse::<i32>().unwrap())
        .map_err(Into::into)
    }
}

/// The staff report's filter predicates, shared by the list and count queries so
/// they cannot diverge.
///
/// `enabled_platforms` narrows with **AND** semantics: a publisher matches only
/// if it has an enabled assignment for **every** requested platform. The
/// assignment primary key is `(publisher_id, platform)`, so a grouped count of
/// the matching enabled rows is exactly the number of distinct requested
/// platforms the publisher has enabled.
fn filtered_publishers<'a>(
    publishers: Vec<Uuid>,
    packages: Vec<ThothPackage>,
    enabled_platforms: Vec<DistributionPlatform>,
    job_statuses: Vec<DistributionJobStatus>,
    without_back_catalogue_job: Option<bool>,
) -> publisher::BoxedQuery<'a, diesel::pg::Pg> {
    let mut query = publisher::table.into_boxed();
    if !publishers.is_empty() {
        query = query.filter(publisher::publisher_id.eq_any(publishers));
    }
    if !packages.is_empty() {
        query = query.filter(publisher::subscription_package.eq_any(packages));
    }
    // `BE-04`: the latest back-catalogue job's status, with **OR** semantics
    // within the list. That is deliberately the opposite of `enabled_platforms`
    // just below, and it is not an inconsistency: a status is single-valued per
    // job, so AND over two statuses would match nothing.
    //
    // "Latest" is the same total order the report's field and its loader use —
    // `created_at DESC`, then the job id `DESC` — so the selected row is
    // deterministic even for two jobs sharing a `created_at`.
    if !job_statuses.is_empty() {
        let latest_status = distribution_job::table
            .filter(distribution_job::publisher_id.eq(publisher::publisher_id))
            .filter(distribution_job::kind.eq(DistributionJobKind::PublisherBackCatalogue))
            .order((
                distribution_job::created_at.desc(),
                distribution_job::distribution_job_id.desc(),
            ))
            .select(distribution_job::status.nullable())
            .single_value();
        query = query.filter(latest_status.eq_any(job_statuses));
    }
    // `BE-04`: presence or absence of any back-catalogue job at all. Combining
    // `true` here with a non-empty `job_statuses` is a documented contradiction
    // that matches zero publishers; it is deterministic, and it is not an error.
    if let Some(without) = without_back_catalogue_job {
        let has_any_job = distribution_job::table
            .filter(distribution_job::publisher_id.eq(publisher::publisher_id))
            .filter(distribution_job::kind.eq(DistributionJobKind::PublisherBackCatalogue))
            .select(distribution_job::distribution_job_id);
        query = if without {
            query.filter(diesel::dsl::not(diesel::dsl::exists(has_any_job)))
        } else {
            query.filter(diesel::dsl::exists(has_any_job))
        };
    }
    let required: Vec<DistributionPlatform> = deduplicate_platforms(&enabled_platforms);
    if !required.is_empty() {
        let required_count = required.len() as i64;
        query = query.filter(
            publisher::publisher_id.eq_any(
                publisher_distribution_platform::table
                    .filter(publisher_distribution_platform::enabled.eq(true))
                    .filter(publisher_distribution_platform::platform.eq_any(required))
                    .group_by(publisher_distribution_platform::publisher_id)
                    .having(diesel::dsl::count_star().eq(required_count))
                    .select(publisher_distribution_platform::publisher_id),
            ),
        );
    }
    query
}

/// The requested platforms deduplicated, in canonical order, with **no** linked
/// closure: a report filter selects publishers, it does not assert desired
/// state.
fn deduplicate_platforms(requested: &[DistributionPlatform]) -> Vec<DistributionPlatform> {
    let requested: HashSet<DistributionPlatform> = requested.iter().copied().collect();
    DistributionPlatform::ALL
        .into_iter()
        .filter(|platform| requested.contains(platform))
        .collect()
}

/// The latest recorded change for each publisher in the page, in one set-based
/// statement.
///
/// The `DISTINCT ON` order is a **total** order — `publisher_id`, then
/// `created_at DESC`, then the history id `DESC` — so the selected row is
/// deterministic even for two rows sharing a `created_at`. The composite index
/// created by the migration supports exactly this lookup.
fn latest_changes(
    connection: &mut PgConnection,
    publisher_ids: &[Uuid],
) -> ThothResult<HashMap<Uuid, PublisherServiceConfigurationChange>> {
    if publisher_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows: Vec<LatestChangeRow> = publisher_service_configuration_history::table
        .filter(publisher_service_configuration_history::publisher_id.eq_any(publisher_ids))
        .distinct_on(publisher_service_configuration_history::publisher_id)
        .order((
            publisher_service_configuration_history::publisher_id.asc(),
            publisher_service_configuration_history::created_at.desc(),
            publisher_service_configuration_history::publisher_service_configuration_history_id
                .desc(),
        ))
        .select((
            publisher_service_configuration_history::publisher_id,
            publisher_service_configuration_history::created_at,
            publisher_service_configuration_history::actor,
            publisher_service_configuration_history::source,
        ))
        .load::<LatestChangeRow>(connection)?;

    Ok(rows
        .into_iter()
        .map(|(publisher_id, changed_at, actor, source)| {
            (
                publisher_id,
                PublisherServiceConfigurationChange {
                    changed_at,
                    actor,
                    source,
                },
            )
        })
        .collect())
}
