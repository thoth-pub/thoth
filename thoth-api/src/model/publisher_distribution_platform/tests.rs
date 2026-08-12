use std::str::FromStr;
use std::sync::Arc;

use diesel::sql_query;
use diesel::RunQueryDsl;
use uuid::Uuid;

use super::*;
use crate::db::PgPool;
use crate::model::tests::db as test_db;
use thoth_errors::ThothError;

// --------------------------------------------------------------------------
// Inventory and representation
// --------------------------------------------------------------------------

/// The approved inventory, in canonical declaration order.
const CANONICAL_CODES: [&str; 17] = [
    "INTERNET_ARCHIVE",
    "OAPEN",
    "DOAB",
    "SCIENCE_OPEN",
    "CAMBRIDGE_UNIVERSITY_LIBRARY",
    "CROSSREF",
    "FIGSHARE",
    "ZENODO",
    "PROJECT_MUSE",
    "JSTOR",
    "EBSCO_HOST",
    "PROQUEST_EBOOK_CENTRAL",
    "GOOGLE_PLAY",
    "BKCI",
    "OCLC_KB",
    "EX_LIBRIS_KB",
    "JISC_NBK",
];

#[test]
fn inventory_is_exactly_seventeen_values_in_canonical_order() {
    assert_eq!(DistributionPlatform::ALL.len(), 17);
    let codes: Vec<String> = DistributionPlatform::ALL
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(codes, CANONICAL_CODES);
}

#[test]
fn inventory_contains_no_fallback_or_excluded_value() {
    let codes: Vec<String> = DistributionPlatform::ALL
        .iter()
        .map(ToString::to_string)
        .collect();
    for forbidden in [
        "OTHER",
        "UNKNOWN",
        "PROVISIONAL",
        // The ten ADR-0004 exclusions.
        "EBSCO_KB",
        "PROQUEST_SERIALS_SOLUTIONS_KB",
        "OVERDRIVE",
        "BDS_LIVE",
        "RNIB_BOOKSHARE",
        "SCIELO_BOOKS",
        "ZOTERO",
        "THOTH",
        "PUBLISHER_WEBSITE",
    ] {
        assert!(
            !codes.iter().any(|code| code == forbidden),
            "{forbidden} must not be a DistributionPlatform variant"
        );
    }
}

#[test]
fn string_conversion_round_trips_and_rejects_unknown_values() {
    for platform in DistributionPlatform::ALL {
        let code = platform.to_string();
        assert_eq!(
            DistributionPlatform::from_str(&code).expect("known code"),
            platform
        );
    }
    for unknown in ["OTHER", "", "oapen", "Oapen", "SOMETHING_ELSE"] {
        assert!(
            DistributionPlatform::from_str(unknown).is_err(),
            "{unknown} must not parse"
        );
    }
}

#[test]
fn serde_round_trips_and_rejects_unknown_values() {
    for platform in DistributionPlatform::ALL {
        let json = serde_json::to_string(&platform).expect("serialize");
        assert_eq!(json, format!("\"{platform}\""));
        let parsed: DistributionPlatform = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, platform);
    }
    assert!(serde_json::from_str::<DistributionPlatform>("\"OTHER\"").is_err());
    assert!(serde_json::from_str::<DistributionPlatform>("\"\"").is_err());
}

#[test]
fn there_is_no_default_distribution_platform() {
    // A default destination would let an unset or unrecognised value silently
    // resolve to a real distribution target, so the enum must neither derive
    // nor implement `Default`.
    let source = include_str!("mod.rs");
    assert!(!source.contains("impl Default for DistributionPlatform"));
    let (before_enum, _) = source
        .split_once("pub enum DistributionPlatform {")
        .expect("enum declaration");
    let derives = before_enum
        .rsplit_once("#[derive(")
        .expect("derive attribute")
        .1;
    assert!(
        !derives.contains("Default"),
        "DistributionPlatform must not derive Default"
    );
}

#[test]
fn distribution_platform_declares_no_metric_platform_conversion() {
    // `DistributionPlatform` and Thoth Metrics' platform vocabulary are
    // deliberately separate domains: no shared universal enum and no
    // name-based conversion.
    let source = include_str!("mod.rs");
    assert!(!source.contains("impl From<DistributionPlatform> for MetricPlatform"));
    assert!(!source.contains("impl From<MetricPlatform> for DistributionPlatform"));
}

// --------------------------------------------------------------------------
// Descriptors
// --------------------------------------------------------------------------

