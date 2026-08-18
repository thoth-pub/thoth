//! Publisher distribution-platform assignments (`BE-02`).
//!
//! This module owns the closed [`DistributionPlatform`] inventory approved by
//! [ADR-0004] and the final repository inventory, the code-owned platform
//! descriptors, and the persisted `publisher_distribution_platform` assignment
//! model.
//!
//! BE-02 is an inactive additive foundation: nothing here creates a
//! distribution job, performs dissemination, or activates a destination.
//!
//! [ADR-0004]: ../../../docs/engineering/decisions/ADR-0004-distribution-platform-inventory.md

use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::model::Timestamp;

/// A destination to which a publisher's works may be distributed.
///
/// The inventory is closed and exhaustive. There is deliberately no `OTHER`,
/// `UNKNOWN`, `PROVISIONAL`, catch-all, fallback or `Default` variant: an
/// unrecognised database, serde, string or GraphQL value must fail rather than
/// silently resolve to a nearest destination. This enum is also deliberately
/// distinct from `MetricPlatform`; no name-based conversion exists between
/// them.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
    graphql(description = "Destination to which a publisher's works may be distributed"),
    ExistingTypePath = "crate::schema::sql_types::DistributionPlatform"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum DistributionPlatform {
    #[cfg_attr(
        feature = "backend",
        db_rename = "INTERNET_ARCHIVE",
        graphql(description = "Internet Archive")
    )]
    InternetArchive,
    #[cfg_attr(
        feature = "backend",
        db_rename = "OAPEN",
        graphql(description = "OAPEN; linked with DOAB")
    )]
    Oapen,
    #[cfg_attr(
        feature = "backend",
        db_rename = "DOAB",
        graphql(description = "Directory of Open Access Books; linked with OAPEN")
    )]
    Doab,
    #[cfg_attr(
        feature = "backend",
        db_rename = "SCIENCE_OPEN",
        graphql(description = "ScienceOpen")
    )]
    ScienceOpen,
    #[cfg_attr(
        feature = "backend",
        db_rename = "CAMBRIDGE_UNIVERSITY_LIBRARY",
        graphql(description = "Cambridge University Library")
    )]
    CambridgeUniversityLibrary,
    #[cfg_attr(
        feature = "backend",
        db_rename = "CROSSREF",
        graphql(description = "Crossref")
    )]
    Crossref,
    #[cfg_attr(
        feature = "backend",
        db_rename = "FIGSHARE",
        graphql(description = "Figshare")
    )]
    Figshare,
    #[cfg_attr(
        feature = "backend",
        db_rename = "ZENODO",
        graphql(description = "Zenodo")
    )]
    Zenodo,
    #[cfg_attr(
        feature = "backend",
        db_rename = "PROJECT_MUSE",
        graphql(description = "Project MUSE")
    )]
    ProjectMuse,
    #[cfg_attr(
        feature = "backend",
        db_rename = "JSTOR",
        graphql(description = "JSTOR")
    )]
    Jstor,
    #[cfg_attr(
        feature = "backend",
        db_rename = "EBSCO_HOST",
        graphql(description = "EBSCOHost")
    )]
    EbscoHost,
    #[cfg_attr(
        feature = "backend",
        db_rename = "PROQUEST_EBOOK_CENTRAL",
        graphql(description = "ProQuest Ebook Central")
    )]
    ProquestEbookCentral,
    #[cfg_attr(
        feature = "backend",
        db_rename = "GOOGLE_PLAY",
        graphql(description = "Google Play Books")
    )]
    GooglePlay,
    #[cfg_attr(
        feature = "backend",
        db_rename = "BKCI",
        graphql(description = "Book Citation Index")
    )]
    Bkci,
    #[cfg_attr(
        feature = "backend",
        db_rename = "OCLC_KB",
        graphql(description = "OCLC Knowledge Base")
    )]
    OclcKb,
    #[cfg_attr(
        feature = "backend",
        db_rename = "EX_LIBRIS_KB",
        graphql(description = "Ex Libris Knowledge Base")
    )]
    ExLibrisKb,
    #[cfg_attr(
        feature = "backend",
        db_rename = "JISC_NBK",
        graphql(description = "Jisc NBK; included but currently inactive and non-assignable")
    )]
    JiscNbk,
}

