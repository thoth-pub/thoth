//! The code-owned kind registry (spec section 8).
//!
//! A kind is a code string, never a database enum: a binary leaves kinds it
//! does not know alone, and tests register test kinds. Engine functions take
//! `&AsyncJobRegistry`; production passes [`PRODUCTION_REGISTRY`], which is
//! empty in Release N, so the engine is inert.

use std::collections::{BTreeMap, HashSet};
use std::fmt;

use serde::{Deserialize, Serialize};

use super::{is_kind_format, is_namespace, is_non_blank, EffectClass, RevisionMode, MAX_TARGETS};

/// The closed set of executor domains. `ThothWorker` is in-process and never
/// claimable through GraphQL; `Dissemination` maps to `DISSEMINATION_WORKER`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ExecutorDomain {
    ThothWorker,
    Dissemination,
}

/// The risk class of the content a kind handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskClass {
    Trusted,
    UntrustedContent,
}

/// Which pre-effect states an operator may cancel (T14, T14w, T14r). An
/// operator stop after effect start (T14e) does not consult this policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CancelPolicy {
    pub pending: bool,
    pub retry_scheduled: bool,
    pub waiting: bool,
    pub running: bool,
}

impl CancelPolicy {
    /// Every pre-effect state may be cancelled.
    pub const ALL: CancelPolicy = CancelPolicy {
        pending: true,
        retry_scheduled: true,
        waiting: true,
        running: true,
    };
    /// No state may be cancelled (an operator stop after effect start still
    /// applies).
    pub const NONE: CancelPolicy = CancelPolicy {
        pending: false,
        retry_scheduled: false,
        waiting: false,
        running: false,
    };
}

/// The outcomes a staff `RESOLVE_RECONCILIATION` command may record (T18).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StaffOutcome {
    Succeeded,
    Failed,
    Cancelled,
    Waiting,
    AcceptRegression,
}

impl StaffOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            StaffOutcome::Succeeded => "SUCCEEDED",
            StaffOutcome::Failed => "FAILED",
            StaffOutcome::Cancelled => "CANCELLED",
            StaffOutcome::Waiting => "WAITING",
            StaffOutcome::AcceptRegression => "ACCEPT_REGRESSION",
        }
    }
}

/// What an evidence object attests (section 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvidenceUse {
    /// The job's own effect holds at the provider.
    Applied,
    /// No effect exists at the provider.
    NoEffect,
    /// An effect was confirmed but does not hold at the provider.
    NotApplied,
}

/// What a kind's `validate_evidence` sees of the job, so it can check that an
/// evidence object refers to this job's own revision and targets.
#[derive(Debug, Clone, Copy)]
pub struct EvidenceContext<'a> {
    pub payload_version: i16,
    pub payload: &'a serde_json::Value,
    pub source_order: Option<i64>,
    pub source_fingerprint: Option<&'a str>,
    pub source_ref: Option<&'a str>,
    pub target_keys: &'a [String],
    pub provider_ref: Option<&'a str>,
}

/// A kind function's refusal. Its message is code-owned text: a validator
/// never copies payload or evidence content into it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayloadError(pub String);

impl PayloadError {
    pub fn new(message: impl Into<String>) -> Self {
        PayloadError(message.into())
    }
}