/// The binding descriptor table of specification section 8.
#[allow(clippy::type_complexity)]
const APPROVED_DESCRIPTORS: [(
    DistributionPlatform,
    &str,
    BackCatalogueBehaviour,
    Option<DistributionPlatformGroup>,
    DistributionAdapterProfile,
    MechanismReadiness,
    AssignmentAvailability,
); 17] = [
    (
        DistributionPlatform::InternetArchive,
        "Internet Archive",
        BackCatalogueBehaviour::AutomaticPush,
        None,
        DistributionAdapterProfile::IaApi,
        MechanismReadiness::Active,
        AssignmentAvailability::Assignable,
    ),
    (
        DistributionPlatform::Oapen,
        "OAPEN",
        BackCatalogueBehaviour::AutomaticPush,
        Some(DistributionPlatformGroup::OapenDoab),
        DistributionAdapterProfile::OapenDoabSword,
        MechanismReadiness::Active,
        AssignmentAvailability::Assignable,
    ),
    (
        DistributionPlatform::Doab,
        "DOAB",
        BackCatalogueBehaviour::AutomaticPush,
        Some(DistributionPlatformGroup::OapenDoab),
        DistributionAdapterProfile::OapenDoabSword,
        MechanismReadiness::Active,
        AssignmentAvailability::Assignable,
    ),
    (
        DistributionPlatform::ScienceOpen,
        "ScienceOpen",
        BackCatalogueBehaviour::Manual,
        None,
        DistributionAdapterProfile::ScienceOpenFtp,
        MechanismReadiness::Active,
        AssignmentAvailability::Assignable,
    ),
    (
        DistributionPlatform::CambridgeUniversityLibrary,
        "Cambridge University Library",
        BackCatalogueBehaviour::AutomaticPush,
        None,
        DistributionAdapterProfile::CulSword,
        MechanismReadiness::Active,
        AssignmentAvailability::Assignable,
    ),
    (
        DistributionPlatform::Crossref,
        "Crossref",
        BackCatalogueBehaviour::AutomaticPush,
        None,
        DistributionAdapterProfile::CrossrefDoiDeposit,
        MechanismReadiness::Active,
        AssignmentAvailability::Assignable,
    ),
    (
        DistributionPlatform::Figshare,
        "Figshare",
        BackCatalogueBehaviour::AutomaticPush,
        None,
        DistributionAdapterProfile::FigshareApi,
        MechanismReadiness::Active,
        AssignmentAvailability::Assignable,
    ),
    (
        DistributionPlatform::Zenodo,
        "Zenodo",
        BackCatalogueBehaviour::AutomaticPush,
        None,
        DistributionAdapterProfile::ZenodoApi,
        MechanismReadiness::Active,
        AssignmentAvailability::Assignable,
    ),
    (
        DistributionPlatform::ProjectMuse,
        "Project MUSE",
        BackCatalogueBehaviour::AutomaticPush,
        None,
        DistributionAdapterProfile::MuseFtp,
        MechanismReadiness::Active,
        AssignmentAvailability::Assignable,
    ),
    (
        DistributionPlatform::Jstor,
        "JSTOR",
        BackCatalogueBehaviour::AutomaticPush,
        None,
        DistributionAdapterProfile::JstorFtp,
        MechanismReadiness::Active,
        AssignmentAvailability::Assignable,
    ),
    (
        DistributionPlatform::EbscoHost,
        "EBSCOHost",
        BackCatalogueBehaviour::AutomaticPush,
        None,
        DistributionAdapterProfile::EbscoHostSftp,
        MechanismReadiness::Active,
        AssignmentAvailability::Assignable,
    ),
    (
        DistributionPlatform::ProquestEbookCentral,
        "ProQuest Ebook Central",
        BackCatalogueBehaviour::AutomaticPush,
        None,
        DistributionAdapterProfile::ProquestEbookCentralFtp,
        MechanismReadiness::Active,
        AssignmentAvailability::Assignable,
    ),
    (
        DistributionPlatform::GooglePlay,
        "Google Play Books",
        BackCatalogueBehaviour::AutomaticPush,
        None,
        DistributionAdapterProfile::GooglePlayGcs,
        MechanismReadiness::Active,
        AssignmentAvailability::Assignable,
    ),
    (
        DistributionPlatform::Bkci,
        "Book Citation Index",
        BackCatalogueBehaviour::AutomaticPush,
        None,
        DistributionAdapterProfile::BkciFtp,
        MechanismReadiness::Active,
        AssignmentAvailability::Assignable,
    ),
    (
        DistributionPlatform::OclcKb,
        "OCLC Knowledge Base",
        BackCatalogueBehaviour::PullFeed,
        None,
        DistributionAdapterProfile::OclcKbartPublic,
        MechanismReadiness::Active,
        AssignmentAvailability::Assignable,
    ),
    (
        DistributionPlatform::ExLibrisKb,
        "Ex Libris Knowledge Base",
        BackCatalogueBehaviour::PullFeed,
        None,
        DistributionAdapterProfile::OclcKbartPublic,
        MechanismReadiness::Active,
        AssignmentAvailability::Assignable,
    ),
    (
        DistributionPlatform::JiscNbk,
        "Jisc NBK",
        BackCatalogueBehaviour::AutomaticPush,
        None,
        DistributionAdapterProfile::JiscNbkMarcS3,
        MechanismReadiness::Inactive,
        AssignmentAvailability::NonAssignable,
    ),
];

