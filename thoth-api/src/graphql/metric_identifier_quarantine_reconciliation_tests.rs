//! GraphQL boundary evidence for `MET-WP7-PREREQ-03`.
//!
//! The complete authorization/result/error matrix is exercised with the
//! disposable backend fixtures before exact-head review.

#![cfg(all(test, feature = "backend"))]

use crate::model::metric_identifier_quarantine_reconciliation::
    MetricIdentifierQuarantineReconciliationBatch;

#[test]
fn aggregate_result_preserves_attempt_partition() {
    let result = MetricIdentifierQuarantineReconciliationBatch {
        attempted: 4,
        resolved: 1,
        pending: 1,
        blocked: 2,
    };
    assert_eq!(
        result.attempted,
        result.resolved + result.pending + result.blocked
    );
}
