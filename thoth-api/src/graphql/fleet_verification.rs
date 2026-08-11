//! Effective-mode fleet verification (`THOTH-GQL-OPS-03`).
//!
//! # What this module is for
//!
//! [`MutationGuardMode`] is read **once at process start** and there is no
//! reload path, so a process's effective mode is a fixed property of that
//! process rather than a queryable setting. Configured intent is therefore not
//! proof of process-effective mode: where the two diverge they diverge with no
//! symptom, which is the **silent-adoption failure class**.
//!
//! This module supplies the two halves needed to catch that class fleet-wide:
//!
//! ```text
//! producer   EffectiveModeObservation::for_process(mode)
//!            -> one record line, emitted once at startup on the process's
//!               OWN log stream
//!
//! verifier   verify_fleet(enumerated, observations)
//!            -> CONSISTENT / MIXED / NOT ESTABLISHED over a population the
//!               ORCHESTRATOR enumerated
//! ```
//!
//! # Disclosure boundary (`THOTH-GQL-OPS-03` section 3.2)
//!
//! The observation is carried **out of band**, on the serving process's own log
//! stream, which the orchestration plane already collects per instance. It
//! never crosses the public listener: this module adds **no** HTTP route, no
//! GraphQL field, no header and no new authorization decision, so no public
//! unauthenticated caller can reach it at all.
//!
//! A record carries exactly two things, and the types make a third impossible:
//!
//! ```text
//! mode       the process's ACTUAL effective MutationGuardMode
//! instance   the minimum runtime identity needed to correlate one
//!            observation with one orchestrator-enumerated instance
//! ```
//!
//! No secret, credential, environment-variable value, deployment
//! configuration, request data, GraphQL document, variable, mutation argument,
//! publisher or user datum, or topology/infrastructure metadata beyond that
//! minimum can be carried, because [`EffectiveModeObservation`] has no field
//! able to hold one — the same structural argument the guard's own
//! `GuardEvent` makes.
//!
//! # One source of truth
//!
//! `for_process` takes the effective mode as its only mode input, the struct
//! has no setter and no interior mutability, and the serving process builds its
//! observation from the **same stored value** it installs on the request path.
//! The reported mode and the request-path mode are therefore the same value by
//! construction and cannot be set independently
//! (`THOTH-GQL-OPS-03` section 5 invariant 6).
//!
//! # Fail closed
//!
//! Verification never reports consistency it did not observe. An enumerated
//! member with no attributable observation is [`MemberMode::Unknown`] — a
//! distinct variant from `MemberMode::Established(MutationGuardMode::Off)`, so
//! `UNKNOWN` can never decay into `OFF` — and any incomplete or ambiguous
//! coverage yields [`FleetOutcome::NotEstablished`] rather than a partial
//! success. That is also what makes a **pre-guard** release safe: it emits no
//! record, so it is reported `UNKNOWN` and is never described as `OFF`
//! (`THOTH-GQL-OPS-03` section 5 invariant 12).

use std::sync::OnceLock;

use super::MutationGuardMode;

/// The leading token of an effective-mode record.
///
/// Deliberately distinctive so a collector can locate the record inside an
/// ordinary log line without parsing the logger's own prefix.
pub const EFFECTIVE_MODE_RECORD_TAG: &str = "THOTH_MUTATION_GUARD_EFFECTIVE_MODE";

/// The canonical token for a mode.
///
/// Written as an exhaustive match rather than a catch-all so that adding a mode
/// forces this decision to be made again. Its inverse is the **existing**
/// [`std::str::FromStr`] implementation the command path already uses, so a
/// record round-trips through the same parser production consumes.
fn mode_token(mode: MutationGuardMode) -> &'static str {
    match mode {
        MutationGuardMode::Off => "OFF",
        MutationGuardMode::Observe => "OBSERVE",
        MutationGuardMode::Enforce => "ENFORCE",
    }
}

