//! `THOTH-GQL-OPS-03` unit tests for the effective-mode observation and the
//! fleet verifier.
//!
//! These tests never start a server and never touch a database: the producer is
//! a pure function of the stored mode plus this process's own identity, and the
//! verifier is a pure function of an enumeration and a set of observations. The
//! server-level proof that the reported mode is the **same stored value** the
//! request path uses lives with the running application, in `thoth-api-server`.

use super::*;

/// Build an observation as if it had been collected from `instance`, by going
/// through the real record format and the real parser. Nothing here can invent
/// an observation shape a serving process could not have emitted.
fn observation(instance: &str, mode: MutationGuardMode) -> EffectiveModeObservation {
    let record = format!(
        "{EFFECTIVE_MODE_RECORD_TAG} mode={} instance={instance}",
        mode_token(mode)
    );
    EffectiveModeObservation::parse_record(&record).expect("a well-formed record must parse")
}

fn enumerate(instances: &[&str]) -> Vec<InstanceIdentity> {
    instances
        .iter()
        .map(|instance| InstanceIdentity::new(instance).expect("a usable test identity"))
        .collect()
}

const ALL_MODES: [MutationGuardMode; 3] = [
    MutationGuardMode::Off,
    MutationGuardMode::Observe,
    MutationGuardMode::Enforce,
];

// --- the observation a serving process produces ----------------------------

#[test]
fn the_reported_mode_is_the_mode_the_observation_was_built_from() {
    for mode in ALL_MODES {
        assert_eq!(
            EffectiveModeObservation::for_process(mode).mode(),
            mode,
            "the observation must report the effective mode it was given, \
             not a re-derivation of it"
        );
    }
}

#[test]
fn the_observation_carries_no_second_independently_settable_mode() {
    // `for_process` takes the effective mode as its only mode input and the
    // struct exposes no setter, so the reported mode moves if and only if the
    // stored mode moves. Three distinct inputs give three distinct reports.
    let reported: Vec<MutationGuardMode> = ALL_MODES
        .iter()
        .map(|mode| EffectiveModeObservation::for_process(*mode).mode())
        .collect();
    assert_eq!(reported, ALL_MODES.to_vec());
}

#[test]
fn the_runtime_identity_is_stable_for_the_life_of_the_process() {
    let first = EffectiveModeObservation::for_process(MutationGuardMode::Off);
    let second = EffectiveModeObservation::for_process(MutationGuardMode::Enforce);
    assert_eq!(
        first.instance(),
        second.instance(),
        "repeated observation of one process must yield one identity"
    );
    assert_eq!(process_instance_identity(), process_instance_identity());
    // Whatever this platform reports, an identity that exists is usable and an
    // absent one is absent — never an empty or placeholder value.
    if let Some(identity) = process_instance_identity() {
        assert!(!identity.as_str().is_empty());
        assert_eq!(
            InstanceIdentity::new(identity.as_str()).as_ref(),
            Some(identity)
        );
    }
}

/// Write `contents` to a uniquely named temporary file and return its path.
fn hostname_source(label: &str, contents: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "thoth-ops-03-{}-{}-{label}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::write(&path, contents).expect("temporary host-name source");
    path
}

#[test]
fn the_identity_is_read_from_the_first_source_that_publishes_a_usable_host_name() {
    let container_style = hostname_source("hostname", "thoth-api-7f4c9\n");
    let unusable = hostname_source("unusable", "not a host name\n");
    let absent = std::env::temp_dir().join("thoth-ops-03-definitely-absent-source");
    let container_style_path = container_style.to_str().expect("utf-8 path");
    let unusable_path = unusable.to_str().expect("utf-8 path");
    let absent_path = absent.to_str().expect("utf-8 path");

    // A missing source falls through to the next one, and the value is trimmed.
    assert_eq!(
        identity_from_sources(&[absent_path, container_style_path])
            .as_ref()
            .map(InstanceIdentity::as_str),
        Some("thoth-api-7f4c9")
    );
    // A source whose contents are not a usable identity is refused, not
    // salvaged, and the search continues.
    assert_eq!(
        identity_from_sources(&[unusable_path, container_style_path])
            .as_ref()
            .map(InstanceIdentity::as_str),
        Some("thoth-api-7f4c9")
    );
    // No usable source at all means no identity — never a placeholder.
    assert_eq!(identity_from_sources(&[absent_path, unusable_path]), None);
    assert_eq!(identity_from_sources(&[]), None);

    for path in [container_style, unusable] {
        let _ = std::fs::remove_file(path);
    }
}

