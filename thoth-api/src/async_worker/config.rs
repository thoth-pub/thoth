//! The worker's configuration (spec section 4) and the AWS credential-source
//! check (spec section 6.2 step 2).
//!
//! Every value arrives as text from the command line or its environment
//! variable and is validated here, so a malformed value refuses to start
//! with an error that names the argument. No error ever repeats a value of
//! `--database-url`, and [`WorkerConfig`]'s `Debug` redacts it.

use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;

/// The default `--worker-name`.
pub const DEFAULT_WORKER_NAME: &str = "thoth-worker";
/// The default `--slots`.
pub const DEFAULT_SLOTS: &str = "4";
/// The default `--poll-seconds`.
pub const DEFAULT_POLL_SECONDS: &str = "5";
/// The default `--shutdown-grace-seconds`.
pub const DEFAULT_SHUTDOWN_GRACE_SECONDS: &str = "20";
/// The default `--liveness-file`.
pub const DEFAULT_LIVENESS_FILE: &str = "/tmp/thoth-worker-liveness";
/// The default `thoth worker health --max-age-seconds`.
pub const DEFAULT_HEALTH_MAX_AGE_SECONDS: &str = "60";

/// The executor principal of the trusted pool (spec section 5.2).
pub const WORKER_PRINCIPAL: &str = "process:thoth-worker";

/// The AWS variables whose presence (set and non-empty) refuses startup:
/// static keys, a session token, and the profile and file selectors. Worker
/// AWS authority comes from the ECS task role only (ADR-0012 3.10).
pub const AWS_CREDENTIAL_VARIABLES: [&str; 6] = [
    "AWS_ACCESS_KEY_ID",
    "AWS_SECRET_ACCESS_KEY",
    "AWS_SESSION_TOKEN",
    "AWS_PROFILE",
    "AWS_SHARED_CREDENTIALS_FILE",
    "AWS_CONFIG_FILE",
];

/// The raw arguments of `thoth start worker`, as text.
#[derive(Clone, PartialEq, Eq)]
pub struct WorkerArgs {
    pub database_url: Option<String>,
    pub worker_name: String,
    pub kinds: String,
    pub capabilities: String,
    pub slots: String,
    pub poll_seconds: String,
    pub shutdown_grace_seconds: String,
    pub liveness_file: String,
}

impl WorkerArgs {
    /// The section 4 defaults with the given database URL.
    pub fn with_defaults(database_url: Option<String>) -> Self {
        WorkerArgs {
            database_url,
            worker_name: DEFAULT_WORKER_NAME.to_string(),
            kinds: String::new(),
            capabilities: String::new(),
            slots: DEFAULT_SLOTS.to_string(),
            poll_seconds: DEFAULT_POLL_SECONDS.to_string(),
            shutdown_grace_seconds: DEFAULT_SHUTDOWN_GRACE_SECONDS.to_string(),
            liveness_file: DEFAULT_LIVENESS_FILE.to_string(),
        }
    }
}

impl fmt::Debug for WorkerArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkerArgs")
            .field("database_url", &"<redacted>")
            .field("worker_name", &self.worker_name)
            .field("kinds", &self.kinds)
            .field("capabilities", &self.capabilities)
            .field("slots", &self.slots)
            .field("poll_seconds", &self.poll_seconds)
            .field("shutdown_grace_seconds", &self.shutdown_grace_seconds)
            .field("liveness_file", &self.liveness_file)
            .finish()
    }
}

/// The validated configuration.
#[derive(Clone, PartialEq, Eq)]
pub struct WorkerConfig {
    database_url: String,
    /// `claimed_by`: stable across restarts.
    pub worker_name: String,
    /// The kinds this worker may claim and recover; empty means idle.
    pub kinds: Vec<String>,
    /// The runtime capabilities this deployment provides.
    pub capabilities: Vec<String>,
    /// Handler calls in flight at once.
    pub slots: usize,
    /// The idle poll interval in seconds.
    pub poll_seconds: u64,
    /// Handler time after a stop signal before the fallback phase.
    pub shutdown_grace_seconds: u64,
    /// Where the liveness ticker writes.
    pub liveness_file: PathBuf,
}

impl fmt::Debug for WorkerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkerConfig")
            .field("database_url", &"<redacted>")
            .field("worker_name", &self.worker_name)
            .field("kinds", &self.kinds)
            .field("capabilities", &self.capabilities)
            .field("slots", &self.slots)
            .field("poll_seconds", &self.poll_seconds)
            .field("shutdown_grace_seconds", &self.shutdown_grace_seconds)
            .field("liveness_file", &self.liveness_file)
            .finish()
    }
}

