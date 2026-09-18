//! The work-level execution profile registry (R52B section 7).
//!
//! A work-level execution profile is programme-local, code-owned and statically
//! registered. "This platform has a work-level execution profile" is
//! represented only by [`execution_profile`] and by the per-profile arms of the
//! database target-set trigger (R52B section 11.4). It is never inferred from a
//! platform's back-catalogue behaviour, adapter profile or linked group.

use crate::model::publisher_distribution_platform::DistributionPlatform;

/// Whether a fenced attempt that vanished may be replayed (R52B section 11.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FencedRecovery {
    /// A new claim may replay a fenced abandonment.
    ReplaySafe,
    /// A fenced abandonment blocks every claim of its `(work, profile)` until
    /// the profile's reconciliation clears it.
    ReplayBlocked,
}

/// One statically registered work-level execution profile.
#[derive(Debug, PartialEq, Eq)]
pub struct WorkLevelExecutionProfile {
    /// The stable durable execution key.
    pub key: DistributionPlatform,
    /// The declared target set: non-empty, containing the key, in canonical
    /// platform order and duplicate-free.
    pub targets: &'static [DistributionPlatform],
    /// The fenced-recovery class.
    pub fenced_recovery: FencedRecovery,
    /// Where the fenced-recovery class is proved.
    pub recovery_proof_reference: &'static str,
}

/// The Crossref execution profile (R52B sections 7.3 and 14.1).
pub static CROSSREF_PROFILE: WorkLevelExecutionProfile = WorkLevelExecutionProfile {
    key: DistributionPlatform::Crossref,
    targets: &[DistributionPlatform::Crossref],
    fenced_recovery: FencedRecovery::ReplayBlocked,
    recovery_proof_reference: "R52B §11.7",
};

/// The registered work-level execution profile of `platform`, if any.
///
/// The match is exhaustive with no wildcard arm, so adding a destination fails
/// compilation until it is classified here.
pub fn execution_profile(
    platform: DistributionPlatform,
) -> Option<&'static WorkLevelExecutionProfile> {
    match platform {
        DistributionPlatform::InternetArchive => None,
        DistributionPlatform::Oapen => None,
        DistributionPlatform::Doab => None,
        DistributionPlatform::ScienceOpen => None,
        DistributionPlatform::CambridgeUniversityLibrary => None,
        DistributionPlatform::Crossref => Some(&CROSSREF_PROFILE),
        DistributionPlatform::Figshare => None,
        DistributionPlatform::Zenodo => None,
        DistributionPlatform::ProjectMuse => None,
        DistributionPlatform::Jstor => None,
        DistributionPlatform::EbscoHost => None,
        DistributionPlatform::ProquestEbookCentral => None,
        DistributionPlatform::GooglePlay => None,
        DistributionPlatform::Bkci => None,
        DistributionPlatform::OclcKb => None,
        DistributionPlatform::ExLibrisKb => None,
        DistributionPlatform::JiscNbk => None,
    }
}