#[test]
#[cfg(target_os = "linux")]
fn a_linux_process_can_always_identify_itself() {
    // The service runs in Linux containers, where the host name is the
    // orchestrator-assigned instance name. An observation from such a process
    // must be attributable; only a platform publishing no host name at all may
    // produce an unattributable one.
    let identity = process_instance_identity().expect(
        "a Linux process must resolve a runtime identity, \
         otherwise every production observation would be UNKNOWN",
    );
    assert!(!identity.as_str().is_empty());
    assert!(
        EffectiveModeObservation::for_process(MutationGuardMode::Off)
            .instance()
            .is_some()
    );
}

#[test]
fn a_record_carries_only_the_effective_mode_and_the_correlation_identity() {
    // Field-by-field minimum disclosure (`THOTH-GQL-OPS-03` AC-23): the exact
    // rendered record, with nothing else in it.
    let record = observation("thoth-api-7f4c9", MutationGuardMode::Observe).record();
    assert_eq!(
        record,
        "THOTH_MUTATION_GUARD_EFFECTIVE_MODE mode=OBSERVE instance=thoth-api-7f4c9"
    );
    let fields: Vec<&str> = record.split_whitespace().skip(1).collect();
    assert_eq!(fields, ["mode=OBSERVE", "instance=thoth-api-7f4c9"]);
}

#[test]
fn a_record_round_trips_through_the_command_paths_own_mode_parser() {
    for mode in ALL_MODES {
        let emitted = observation("thoth-api-1", mode);
        let recovered = EffectiveModeObservation::parse_record(&emitted.record())
            .expect("an emitted record must parse");
        assert_eq!(recovered, emitted);
        assert_eq!(recovered.mode(), mode);
        // The token is the same string the CLI accepts, parsed by the same
        // `FromStr` the command path uses.
        assert_eq!(
            mode_token(mode).parse::<MutationGuardMode>(),
            Ok(mode),
            "the record token must be the value the command path accepts"
        );
    }
}

#[test]
fn a_record_is_recoverable_from_an_ordinary_prefixed_log_line() {
    let line = "[2026-08-11T09:00:00Z INFO  thoth_api_server] \
                THOTH_MUTATION_GUARD_EFFECTIVE_MODE mode=ENFORCE instance=thoth-api-9b2";
    let recovered = EffectiveModeObservation::parse_record(line).expect("must parse");
    assert_eq!(recovered.mode(), MutationGuardMode::Enforce);
    assert_eq!(
        recovered.instance().map(InstanceIdentity::as_str),
        Some("thoth-api-9b2")
    );
}

#[test]
fn a_process_that_cannot_identify_itself_emits_an_unattributable_record() {
    let record = "THOTH_MUTATION_GUARD_EFFECTIVE_MODE mode=OFF";
    let recovered = EffectiveModeObservation::parse_record(record).expect("must parse");
    assert_eq!(recovered.mode(), MutationGuardMode::Off);
    assert_eq!(recovered.instance(), None);

    // An unattributable observation cannot answer for any enumerated member,
    // so the member stays UNKNOWN and nothing is established.
    let verification = verify_fleet(&enumerate(&["thoth-api-1"]), &[recovered]);
    assert_eq!(verification.members()[0].mode, MemberMode::Unknown);
    assert_eq!(verification.outcome(), FleetOutcome::NotEstablished);
    assert_eq!(verification.unattributed_observations(), 1);
}

#[test]
fn a_malformed_or_forged_record_becomes_no_observation_rather_than_a_wrong_one() {
    for line in [
        // no tag at all
        "mode=OFF instance=thoth-api-1",
        // no mode
        "THOTH_MUTATION_GUARD_EFFECTIVE_MODE instance=thoth-api-1",
        // unparsable mode
        "THOTH_MUTATION_GUARD_EFFECTIVE_MODE mode=SOMETHING_ELSE",
        // a key this module never emits
        "THOTH_MUTATION_GUARD_EFFECTIVE_MODE mode=OFF database_url=postgres://x",
        // a repeated key
        "THOTH_MUTATION_GUARD_EFFECTIVE_MODE mode=OFF mode=ENFORCE",
        // an identity that is not a single host-name-like token
        "THOTH_MUTATION_GUARD_EFFECTIVE_MODE mode=OFF instance=not/an/identity",
        // not a key=value field
        "THOTH_MUTATION_GUARD_EFFECTIVE_MODE mode=OFF trailing",
    ] {
        assert_eq!(
            EffectiveModeObservation::parse_record(line),
            None,
            "`{line}` must not produce an observation"
        );
    }
}

#[test]
fn an_identity_cannot_be_made_to_carry_a_payload() {
    for rejected in [
        "",
        " ",
        "two tokens",
        "key=value",
        "postgres://thoth:thoth@localhost/thoth",
        "a\nb",
        &"x".repeat(254),
    ] {
        assert_eq!(
            InstanceIdentity::new(rejected),
            None,
            "`{rejected}` must not be accepted as a runtime identity"
        );
    }
    for accepted in ["thoth-api-7f4c9", "ip-10-0-1-23.eu-west-1.compute.internal"] {
        assert!(InstanceIdentity::new(accepted).is_some());
    }
}

