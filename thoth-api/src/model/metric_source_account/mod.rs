//! Metric source accounts (`MET-WP1-02`, administered by `MET-WP1-13`).
//!
//! This module owns the persisted `metric_source_account` model: one concrete
//! partition/account of a [`metric_source`](crate::model::metric_source),
//! routed to the [`metric_platform`](crate::model::metric_platform) on which
//! its activity was observed. Account identity is `(source_id, external_key)`
//! and is enforced by the database; the globally unique stable [`code`] added
//! by `MET-WP2-01A` sits alongside it and does not replace it.
//!
//! `configuration` is generic **non-secret** routing/configuration JSON only
//! at the database, but `MET-WP1-13` never exposes it raw. The administration
//! surface accepts only the closed, typed [`MetricSourceAccountConfigurationInput`]
//! and returns only the closed [`MetricSourceAccountConfiguration`]; the
//! coordinator canonicalizes the input into exactly one of the approved stored
//! JSON shapes and decodes every stored value through the fail-closed decoder
//! below before it may be returned, updated or copied into audit history. There
//! is no arbitrary JSON input, no raw JSON output, no alternate escape hatch and
//! no credential-bearing field: the safety boundary is structural.
//!
//! [`code`]: MetricSourceAccount::code
//!
//! The foreign keys to source, platform and (optionally) publisher are
//! deliberately non-cascading, and nothing here administers
//! `metric_source_checkpoint`.

use serde::Serialize;
use serde_json::{json, Map, Value as JsonValue};
use uuid::Uuid;

use crate::model::metric_source::{MetricSource, MetricSourceAcquisitionType};

/// The fixed `schemaVersion` of the first approved non-empty configuration.
pub const CLOUDFRONT_CONFIGURATION_SCHEMA_VERSION: &str = "cloudfront-source-account/1";
/// The fixed `logging.mode` of that configuration version.
pub const CLOUDFRONT_LEGACY_S3_LOGGING_MODE: &str = "LEGACY_S3";
/// The exact immutable `driver_key` of the one source family that must use the
/// CloudFront configuration. `MET-WP1-13` approves no real source carrying it.
pub const CLOUDFRONT_DRIVER_KEY: &str = "cloudfront";

/// One persisted metric-source-account row.
///
/// `code` is the globally unique stable account identifier required by
/// `thoth-normalized-metrics/1` (`MET-WP2-01A`). It is exact PostgreSQL
/// `TEXT`: the database rejects blank and duplicate codes, and nothing — here
/// or at any later entry point — may trim, case-fold, Unicode-normalize or
/// alias it.
///
/// `external_key` is the source-side partition/account identifier; the
/// database rejects blank keys and duplicate `(source_id, external_key)`
/// pairs. `expected_publisher_id` optionally pins the canonical publisher the
/// account is expected to report for. The approved design deliberately omits
/// `created_at`/`updated_at` on this table.
///
/// The row serializes to camel case so that an audit `before_state` /
/// `after_state` records exactly the persisted canonical state, including the
/// canonical (already decoder-validated) configuration.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MetricSourceAccount {
    pub source_account_id: Uuid,
    pub code: String,
    pub source_id: Uuid,
    pub platform_id: Uuid,
    pub external_key: String,
    pub expected_publisher_id: Option<Uuid>,
    pub configuration: JsonValue,
    pub enabled: bool,
}

/// The closed set of typed configuration representations.
///
/// There is deliberately no `OTHER`, `RAW` or `UNKNOWN` variant.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(
        description = "Which closed configuration representation a metric source account carries"
    )
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricSourceAccountConfigurationKind {
    /// No source-specific configuration: the canonical stored value is `{}`.
    #[cfg_attr(
        feature = "backend",
        graphql(
            name = "EMPTY",
            description = "No source-specific configuration; the stored value is the empty object"
        )
    )]
    Empty,
    /// The `cloudfront-source-account/1` / `LEGACY_S3` shape.
    #[cfg_attr(
        feature = "backend",
        graphql(
            name = "CLOUDFRONT_LEGACY_S3_V1",
            description = "The cloudfront-source-account/1 routing configuration with LEGACY_S3 logging"
        )
    )]
    CloudfrontLegacyS3V1,
}

/// Caller-supplied CloudFront legacy S3 routing values.
///
/// The coordinator derives the fixed `schemaVersion` and `logging.mode`;
/// callers cannot supply them. Each value must contain at least one
/// non-whitespace character and is otherwise stored exactly as supplied.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(
        description = "CloudFront legacy S3 routing values. Each must contain at least one non-whitespace character and is stored exactly as supplied; the hostname must equal the account's immutable externalKey"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricCloudFrontLegacyS3ConfigurationInput {
    pub hostname: String,
    pub bucket: String,
    pub prefix: String,
}

