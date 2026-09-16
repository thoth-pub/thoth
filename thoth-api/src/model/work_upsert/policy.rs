//! Code-owned input validation for the BE-06 operations (Amendment 3 section 7).
//!
//! Every rule here is evaluated in `thoth-api` on the decoded value, after
//! request authorization and before any database access. None trims, normalises
//! or depends on the database's collation provider; the database CHECKs remain
//! defence in depth only.

use thoth_errors::{ThothError, ThothResult};

use crate::model::publisher_distribution_platform::DistributionPlatform;
use crate::model::work_upsert::registry::{self, WorkLevelExecutionProfile};

/// Whether `value` is blank (Amendment 3 section 7.1): empty, or made only of
/// Unicode scalar values that are `char::is_whitespace` or `char::is_control`.
pub(crate) fn is_blank(value: &str) -> bool {
    value.chars().all(|c| c.is_whitespace() || c.is_control())
}

/// Whether `value` is exactly 64 bytes, each `0`-`9` or `a`-`f`: the lower-case
/// hexadecimal SHA-256 shape (Amendment 3 section 7.2).
pub(crate) fn is_sha256_lower_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// The registered profiles an `executionProfiles` list names, deduplicated in
/// canonical platform order (Amendment 3 section 7.3). An empty list and an
/// unregistered member are refused before any database access.
pub(crate) fn registered_profiles(
    profiles: &[DistributionPlatform],
) -> ThothResult<Vec<&'static WorkLevelExecutionProfile>> {
    if profiles.is_empty() {
        return Err(ThothError::WorkUpsertExecutionProfilesRequired);
    }
    let mut registered = Vec::new();
    for platform in DistributionPlatform::ALL {
        if !profiles.contains(&platform) {
            continue;
        }
        registered.push(
            registry::execution_profile(platform)
                .ok_or(ThothError::WorkUpsertProfileNotImplemented)?,
        );
    }
    Ok(registered)
}

/// The released clamp convention: an absent limit is `default`; `<= 0` runs
/// nothing; values above `max` are clamped. No limit is ever refused.
pub(crate) fn clamp_limit(limit: Option<i32>, default: i32, max: i32) -> i64 {
    i64::from(limit.unwrap_or(default).clamp(0, max))
}