// --- fleet verification -----------------------------------------------------

#[test]
fn a_fully_covered_population_agreeing_on_one_mode_is_consistent() {
    let enumerated = enumerate(&["a", "b", "c"]);
    let observations = [
        observation("a", MutationGuardMode::Off),
        observation("b", MutationGuardMode::Off),
        observation("c", MutationGuardMode::Off),
    ];
    let verification = verify_fleet(&enumerated, &observations);
    assert_eq!(
        verification.outcome(),
        FleetOutcome::Consistent(MutationGuardMode::Off)
    );
    assert!(verification.confirms(MutationGuardMode::Off));
    assert!(!verification.confirms(MutationGuardMode::Observe));
    assert_eq!(verification.members().len(), 3);
    assert_eq!(verification.unattributed_observations(), 0);
}

#[test]
fn a_fully_covered_population_disagreeing_is_mixed() {
    let enumerated = enumerate(&["a", "b", "c"]);
    let observations = [
        observation("a", MutationGuardMode::Off),
        observation("b", MutationGuardMode::Observe),
        observation("c", MutationGuardMode::Off),
    ];
    let verification = verify_fleet(&enumerated, &observations);
    assert_eq!(verification.outcome(), FleetOutcome::Mixed);
    // A mixed fleet confirms nothing, in either direction.
    for mode in ALL_MODES {
        assert!(!verification.confirms(mode));
    }
}

#[test]
fn an_uncovered_member_is_unknown_and_the_verification_is_not_established() {
    let enumerated = enumerate(&["a", "b", "c"]);
    let observations = [
        observation("a", MutationGuardMode::Off),
        observation("c", MutationGuardMode::Off),
    ];
    let verification = verify_fleet(&enumerated, &observations);
    assert_eq!(verification.outcome(), FleetOutcome::NotEstablished);
    assert_eq!(verification.members()[1].instance.as_str(), "b");
    assert_eq!(verification.members()[1].mode, MemberMode::Unknown);
    for mode in ALL_MODES {
        assert!(
            !verification.confirms(mode),
            "incomplete coverage must never pass"
        );
    }
}

#[test]
fn unknown_is_structurally_distinct_from_off_and_never_decays_into_it() {
    assert_ne!(
        MemberMode::Unknown,
        MemberMode::Established(MutationGuardMode::Off)
    );
    // And in a real result: the uncovered member is `Unknown`, not `Off`.
    let verification = verify_fleet(
        &enumerate(&["a", "b"]),
        &[observation("a", MutationGuardMode::Off)],
    );
    let uncovered = &verification.members()[1];
    assert_eq!(uncovered.mode, MemberMode::Unknown);
    assert_ne!(
        uncovered.mode,
        MemberMode::Established(MutationGuardMode::Off)
    );
}

#[test]
fn a_pre_guard_instance_is_reported_unknown_rather_than_off() {
    // A pre-guard release contains no mutation guard, so it has no mode at all
    // and emits no record. Its absence must not be read as `OFF`
    // (`THOTH-GQL-OPS-03` section 5 invariant 12).
    let verification = verify_fleet(
        &enumerate(&["guard-enabled", "pre-guard"]),
        &[observation("guard-enabled", MutationGuardMode::Off)],
    );
    assert_eq!(verification.members()[1].mode, MemberMode::Unknown);
    assert_eq!(verification.outcome(), FleetOutcome::NotEstablished);
}

#[test]
fn partial_agreement_is_not_fleet_consistency() {
    // "2 of 3 instances agree" must not pass.
    let verification = verify_fleet(
        &enumerate(&["a", "b", "c"]),
        &[
            observation("a", MutationGuardMode::Observe),
            observation("b", MutationGuardMode::Observe),
        ],
    );
    assert_eq!(verification.outcome(), FleetOutcome::NotEstablished);
    assert!(!verification.confirms(MutationGuardMode::Observe));
}

#[test]
fn evidence_the_enumeration_does_not_account_for_establishes_nothing() {
    // An instance outside the enumeration means the enumeration is stale or
    // incomplete, so the population cannot be pronounced consistent.
    let verification = verify_fleet(
        &enumerate(&["a", "b"]),
        &[
            observation("a", MutationGuardMode::Off),
            observation("b", MutationGuardMode::Off),
            observation("unenumerated", MutationGuardMode::Off),
        ],
    );
    assert_eq!(verification.outcome(), FleetOutcome::NotEstablished);
    assert_eq!(verification.unattributed_observations(), 1);
}