/// The minimum runtime identity that correlates one observation with one
/// orchestrator-enumerated serving instance.
///
/// This is the OS-reported host name of the serving process. It is the
/// orchestrator-assigned instance name in the deployment shapes this service
/// can run under — a container host name is the task/pod identity — so it is
/// the field that maps an observation onto an enumerated member. Without it an
/// observation is anonymous and cannot be attributed.
///
/// The accepted shape is restricted to a single host-name-like token. That is a
/// **disclosure control**, not cosmetics: a value containing whitespace or `=`
/// is rejected, so the type structurally cannot carry a sentence, a
/// configuration fragment or a multi-field payload, and cannot forge extra
/// fields into a record.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InstanceIdentity(String);

impl InstanceIdentity {
    /// The longest permitted identity, matching the maximum length of a DNS
    /// name. Anything longer is not a host name and is refused rather than
    /// truncated.
    const MAX_LEN: usize = 253;

    /// Build an identity, or [`None`] if `value` is not a single host-name-like
    /// token.
    pub fn new(value: &str) -> Option<Self> {
        if value.is_empty() || value.len() > Self::MAX_LEN {
            return None;
        }
        let acceptable = value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_'));
        acceptable.then(|| Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for InstanceIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// This process's runtime identity, resolved exactly once.
///
/// Resolution is a read-only look-up of the host name the operating system
/// reports to the process, cached in a [`OnceLock`], so repeated observation of
/// the same process yields the same answer for the life of that process
/// (`THOTH-GQL-OPS-03` section 6.4).
///
/// It is deliberately **not** read from an environment variable: an
/// environment-variable value is outside the section 3.2 disclosure boundary,
/// and the process's own identity must not depend on deployment configuration
/// choosing to supply it.
///
/// Returns [`None`] where the host name cannot be determined — on a platform
/// that publishes neither source, or where the value is not a usable identity.
/// A process that cannot identify itself emits an **unattributable** record,
/// which verification then reports as `UNKNOWN`. It is never defaulted.
pub fn process_instance_identity() -> Option<&'static InstanceIdentity> {
    static IDENTITY: OnceLock<Option<InstanceIdentity>> = OnceLock::new();
    IDENTITY.get_or_init(resolve_instance_identity).as_ref()
}

/// Where the operating system publishes this process's own host name.
///
/// Both are the container's own host name under every Linux container runtime,
/// which is where this service runs. Neither is secret-bearing, and neither is
/// deployment configuration. A platform publishing neither yields no identity.
const HOSTNAME_SOURCES: [&str; 2] = ["/proc/sys/kernel/hostname", "/etc/hostname"];

fn resolve_instance_identity() -> Option<InstanceIdentity> {
    identity_from_sources(&HOSTNAME_SOURCES)
}

/// Read the host name from the first source that yields a usable identity.
///
/// Split from [`resolve_instance_identity`] only so the resolution can be
/// tested against a known host-name file on any platform, rather than only
/// where the real sources exist.
fn identity_from_sources(paths: &[&str]) -> Option<InstanceIdentity> {
    paths
        .iter()
        .filter_map(|path| std::fs::read_to_string(path).ok())
        .find_map(|contents| InstanceIdentity::new(contents.trim()))
}

/// One serving process's report of its own effective mutation-guard mode.
///
/// Construct it for a live process with [`EffectiveModeObservation::for_process`],
/// or recover one a process already emitted with
/// [`EffectiveModeObservation::parse_record`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectiveModeObservation {
    mode: MutationGuardMode,
    instance: Option<InstanceIdentity>,
}

impl EffectiveModeObservation {
    /// Observe **this** process, given the single stored effective mode.
    ///
    /// `mode` must be the very value the request path holds. It is the only
    /// mode input this type has: there is no setter, no default and no second
    /// signal, so a caller cannot report a mode the process is not using
    /// without passing a different value here — which is why the serving
    /// process derives both from one stored value.
    pub fn for_process(mode: MutationGuardMode) -> Self {
        Self {
            mode,
            instance: process_instance_identity().cloned(),
        }
    }

    /// The process's actual effective mode.
    pub fn mode(&self) -> MutationGuardMode {
        self.mode
    }