impl fmt::Display for PayloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// One kind's policy (section 8). Every numeric bound is checked by
/// [`AsyncJobRegistry::validate`].
#[derive(Clone, Copy)]
pub struct AsyncJobKindPolicy {
    /// Matches the database format check `^[A-Z][A-Z0-9_]{2,63}$`.
    pub kind: &'static str,
    /// For example `"publisher_services"`.
    pub owning_domain: &'static str,
    pub executor_domain: ExecutorDomain,
    pub reconciler_domain: ExecutorDomain,
    pub risk_class: RiskClass,
    pub required_capabilities: &'static [&'static str],
    /// Accepted at creation.
    pub creatable_versions: &'static [i16],
    /// A superset of `creatable_versions`; claimable.
    pub executable_versions: &'static [i16],
    pub effect_class: EffectClass,
    pub revision_mode: RevisionMode,
    /// Every target key's namespace.
    pub target_namespaces: &'static [&'static str],
    /// `false` retires the kind: creation is refused and its jobs drain.
    pub admission_open: bool,
    /// `1..=20`.
    pub retry_budget: i16,
    /// `retry_budget..=100`.
    pub max_execution_claims: i16,
    /// `1..=50`.
    pub max_reconciliation_attempts: i16,
    /// `>= 1`.
    pub backoff_base_seconds: i32,
    /// `>= backoff_base_seconds`, `<= 86400`.
    pub backoff_max_seconds: i32,
    /// `>= 5`; the minimum `WAITING` resume delay.
    pub poll_interval_min_seconds: i32,
    pub lease_default_seconds: i32,
    /// `>= 15`.
    pub lease_min_seconds: i32,
    /// `<= max_attempt_runtime_seconds`.
    pub lease_max_seconds: i32,
    /// `<= 86400`.
    pub max_attempt_runtime_seconds: i32,
    /// `Some` iff `EFFECTFUL`; `<= lease_max_seconds`.
    pub effect_window_seconds: Option<i32>,
    /// `<= 30 days`.
    pub waiting_horizon_seconds: i64,
    /// `>= 1`: `RUNNING` jobs of this kind.
    pub concurrency_limit: i32,
    /// `>= 1`: claimed reconciliations of this kind.
    pub reconciliation_concurrency_limit: i32,
    /// `0..=32`; `>= 1` when `EFFECTFUL`; `<= 1` when `REVISION_BOUND`.
    pub max_targets: usize,
    /// Which pre-effect states may be cancelled.
    pub cancellable: CancelPolicy,
    /// `RETRY_JOB` from `FAILED`/`CANCELLED` at `PRE_WRITE` or after proven
    /// no-effect.
    pub manual_retry_allowed: bool,
    /// `FAILED` after `EFFECT_STARTED` on proven no-effect (T9n).
    pub no_effect_after_start_allowed: bool,
    /// Lease loss at `EFFECT_CONFIRMED` goes to `WAITING`, not RR (T12).
    pub confirmed_effect_resumable: bool,
    /// `SUCCEEDED`, `FAILED`, `CANCELLED`, `WAITING`, `ACCEPT_REGRESSION`.
    pub staff_reconciliation_outcomes: &'static [StaffOutcome],
    pub validate_payload: fn(i16, &serde_json::Value) -> Result<(), PayloadError>,
    /// At most 400 characters.
    pub effect_identity: fn(i16, &serde_json::Value) -> Result<String, PayloadError>,
    pub targets: fn(i16, &serde_json::Value) -> Result<Vec<String>, PayloadError>,
    pub validate_evidence:
        fn(EvidenceUse, &EvidenceContext<'_>, &serde_json::Value) -> Result<(), PayloadError>,
}

impl fmt::Debug for AsyncJobKindPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsyncJobKindPolicy")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

impl AsyncJobKindPolicy {
    pub fn is_effectful(&self) -> bool {
        self.effect_class == EffectClass::Effectful
    }

    pub fn is_revision_bound(&self) -> bool {
        self.revision_mode == RevisionMode::RevisionBound
    }

    pub fn allows_staff_outcome(&self, outcome: StaffOutcome) -> bool {
        self.staff_reconciliation_outcomes.contains(&outcome)
    }

    pub fn is_creatable_version(&self, version: i16) -> bool {
        self.creatable_versions.contains(&version)
    }

    pub fn is_executable_version(&self, version: i16) -> bool {
        self.executable_versions.contains(&version)
    }

    /// `requested` clamped to `[lease_min_seconds, lease_max_seconds]`; the
    /// kind's default when nothing is requested.
    pub fn clamp_lease_seconds(&self, requested: Option<i32>) -> i32 {
        requested
            .unwrap_or(self.lease_default_seconds)
            .clamp(self.lease_min_seconds, self.lease_max_seconds)
    }