/// A group of destinations whose assignment state is normalized together.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(
        description = "Group of destinations whose assignments are enabled and disabled together"
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum DistributionPlatformGroup {
    #[cfg_attr(
        feature = "backend",
        graphql(description = "OAPEN and DOAB, which share one deposit and are enabled together")
    )]
    OapenDoab,
}

/// How a destination receives a publisher's existing back catalogue.
///
/// `BE-04`'s service-configuration coordinator reads this classification when
/// determining whether a newly activated group qualifies for durable
/// back-catalogue job creation. BE-02 creates no job and performs no upload.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(
        description = "How a destination's existing back catalogue is handled. Newly activating a group that contains at least one AUTOMATIC_PUSH destination qualifies that activation for durable back-catalogue job creation; PULL_FEED and MANUAL create no automatic job. This classification itself performs no dissemination"
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum BackCatalogueBehaviour {
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Thoth is expected to push records to the destination")
    )]
    AutomaticPush,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "The destination retrieves a Thoth feed")
    )]
    PullFeed,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Staff action or destination-specific coordination is required")
    )]
    Manual,
}

/// Whether a destination may currently be assigned to a publisher.
///
/// Internal Rust vocabulary. Deliberately **not** a GraphQL enum: the public
/// contract exposes only the derived `assignable` boolean.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssignmentAvailability {
    Assignable,
    NonAssignable,
}

/// Whether the delivery mechanism behind a destination is ready.
///
/// Internal Rust vocabulary. Deliberately **not** a GraphQL enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MechanismReadiness {
    Active,
    Inactive,
}

/// The internal adapter or feed profile that serves a destination.
///
/// Internal Rust vocabulary. Deliberately **not** a GraphQL enum and never
/// exposed: adapter, feed, host, endpoint, bucket, account and credential
/// identities are not part of any BE-02 public contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DistributionAdapterProfile {
    IaApi,
    OapenDoabSword,
    ScienceOpenFtp,
    CulSword,
    CrossrefDoiDeposit,
    FigshareApi,
    ZenodoApi,
    MuseFtp,
    JstorFtp,
    EbscoHostSftp,
    ProquestEbookCentralFtp,
    GooglePlayGcs,
    BkciFtp,
    OclcKbartPublic,
    JiscNbkMarcS3,
}

/// The single static descriptor for one [`DistributionPlatform`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DistributionPlatformDescriptor {
    pub platform: DistributionPlatform,
    pub display_label: &'static str,
    pub linked_group: Option<DistributionPlatformGroup>,
    pub back_catalogue_behaviour: BackCatalogueBehaviour,
    pub assignment_availability: AssignmentAvailability,
    pub mechanism_readiness: MechanismReadiness,
    pub adapter_profile: DistributionAdapterProfile,
}

macro_rules! descriptor {
    (
        $konst:ident,
        $platform:expr,
        $label:literal,
        $group:expr,
        $behaviour:expr,
        $availability:expr,
        $readiness:expr,
        $profile:expr $(,)?
    ) => {
        static $konst: DistributionPlatformDescriptor = DistributionPlatformDescriptor {
            platform: $platform,
            display_label: $label,
            linked_group: $group,
            back_catalogue_behaviour: $behaviour,
            assignment_availability: $availability,
            mechanism_readiness: $readiness,
            adapter_profile: $profile,
        };
    };
}

use AssignmentAvailability::{Assignable, NonAssignable};
use BackCatalogueBehaviour::{AutomaticPush, Manual, PullFeed};
use DistributionAdapterProfile as Profile;
use DistributionPlatform as Platform;
use DistributionPlatformGroup::OapenDoab;
use MechanismReadiness::{Active, Inactive};