/// The closed tagged configuration input.
///
/// GraphQL can syntactically supply the optional payload with any `kind`; the
/// coordinator enforces exactly one valid payload per kind, so `EMPTY` with a
/// payload and `CLOUDFRONT_LEGACY_S3_V1` without one are both rejected.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(
        description = "Closed tagged metric source account configuration. EMPTY must carry no cloudfrontLegacyS3 payload; CLOUDFRONT_LEGACY_S3_V1 requires one. No other representation exists"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricSourceAccountConfigurationInput {
    pub kind: MetricSourceAccountConfigurationKind,
    pub cloudfront_legacy_s3: Option<MetricCloudFrontLegacyS3ConfigurationInput>,
}

/// The decoded CloudFront legacy S3 routing values of a stored configuration.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLObject),
    graphql(description = "CloudFront legacy S3 routing values of a metric source account")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricCloudFrontLegacyS3Configuration {
    pub hostname: String,
    pub bucket: String,
    pub prefix: String,
}

/// The decoded, closed configuration of a metric source account.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLObject),
    graphql(
        description = "The closed typed configuration of a metric source account, decoded from its stored canonical value"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricSourceAccountConfiguration {
    pub kind: MetricSourceAccountConfigurationKind,
    pub cloudfront_legacy_s3: Option<MetricCloudFrontLegacyS3Configuration>,
}

/// Values for a new metric-source-account row (`MET-WP1-13`).
///
/// The caller identifies the source and platform by their exact stable codes;
/// `source_account_id` and the resolved `source_id`/`platform_id` are never
/// accepted from callers.
///
/// `expected_publisher_id` stays nullable at the schema and database level,
/// but Specification Amendment 2 requires it for a
/// `CLOUDFRONT_LEGACY_S3_V1` account: the merged managed `DRIVER` ingestion
/// coordinator fails closed unless the account's expected publisher is present
/// and equals the import's publisher, so this surface refuses to create an
/// approved CloudFront account that could never satisfy that invariant.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(
        description = "Values for a metric source account to be created. Superuser only. The source and platform are named by their exact stable codes. A CLOUDFRONT_LEGACY_S3_V1 account requires expectedPublisherId to name an existing publisher; expectedPublisherId is immutable afterwards"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMetricSourceAccount {
    pub code: String,
    pub source_code: String,
    pub platform_code: String,
    pub external_key: String,
    pub expected_publisher_id: Option<Uuid>,
    pub configuration: MetricSourceAccountConfigurationInput,
    pub enabled: bool,
}

/// A complete replacement of a metric source account's mutable fields
/// (`MET-WP1-13`).
///
/// `code` is the stable selector and is **immutable**. The account's source,
/// platform, `external_key` and `expected_publisher_id` are likewise absent:
/// changing any of them is identity-changing repair, separately reviewed, and
/// changing `code` alone cannot bypass the `(source_id, external_key)` identity.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(
        description = "Complete replacement of a metric source account's mutable values, selected by its stable code. Superuser only. Exactly configuration and enabled are replaced; the code, source, platform, externalKey and expectedPublisherId cannot be changed"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchMetricSourceAccount {
    pub code: String,
    pub configuration: MetricSourceAccountConfigurationInput,
    pub enabled: bool,
}

/// Why a typed or stored configuration was refused.
///
/// Each variant carries a fixed message. None of them ever embeds the stored
/// JSON, its keys, its values or the caller's input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationError {
    /// `EMPTY` was supplied together with a CloudFront payload.
    EmptyWithPayload,
    /// `CLOUDFRONT_LEGACY_S3_V1` was supplied without its payload.
    CloudFrontWithoutPayload,
    /// A hostname, bucket or prefix contains no non-whitespace character.
    BlankRoutingValue,
    /// The configured hostname differs from the account's `external_key`.
    HostnameMismatch,
    /// The resolved source requires `CLOUDFRONT_LEGACY_S3_V1`.
    SourceRequiresCloudFront,
    /// The resolved source permits only `EMPTY`.
    SourceRequiresEmpty,
    /// A `CLOUDFRONT_LEGACY_S3_V1` account is being created without the
    /// expected-publisher safety pin the merged managed `DRIVER` ingestion
    /// coordinator requires (Specification Amendment 2).
    CloudFrontRequiresExpectedPublisher,
    /// A stored value is outside the closed decoder, or is inconsistent with
    /// the account's immutable source or `external_key`.
    UnsupportedStored,
}