#[test]
fn every_platform_has_exactly_one_descriptor_matching_the_approved_table() {
    assert_eq!(APPROVED_DESCRIPTORS.len(), DistributionPlatform::ALL.len());
    for (index, platform) in DistributionPlatform::ALL.into_iter().enumerate() {
        let (expected_platform, label, behaviour, group, profile, readiness, availability) =
            APPROVED_DESCRIPTORS[index];
        assert_eq!(platform, expected_platform, "descriptor table order");
        let descriptor = platform.descriptor();
        assert_eq!(descriptor.platform, platform);
        assert_eq!(descriptor.display_label, label);
        assert_eq!(descriptor.back_catalogue_behaviour, behaviour);
        assert_eq!(descriptor.linked_group, group);
        assert_eq!(descriptor.adapter_profile, profile);
        assert_eq!(descriptor.mechanism_readiness, readiness);
        assert_eq!(descriptor.assignment_availability, availability);
    }
}

#[test]
fn descriptor_lookup_returns_the_same_static_without_allocating() {
    for platform in DistributionPlatform::ALL {
        let first = platform.descriptor();
        let second = platform.descriptor();
        assert!(
            std::ptr::eq(first, second),
            "{platform} descriptor must be one static value"
        );
    }
}

#[test]
fn oapen_and_doab_are_separate_platforms_in_one_linked_group() {
    assert_ne!(DistributionPlatform::Oapen, DistributionPlatform::Doab);
    for platform in [DistributionPlatform::Oapen, DistributionPlatform::Doab] {
        assert_eq!(
            platform.linked_group(),
            Some(DistributionPlatformGroup::OapenDoab)
        );
        assert_eq!(
            platform.linked_members(),
            vec![DistributionPlatform::Oapen, DistributionPlatform::Doab]
        );
    }
    assert_eq!(
        DistributionPlatform::Oapen.descriptor().adapter_profile,
        DistributionPlatform::Doab.descriptor().adapter_profile
    );
}

#[test]
fn oclc_and_ex_libris_share_a_profile_but_are_not_linked() {
    assert_eq!(
        DistributionPlatform::OclcKb.descriptor().adapter_profile,
        DistributionPlatform::ExLibrisKb
            .descriptor()
            .adapter_profile
    );
    for platform in [
        DistributionPlatform::OclcKb,
        DistributionPlatform::ExLibrisKb,
    ] {
        assert_eq!(platform.linked_group(), None);
        assert_eq!(platform.linked_members(), vec![platform]);
    }
}

#[test]
fn only_oapen_and_doab_belong_to_a_linked_group() {
    let linked: Vec<DistributionPlatform> = DistributionPlatform::ALL
        .into_iter()
        .filter(|platform| platform.linked_group().is_some())
        .collect();
    assert_eq!(
        linked,
        vec![DistributionPlatform::Oapen, DistributionPlatform::Doab]
    );
}

#[test]
fn jisc_nbk_is_the_only_inactive_non_assignable_platform() {
    let non_assignable: Vec<DistributionPlatform> = DistributionPlatform::ALL
        .into_iter()
        .filter(|platform| !platform.is_assignable())
        .collect();
    assert_eq!(non_assignable, vec![DistributionPlatform::JiscNbk]);
    let descriptor = DistributionPlatform::JiscNbk.descriptor();
    assert_eq!(descriptor.mechanism_readiness, MechanismReadiness::Inactive);
    assert_eq!(
        descriptor.assignment_availability,
        AssignmentAvailability::NonAssignable
    );
}

#[test]
fn options_expose_descriptor_metadata_in_canonical_order() {
    let options = DistributionPlatformOption::all();
    assert_eq!(options.len(), 17);
    for (index, platform) in DistributionPlatform::ALL.into_iter().enumerate() {
        let option = options[index];
        let descriptor = platform.descriptor();
        assert_eq!(option.platform, platform);
        assert_eq!(option.display_label, descriptor.display_label);
        assert_eq!(option.linked_group, descriptor.linked_group);
        assert_eq!(
            option.back_catalogue_behaviour,
            descriptor.back_catalogue_behaviour
        );
        assert_eq!(option.assignable, platform.is_assignable());
    }
}

