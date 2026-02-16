pub use aws_sdk_cloudfront::Client as CloudFrontClient;
pub use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::{presigning::PresigningConfig, types::ChecksumAlgorithm};
use std::time::Duration as StdDuration;
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use crate::model::imprint::Imprint;

/// Storage configuration extracted from an imprint
pub struct StorageConfig {
    pub s3_bucket: String,
    pub cdn_domain: String,
    pub cloudfront_dist_id: String,
}

impl StorageConfig {
    /// Extract storage configuration from an imprint
    pub fn from_imprint(imprint: &Imprint) -> ThothResult<Self> {
        match (
            &imprint.s3_bucket,
            &imprint.cdn_domain,
            &imprint.cloudfront_dist_id,
        ) {
            (Some(bucket), Some(domain), Some(dist_id)) => Ok(StorageConfig {
                s3_bucket: bucket.clone(),
                cdn_domain: domain.clone(),
                cloudfront_dist_id: dist_id.clone(),
            }),
            _ => Err(ThothError::InternalError(
                "Imprint is not configured for file hosting".to_string(),
            )),
        }
    }
}

async fn load_aws_config(
    access_key_id: &str,
    secret_access_key: &str,
    region: &str,
) -> aws_config::SdkConfig {
    let credentials = aws_credential_types::Credentials::new(
        access_key_id,
        secret_access_key,
        None,
        None,
        "thoth-cli",
    );

    aws_config::ConfigLoader::default()
        .behavior_version(aws_config::BehaviorVersion::latest())
        .credentials_provider(credentials)
        .region(aws_config::Region::new(region.to_string()))
        .load()
        .await
}

/// Create an S3 client configured with explicit credentials and region.
pub async fn create_s3_client(
    access_key_id: &str,
    secret_access_key: &str,
    region: &str,
) -> S3Client {
    let config = load_aws_config(access_key_id, secret_access_key, region).await;
    S3Client::new(&config)
}

/// Create a CloudFront client configured with explicit credentials and region.
pub async fn create_cloudfront_client(
    access_key_id: &str,
    secret_access_key: &str,
    region: &str,
) -> CloudFrontClient {
    let config = load_aws_config(access_key_id, secret_access_key, region).await;
    CloudFrontClient::new(&config)
}

/// Generate a presigned PUT URL for uploading a file to S3
/// required headers:
/// - Content-Type: from declared_mime_type
/// - x-amz-checksum-sha256: base64-encoded SHA-256 checksum
/// - x-amz-sdk-checksum-algorithm: SHA256
pub async fn presign_put_for_upload(
    s3_client: &S3Client,
    bucket: &str,
    temp_key: &str,
    declared_mime_type: &str,
    declared_sha256: &str,
    expires_in_minutes: u64,
) -> ThothResult<String> {
    use base64::{engine::general_purpose, Engine as _};

    // Convert hex SHA-256 to base64
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

    // Presign the request
    let presigned_request = request
        .presigned(presigning_config)
        .await
        .map_err(|e| ThothError::InternalError(format!("Failed to presign request: {}", e)))?;

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

/// Delete an object from S3
pub async fn delete_object(s3_client: &S3Client, bucket: &str, key: &str) -> ThothResult<()> {
    s3_client
        .delete_object()
        .bucket(bucket)
        .key(key)
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

    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };
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

/// Invalidate and clean up an existing canonical object, if one exists.
///
/// When replacing an existing object at a new key, the old object is deleted and both old and
/// new paths are invalidated. When replacing in place (same key), only the canonical path is
/// invalidated.
pub async fn reconcile_replaced_object(
    s3_client: &S3Client,
    cloudfront_client: &CloudFrontClient,
    bucket: &str,
    distribution_id: &str,
    old_object_key: Option<&str>,
    canonical_key: &str,
) -> ThothResult<()> {
    let Some(old_key) = old_object_key else {
        return Ok(());
    };

    if old_key != canonical_key {
        delete_object(s3_client, bucket, old_key).await?;
        invalidate_cloudfront(cloudfront_client, distribution_id, old_key).await?;
    }

    invalidate_cloudfront(cloudfront_client, distribution_id, canonical_key).await?;
    Ok(())
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

/// Compute the canonical object key for an additional resource or featured video file
pub fn canonical_resource_key(
    doi_prefix: &str,
    doi_suffix: &str,
    resource_id: &Uuid,
    extension: &str,
) -> String {
    format!(
        "{}/{}/resources/{}.{}",
        doi_prefix.to_lowercase(),
        doi_suffix.to_lowercase(),
        resource_id,
        extension.to_lowercase()
    )
}

/// Build the full CDN URL from domain and object key
pub fn build_cdn_url(cdn_domain: &str, object_key: &str) -> String {
    // Ensure cdn_domain doesn't end with / and object_key doesn't have a leading /
    let domain = cdn_domain.trim_end_matches('/');
    let domain = domain
        .strip_prefix("https://")
        .or_else(|| domain.strip_prefix("http://"))
        .unwrap_or(domain);
    let key = object_key.trim_start_matches('/');
    format!("https://{}/{}", domain, key)
}

#[cfg(test)]
mod tests;