    /// The reporting instance, or [`None`] when the observation is
    /// unattributable.
    pub fn instance(&self) -> Option<&InstanceIdentity> {
        self.instance.as_ref()
    }

    /// The single out-of-band record line.
    ///
    /// Emitted once per process at startup on the process's own log stream. It
    /// contains the tag, the mode and — when the process could identify itself
    /// — the instance, and nothing else.
    pub fn record(&self) -> String {
        let mut record = format!("{EFFECTIVE_MODE_RECORD_TAG} mode={}", mode_token(self.mode));
        if let Some(instance) = &self.instance {
            record.push_str(" instance=");
            record.push_str(instance.as_str());
        }
        record
    }

    /// Recover an observation from a collected log line.
    ///
    /// The line may carry the logger's own prefix; parsing starts at the tag.
    /// Parsing is strict — an unknown key, a repeated key, an unparsable mode,
    /// an unusable identity or a missing mode all yield [`None`] — so a
    /// malformed or forged line becomes *no observation* rather than a wrong
    /// one, and the member it would have covered stays `UNKNOWN`.
    pub fn parse_record(line: &str) -> Option<Self> {
        let start = line.find(EFFECTIVE_MODE_RECORD_TAG)?;
        let fields = &line[start + EFFECTIVE_MODE_RECORD_TAG.len()..];

        let mut mode: Option<MutationGuardMode> = None;
        let mut instance: Option<InstanceIdentity> = None;
        for field in fields.split_whitespace() {
            let (key, value) = field.split_once('=')?;
            match key {
                "mode" if mode.is_none() => mode = value.parse::<MutationGuardMode>().ok(),
                "instance" if instance.is_none() => instance = Some(InstanceIdentity::new(value)?),
                // An unknown or repeated key means this is not a record this
                // module emitted. Refuse it rather than guess.
                _ => return None,
            }
        }

        Some(Self {
            mode: mode?,
            instance,
        })
    }
}

/// What verification established about one enumerated fleet member.
///
/// `Unknown` is a **variant**, not a sentinel mode, so `UNKNOWN` and `OFF` are
/// distinct in the result shape itself and no code path can coerce one into the
/// other (`THOTH-GQL-OPS-03` AC-7).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemberMode {
    /// The member's process-effective mode was established from an attributable
    /// observation.
    Established(MutationGuardMode),
    /// No attributable observation, or contradictory observations. This is not
    /// a mode and must never be read as one.
    Unknown,
}

/// One enumerated member's verification result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemberVerification {
    pub instance: InstanceIdentity,
    pub mode: MemberMode,
}

/// The verification result for a whole enumerated population.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FleetOutcome {
    /// Every enumerated member established the **same** mode.
    Consistent(MutationGuardMode),
    /// Every enumerated member established a mode, and they are not all equal.
    Mixed,
    /// Nothing is established. Coverage was incomplete, the enumeration was
    /// ambiguous, or evidence arrived that the enumeration does not account
    /// for. This is a failure, never a partial success.
    NotEstablished,
}

/// A completed fleet verification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FleetVerification {
    outcome: FleetOutcome,
    members: Vec<MemberVerification>,
    unattributed_observations: usize,
}

impl FleetVerification {
    pub fn outcome(&self) -> FleetOutcome {
        self.outcome
    }

    /// Every enumerated member, in enumeration order.
    pub fn members(&self) -> &[MemberVerification] {
        &self.members
    }

    /// Observations that no enumerated member accounts for — an unattributable
    /// record, or one from an instance outside the enumeration. Any of these
    /// means the enumeration and the evidence disagree, so nothing is
    /// established.
    pub fn unattributed_observations(&self) -> usize {
        self.unattributed_observations
    }

    /// Whether this verification proves the whole enumerated population is
    /// running `intended`.
    ///
    /// The only outcome that can answer `true` is
    /// [`FleetOutcome::Consistent`], which by construction required complete
    /// coverage. `MIXED` and `NOT ESTABLISHED` both answer `false`, so no
    /// caller can read "most instances agree" as success.
    pub fn confirms(&self, intended: MutationGuardMode) -> bool {
        matches!(self.outcome, FleetOutcome::Consistent(mode) if mode == intended)
    }