#[test]
fn an_ambiguous_or_empty_enumeration_establishes_nothing() {
    let repeated = verify_fleet(
        &enumerate(&["a", "a"]),
        &[observation("a", MutationGuardMode::Off)],
    );
    assert_eq!(repeated.outcome(), FleetOutcome::NotEstablished);

    let empty = verify_fleet(&[], &[]);
    assert_eq!(empty.outcome(), FleetOutcome::NotEstablished);
    assert!(empty.members().is_empty());
    for mode in ALL_MODES {
        assert!(!empty.confirms(mode));
    }
}

#[test]
fn contradictory_observations_for_one_member_leave_it_unknown() {
    let verification = verify_fleet(
        &enumerate(&["a"]),
        &[
            observation("a", MutationGuardMode::Off),
            observation("a", MutationGuardMode::Enforce),
        ],
    );
    assert_eq!(verification.members()[0].mode, MemberMode::Unknown);
    assert_eq!(verification.outcome(), FleetOutcome::NotEstablished);

    // Repeated but agreeing observations of one member are not a contradiction.
    let agreeing = verify_fleet(
        &enumerate(&["a"]),
        &[
            observation("a", MutationGuardMode::Enforce),
            observation("a", MutationGuardMode::Enforce),
        ],
    );
    assert_eq!(
        agreeing.outcome(),
        FleetOutcome::Consistent(MutationGuardMode::Enforce)
    );
}

#[test]
fn silent_adoption_is_visible_as_a_divergence_from_declared_intent() {
    // Declared intent OBSERVE; every process actually computed OFF. The
    // verification is internally consistent — and wrong against intent, which
    // is exactly the failure class that is otherwise invisible.
    let enumerated = enumerate(&["a", "b"]);
    let observations = [
        observation("a", MutationGuardMode::Off),
        observation("b", MutationGuardMode::Off),
    ];
    let verification = verify_fleet(&enumerated, &observations);

    assert_eq!(
        verification.outcome(),
        FleetOutcome::Consistent(MutationGuardMode::Off),
        "the effective value wins: the report is what the processes compute"
    );
    assert!(
        !verification.confirms(MutationGuardMode::Observe),
        "a declared OBSERVE must not be confirmed by an OFF fleet"
    );
    let divergences = verification.divergences_from_declared_intent(MutationGuardMode::Observe);
    assert_eq!(divergences.len(), 2);
    assert!(divergences
        .iter()
        .all(|member| member.mode == MemberMode::Established(MutationGuardMode::Off)));

    // No divergence is reported where intent and effective mode agree.
    assert!(verification
        .divergences_from_declared_intent(MutationGuardMode::Off)
        .is_empty());
}

#[test]
fn an_unknown_member_is_an_absence_of_evidence_not_a_divergence() {
    let verification = verify_fleet(
        &enumerate(&["a", "b"]),
        &[observation("a", MutationGuardMode::Observe)],
    );
    assert!(verification
        .divergences_from_declared_intent(MutationGuardMode::Observe)
        .is_empty());
    // ...but the missing member still fails the verification outright.
    assert_eq!(verification.outcome(), FleetOutcome::NotEstablished);
}

#[test]
fn verification_is_repeatable_and_order_independent_in_its_conclusion() {
    let enumerated = enumerate(&["a", "b", "c"]);
    let observations = [
        observation("c", MutationGuardMode::Enforce),
        observation("a", MutationGuardMode::Enforce),
        observation("b", MutationGuardMode::Enforce),
    ];
    let first = verify_fleet(&enumerated, &observations);
    let second = verify_fleet(&enumerated, &observations);
    assert_eq!(first, second, "verification must be a pure function");
    assert_eq!(
        first.outcome(),
        FleetOutcome::Consistent(MutationGuardMode::Enforce)
    );
    // Members are reported in enumeration order regardless of collection order.
    let reported: Vec<&str> = first
        .members()
        .iter()
        .map(|member| member.instance.as_str())
        .collect();
    assert_eq!(reported, ["a", "b", "c"]);
}

#[test]
fn store_availability_follows_from_the_reported_mode_alone() {
    // Runbook section 4.4: confirming an instance's effective mode is itself
    // the confirmation of its store availability, because availability is
    // derived from the mode and from nothing else.
    for (mode, available) in [
        (MutationGuardMode::Off, false),
        (MutationGuardMode::Observe, false),
        (MutationGuardMode::Enforce, true),
    ] {
        let verification = verify_fleet(&enumerate(&["a"]), &[observation("a", mode)]);
        let MemberMode::Established(reported) = verification.members()[0].mode else {
            panic!("the member's mode must be established");
        };
        assert_eq!(reported.store_available(), available);
    }
}
