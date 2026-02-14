use super::*;
use crate::model::imprint::Imprint;
use thoth_errors::ThothError;
use uuid::Uuid;

#[test]
fn storage_config_from_imprint_returns_values() {
    let imprint = Imprint {
        s3_bucket: Some("bucket".to_string()),
        cdn_domain: Some("cdn.example.org".to_string()),
        cloudfront_dist_id: Some("dist-123".to_string()),
        ..Default::default()
    };

    let config = StorageConfig::from_imprint(&imprint).expect("Expected storage config");
    assert_eq!(config.s3_bucket, "bucket");
    assert_eq!(config.cdn_domain, "cdn.example.org");
    assert_eq!(config.cloudfront_dist_id, "dist-123");
}

#[test]
fn storage_config_from_imprint_requires_all_fields() {
    let imprint = Imprint {
        s3_bucket: Some("bucket".to_string()),
        cdn_domain: None,
        cloudfront_dist_id: Some("dist-123".to_string()),
        ..Default::default()
    };

    let err = StorageConfig::from_imprint(&imprint)
        .err()
        .expect("Expected missing config error");
    assert_eq!(
        err,
        ThothError::InternalError("Imprint is not configured for file hosting".to_string())
    );
}

#[test]
fn temp_key_prefixes_uploads() {
    let upload_id = Uuid::parse_str("6f4e7ad7-8e68-4c1e-8efc-49f7c59b0c88").unwrap();
    assert_eq!(temp_key(&upload_id), format!("uploads/{}", upload_id));
}

#[test]
fn canonical_publication_key_lowercases_parts() {
    let key = canonical_publication_key("10.1234", "AbC/Def", "PDF");
    assert_eq!(key, "10.1234/abc/def.pdf");
}

#[test]
fn canonical_frontcover_key_lowercases_parts() {
    let key = canonical_frontcover_key("10.1234", "AbC/Def", "PNG");
    assert_eq!(key, "10.1234/abc/def_frontcover.png");
}

#[test]
fn build_cdn_url_normalizes_domain_and_key() {
    let https_url = build_cdn_url("https://cdn.example.org/", "/files/doc.pdf");
    assert_eq!(https_url, "https://cdn.example.org/files/doc.pdf");

    let http_url = build_cdn_url("http://cdn.example.org", "files/doc.pdf");
    assert_eq!(http_url, "https://cdn.example.org/files/doc.pdf");
}
