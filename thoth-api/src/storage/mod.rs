use aws_config::Region;
use aws_sdk_cloudfront::Client as CloudFrontClient;
pub use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::{presigning::PresigningConfig, types::ChecksumAlgorithm};
use std::env;
use std::time::Duration as StdDuration;
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use crate::model::imprint::Imprint;
mod encryption;

pub use encryption::{decrypt_credential, encrypt_credential};

pub struct StorageConfig {
    pub s3_bucket: String,
    pub s3_region: String,
    pub cdn_domain: String,
    pub cloudfront_dist_id: String,
    pub aws_access_key_id: Option<String>,
    pub aws_secret_access_key: Option<String>,
}

impl StorageConfig {
    pub fn from_imprint(imprint: &Imprint) -> ThothResult<Self> {
        let aws_access_key_id = env::var("AWS_ACCESS_KEY_ID")
            .ok()
            .filter(|value| !value.is_empty());
        let aws_secret_access_key = env::var("AWS_SECRET_ACCESS_KEY")
            .ok()
            .filter(|value| !value.is_empty());

        match (
            &imprint.s3_bucket,
            &imprint.s3_region,
            &imprint.cdn_domain,
            &imprint.cloudfront_dist_id,
        ) {
            (Some(bucket), Some(region), Some(domain), Some(dist_id)) => Ok(StorageConfig {
                s3_bucket: bucket.clone(),
                s3_region: region.clone(),
                cdn_domain: domain.clone(),
                cloudfront_dist_id: dist_id.clone(),
                aws_access_key_id,
                aws_secret_access_key,
            }),
            _ => Err(ThothError::InternalError(
                "Imprint is not configured for file hosting".to_string(),
            )),
        }
    }
}

/// Create an S3 client configured for the given region
///
/// AWS credentials are automatically loaded from the default credential chain:
/// - Environment variables (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY)
/// - AWS credentials file (~/.aws/credentials)
/// - IAM roles (when running on EC2/ECS/Lambda)
/// - Other standard AWS credential sources
pub async fn create_s3_client(
    region: &str,
    access_key_id: Option<&str>,
    secret_access_key: Option<&str>,
) -> S3Client {
    create_s3_client_with_credentials(region, access_key_id, secret_access_key).await
}

/// Create an S3 client with custom credentials from StorageConfig
pub async fn create_s3_client_with_credentials(
    region: &str,
    access_key_id: Option<&str>,
    secret_access_key: Option<&str>,
) -> S3Client {
    eprintln!("S3_DEBUG: Creating S3 client for region: {}", region);

    let mut config_builder = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(Region::new(region.to_string()));

    // Use custom credentials if provided
    if let (Some(access_key), Some(secret_key)) = (access_key_id, secret_access_key) {
        eprintln!("S3_DEBUG: Using custom credentials from database");
        use aws_credential_types::Credentials;
        let credentials = Credentials::new(access_key, secret_key, None, None, "thoth-storage");
        config_builder = config_builder.credentials_provider(credentials);
    } else {
        eprintln!("S3_DEBUG: Using default credential chain");
    }

    let config = config_builder.load().await;

    let s3_config = aws_sdk_s3::config::Builder::from(&config)
        .force_path_style(true)
        .build();

    eprintln!("S3_DEBUG: S3 client created with path-style addressing");
    S3Client::from_conf(s3_config)
}

/// Create a CloudFront client
pub async fn create_cloudfront_client() -> CloudFrontClient {
    create_cloudfront_client_with_credentials(None, None).await
}

/// Create a CloudFront client with custom credentials from StorageConfig
pub async fn create_cloudfront_client_with_credentials(
    access_key_id: Option<&str>,
    secret_access_key: Option<&str>,
) -> CloudFrontClient {
    let mut config_builder = aws_config::defaults(aws_config::BehaviorVersion::latest());

    // Use custom credentials if provided
    if let (Some(access_key), Some(secret_key)) = (access_key_id, secret_access_key) {
        eprintln!("CLOUDFRONT_DEBUG: Using custom credentials from database");
        use aws_credential_types::Credentials;
        let credentials = Credentials::new(access_key, secret_key, None, None, "thoth-storage");
        config_builder = config_builder.credentials_provider(credentials);
    }

    let config = config_builder.load().await;
    CloudFrontClient::new(&config)
}