descriptor!(
    INTERNET_ARCHIVE,
    Platform::InternetArchive,
    "Internet Archive",
    None,
    AutomaticPush,
    Assignable,
    Active,
    Profile::IaApi,
);
descriptor!(
    OAPEN,
    Platform::Oapen,
    "OAPEN",
    Some(OapenDoab),
    AutomaticPush,
    Assignable,
    Active,
    Profile::OapenDoabSword,
);
descriptor!(
    DOAB,
    Platform::Doab,
    "DOAB",
    Some(OapenDoab),
    AutomaticPush,
    Assignable,
    Active,
    Profile::OapenDoabSword,
);
descriptor!(
    SCIENCE_OPEN,
    Platform::ScienceOpen,
    "ScienceOpen",
    None,
    Manual,
    Assignable,
    Active,
    Profile::ScienceOpenFtp,
);
descriptor!(
    CAMBRIDGE_UNIVERSITY_LIBRARY,
    Platform::CambridgeUniversityLibrary,
    "Cambridge University Library",
    None,
    AutomaticPush,
    Assignable,
    Active,
    Profile::CulSword,
);
descriptor!(
    CROSSREF,
    Platform::Crossref,
    "Crossref",
    None,
    AutomaticPush,
    Assignable,
    Active,
    Profile::CrossrefDoiDeposit,
);
descriptor!(
    FIGSHARE,
    Platform::Figshare,
    "Figshare",
    None,
    AutomaticPush,
    Assignable,
    Active,
    Profile::FigshareApi,
);
descriptor!(
    ZENODO,
    Platform::Zenodo,
    "Zenodo",
    None,
    AutomaticPush,
    Assignable,
    Active,
    Profile::ZenodoApi,
);
descriptor!(
    PROJECT_MUSE,
    Platform::ProjectMuse,
    "Project MUSE",
    None,
    AutomaticPush,
    Assignable,
    Active,
    Profile::MuseFtp,
);
descriptor!(
    JSTOR,
    Platform::Jstor,
    "JSTOR",
    None,
    AutomaticPush,
    Assignable,
    Active,
    Profile::JstorFtp,
);
descriptor!(
    EBSCO_HOST,
    Platform::EbscoHost,
    "EBSCOHost",
    None,
    AutomaticPush,
    Assignable,
    Active,
    Profile::EbscoHostSftp,
);
descriptor!(
    PROQUEST_EBOOK_CENTRAL,
    Platform::ProquestEbookCentral,
    "ProQuest Ebook Central",
    None,
    AutomaticPush,
    Assignable,
    Active,
    Profile::ProquestEbookCentralFtp,
);
descriptor!(
    GOOGLE_PLAY,
    Platform::GooglePlay,
    "Google Play Books",
    None,
    AutomaticPush,
    Assignable,
    Active,
    Profile::GooglePlayGcs,
);
descriptor!(
    BKCI,
    Platform::Bkci,
    "Book Citation Index",
    None,
    AutomaticPush,
    Assignable,
    Active,
    Profile::BkciFtp,
);
descriptor!(
    OCLC_KB,
    Platform::OclcKb,
    "OCLC Knowledge Base",
    None,
    PullFeed,
    Assignable,
    Active,
    Profile::OclcKbartPublic,
);
descriptor!(
    EX_LIBRIS_KB,
    Platform::ExLibrisKb,
    "Ex Libris Knowledge Base",
    None,
    PullFeed,
    Assignable,
    Active,
    Profile::OclcKbartPublic,
);
descriptor!(
    JISC_NBK,
    Platform::JiscNbk,
    "Jisc NBK",
    None,
    AutomaticPush,
    NonAssignable,
    Inactive,
    Profile::JiscNbkMarcS3,
);

impl DistributionPlatform {
    /// Every destination, in canonical declaration order.
    ///
    /// This order is binding: it is the PostgreSQL enum label order and the
    /// deterministic platform order of `Publisher.distributionPlatforms` and
    /// `distributionPlatformOptions`.
    pub const ALL: [DistributionPlatform; 17] = [
        Platform::InternetArchive,
        Platform::Oapen,
        Platform::Doab,
        Platform::ScienceOpen,
        Platform::CambridgeUniversityLibrary,
        Platform::Crossref,
        Platform::Figshare,
        Platform::Zenodo,
        Platform::ProjectMuse,
        Platform::Jstor,
        Platform::EbscoHost,
        Platform::ProquestEbookCentral,
        Platform::GooglePlay,
        Platform::Bkci,
        Platform::OclcKb,
        Platform::ExLibrisKb,
        Platform::JiscNbk,
    ];