    /// Every bound of section 8 that concerns this policy alone.
    fn validate_bounds(&self, problems: &mut Vec<String>) {
        let kind = self.kind;
        let mut fail = |message: String| problems.push(format!("{kind}: {message}"));
        if !is_kind_format(kind) {
            fail("kind does not match ^[A-Z][A-Z0-9_]{2,63}$".to_string());
        }
        if !is_non_blank(self.owning_domain) {
            fail("owning_domain is blank".to_string());
        }
        if self
            .required_capabilities
            .iter()
            .any(|capability| !is_non_blank(capability))
        {
            fail("a required capability is blank".to_string());
        }
        if self.executable_versions.is_empty() {
            fail("executable_versions is empty".to_string());
        }
        if self
            .executable_versions
            .iter()
            .chain(self.creatable_versions)
            .any(|version| *version < 1)
        {
            fail("a payload version is below 1".to_string());
        }
        if self
            .creatable_versions
            .iter()
            .any(|version| !self.executable_versions.contains(version))
        {
            fail("executable_versions is not a superset of creatable_versions".to_string());
        }
        if !(1..=20).contains(&self.retry_budget) {
            fail("retry_budget is outside 1..=20".to_string());
        }
        if !(self.retry_budget..=100).contains(&self.max_execution_claims) {
            fail("max_execution_claims is outside retry_budget..=100".to_string());
        }
        if !(1..=50).contains(&self.max_reconciliation_attempts) {
            fail("max_reconciliation_attempts is outside 1..=50".to_string());
        }
        if self.backoff_base_seconds < 1 {
            fail("backoff_base_seconds is below 1".to_string());
        }
        if !(self.backoff_base_seconds..=86_400).contains(&self.backoff_max_seconds) {
            fail("backoff_max_seconds is outside backoff_base_seconds..=86400".to_string());
        }
        if self.poll_interval_min_seconds < 5 {
            fail("poll_interval_min_seconds is below 5".to_string());
        }
        if self.lease_min_seconds < 15 {
            fail("lease_min_seconds is below 15".to_string());
        }
        if self.lease_max_seconds < self.lease_min_seconds {
            fail("lease_max_seconds is below lease_min_seconds".to_string());
        }
        if self.lease_max_seconds > self.max_attempt_runtime_seconds {
            fail("lease_max_seconds exceeds max_attempt_runtime_seconds".to_string());
        }
        if !(self.lease_min_seconds..=self.lease_max_seconds).contains(&self.lease_default_seconds)
        {
            fail(
                "lease_default_seconds is outside lease_min_seconds..=lease_max_seconds"
                    .to_string(),
            );
        }
        if self.max_attempt_runtime_seconds > 86_400 {
            fail("max_attempt_runtime_seconds exceeds 86400".to_string());
        }
        match (self.effect_class, self.effect_window_seconds) {
            (EffectClass::Effectful, Some(window)) => {
                if window < 1 || window > self.lease_max_seconds {
                    fail("effect_window_seconds is outside 1..=lease_max_seconds".to_string());
                }
            }
            (EffectClass::Effectful, None) => {
                fail("an EFFECTFUL kind needs effect_window_seconds".to_string())
            }
            (EffectClass::ReadOnly, Some(_)) => {
                fail("a READ_ONLY kind has effect_window_seconds".to_string())
            }
            (EffectClass::ReadOnly, None) => {}
        }
        if !(1..=30 * 86_400).contains(&self.waiting_horizon_seconds) {
            fail("waiting_horizon_seconds is outside 1..=30 days".to_string());
        }
        if self.concurrency_limit < 1 {
            fail("concurrency_limit is below 1".to_string());
        }
        if self.reconciliation_concurrency_limit < 1 {
            fail("reconciliation_concurrency_limit is below 1".to_string());
        }
        if self.max_targets > MAX_TARGETS {
            fail("max_targets exceeds 32".to_string());
        }
        if self.is_effectful() && self.max_targets < 1 {
            fail("an EFFECTFUL kind needs max_targets >= 1".to_string());
        }
        if self.is_revision_bound() && self.max_targets > 1 {
            fail("a REVISION_BOUND kind has max_targets <= 1".to_string());
        }
        if self
            .target_namespaces
            .iter()
            .any(|namespace| !is_namespace(namespace))
        {
            fail("a target namespace does not match ^[a-z][a-z0-9_]{1,31}$".to_string());
        }
        if self.max_targets >= 1 && self.target_namespaces.is_empty() {
            fail("a kind with targets declares no target namespace".to_string());
        }
        let outcomes = self.staff_reconciliation_outcomes;
        if self.is_effectful()
            && !(outcomes.contains(&StaffOutcome::Succeeded)
                && outcomes.contains(&StaffOutcome::Failed))
        {
            fail("an EFFECTFUL kind must list SUCCEEDED and FAILED staff outcomes".to_string());
        }
        let accept = outcomes.contains(&StaffOutcome::AcceptRegression);
        if self.is_effectful() && self.is_revision_bound() && !accept {
            fail("an EFFECTFUL REVISION_BOUND kind must list ACCEPT_REGRESSION".to_string());
        }
        if accept && !(self.is_effectful() && self.is_revision_bound()) {
            fail("only an EFFECTFUL REVISION_BOUND kind may list ACCEPT_REGRESSION".to_string());
        }
    }
}