// --------------------------------------------------------------------------
// Database-backed lifecycle
// --------------------------------------------------------------------------

fn row(
    pool: &PgPool,
    publisher_id: Uuid,
    platform: DistributionPlatform,
) -> Option<PublisherDistributionPlatform> {
    PublisherDistributionPlatform::all_for_publisher(pool, publisher_id)
        .expect("load assignments")
        .into_iter()
        .find(|row| row.platform == platform)
}

fn enabled_row(
    pool: &PgPool,
    publisher_id: Uuid,
    platform: DistributionPlatform,
) -> PublisherDistributionPlatform {
    let row = row(pool, publisher_id, platform).expect("assignment row");
    assert!(row.enabled);
    assert!(row.disabled_at.is_none());
    row
}

#[test]
fn absent_to_enabled_inserts_one_activated_row() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);

    PublisherDistributionPlatform::enable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::Crossref,
    )
    .expect("enable");

    let row = enabled_row(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::Crossref,
    );
    assert_eq!(row.publisher_id, publisher.publisher_id);
    assert_eq!(row.enabled_at, row.created_at);
    assert_eq!(row.enabled_at, row.updated_at);
    assert_eq!(
        PublisherDistributionPlatform::all_for_publisher(&pool, publisher.publisher_id)
            .expect("load")
            .len(),
        1
    );
}

#[test]
fn enabled_to_enabled_is_an_idempotent_no_op_that_moves_no_timestamp() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let platform = DistributionPlatform::Zenodo;

    PublisherDistributionPlatform::enable(&pool, publisher.publisher_id, platform)
        .expect("first enable");
    let before = enabled_row(&pool, publisher.publisher_id, platform);

    PublisherDistributionPlatform::enable(&pool, publisher.publisher_id, platform)
        .expect("second enable");
    let after = enabled_row(&pool, publisher.publisher_id, platform);

    assert_eq!(after, before);
}

#[test]
fn enabled_to_disabled_retains_the_row_and_its_activation() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let platform = DistributionPlatform::Figshare;

    PublisherDistributionPlatform::enable(&pool, publisher.publisher_id, platform).expect("enable");
    let before = enabled_row(&pool, publisher.publisher_id, platform);

    PublisherDistributionPlatform::disable(&pool, publisher.publisher_id, platform)
        .expect("disable");
    let after = row(&pool, publisher.publisher_id, platform).expect("retained row");

    assert!(!after.enabled);
    assert!(after.disabled_at.is_some());
    assert_eq!(after.activation_id, before.activation_id);
    assert_eq!(after.enabled_at, before.enabled_at);
    assert_eq!(after.created_at, before.created_at);
    assert!(after.updated_at >= before.updated_at);
}

#[test]
fn disabled_to_disabled_is_an_idempotent_no_op_that_moves_no_timestamp() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let platform = DistributionPlatform::Jstor;

    PublisherDistributionPlatform::enable(&pool, publisher.publisher_id, platform).expect("enable");
    PublisherDistributionPlatform::disable(&pool, publisher.publisher_id, platform)
        .expect("first disable");
    let before = row(&pool, publisher.publisher_id, platform).expect("row");

    PublisherDistributionPlatform::disable(&pool, publisher.publisher_id, platform)
        .expect("second disable");
    let after = row(&pool, publisher.publisher_id, platform).expect("row");

    assert_eq!(after, before);
}

#[test]
fn disabled_to_enabled_creates_a_new_activation_on_the_retained_row() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let platform = DistributionPlatform::Bkci;

    PublisherDistributionPlatform::enable(&pool, publisher.publisher_id, platform).expect("enable");
    let first = enabled_row(&pool, publisher.publisher_id, platform);
    PublisherDistributionPlatform::disable(&pool, publisher.publisher_id, platform)
        .expect("disable");
    PublisherDistributionPlatform::enable(&pool, publisher.publisher_id, platform)
        .expect("re-enable");
    let second = enabled_row(&pool, publisher.publisher_id, platform);

    assert_ne!(second.activation_id, first.activation_id);
    assert!(second.enabled_at >= first.enabled_at);
    assert!(second.disabled_at.is_none());
    assert_eq!(second.created_at, first.created_at);
    assert_eq!(
        PublisherDistributionPlatform::all_for_publisher(&pool, publisher.publisher_id)
            .expect("load")
            .len(),
        1,
        "re-enable must reuse the retained row, not add a second one"
    );
}

#[test]
fn absent_to_disabled_succeeds_without_creating_a_never_activated_row() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);

    PublisherDistributionPlatform::disable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::GooglePlay,
    )
    .expect("disable absent");

    assert!(
        PublisherDistributionPlatform::all_for_publisher(&pool, publisher.publisher_id)
            .expect("load")
            .is_empty()
    );
}