/// Generate a presigned PUT URL for uploading a file to S3
/// required headers:
/// - Content-Type: from declared_mime_type
/// - x-amz-checksum-sha256: base64-encoded SHA-256 checksum
pub async fn presign_put_for_upload(
    s3_client: &S3Client,
    bucket: &str,
    temp_key: &str,
    declared_mime_type: &str,
    declared_sha256: &str,
    expires_in_minutes: u64,
) -> ThothResult<String> {
    eprintln!(
        "PRESIGN_DEBUG: Creating presigned URL for bucket: {}, key: {}",
        bucket, temp_key
    );
    use base64::{engine::general_purpose, Engine as _};

    // hex SHA-256 to base64
    let sha256_bytes = hex::decode(declared_sha256)
        .map_err(|e| ThothError::InternalError(format!("Invalid SHA-256 hex: {}", e)))?;
    let sha256_base64 = general_purpose::STANDARD.encode(&sha256_bytes);

    let expires_in = StdDuration::from_secs(expires_in_minutes * 60);

    let presigning_config = PresigningConfig::expires_in(expires_in).map_err(|e| {
        ThothError::InternalError(format!("Failed to create presigning config: {}", e))
    })?;

    let request = s3_client
        .put_object()
        .bucket(bucket)
        .key(temp_key)
        .content_type(declared_mime_type)
        .checksum_sha256(sha256_base64)
        .checksum_algorithm(ChecksumAlgorithm::Sha256);

    let presigned_request = request.presigned(presigning_config).await.map_err(|e| {
        eprintln!("PRESIGN_DEBUG: Presigning failed with error: {:?}", e);
        eprintln!("PRESIGN_DEBUG: Bucket: {}, Key: {}", bucket, temp_key);
        ThothError::InternalError(format!("Failed to presign request: {}", e))
    })?;

    Ok(presigned_request.uri().to_string())
}

/// Copy an object from temporary upload location to final canonical location
pub async fn copy_temp_object_to_final(
    s3_client: &S3Client,
    bucket: &str,
    temp_key: &str,
    final_key: &str,
) -> ThothResult<()> {
    let copy_source = format!("{}/{}", bucket, temp_key);

    s3_client
        .copy_object()
        .bucket(bucket)
        .copy_source(copy_source)
        .key(final_key)
        .send()
        .await
        .map_err(|e| ThothError::InternalError(format!("Failed to copy object: {}", e)))?;

    Ok(())
}

/// Delete a temporary upload object from S3
pub async fn delete_temp_object(
    s3_client: &S3Client,
    bucket: &str,
    temp_key: &str,
) -> ThothResult<()> {
    s3_client
        .delete_object()
        .bucket(bucket)
        .key(temp_key)
        .send()
        .await
        .map_err(|e| ThothError::InternalError(format!("Failed to delete object: {}", e)))?;

    Ok(())
}

/// Get object metadata (HeadObject) from S3
pub async fn head_object(
    s3_client: &S3Client,
    bucket: &str,
    key: &str,
) -> ThothResult<(i64, String)> {
    let response = s3_client
        .head_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| ThothError::InternalError(format!("Failed to head object: {}", e)))?;

    let bytes = response.content_length().unwrap_or(0);
    let mime_type = response
        .content_type()
        .unwrap_or("application/octet-stream")
        .to_string();

    Ok((bytes, mime_type))
}

