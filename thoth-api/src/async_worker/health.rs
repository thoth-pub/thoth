//! Liveness (spec section 7.2): the file format, the atomic write, the
//! withhold rules the ticker applies, and `thoth worker health`.
//!
//! The liveness file holds the current Unix time in decimal seconds. The
//! ticker writes it to a temporary file beside the target and renames it into
//! place, so a reader never sees a partial write. The health check opens no
//! database connection and no socket: it only reads the file.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::config::{parse_liveness_file, DEFAULT_HEALTH_MAX_AGE_SECONDS, DEFAULT_LIVENESS_FILE};

/// The smallest and largest `--max-age-seconds`. The minimum is three ticker
/// periods (`3 x LIVENESS_CADENCE`).
pub const HEALTH_MAX_AGE_RANGE: (u64, u64) = (15, 600);

/// The text written for a Unix time.
pub fn liveness_text(unix_seconds: u64) -> String {
    format!("{unix_seconds}\n")
}

/// The Unix time a liveness text holds, if it parses.
pub fn parse_liveness_text(text: &str) -> Option<u64> {
    text.trim().parse::<u64>().ok()
}

/// The current Unix time in seconds.
pub fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

/// The temporary file the ticker writes before the rename.
pub(crate) fn temporary_path(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    name.push(".tmp");
    path.with_file_name(name)
}

/// Write `unix_seconds` to `path` atomically: a temporary file next to it,
/// then a rename. The parent directory is created when missing.
pub fn write_liveness(path: &Path, unix_seconds: u64) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let temporary = temporary_path(path);
    fs::write(&temporary, liveness_text(unix_seconds))?;
    fs::rename(&temporary, path)
}

/// True when the file exists, parses and is at most `max_age_seconds` old
/// at `now_unix`.
pub fn is_healthy(path: &Path, max_age_seconds: u64, now_unix: u64) -> bool {
    match fs::read_to_string(path) {
        Ok(text) => match parse_liveness_text(&text) {
            Some(written) => now_unix.saturating_sub(written) <= max_age_seconds,
            None => false,
        },
        Err(_) => false,
    }
}

/// Validate the arguments of `thoth worker health`.
pub fn parse_health_args(
    liveness_file: &str,
    max_age_seconds: &str,
) -> Result<(PathBuf, u64), Vec<String>> {
    let mut problems = Vec::new();
    let path = parse_liveness_file(
        liveness_file,
        "--liveness-file (ASYNC_WORKER_LIVENESS_FILE)",
        &mut problems,
    );
    let (min, max) = HEALTH_MAX_AGE_RANGE;
    let max_age = match max_age_seconds.trim().parse::<u64>() {
        Ok(value) if (min..=max).contains(&value) => value,
        _ => {
            problems.push(format!(
                "--max-age-seconds (ASYNC_WORKER_HEALTH_MAX_AGE_SECONDS) must be an integer in \
                 {min}..={max}"
            ));
            min
        }
    };
    if problems.is_empty() {
        Ok((path, max_age))
    } else {
        Err(problems)
    }
}

/// `thoth worker health`: exit code 0 when healthy, otherwise 1. A
/// malformed argument also answers 1, with one line per problem on standard
/// error.
pub fn health_command(liveness_file: Option<&str>, max_age_seconds: Option<&str>) -> i32 {
    let liveness_file = liveness_file.unwrap_or(DEFAULT_LIVENESS_FILE);
    let max_age_seconds = max_age_seconds.unwrap_or(DEFAULT_HEALTH_MAX_AGE_SECONDS);
    match parse_health_args(liveness_file, max_age_seconds) {
        Ok((path, max_age)) => {
            if is_healthy(&path, max_age, unix_now()) {
                0
            } else {
                1
            }
        }
        Err(problems) => {
            for problem in problems {
                eprintln!("async worker: {problem}");
            }
            1
        }
    }
}

/// What the ticker knows at one tick (spec section 7.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LivenessInputs {
    /// Age of the claim-loop progress (or of the startup wait loops, or of
    /// idle mode).
    pub loop_age: Duration,
    /// Age of the heartbeat progress; `None` before the heartbeat starts.
    pub heartbeat_age: Option<Duration>,
    /// Claims in flight.
    pub in_flight: usize,
    /// How long each hung slot has been hung.
    pub hung: Vec<Duration>,
    /// The configured slot count.
    pub slots: usize,
    /// A runtime thread panicked.
    pub panicked: bool,
}

/// The limits the ticker applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LivenessLimits {
    pub loop_stall: Duration,
    pub heartbeat_stall: Duration,
    pub hung_slot_limit: Duration,
}

/// Why the ticker withholds its write, or `None` to write.
pub(crate) fn withhold_reason(
    inputs: &LivenessInputs,
    limits: &LivenessLimits,
) -> Option<&'static str> {
    if inputs.panicked {
        return Some("RUNTIME_THREAD_PANICKED");
    }
    if inputs.loop_age > limits.loop_stall {
        return Some("LOOP_STALLED");
    }
    if inputs.in_flight > 0
        && inputs
            .heartbeat_age
            .is_some_and(|age| age > limits.heartbeat_stall)
    {
        return Some("HEARTBEAT_STALLED");
    }
    if inputs.hung.iter().any(|age| *age > limits.hung_slot_limit) {
        return Some("SLOT_HUNG_TOO_LONG");
    }
    if inputs.slots > 0 && inputs.hung.len() >= inputs.slots {
        return Some("ALL_SLOTS_HUNG");
    }
    None
}
