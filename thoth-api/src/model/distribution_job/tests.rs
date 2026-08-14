//! Domain tests for durable distribution jobs (`BE-04`).

use super::*;

#[test]
fn back_catalogue_deduplication_key_matches_the_specified_formula() {
    let publisher_id = Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap();
    let activation_id = Uuid::parse_str("22222222-2222-4222-8222-222222222222").unwrap();

    let key = DistributionJob::back_catalogue_deduplication_key(publisher_id, activation_id);

    assert_eq!(
        key,
        "PUBLISHER_BACK_CATALOGUE:11111111-1111-4111-8111-111111111111:22222222-2222-4222-8222-222222222222"
    );
    // The rendered length is fixed, well inside the 256-character check.
    assert_eq!(key.chars().count(), 98);
}

#[test]
fn distribution_job_creation_parses_exactly_off_and_on() {
    assert_eq!(
        "OFF".parse::<DistributionJobCreation>().unwrap(),
        DistributionJobCreation::Off
    );
    assert_eq!(
        "ON".parse::<DistributionJobCreation>().unwrap(),
        DistributionJobCreation::On
    );

    for invalid in ["off", "on", "On", "TRUE", "1", "", "ENABLED", "OFF "] {
        assert!(
            invalid.parse::<DistributionJobCreation>().is_err(),
            "{invalid} must not parse"
        );
    }
}

#[test]
fn distribution_job_creation_defaults_to_off() {
    assert_eq!(
        DistributionJobCreation::default(),
        DistributionJobCreation::Off
    );
    assert!(!DistributionJobCreation::default().is_on());
    assert!(DistributionJobCreation::On.is_on());
}

#[test]
fn distribution_job_creation_round_trips_through_display() {
    for value in [DistributionJobCreation::Off, DistributionJobCreation::On] {
        assert_eq!(value.to_string().parse::<DistributionJobCreation>().unwrap(), value);
    }
}

#[test]
fn terminal_statuses_are_exactly_the_three_completed_ones() {
    assert!(!DistributionJobStatus::Pending.is_terminal());
    assert!(!DistributionJobStatus::Running.is_terminal());
    assert!(DistributionJobStatus::Succeeded.is_terminal());
    assert!(DistributionJobStatus::Failed.is_terminal());
    assert!(DistributionJobStatus::Cancelled.is_terminal());
}

#[test]
fn code_owned_bounds_are_the_specified_values() {
    assert_eq!(DISTRIBUTION_JOB_MAX_ATTEMPTS, 5);
    assert_eq!(DISTRIBUTION_JOB_LEASE_DEFAULT_SECONDS, 900);
    assert_eq!(DISTRIBUTION_JOB_LEASE_MIN_SECONDS, 60);
    assert_eq!(DISTRIBUTION_JOB_LEASE_MAX_SECONDS, 3600);
    assert_eq!(DISTRIBUTION_JOB_CLAIM_DEFAULT_BATCH, 10);
    assert_eq!(DISTRIBUTION_JOB_CLAIM_MAX_BATCH, 50);
    assert_eq!(DISTRIBUTION_JOB_LEASE_RECOVERY_BATCH, 50);
    assert_eq!(DISTRIBUTION_JOB_RETRY_BASE_SECONDS, 300);
    assert_eq!(DISTRIBUTION_JOB_RETRY_MAX_SECONDS, 21_600);
    assert_eq!(DISTRIBUTION_JOB_ERROR_CODE_MAX_CHARS, 64);
    assert_eq!(DISTRIBUTION_JOB_ERROR_DETAIL_MAX_CHARS, 2048);
}