/// Invalidate CloudFront cache for a given path
pub async fn invalidate_cloudfront(
    cloudfront_client: &CloudFrontClient,
    distribution_id: &str,
    path: &str,
) -> ThothResult<String> {
    use aws_sdk_cloudfront::types::Paths;

    let paths = Paths::builder()
        .quantity(1)
        .items(path)
        .build()
        .map_err(|e| ThothError::InternalError(format!("Failed to build paths: {}", e)))?;

    let response = cloudfront_client
        .create_invalidation()
        .distribution_id(distribution_id)
        .invalidation_batch(
            aws_sdk_cloudfront::types::InvalidationBatch::builder()
                .paths(paths)
                .caller_reference(format!("thoth-{}", Uuid::new_v4()))
                .build()
                .map_err(|e| {
                    ThothError::InternalError(format!("Failed to build invalidation batch: {}", e))
                })?,
        )
        .send()
        .await
        .map_err(|e| ThothError::InternalError(format!("Failed to create invalidation: {}", e)))?;

    let invalidation_id = response
        .invalidation()
        .map(|inv| inv.id().to_string())
        .ok_or_else(|| ThothError::InternalError("No invalidation ID returned".to_string()))?;

    Ok(invalidation_id)
}

/// Compute the temporary S3 key for an upload
pub fn temp_key(file_upload_id: &Uuid) -> String {
    format!("uploads/{}", file_upload_id)
}

/// Compute the canonical object key for a publication file
pub fn canonical_publication_key(doi_prefix: &str, doi_suffix: &str, extension: &str) -> String {
    format!(
        "{}/{}.{}",
        doi_prefix.to_lowercase(),
        doi_suffix.to_lowercase(),
        extension.to_lowercase()
    )
}

/// Compute the canonical object key for a frontcover file
pub fn canonical_frontcover_key(doi_prefix: &str, doi_suffix: &str, extension: &str) -> String {
    format!(
        "{}/{}_frontcover.{}",
        doi_prefix.to_lowercase(),
        doi_suffix.to_lowercase(),
        extension.to_lowercase()
    )
}