impl ConfigurationError {
    /// The bounded, fixed client-facing message.
    pub const fn message(self) -> &'static str {
        match self {
            Self::EmptyWithPayload => {
                "An EMPTY metric source account configuration must not carry a CloudFront legacy S3 payload."
            }
            Self::CloudFrontWithoutPayload => {
                "A CLOUDFRONT_LEGACY_S3_V1 metric source account configuration requires its CloudFront legacy S3 payload."
            }
            Self::BlankRoutingValue => {
                "A CloudFront legacy S3 configuration requires non-blank hostname, bucket and prefix values."
            }
            Self::HostnameMismatch => {
                "The configured CloudFront hostname must equal the metric source account's external key exactly."
            }
            Self::SourceRequiresCloudFront => {
                "A metric source account of a DRIVER source with the cloudfront driver key must use the CLOUDFRONT_LEGACY_S3_V1 configuration."
            }
            Self::SourceRequiresEmpty => {
                "Only a metric source account of a DRIVER source with the cloudfront driver key may use the CLOUDFRONT_LEGACY_S3_V1 configuration; every other source uses EMPTY."
            }
            Self::CloudFrontRequiresExpectedPublisher => {
                "A CLOUDFRONT_LEGACY_S3_V1 metric source account must be created with an expectedPublisherId naming an existing publisher."
            }
            Self::UnsupportedStored => {
                "The stored configuration of this metric source account is not supported by this administration surface and requires separately reviewed repair."
            }
        }
    }
}

fn is_nonblank(value: &str) -> bool {
    value.chars().any(|c| !c.is_whitespace())
}

/// The configuration kind the closed compatibility matrix requires for a
/// resolved source: exactly `DRIVER` + `driver_key = "cloudfront"` requires
/// the CloudFront shape, and every other currently supported source uses
/// `EMPTY`. Compatibility is decided from the canonical source row's immutable
/// fields, never inferred from a source `code`.
pub fn required_configuration_kind(source: &MetricSource) -> MetricSourceAccountConfigurationKind {
    if source.acquisition_type == MetricSourceAcquisitionType::Driver
        && source.driver_key.as_deref() == Some(CLOUDFRONT_DRIVER_KEY)
    {
        MetricSourceAccountConfigurationKind::CloudfrontLegacyS3V1
    } else {
        MetricSourceAccountConfigurationKind::Empty
    }
}

/// Reject a kind the resolved source does not permit.
pub fn check_source_compatibility(
    source: &MetricSource,
    kind: MetricSourceAccountConfigurationKind,
) -> Result<(), ConfigurationError> {
    let required = required_configuration_kind(source);
    if kind == required {
        Ok(())
    } else if required == MetricSourceAccountConfigurationKind::CloudfrontLegacyS3V1 {
        Err(ConfigurationError::SourceRequiresCloudFront)
    } else {
        Err(ConfigurationError::SourceRequiresEmpty)
    }
}

impl MetricSourceAccountConfigurationInput {
    /// Validate the typed input against the exact truth table and produce the
    /// canonical stored JSON together with its decoded form.
    ///
    /// `EMPTY` persists exactly `{}`. `CLOUDFRONT_LEGACY_S3_V1` persists
    /// exactly the approved key set with the fixed schema version and logging
    /// mode and the three supplied routing values, which are not trimmed,
    /// case-folded, Unicode-normalized or otherwise rewritten. The hostname
    /// must equal `external_key` by exact string equality.
    pub fn canonicalize(
        &self,
        external_key: &str,
    ) -> Result<(JsonValue, MetricSourceAccountConfiguration), ConfigurationError> {
        match (self.kind, &self.cloudfront_legacy_s3) {
            (MetricSourceAccountConfigurationKind::Empty, None) => Ok((
                json!({}),
                MetricSourceAccountConfiguration {
                    kind: MetricSourceAccountConfigurationKind::Empty,
                    cloudfront_legacy_s3: None,
                },
            )),
            (MetricSourceAccountConfigurationKind::Empty, Some(_)) => {
                Err(ConfigurationError::EmptyWithPayload)
            }
            (MetricSourceAccountConfigurationKind::CloudfrontLegacyS3V1, None) => {
                Err(ConfigurationError::CloudFrontWithoutPayload)
            }
            (MetricSourceAccountConfigurationKind::CloudfrontLegacyS3V1, Some(routing)) => {
                if !(is_nonblank(&routing.hostname)
                    && is_nonblank(&routing.bucket)
                    && is_nonblank(&routing.prefix))
                {
                    return Err(ConfigurationError::BlankRoutingValue);
                }
                if routing.hostname != external_key {
                    return Err(ConfigurationError::HostnameMismatch);
                }
                let decoded = MetricCloudFrontLegacyS3Configuration {
                    hostname: routing.hostname.clone(),
                    bucket: routing.bucket.clone(),
                    prefix: routing.prefix.clone(),
                };
                Ok((
                    json!({
                        "schemaVersion": CLOUDFRONT_CONFIGURATION_SCHEMA_VERSION,
                        "hostname": decoded.hostname,
                        "logging": {
                            "mode": CLOUDFRONT_LEGACY_S3_LOGGING_MODE,
                            "bucket": decoded.bucket,
                            "prefix": decoded.prefix,
                        },
                    }),
                    MetricSourceAccountConfiguration {
                        kind: MetricSourceAccountConfigurationKind::CloudfrontLegacyS3V1,
                        cloudfront_legacy_s3: Some(decoded),
                    },
                ))
            }
        }
    }
}