#[test]
fn transition_sequence_never_violates_the_row_invariant() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let platform = DistributionPlatform::InternetArchive;

    for _ in 0..3 {
        PublisherDistributionPlatform::enable(&pool, publisher.publisher_id, platform)
            .expect("enable");
        PublisherDistributionPlatform::enable(&pool, publisher.publisher_id, platform)
            .expect("enable again");
        PublisherDistributionPlatform::disable(&pool, publisher.publisher_id, platform)
            .expect("disable");
        PublisherDistributionPlatform::disable(&pool, publisher.publisher_id, platform)
            .expect("disable again");
    }

    let mut connection = pool.get().expect("connection");
    let violations = sql_query(
        "SELECT count(*) AS count FROM publisher_distribution_platform \
         WHERE enabled <> (disabled_at IS NULL)",
    )
    .get_result::<CountRow>(&mut connection)
    .expect("invariant query");
    assert_eq!(violations.count, 0);
}

#[derive(diesel::QueryableByName)]
struct CountRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

#[test]
fn database_rejects_rows_whose_enabled_flag_contradicts_disabled_at() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let mut connection = pool.get().expect("connection");

    let enabled_but_disabled_at = sql_query(format!(
        "INSERT INTO publisher_distribution_platform \
         (publisher_id, platform, enabled, activation_id, enabled_at, disabled_at) \
         VALUES ('{}', 'OAPEN', true, gen_random_uuid(), now(), now())",
        publisher.publisher_id
    ))
    .execute(&mut connection);
    assert!(enabled_but_disabled_at.is_err());

    let disabled_without_disabled_at = sql_query(format!(
        "INSERT INTO publisher_distribution_platform \
         (publisher_id, platform, enabled, activation_id, enabled_at, disabled_at) \
         VALUES ('{}', 'OAPEN', false, gen_random_uuid(), now(), NULL)",
        publisher.publisher_id
    ))
    .execute(&mut connection);
    assert!(disabled_without_disabled_at.is_err());
}

#[test]
fn deleting_a_publisher_cascades_to_its_assignments() {
    use crate::model::Crud;
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    PublisherDistributionPlatform::enable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::Oapen,
    )
    .expect("enable");
    assert_eq!(
        PublisherDistributionPlatform::all_for_publisher(&pool, publisher.publisher_id)
            .expect("load")
            .len(),
        2,
        "OAPEN enables the linked pair"
    );

    let publisher_id = publisher.publisher_id;
    publisher.delete(&pool).expect("delete");

    assert!(
        PublisherDistributionPlatform::all_for_publisher(&pool, publisher_id)
            .expect("load")
            .is_empty()
    );
}

#[test]
fn enabling_a_platform_for_an_unknown_publisher_fails() {
    let (_guard, pool) = test_db::setup_test_db();
    let error = PublisherDistributionPlatform::enable(
        &pool,
        Uuid::new_v4(),
        DistributionPlatform::Crossref,
    )
    .expect_err("unknown publisher");
    assert_eq!(error, ThothError::EntityNotFound);
}

#[test]
fn transitions_for_different_publishers_are_independent() {
    let (_guard, pool) = test_db::setup_test_db();
    let first = test_db::create_publisher(&pool);
    let second = test_db::create_publisher(&pool);

    PublisherDistributionPlatform::enable(
        &pool,
        first.publisher_id,
        DistributionPlatform::Crossref,
    )
    .expect("enable first");

    assert_eq!(
        PublisherDistributionPlatform::enabled_assignments(&pool, first.publisher_id)
            .expect("load first")
            .len(),
        1
    );
    assert!(
        PublisherDistributionPlatform::enabled_assignments(&pool, second.publisher_id)
            .expect("load second")
            .is_empty()
    );
}