    /// The one static descriptor for this destination.
    ///
    /// The match is compile-time exhaustive with one arm per variant and no
    /// wildcard: adding a destination without a descriptor cannot compile.
    /// The returned reference borrows a `static`, so lookup allocates nothing.
    pub fn descriptor(self) -> &'static DistributionPlatformDescriptor {
        match self {
            Platform::InternetArchive => &INTERNET_ARCHIVE,
            Platform::Oapen => &OAPEN,
            Platform::Doab => &DOAB,
            Platform::ScienceOpen => &SCIENCE_OPEN,
            Platform::CambridgeUniversityLibrary => &CAMBRIDGE_UNIVERSITY_LIBRARY,
            Platform::Crossref => &CROSSREF,
            Platform::Figshare => &FIGSHARE,
            Platform::Zenodo => &ZENODO,
            Platform::ProjectMuse => &PROJECT_MUSE,
            Platform::Jstor => &JSTOR,
            Platform::EbscoHost => &EBSCO_HOST,
            Platform::ProquestEbookCentral => &PROQUEST_EBOOK_CENTRAL,
            Platform::GooglePlay => &GOOGLE_PLAY,
            Platform::Bkci => &BKCI,
            Platform::OclcKb => &OCLC_KB,
            Platform::ExLibrisKb => &EX_LIBRIS_KB,
            Platform::JiscNbk => &JISC_NBK,
        }
    }

    /// The linked group this destination belongs to, if any.
    pub fn linked_group(self) -> Option<DistributionPlatformGroup> {
        self.descriptor().linked_group
    }

    /// Every member of this destination's linked group, in canonical order.
    ///
    /// A destination with no linked group is its own only member.
    pub fn linked_members(self) -> Vec<DistributionPlatform> {
        match self.linked_group() {
            Some(group) => DistributionPlatform::ALL
                .into_iter()
                .filter(|platform| platform.linked_group() == Some(group))
                .collect(),
            None => vec![self],
        }
    }

    /// Whether a publisher assignment may currently be enabled here.
    pub fn is_assignable(self) -> bool {
        self.descriptor().assignment_availability == AssignmentAvailability::Assignable
    }
}

/// The publicly exposed projection of one platform descriptor.
///
/// This is derived entirely from code-owned descriptors and reads no database.
/// It carries only the destination, its label, its linked group, its
/// back-catalogue behaviour and whether it may currently be assigned. Mechanism
/// readiness and adapter/feed identity are internal and deliberately absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DistributionPlatformOption {
    pub platform: DistributionPlatform,
    pub display_label: &'static str,
    pub linked_group: Option<DistributionPlatformGroup>,
    pub back_catalogue_behaviour: BackCatalogueBehaviour,
    pub assignable: bool,
}

impl DistributionPlatformOption {
    /// Every destination option, in canonical declaration order.
    pub fn all() -> Vec<DistributionPlatformOption> {
        DistributionPlatform::ALL
            .into_iter()
            .map(DistributionPlatformOption::from)
            .collect()
    }
}

impl From<DistributionPlatform> for DistributionPlatformOption {
    fn from(platform: DistributionPlatform) -> Self {
        let descriptor = platform.descriptor();
        Self {
            platform,
            display_label: descriptor.display_label,
            linked_group: descriptor.linked_group,
            back_catalogue_behaviour: descriptor.back_catalogue_behaviour,
            assignable: descriptor.assignment_availability == AssignmentAvailability::Assignable,
        }
    }
}

/// The publicly exposed projection of one enabled assignment.
///
/// This carries only the destination and the timestamp at which the current
/// activation began. `activation_id`, `disabled_at`, retained history,
/// package/capability state and adapter identity are deliberately absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublisherDistributionPlatformAssignment {
    pub platform: DistributionPlatform,
    pub enabled_at: Timestamp,
}

/// One persisted publisher/destination assignment row.
///
/// A row exists only because the destination has been enabled at least once.
/// Disabled rows are retained rather than deleted, so `enabled` is true if and
/// only if `disabled_at IS NULL` — an invariant the database enforces through
/// `publisher_distribution_platform_enabled_state_check`.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublisherDistributionPlatform {
    pub publisher_id: Uuid,
    pub platform: DistributionPlatform,
    pub enabled: bool,
    pub activation_id: Uuid,
    pub enabled_at: Timestamp,
    pub disabled_at: Option<Timestamp>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl PublisherDistributionPlatform {
    /// The public projection of this row.
    pub fn to_assignment(&self) -> PublisherDistributionPlatformAssignment {
        PublisherDistributionPlatformAssignment {
            platform: self.platform,
            enabled_at: self.enabled_at,
        }
    }
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(all(test, feature = "backend"))]
mod tests;
