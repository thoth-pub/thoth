//! `THOTH-GQL-OPS-03` — the effective-mode record, emitted by a **real
//! process**.
//!
//! The unit tests in `thoth-api` prove the record format and the verifier, and
//! the tests in `thoth-api-server` prove that the reported mode is the same
//! stored value the request path uses. What only this level can show is that an
//! actual `thoth` process, started the way the deployment starts one, emits an
//! effective-mode record carrying the mode it really computed — and that the
//! verifier consumes those real records.
//!
//! ## How the process is run, and why it exits
//!
//! Each process is started on the real `start graphql-api` command path with a
//! deliberately invalid private key. The record is emitted immediately after
//! logging is initialised and **before** the first startup step that can fail,
//! so the process reports its mode and then aborts on the key. If the record
//! were ever moved after a fallible step, these tests would stop finding it.
//!
//! **That ordering also means a record proves nothing about fleet membership.**
//! Every process in this file emits a record and then *fails to start*. A record
//! therefore attests only "some process computed this mode", never "this is a
//! current serving instance". Live orchestrator enumeration remains the sole
//! authority on the population; records are evidence *about members of it*.
//!
//! ## NOT TRUSTED EVIDENCE
//!
//! Independent review 4906399962 (`CHANGES REQUIRED`) established that a public
//! caller can inject record-looking text into this same log stream (finding 1),
//! and that the real per-instance collection and identity-correlation contract
//! is unevidenced (finding 2). These tests show what a real process *emits*;
//! they do not, and cannot, show that a collector may trust it.
//!
//! Nothing here binds a socket, reaches a database, reads a secret, contacts a
//! network service or touches any environment: every process is local,
//! disposable and dead within milliseconds.

use std::process::Command;

use thoth::api::graphql::{
    verify_fleet, EffectiveModeObservation, FleetOutcome, InstanceIdentity, MemberMode,
    MutationGuardMode,
};

/// Start one real `thoth` GraphQL API process in `mode` and return its
/// effective-mode observation, recovered from its own log stream.
fn observe_real_process(mode: &str) -> EffectiveModeObservation {
    observe_real_process_with_declared_intent(mode, mode)
}