#[test]
fn concurrent_enables_of_one_platform_produce_one_activation() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let platform = DistributionPlatform::Zenodo;

    let handles: Vec<_> = (0..4)
        .map(|_| {
            let pool = Arc::clone(&pool);
            let publisher_id = publisher.publisher_id;
            std::thread::spawn(move || {
                PublisherDistributionPlatform::enable(&pool, publisher_id, platform)
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("thread").expect("enable");
    }

    let rows = PublisherDistributionPlatform::all_for_publisher(&pool, publisher.publisher_id)
        .expect("load");
    assert_eq!(rows.len(), 1);
    assert!(rows[0].enabled);
}

#[test]
fn concurrent_enable_and_disable_serialize_to_a_valid_state() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let platform = DistributionPlatform::ProjectMuse;
    PublisherDistributionPlatform::enable(&pool, publisher.publisher_id, platform)
        .expect("seed enable");

    let enabling = {
        let pool = Arc::clone(&pool);
        let publisher_id = publisher.publisher_id;
        std::thread::spawn(move || {
            PublisherDistributionPlatform::enable(&pool, publisher_id, platform)
        })
    };
    let disabling = {
        let pool = Arc::clone(&pool);
        let publisher_id = publisher.publisher_id;
        std::thread::spawn(move || {
            PublisherDistributionPlatform::disable(&pool, publisher_id, platform)
        })
    };
    enabling.join().expect("thread").expect("enable");
    disabling.join().expect("thread").expect("disable");

    let row = row(&pool, publisher.publisher_id, platform).expect("row");
    assert_eq!(row.enabled, row.disabled_at.is_none());
}

// --------------------------------------------------------------------------
// Linked OAPEN/DOAB normalization
// --------------------------------------------------------------------------

fn linked_pair(
    pool: &PgPool,
    publisher_id: Uuid,
) -> (PublisherDistributionPlatform, PublisherDistributionPlatform) {
    (
        row(pool, publisher_id, DistributionPlatform::Oapen).expect("OAPEN row"),
        row(pool, publisher_id, DistributionPlatform::Doab).expect("DOAB row"),
    )
}

fn assert_normalized_enabled(
    oapen: &PublisherDistributionPlatform,
    doab: &PublisherDistributionPlatform,
) {
    assert!(oapen.enabled && doab.enabled);
    assert!(oapen.disabled_at.is_none() && doab.disabled_at.is_none());
    assert_eq!(oapen.activation_id, doab.activation_id);
    assert_eq!(oapen.enabled_at, doab.enabled_at);
}

#[test]
fn enabling_either_linked_member_enables_both_with_one_shared_activation() {
    let (_guard, pool) = test_db::setup_test_db();
    for platform in [DistributionPlatform::Oapen, DistributionPlatform::Doab] {
        let publisher = test_db::create_publisher(&pool);
        PublisherDistributionPlatform::enable(&pool, publisher.publisher_id, platform)
            .expect("linked enable");
        let (oapen, doab) = linked_pair(&pool, publisher.publisher_id);
        assert_normalized_enabled(&oapen, &doab);
    }
}

#[test]
fn disabling_either_linked_member_disables_both() {
    let (_guard, pool) = test_db::setup_test_db();
    for platform in [DistributionPlatform::Oapen, DistributionPlatform::Doab] {
        let publisher = test_db::create_publisher(&pool);
        PublisherDistributionPlatform::enable(
            &pool,
            publisher.publisher_id,
            DistributionPlatform::Oapen,
        )
        .expect("linked enable");
        PublisherDistributionPlatform::disable(&pool, publisher.publisher_id, platform)
            .expect("linked disable");
        let (oapen, doab) = linked_pair(&pool, publisher.publisher_id);
        assert!(!oapen.enabled && !doab.enabled);
        assert!(oapen.disabled_at.is_some() && doab.disabled_at.is_some());
        assert_eq!(oapen.disabled_at, doab.disabled_at);
        assert_eq!(oapen.activation_id, doab.activation_id);
    }
}

#[test]
fn linked_enable_is_a_no_op_only_when_the_pair_is_normalized_fully_enabled() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    PublisherDistributionPlatform::enable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::Oapen,
    )
    .expect("linked enable");
    let before = linked_pair(&pool, publisher.publisher_id);

    PublisherDistributionPlatform::enable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::Doab,
    )
    .expect("second linked enable");
    let after = linked_pair(&pool, publisher.publisher_id);

    assert_eq!(after, before, "normalized pair must not be rewritten");
}

#[test]
fn linked_disable_is_a_no_op_when_no_member_is_enabled() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    PublisherDistributionPlatform::enable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::Oapen,
    )
    .expect("enable");
    PublisherDistributionPlatform::disable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::Oapen,
    )
    .expect("disable");
    let before = linked_pair(&pool, publisher.publisher_id);

    PublisherDistributionPlatform::disable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::Doab,
    )
    .expect("second disable");
    let after = linked_pair(&pool, publisher.publisher_id);

    assert_eq!(after, before);
}

