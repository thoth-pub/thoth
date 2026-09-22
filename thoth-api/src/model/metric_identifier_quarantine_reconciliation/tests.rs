//! Focused unit evidence for the closed reconciliation state partition.
//! Database/concurrency/canonical-application acceptance coverage is added
//! alongside the coordinator fixtures in this task before source review.

use super::{AttemptBucket, MetricIdentifierQuarantineReconciliationState as State};

#[test]
fn closed_state_partition_is_exhaustive_and_stable() {
    let cases = [
        (State::PendingUnknownDoi, false, AttemptBucket::Pending),
        (State::BlockedAmbiguousDoi, false, AttemptBucket::Blocked),
        (State::BlockedPublisherScopeMismatch, false, AttemptBucket::Blocked),
        (State::BlockedSourceConflict, false, AttemptBucket::Blocked),
        (State::BlockedOverlappingPeriod, false, AttemptBucket::Blocked),
        (State::BlockedSameImportOrder, false, AttemptBucket::Blocked),
        (State::BlockedImportOrderAmbiguous, false, AttemptBucket::Blocked),
        (State::BlockedDeltaOverflow, false, AttemptBucket::Blocked),
        (State::BlockedInconsistentEvidence, false, AttemptBucket::Blocked),
        (State::ResolvedWinner, true, AttemptBucket::Resolved),
        (State::ResolvedDuplicate, true, AttemptBucket::Resolved),
        (State::ResolvedRevision, true, AttemptBucket::Resolved),
        (State::ResolvedSuperseded, true, AttemptBucket::Resolved),
    ];

    for (state, terminal, bucket) in cases {
        assert_eq!(state.is_terminal(), terminal);
        assert_eq!(state.bucket(), bucket);
        assert_eq!(state.to_string().parse::<State>(), Ok(state));
    }
}