/// Start one real process whose **declared configured intent** is
/// `declared_intent` but whose effective mode is `mode`.
///
/// The divergence is constructed deliberately, out of two genuinely different
/// inputs: the environment variable the deployment would set carries the
/// declared intent, and an explicit command-line value — which `clap` gives
/// precedence over the environment — carries what the process actually
/// computes. Where the two are equal there is no divergence, which is the
/// ordinary case above.
///
/// It does **not** depend on, reintroduce or weaken the `init` argument-parsing
/// defect that `THOTH-GQL-OPS-02` closed. That defect made a *configured* value
/// unreadable; this fixture relies on argument parsing working exactly as
/// `THOTH-GQL-OPS-02` left it, and it stays valid however the startup path is
/// later refactored, because both inputs are real and neither is a bug.
fn observe_real_process_with_declared_intent(
    mode: &str,
    declared_intent: &str,
) -> EffectiveModeObservation {
    let output = Command::new(env!("CARGO_BIN_EXE_thoth"))
        .args([
            "start",
            "graphql-api",
            "--mutation-guard-mode",
            mode,
            // Every argument the command path unwraps. None is a real value:
            // the process never connects to anything.
            "--database-url",
            "postgres://unused:unused@127.0.0.1:1/unused",
            "--private-key",
            "this is not base64",
            "--aws-access-key-id",
            "unused",
            "--aws-secret-access-key",
            "unused",
            "--aws-region",
            "unused",
        ])
        // The record is emitted at `info`, which is the server's own default
        // filter. Pinning it here keeps the test independent of whatever the
        // developer's shell happens to export.
        .env("RUST_LOG", "info")
        // What the deployment configuration declares this process should be.
        .env("THOTH_GRAPHQL_MUTATION_GUARD_MODE", declared_intent)
        .output()
        .expect("the thoth binary should be runnable");

    assert!(
        !output.status.success(),
        "this process is expected to abort on the invalid private key"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    let observation = stderr
        .lines()
        .find_map(EffectiveModeObservation::parse_record)
        .unwrap_or_else(|| {
            panic!("no effective-mode record found in the process log stream:\n{stderr}")
        });

    // The record is emitted exactly once per process.
    let records = stderr
        .lines()
        .filter(|line| EffectiveModeObservation::parse_record(line).is_some())
        .count();
    assert_eq!(records, 1, "exactly one record per process");

    observation
}

/// The identity a local process reports, if this platform publishes one.
///
/// A developer platform that publishes no host name makes every local
/// observation unattributable, which is a specified outcome rather than a
/// failure — it is the same path a process that cannot identify itself takes in
/// production, and it resolves to `UNKNOWN`.
fn local_identity(observation: &EffectiveModeObservation) -> Option<InstanceIdentity> {
    observation.instance().cloned()
}

#[test]
fn a_real_process_reports_the_effective_mode_it_actually_computed() {
    for (argument, expected) in [
        ("OFF", MutationGuardMode::Off),
        ("OBSERVE", MutationGuardMode::Observe),
        ("ENFORCE", MutationGuardMode::Enforce),
    ] {
        let observation = observe_real_process(argument);
        assert_eq!(
            observation.mode(),
            expected,
            "a process started in {argument} must report {expected:?}"
        );
        // The record carries the mode and, at most, the correlation identity.
        let record = observation.record();
        let fields: Vec<&str> = record.split_whitespace().skip(1).collect();
        assert_eq!(fields.first(), Some(&format!("mode={argument}").as_str()));
        assert!(
            fields.len() <= 2,
            "a record carries no field beyond the mode and the identity: {record}"
        );
    }
}

#[test]
fn a_single_enumerated_instance_running_one_real_process_is_verifiable() {
    let observation = observe_real_process("OBSERVE");
    let Some(identity) = local_identity(&observation) else {
        // Unattributable on this platform: verification must then establish
        // nothing rather than guess.
        let verification = verify_fleet(
            &[InstanceIdentity::new("enumerated-member").unwrap()],
            &[observation],
        );
        assert_eq!(verification.outcome(), FleetOutcome::NotEstablished);
        assert_eq!(verification.members()[0].mode, MemberMode::Unknown);
        return;
    };

    let verification = verify_fleet(std::slice::from_ref(&identity), &[observation]);
    assert_eq!(
        verification.outcome(),
        FleetOutcome::Consistent(MutationGuardMode::Observe)
    );
    assert!(verification.confirms(MutationGuardMode::Observe));
    assert!(!verification.confirms(MutationGuardMode::Off));
}

#[test]
fn real_records_that_disagree_about_one_enumerated_member_establish_nothing() {
    // Two real processes, started in different modes. Whether this platform
    // gives them an identity or not, one enumerated member cannot be both:
    // either the observations contradict each other, or they are unattributable.
    // Both are fail-closed, and neither may be resolved to a mode.
    let first = observe_real_process("OFF");
    let second = observe_real_process("ENFORCE");
    assert_ne!(first.mode(), second.mode());

    let enumerated = vec![local_identity(&first)
        .unwrap_or_else(|| InstanceIdentity::new("enumerated-member").unwrap())];
    let verification = verify_fleet(&enumerated, &[first, second]);

    assert_eq!(verification.outcome(), FleetOutcome::NotEstablished);
    assert_eq!(verification.members()[0].mode, MemberMode::Unknown);
    for mode in [
        MutationGuardMode::Off,
        MutationGuardMode::Observe,
        MutationGuardMode::Enforce,
    ] {
        assert!(!verification.confirms(mode));
    }
}

#[test]
fn silent_adoption_is_caught_because_the_process_reports_what_it_computed() {
    // The deployment declares OBSERVE. The process actually computes OFF. It
    // starts healthily and serves normally: without this mechanism the
    // divergence has no symptom at all.
    let declared = MutationGuardMode::Observe;
    let observation = observe_real_process_with_declared_intent("OFF", "OBSERVE");

    assert_eq!(
        observation.mode(),
        MutationGuardMode::Off,
        "the record must report the mode the process COMPUTED, not the \
         mode the configuration declared"
    );
    assert_ne!(observation.mode(), declared);

    let enumerated = vec![observation
        .instance()
        .cloned()
        .unwrap_or_else(|| InstanceIdentity::new("enumerated-member").unwrap())];
    let verification = verify_fleet(&enumerated, &[observation]);

    // Whatever this platform can attribute, the declared intent is never
    // confirmed — the divergence is visible rather than silent.
    assert!(
        !verification.confirms(declared),
        "a declared OBSERVE must never be confirmed by a process running OFF"
    );
    match verification.members()[0].mode {
        MemberMode::Established(effective) => {
            assert_eq!(effective, MutationGuardMode::Off);
            assert_eq!(
                verification
                    .divergences_from_declared_intent(declared)
                    .len(),
                1,
                "the intent/effective divergence must be reported"
            );
        }
        // Unattributable on this platform: nothing is established, which also
        // refuses to confirm the declared intent.
        MemberMode::Unknown => {
            assert_eq!(verification.outcome(), FleetOutcome::NotEstablished);
        }
    }
}

#[test]
fn a_real_process_record_is_reproducible_across_restarts() {
    // The mode is a fixed property of a process, so restarting the same
    // configuration reports the same mode. This is what lets an operator
    // re-read an instance during a rollout and compare answers.
    let first = observe_real_process("ENFORCE");
    let second = observe_real_process("ENFORCE");
    assert_eq!(first.mode(), second.mode());
    assert_eq!(first.instance(), second.instance());
    assert_eq!(first.record(), second.record());
}