/// Force a one-sided linked state that the supported domain path cannot
/// produce, so repair behaviour can be proven.
fn write_raw_assignment(
    pool: &PgPool,
    publisher_id: Uuid,
    platform: &str,
    activation_id: Uuid,
    enabled_at_sql: &str,
) {
    let mut connection = pool.get().expect("connection");
    sql_query(format!(
        "INSERT INTO publisher_distribution_platform \
         (publisher_id, platform, enabled, activation_id, enabled_at, disabled_at) \
         VALUES ('{publisher_id}', '{platform}', true, '{activation_id}', {enabled_at_sql}, NULL) \
         ON CONFLICT (publisher_id, platform) DO UPDATE SET \
         enabled = true, activation_id = EXCLUDED.activation_id, \
         enabled_at = EXCLUDED.enabled_at, disabled_at = NULL"
    ))
    .execute(&mut connection)
    .expect("raw assignment write");
}

#[test]
fn linked_enable_repairs_a_one_sided_pair() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    write_raw_assignment(
        &pool,
        publisher.publisher_id,
        "OAPEN",
        Uuid::new_v4(),
        "now()",
    );
    assert!(row(&pool, publisher.publisher_id, DistributionPlatform::Doab).is_none());

    PublisherDistributionPlatform::enable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::Oapen,
    )
    .expect("repairing enable");

    let (oapen, doab) = linked_pair(&pool, publisher.publisher_id);
    assert_normalized_enabled(&oapen, &doab);
}

#[test]
fn linked_enable_repairs_a_disabled_member() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    PublisherDistributionPlatform::enable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::Oapen,
    )
    .expect("enable");
    let mut connection = pool.get().expect("connection");
    sql_query(format!(
        "UPDATE publisher_distribution_platform SET enabled = false, disabled_at = now() \
         WHERE publisher_id = '{}' AND platform = 'DOAB'",
        publisher.publisher_id
    ))
    .execute(&mut connection)
    .expect("force one-sided disable");

    PublisherDistributionPlatform::enable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::Doab,
    )
    .expect("repairing enable");

    let (oapen, doab) = linked_pair(&pool, publisher.publisher_id);
    assert_normalized_enabled(&oapen, &doab);
}

#[test]
fn linked_enable_normalizes_a_split_activation_pair() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let activation_a = Uuid::new_v4();
    let activation_b = Uuid::new_v4();
    assert_ne!(activation_a, activation_b);
    write_raw_assignment(
        &pool,
        publisher.publisher_id,
        "OAPEN",
        activation_a,
        "now()",
    );
    write_raw_assignment(&pool, publisher.publisher_id, "DOAB", activation_b, "now()");

    PublisherDistributionPlatform::enable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::Oapen,
    )
    .expect("normalizing enable");

    let (oapen, doab) = linked_pair(&pool, publisher.publisher_id);
    assert_normalized_enabled(&oapen, &doab);
    assert_ne!(
        oapen.activation_id, activation_a,
        "normalization must generate one new activation"
    );
    assert_ne!(oapen.activation_id, activation_b);
}

#[test]
fn linked_enable_normalizes_a_split_enabled_at_pair() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let shared_activation = Uuid::new_v4();
    write_raw_assignment(
        &pool,
        publisher.publisher_id,
        "OAPEN",
        shared_activation,
        "now() - interval '1 day'",
    );
    write_raw_assignment(
        &pool,
        publisher.publisher_id,
        "DOAB",
        shared_activation,
        "now()",
    );
    let before = linked_pair(&pool, publisher.publisher_id);
    assert_eq!(before.0.activation_id, before.1.activation_id);
    assert_ne!(before.0.enabled_at, before.1.enabled_at);

    PublisherDistributionPlatform::enable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::Doab,
    )
    .expect("normalizing enable");

    let (oapen, doab) = linked_pair(&pool, publisher.publisher_id);
    assert_normalized_enabled(&oapen, &doab);
    assert_ne!(oapen.activation_id, shared_activation);
}

#[test]
fn a_failure_writing_the_second_linked_row_rolls_the_whole_transition_back() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let mut connection = pool.get().expect("connection");

    // Inject a backend failure that only fires for the second member of the
    // linked pair, so the first row's write is already in the transaction when
    // the transition fails.
    sql_query(
        "CREATE OR REPLACE FUNCTION be02_reject_doab() RETURNS trigger AS $$ \
         BEGIN IF NEW.platform = 'DOAB' THEN \
         RAISE EXCEPTION 'injected second-row failure'; END IF; RETURN NEW; END; \
         $$ LANGUAGE plpgsql",
    )
    .execute(&mut connection)
    .expect("create injection function");
    sql_query(
        "CREATE TRIGGER be02_reject_doab BEFORE INSERT OR UPDATE \
         ON publisher_distribution_platform FOR EACH ROW EXECUTE FUNCTION be02_reject_doab()",
    )
    .execute(&mut connection)
    .expect("create injection trigger");

    let outcome = PublisherDistributionPlatform::enable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::Oapen,
    );

    sql_query("DROP TRIGGER be02_reject_doab ON publisher_distribution_platform")
        .execute(&mut connection)
        .expect("drop injection trigger");
    sql_query("DROP FUNCTION be02_reject_doab()")
        .execute(&mut connection)
        .expect("drop injection function");

    assert!(outcome.is_err(), "injected failure must propagate");
    assert!(
        PublisherDistributionPlatform::all_for_publisher(&pool, publisher.publisher_id)
            .expect("load")
            .is_empty(),
        "no one-sided linked state may survive a failed transition"
    );
}