/// Build the full CDN URL from domain and object key
pub fn build_cdn_url(cdn_domain: &str, object_key: &str) -> String {
    // Ensure cdn_domain doesn't end with / and object_key doesn't have a leading /
    let domain = cdn_domain.trim_end_matches('/');
    let key = object_key.trim_start_matches('/');
    format!("https://{}/{}", domain, key)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::model::Timestamp;
    use std::env;
    use std::sync::{Mutex, MutexGuard};
    use uuid::Uuid;

    lazy_static::lazy_static! {
        static ref ENV_LOCK: Mutex<()> = Mutex::new(());
    }

    pub(crate) fn env_lock() -> MutexGuard<'static, ()> {
        ENV_LOCK.lock().unwrap()
    }

    fn create_test_imprint_with_storage() -> Imprint {
        Imprint {
            imprint_id: Uuid::new_v4(),
            publisher_id: Uuid::new_v4(),
            imprint_name: "Test Imprint".to_string(),
            imprint_url: None,
            crossmark_doi: None,
            s3_bucket: Some("test-bucket".to_string()),
            s3_region: Some("us-east-1".to_string()),
            cdn_domain: Some("cdn.example.com".to_string()),
            cloudfront_dist_id: Some("E1234567890ABC".to_string()),
            created_at: Timestamp::default(),
            updated_at: Timestamp::default(),
        }
    }

    fn create_test_imprint_without_storage() -> Imprint {
        Imprint {
            imprint_id: Uuid::new_v4(),
            publisher_id: Uuid::new_v4(),
            imprint_name: "Test Imprint".to_string(),
            imprint_url: None,
            crossmark_doi: None,
            s3_bucket: None,
            s3_region: None,
            cdn_domain: None,
            cloudfront_dist_id: None,
            created_at: Timestamp::default(),
            updated_at: Timestamp::default(),
        }
    }

    #[test]
    fn test_storage_config_from_imprint_success() {
        let _guard = env_lock();
        env::remove_var("AWS_ACCESS_KEY_ID");
        env::remove_var("AWS_SECRET_ACCESS_KEY");
        let imprint = create_test_imprint_with_storage();
        let config = StorageConfig::from_imprint(&imprint).unwrap();

        assert_eq!(config.s3_bucket, "test-bucket");
        assert_eq!(config.s3_region, "us-east-1");
        assert_eq!(config.cdn_domain, "cdn.example.com");
        assert_eq!(config.cloudfront_dist_id, "E1234567890ABC");
        assert_eq!(config.aws_access_key_id, None);
        assert_eq!(config.aws_secret_access_key, None);
    }

    #[test]
    fn test_storage_config_from_imprint_missing_config() {
        let imprint = create_test_imprint_without_storage();
        let result = StorageConfig::from_imprint(&imprint);

        assert!(result.is_err());
        assert!(matches!(result, Err(ThothError::InternalError(_))));
    }

    #[test]
    fn test_storage_config_from_imprint_partial_config() {
        let mut imprint = create_test_imprint_without_storage();
        imprint.s3_bucket = Some("test-bucket".to_string());
        // Missing other required fields

        let result = StorageConfig::from_imprint(&imprint);
        assert!(result.is_err());
    }

    #[test]
    #[cfg(feature = "backend")]
    fn test_storage_config_from_imprint_with_credentials() {
        let _guard = env_lock();
        env::set_var("AWS_ACCESS_KEY_ID", "AKIAIOSFODNN7EXAMPLE");
        env::set_var(
            "AWS_SECRET_ACCESS_KEY",
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
        );
        let imprint = create_test_imprint_with_storage();

        let config = StorageConfig::from_imprint(&imprint).unwrap();

        assert_eq!(
            config.aws_access_key_id,
            Some("AKIAIOSFODNN7EXAMPLE".to_string())
        );
        assert_eq!(
            config.aws_secret_access_key,
            Some("wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string())
        );

        env::remove_var("AWS_ACCESS_KEY_ID");
        env::remove_var("AWS_SECRET_ACCESS_KEY");
    }

    #[test]
    fn test_temp_key() {
        let file_upload_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let key = temp_key(&file_upload_id);
        assert_eq!(key, "uploads/550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_canonical_publication_key() {
        let key = canonical_publication_key("10.12345", "Test-Suffix.01", "PDF");
        assert_eq!(key, "10.12345/test-suffix.01.pdf");

        let key2 = canonical_publication_key("10.1000", "182", "epub");
        assert_eq!(key2, "10.1000/182.epub");

        // Test case insensitivity
        let key3 = canonical_publication_key("10.ABCDEF", "UPPERCASE-SUFFIX", "XML");
        assert_eq!(key3, "10.abcdef/uppercase-suffix.xml");
    }

    #[test]
    fn test_canonical_frontcover_key() {
        let key = canonical_frontcover_key("10.12345", "Test-Suffix.01", "JPG");
        assert_eq!(key, "10.12345/test-suffix.01_frontcover.jpg");

        let key2 = canonical_frontcover_key("10.1000", "182", "png");
        assert_eq!(key2, "10.1000/182_frontcover.png");

        // Test case insensitivity
        let key3 = canonical_frontcover_key("10.ABCDEF", "UPPERCASE-SUFFIX", "WEBP");
        assert_eq!(key3, "10.abcdef/uppercase-suffix_frontcover.webp");
    }

    #[test]
    fn test_build_cdn_url() {
        // Normal case
        let url = build_cdn_url("cdn.example.com", "10.12345/test.pdf");
        assert_eq!(url, "https://cdn.example.com/10.12345/test.pdf");

        // Domain with trailing slash
        let url2 = build_cdn_url("cdn.example.com/", "10.12345/test.pdf");
        assert_eq!(url2, "https://cdn.example.com/10.12345/test.pdf");

        // Key with leading slash
        let url3 = build_cdn_url("cdn.example.com", "/10.12345/test.pdf");
        assert_eq!(url3, "https://cdn.example.com/10.12345/test.pdf");

        // Both with slashes
        let url4 = build_cdn_url("cdn.example.com/", "/10.12345/test.pdf");
        assert_eq!(url4, "https://cdn.example.com/10.12345/test.pdf");

        // Multiple trailing slashes
        let url5 = build_cdn_url("cdn.example.com///", "///10.12345/test.pdf");
        assert_eq!(url5, "https://cdn.example.com/10.12345/test.pdf");
    }

    #[tokio::test]
    #[cfg(feature = "backend")]
    async fn test_create_s3_client_with_credentials() {
        // Test that function doesn't panic and returns a client
        // Note: This creates a real AWS client, but doesn't make actual AWS calls
        let _client = create_s3_client_with_credentials("us-east-1", None, None).await;
        // If we get here, the client was created successfully
    }

    #[tokio::test]
    #[cfg(feature = "backend")]
    async fn test_create_s3_client_with_custom_credentials() {
        // Test with custom credentials
        let _client = create_s3_client_with_credentials(
            "us-east-1",
            Some("AKIAIOSFODNN7EXAMPLE"),
            Some("wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"),
        )
        .await;
        // If we get here, the client was created successfully
    }

    #[tokio::test]
    #[cfg(feature = "backend")]
    async fn test_create_cloudfront_client_with_credentials() {
        // Test that function doesn't panic and returns a client
        let _client = create_cloudfront_client_with_credentials(None, None).await;
        // If we get here, the client was created successfully
    }

    #[tokio::test]
    #[cfg(feature = "backend")]
    async fn test_create_cloudfront_client_with_custom_credentials() {
        // Test with custom credentials
        let _client = create_cloudfront_client_with_credentials(
            Some("AKIAIOSFODNN7EXAMPLE"),
            Some("wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"),
        )
        .await;
        // If we get here, the client was created successfully
    }

    #[test]
    fn test_storage_config_from_imprint_partial_missing_fields() {
        // Test with only some fields set
        let mut imprint = create_test_imprint_without_storage();
        imprint.s3_bucket = Some("test-bucket".to_string());
        imprint.s3_region = Some("us-east-1".to_string());
        // Missing cdn_domain and cloudfront_dist_id

        let result = StorageConfig::from_imprint(&imprint);
        assert!(result.is_err());
    }

    #[test]
    fn test_canonical_publication_key_edge_cases() {
        // Empty suffix
        let key = canonical_publication_key("10.12345", "", "pdf");
        assert_eq!(key, "10.12345/.pdf");

        // Suffix with dots
        let key2 = canonical_publication_key("10.12345", "test.suffix.v2", "epub");
        assert_eq!(key2, "10.12345/test.suffix.v2.epub");

        // Extension with dots (should be handled as-is)
        let key3 = canonical_publication_key("10.12345", "test", "tar.gz");
        assert_eq!(key3, "10.12345/test.tar.gz");
    }

    #[test]
    fn test_canonical_frontcover_key_edge_cases() {
        // Empty suffix
        let key = canonical_frontcover_key("10.12345", "", "jpg");
        assert_eq!(key, "10.12345/_frontcover.jpg");

        // Suffix with special characters
        let key2 = canonical_frontcover_key("10.12345", "test-suffix_v2", "png");
        assert_eq!(key2, "10.12345/test-suffix_v2_frontcover.png");
    }

    #[test]
    fn test_build_cdn_url_edge_cases() {
        // Empty key
        let url = build_cdn_url("cdn.example.com", "");
        assert_eq!(url, "https://cdn.example.com/");

        // Key with multiple slashes
        let url2 = build_cdn_url("cdn.example.com", "///path/to/file.pdf");
        assert_eq!(url2, "https://cdn.example.com/path/to/file.pdf");

        // Domain with protocol (should still work)
        let url3 = build_cdn_url("https://cdn.example.com", "file.pdf");
        assert_eq!(url3, "https://https://cdn.example.com/file.pdf");
    }

    #[test]
    fn test_temp_key_different_uuids() {
        let uuid1 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let uuid2 = Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap();

        let key1 = temp_key(&uuid1);
        let key2 = temp_key(&uuid2);

        assert_ne!(key1, key2);
        assert_eq!(key1, "uploads/550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(key2, "uploads/6ba7b810-9dad-11d1-80b4-00c04fd430c8");
    }
}