impl WorkerConfig {
    /// Validate every argument and report every problem, each naming its
    /// argument and never a value of `--database-url`.
    pub fn from_args(args: &WorkerArgs) -> Result<WorkerConfig, Vec<String>> {
        let mut problems = Vec::new();
        let database_url = match args.database_url.as_deref() {
            Some(url) if !url.is_empty() => url.to_string(),
            _ => {
                problems.push("--database-url (DATABASE_URL) must be set".to_string());
                String::new()
            }
        };
        if !is_worker_name(&args.worker_name) {
            problems.push(
                "--worker-name (ASYNC_WORKER_NAME) must match ^[a-z][a-z0-9-]{2,63}$".to_string(),
            );
        }
        let kinds = parse_list(
            &args.kinds,
            "--kinds (ASYNC_WORKER_KINDS)",
            "^[A-Z][A-Z0-9_]{2,63}$",
            is_kind_name,
            &mut problems,
        );
        let capabilities = parse_list(
            &args.capabilities,
            "--capabilities (ASYNC_WORKER_CAPABILITIES)",
            "^[a-z][a-z0-9_.]{0,63}$",
            is_capability,
            &mut problems,
        );
        let slots = parse_bounded(
            &args.slots,
            "--slots (ASYNC_WORKER_SLOTS)",
            1,
            16,
            &mut problems,
        );
        let poll_seconds = parse_bounded(
            &args.poll_seconds,
            "--poll-seconds (ASYNC_WORKER_POLL_SECONDS)",
            1,
            60,
            &mut problems,
        );
        let shutdown_grace_seconds = parse_bounded(
            &args.shutdown_grace_seconds,
            "--shutdown-grace-seconds (ASYNC_WORKER_SHUTDOWN_GRACE_SECONDS)",
            1,
            80,
            &mut problems,
        );
        let liveness_file = parse_liveness_file(
            &args.liveness_file,
            "--liveness-file (ASYNC_WORKER_LIVENESS_FILE)",
            &mut problems,
        );
        if !problems.is_empty() {
            return Err(problems);
        }
        Ok(WorkerConfig {
            database_url,
            worker_name: args.worker_name.clone(),
            kinds,
            capabilities,
            slots: slots as usize,
            poll_seconds,
            shutdown_grace_seconds,
            liveness_file,
        })
    }

    /// The primary database URL. Never logged.
    pub(crate) fn database_url(&self) -> &str {
        &self.database_url
    }

    /// True when no kind is listed: the worker idles and opens no database
    /// connection.
    pub fn is_idle(&self) -> bool {
        self.kinds.is_empty()
    }
}

/// `^[a-z][a-z0-9-]{2,63}$`.
pub fn is_worker_name(value: &str) -> bool {
    let mut chars = value.chars();
    (3..=64).contains(&value.chars().count())
        && matches!(chars.next(), Some(first) if first.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// `^[A-Z][A-Z0-9_]{2,63}$`.
pub fn is_kind_name(value: &str) -> bool {
    let mut chars = value.chars();
    (3..=64).contains(&value.chars().count())
        && matches!(chars.next(), Some(first) if first.is_ascii_uppercase())
        && chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

/// `^[a-z][a-z0-9_.]{0,63}$`.
pub fn is_capability(value: &str) -> bool {
    let mut chars = value.chars();
    (1..=64).contains(&value.chars().count())
        && matches!(chars.next(), Some(first) if first.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.')
}

/// A comma list: empty (or blank) text is the empty list; every item is
/// trimmed, must match `format` and appear once.
fn parse_list(
    text: &str,
    name: &str,
    format: &str,
    valid: fn(&str) -> bool,
    problems: &mut Vec<String>,
) -> Vec<String> {
    if text.trim().is_empty() {
        return Vec::new();
    }
    let mut items: Vec<String> = Vec::new();
    let mut malformed = false;
    let mut duplicate = false;
    for item in text.split(',').map(str::trim) {
        if !valid(item) {
            malformed = true;
        } else if items.iter().any(|seen| seen == item) {
            duplicate = true;
        } else {
            items.push(item.to_string());
        }
    }
    if malformed {
        problems.push(format!(
            "{name} must be a comma list whose items each match {format}"
        ));
    }
    if duplicate {
        problems.push(format!("{name} lists an item more than once"));
    }
    items
}

/// A decimal integer in `min..=max`.
fn parse_bounded(text: &str, name: &str, min: u64, max: u64, problems: &mut Vec<String>) -> u64 {
    match text.trim().parse::<u64>() {
        Ok(value) if (min..=max).contains(&value) => value,
        _ => {
            problems.push(format!("{name} must be an integer in {min}..={max}"));
            min
        }
    }
}

/// An absolute path naming a file.
pub(crate) fn parse_liveness_file(text: &str, name: &str, problems: &mut Vec<String>) -> PathBuf {
    let path = PathBuf::from(text);
    if text.is_empty() || !path.is_absolute() || path.file_name().is_none() {
        problems.push(format!("{name} must be an absolute file path"));
    }
    path
}

/// Refuse to start when any of [`AWS_CREDENTIAL_VARIABLES`] is set and
/// non-empty. A pure function over a lookup of single variables (it never
/// reads the whole environment); each message names the variable, never its
/// value. `AWS_CONTAINER_CREDENTIALS_*` and `AWS_REGION` are allowed: they
/// are how the task role is reached.
pub fn check_aws_sources(lookup: impl Fn(&str) -> Option<OsString>) -> Result<(), Vec<String>> {
    let problems: Vec<String> = AWS_CREDENTIAL_VARIABLES
        .iter()
        .filter(|name| lookup(name).is_some_and(|value| !value.is_empty()))
        .map(|name| {
            format!(
                "{name} is set: the worker takes AWS authority from the ECS task role only, \
                 so no static, profile or file credential source may be configured"
            )
        })
        .collect();
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}