/// A nonblank string member of a JSON object, or `None`.
fn nonblank_member<'a>(object: &'a Map<String, JsonValue>, key: &str) -> Option<&'a str> {
    object
        .get(key)
        .and_then(JsonValue::as_str)
        .filter(|value| is_nonblank(value))
}

/// Whether an object holds exactly the given keys and no others.
fn has_exact_keys(object: &Map<String, JsonValue>, keys: &[&str]) -> bool {
    object.len() == keys.len() && keys.iter().all(|key| object.contains_key(*key))
}

impl MetricSourceAccountConfiguration {
    /// Decode one stored `metric_source_account.configuration` value through
    /// the closed decoder.
    ///
    /// Exactly two stored forms are accepted: the semantic empty object, and
    /// the exact `cloudfront-source-account/1` shape with the fixed
    /// `LEGACY_S3` mode, exactly the approved keys at every level, nonblank
    /// routing values and a hostname equal to `external_key`. Anything else —
    /// an unknown schema version or mode, an extra or missing key, a blank or
    /// non-string routing value, a non-object, or a structurally valid shape
    /// that conflicts with the account's immutable `external_key` — is
    /// unsupported and fails closed. The error never carries the stored value.
    pub fn decode_stored(
        stored: &JsonValue,
        external_key: &str,
    ) -> Result<Self, ConfigurationError> {
        let object = stored
            .as_object()
            .ok_or(ConfigurationError::UnsupportedStored)?;
        if object.is_empty() {
            return Ok(Self {
                kind: MetricSourceAccountConfigurationKind::Empty,
                cloudfront_legacy_s3: None,
            });
        }
        if !has_exact_keys(object, &["schemaVersion", "hostname", "logging"]) {
            return Err(ConfigurationError::UnsupportedStored);
        }
        if object.get("schemaVersion").and_then(JsonValue::as_str)
            != Some(CLOUDFRONT_CONFIGURATION_SCHEMA_VERSION)
        {
            return Err(ConfigurationError::UnsupportedStored);
        }
        let hostname =
            nonblank_member(object, "hostname").ok_or(ConfigurationError::UnsupportedStored)?;
        let logging = object
            .get("logging")
            .and_then(JsonValue::as_object)
            .ok_or(ConfigurationError::UnsupportedStored)?;
        if !has_exact_keys(logging, &["mode", "bucket", "prefix"])
            || logging.get("mode").and_then(JsonValue::as_str)
                != Some(CLOUDFRONT_LEGACY_S3_LOGGING_MODE)
        {
            return Err(ConfigurationError::UnsupportedStored);
        }
        let bucket =
            nonblank_member(logging, "bucket").ok_or(ConfigurationError::UnsupportedStored)?;
        let prefix =
            nonblank_member(logging, "prefix").ok_or(ConfigurationError::UnsupportedStored)?;
        if hostname != external_key {
            return Err(ConfigurationError::UnsupportedStored);
        }
        Ok(Self {
            kind: MetricSourceAccountConfigurationKind::CloudfrontLegacyS3V1,
            cloudfront_legacy_s3: Some(MetricCloudFrontLegacyS3Configuration {
                hostname: hostname.to_string(),
                bucket: bucket.to_string(),
                prefix: prefix.to_string(),
            }),
        })
    }
}

impl MetricSourceAccount {
    /// Decode this row's stored configuration through the closed decoder.
    pub fn decoded_configuration(
        &self,
    ) -> Result<MetricSourceAccountConfiguration, ConfigurationError> {
        MetricSourceAccountConfiguration::decode_stored(&self.configuration, &self.external_key)
    }
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