/// A slice of policies looked up by kind.
#[derive(Debug, Clone, Copy)]
pub struct AsyncJobRegistry {
    policies: &'static [AsyncJobKindPolicy],
}

/// The production registry: **empty** in Release N.
pub static PRODUCTION_REGISTRY: AsyncJobRegistry = AsyncJobRegistry::new(&[]);

impl AsyncJobRegistry {
    pub const fn new(policies: &'static [AsyncJobKindPolicy]) -> Self {
        AsyncJobRegistry { policies }
    }

    pub fn policies(&self) -> &'static [AsyncJobKindPolicy] {
        self.policies
    }

    pub fn get(&self, kind: &str) -> Option<&'static AsyncJobKindPolicy> {
        self.policies.iter().find(|policy| policy.kind == kind)
    }

    pub fn is_empty(&self) -> bool {
        self.policies.is_empty()
    }

    /// Validate every policy against the bounds of section 8 and the
    /// cross-kind rules: kinds are unique, and every kind declaring a target
    /// namespace shares that namespace's owning domain and revision mode. It
    /// passes vacuously on the empty production registry and is the gate for
    /// every future kind. Returns every problem found.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut problems = Vec::new();
        let mut kinds = HashSet::new();
        for policy in self.policies {
            if !kinds.insert(policy.kind) {
                problems.push(format!("{}: the kind is registered twice", policy.kind));
            }
            policy.validate_bounds(&mut problems);
        }
        let mut namespaces: BTreeMap<&str, (&str, &str, RevisionMode)> = BTreeMap::new();
        for policy in self.policies {
            for namespace in policy.target_namespaces {
                match namespaces.get(namespace) {
                    None => {
                        namespaces.insert(
                            namespace,
                            (policy.kind, policy.owning_domain, policy.revision_mode),
                        );
                    }
                    Some((first_kind, domain, mode)) => {
                        if *domain != policy.owning_domain {
                            problems.push(format!(
                                "{}: target namespace {namespace} is owned by {domain} \
                                 (kind {first_kind}), not {}",
                                policy.kind, policy.owning_domain
                            ));
                        }
                        if *mode != policy.revision_mode {
                            problems.push(format!(
                                "{}: target namespace {namespace} has revision mode {} \
                                 (kind {first_kind}), not {}",
                                policy.kind,
                                mode.as_str(),
                                policy.revision_mode.as_str()
                            ));
                        }
                    }
                }
            }
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(problems)
        }
    }
}