#[test]
fn concurrent_linked_enables_produce_one_shared_group_activation() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);

    let handles: Vec<_> = [
        DistributionPlatform::Oapen,
        DistributionPlatform::Doab,
        DistributionPlatform::Oapen,
        DistributionPlatform::Doab,
    ]
    .into_iter()
    .map(|platform| {
        let pool = Arc::clone(&pool);
        let publisher_id = publisher.publisher_id;
        std::thread::spawn(move || {
            PublisherDistributionPlatform::enable(&pool, publisher_id, platform)
        })
    })
    .collect();
    for handle in handles {
        handle.join().expect("thread").expect("enable");
    }

    let (oapen, doab) = linked_pair(&pool, publisher.publisher_id);
    assert_normalized_enabled(&oapen, &doab);
    assert_eq!(
        PublisherDistributionPlatform::all_for_publisher(&pool, publisher.publisher_id)
            .expect("load")
            .len(),
        2
    );
}

// --------------------------------------------------------------------------
// OCLC / Ex Libris independence and JISC NBK
// --------------------------------------------------------------------------

#[test]
fn oclc_and_ex_libris_assignments_and_activations_are_independent() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);

    PublisherDistributionPlatform::enable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::OclcKb,
    )
    .expect("enable OCLC");
    assert!(row(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::ExLibrisKb
    )
    .is_none());

    PublisherDistributionPlatform::enable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::ExLibrisKb,
    )
    .expect("enable Ex Libris");
    let oclc = enabled_row(&pool, publisher.publisher_id, DistributionPlatform::OclcKb);
    let ex_libris = enabled_row(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::ExLibrisKb,
    );
    assert_ne!(oclc.activation_id, ex_libris.activation_id);

    PublisherDistributionPlatform::disable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::OclcKb,
    )
    .expect("disable OCLC");
    assert!(
        !row(&pool, publisher.publisher_id, DistributionPlatform::OclcKb)
            .expect("row")
            .enabled
    );
    assert!(
        enabled_row(
            &pool,
            publisher.publisher_id,
            DistributionPlatform::ExLibrisKb
        )
        .enabled
    );
}

#[test]
fn enabling_jisc_nbk_fails_closed_before_any_write() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);

    let error = PublisherDistributionPlatform::enable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::JiscNbk,
    )
    .expect_err("JISC NBK is not assignable");

    assert_eq!(
        error,
        ThothError::DistributionPlatformNotAssignable("JISC_NBK".to_string())
    );
    assert!(
        PublisherDistributionPlatform::all_for_publisher(&pool, publisher.publisher_id)
            .expect("load")
            .is_empty(),
        "a rejected enable must write nothing"
    );
    assert!(
        PublisherDistributionPlatform::enabled_assignments(&pool, publisher.publisher_id)
            .expect("load")
            .is_empty()
    );
}

// --------------------------------------------------------------------------
// Read paths
// --------------------------------------------------------------------------

#[test]
fn enabled_assignments_exclude_disabled_rows_and_use_canonical_order() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);

    for platform in [
        DistributionPlatform::Zenodo,
        DistributionPlatform::InternetArchive,
        DistributionPlatform::Oapen,
        DistributionPlatform::Crossref,
    ] {
        PublisherDistributionPlatform::enable(&pool, publisher.publisher_id, platform)
            .expect("enable");
    }
    PublisherDistributionPlatform::disable(
        &pool,
        publisher.publisher_id,
        DistributionPlatform::Crossref,
    )
    .expect("disable");

    let platforms: Vec<DistributionPlatform> =
        PublisherDistributionPlatform::enabled_assignments(&pool, publisher.publisher_id)
            .expect("load")
            .into_iter()
            .map(|assignment| assignment.platform)
            .collect();

    assert_eq!(
        platforms,
        vec![
            DistributionPlatform::InternetArchive,
            DistributionPlatform::Oapen,
            DistributionPlatform::Doab,
            DistributionPlatform::Zenodo,
        ]
    );
}

#[test]
fn a_publisher_with_no_assignments_loads_an_empty_vector() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    assert!(
        PublisherDistributionPlatform::enabled_assignments(&pool, publisher.publisher_id)
            .expect("load")
            .is_empty()
    );
}