    /// Members whose **established** effective mode differs from a declared
    /// configured intent.
    ///
    /// This is the silent-adoption check. The intent is supplied by the
    /// operator from the change they believe they made; the effective mode
    /// comes from what the process actually computed. Nothing here re-reads
    /// configuration, so the two can be compared rather than confused.
    ///
    /// An `Unknown` member is not a divergence — it is an absence of evidence,
    /// and it has already forced the outcome to `NOT ESTABLISHED`.
    pub fn divergences_from_declared_intent(
        &self,
        declared: MutationGuardMode,
    ) -> Vec<&MemberVerification> {
        self.members
            .iter()
            .filter(
                |member| matches!(member.mode, MemberMode::Established(mode) if mode != declared),
            )
            .collect()
    }
}

/// Verify an enumerated serving population against the observations collected
/// from it.
///
/// `enumerated` is the running instance set read from **live orchestrator
/// state**. Passing it is what makes complete coverage a requirement rather
/// than an aspiration: every member is answered for, and a member with no
/// attributable observation is `UNKNOWN`, which forces
/// [`FleetOutcome::NotEstablished`]. Sampling traffic through the shared load
/// balancer cannot produce this input, and so cannot satisfy the task
/// (`THOTH-GQL-OPS-03` AC-4).
///
/// Nothing is established when:
///
/// - the enumeration is empty — there is no population to be consistent about;
/// - the enumeration repeats an identity, so an observation cannot be
///   attributed to one member;
/// - any member has no attributable observation, or has observations that
///   disagree;
/// - any observation is unattributable or falls outside the enumeration, which
///   means the enumeration is stale or incomplete.
pub fn verify_fleet(
    enumerated: &[InstanceIdentity],
    observations: &[EffectiveModeObservation],
) -> FleetVerification {
    // Enumeration order is preserved for the operator; a repeat makes the
    // enumeration ambiguous.
    let mut unique: Vec<&InstanceIdentity> = Vec::with_capacity(enumerated.len());
    for instance in enumerated {
        if !unique.contains(&instance) {
            unique.push(instance);
        }
    }
    let ambiguous_enumeration = unique.len() != enumerated.len();

    let unattributed_observations = observations
        .iter()
        .filter(|observation| match observation.instance() {
            Some(instance) => !unique.contains(&instance),
            None => true,
        })
        .count();

    let members: Vec<MemberVerification> = unique
        .into_iter()
        .map(|instance| {
            let mut modes = observations
                .iter()
                .filter(|observation| observation.instance() == Some(instance))
                .map(EffectiveModeObservation::mode);
            let mode = match modes.next() {
                // No observation at all: unknown, never a mode.
                None => MemberMode::Unknown,
                // Contradictory observations for one member establish nothing.
                Some(first) if modes.all(|mode| mode == first) => MemberMode::Established(first),
                Some(_) => MemberMode::Unknown,
            };
            MemberVerification {
                instance: instance.clone(),
                mode,
            }
        })
        .collect();

    let outcome = fleet_outcome(
        &members,
        ambiguous_enumeration || unattributed_observations > 0,
    );

    FleetVerification {
        outcome,
        members,
        unattributed_observations,
    }
}

/// Reduce per-member results to one outcome, failing closed.
fn fleet_outcome(members: &[MemberVerification], coverage_failed: bool) -> FleetOutcome {
    if coverage_failed || members.is_empty() {
        return FleetOutcome::NotEstablished;
    }
    let mut established = Vec::with_capacity(members.len());
    for member in members {
        match member.mode {
            // A single unknown member is enough: complete coverage is required,
            // so "2 of 3 agree" is not consistency.
            MemberMode::Unknown => return FleetOutcome::NotEstablished,
            MemberMode::Established(mode) => established.push(mode),
        }
    }
    let first = established[0];
    match established.iter().all(|mode| *mode == first) {
        true => FleetOutcome::Consistent(first),
        false => FleetOutcome::Mixed,
    }
}

#[cfg(test)]
mod tests;
