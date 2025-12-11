#[cfg(feature = "backend")]
use aws_config::Region;
#[cfg(feature = "backend")]
use aws_sdk_cloudfront::Client as CloudFrontClient;
#[cfg(feature = "backend")]
use aws_sdk_s3::{
    presigning::PresigningConfig,
    types::ChecksumAlgorithm,
    Client as S3Client,
};
#[cfg(feature = "backend")]
use std::time::Duration as StdDuration;
#[cfg(feature = "backend")]
use thoth_errors::{ThothError, ThothResult};
#[cfg(feature = "backend")]
use uuid::Uuid;

#[cfg(feature = "backend")]
use crate::model::imprint::Imprint;

/// Storage configuration extracted from an imprint
#[cfg(feature = "backend")]
pub struct StorageConfig {
    pub s3_bucket: String,
    pub s3_region: String,
    pub cdn_domain: String,
    pub cloudfront_dist_id: String,
}

#[cfg(feature = "backend")]
impl StorageConfig {
    /// Extract storage configuration from an imprint
    pub fn from_imprint(imprint: &Imprint) -> ThothResult<Self> {
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
            }),
            _ => Err(ThothError::InternalError(
                "Imprint is not configured for file hosting".to_string(),
            )),
        }
    }
}

/// Create an S3 client configured for the given region
#[cfg(feature = "backend")]
pub async fn create_s3_client(region: &str) -> S3Client {
    eprintln!("S3_DEBUG: Creating S3 client for region: {}", region);

    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(Region::new(region.to_string()))
        .load()
        .await;

    // Create S3 client with path-style addressing
    let s3_config = aws_sdk_s3::config::Builder::from(&config)
        .force_path_style(true)
        .build();

    eprintln!("S3_DEBUG: S3 client created with path-style addressing");
    S3Client::from_conf(s3_config)
}

/// Create a CloudFront client
#[cfg(feature = "backend")]
pub async fn create_cloudfront_client() -> CloudFrontClient {
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    CloudFrontClient::new(&config)
}

/// Generate a presigned PUT URL for uploading a file to S3
///
/// The URL includes required headers:
/// - Content-Type: from declared_mime_type
/// - x-amz-checksum-sha256: base64-encoded SHA-256 checksum
#[cfg(feature = "backend")]
pub async fn presign_put_for_upload(
    s3_client: &S3Client,
    bucket: &str,
    temp_key: &str,
    declared_mime_type: &str,
    declared_sha256: &str,
    expires_in_minutes: u64,
) -> ThothResult<String> {
    eprintln!("PRESIGN_DEBUG: Creating presigned URL for bucket: {}, key: {}", bucket, temp_key);
    use base64::{engine::general_purpose, Engine as _};

    // Convert hex SHA-256 to base64
    let sha256_bytes = hex::decode(declared_sha256)
        .map_err(|e| ThothError::InternalError(format!("Invalid SHA-256 hex: {}", e)))?;
    let sha256_base64 = general_purpose::STANDARD.encode(&sha256_bytes);

    let expires_in = StdDuration::from_secs(expires_in_minutes * 60);

    let presigning_config = PresigningConfig::expires_in(expires_in)
        .map_err(|e| ThothError::InternalError(format!("Failed to create presigning config: {}", e)))?;

    let request = s3_client
        .put_object()
        .bucket(bucket)
        .key(temp_key)
        .content_type(declared_mime_type)
        .checksum_sha256(sha256_base64)
        .checksum_algorithm(ChecksumAlgorithm::Sha256);

    // Presign the request
    println!("DEBUG: About to presign request...");
    let presigned_request = request
        .presigned(presigning_config)
        .await
        .map_err(|e| {
            eprintln!("PRESIGN_DEBUG: Presigning failed with error: {:?}", e);
            eprintln!("PRESIGN_DEBUG: Bucket: {}, Key: {}", bucket, temp_key);
            ThothError::InternalError(format!("Failed to presign request: {}", e))
        })?;

    Ok(presigned_request.uri().to_string())
}

/// Copy an object from temporary upload location to final canonical location
#[cfg(feature = "backend")]
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
#[cfg(feature = "backend")]
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
#[cfg(feature = "backend")]
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

    let bytes = response.content_length().unwrap_or(0) as i64;
    let mime_type = response
        .content_type()
        .unwrap_or("application/octet-stream")
        .to_string();

    Ok((bytes, mime_type))
}

/// Invalidate CloudFront cache for a given path
#[cfg(feature = "backend")]
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
                .map_err(|e| ThothError::InternalError(format!("Failed to build invalidation batch: {}", e)))?,
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
#[cfg(feature = "backend")]
pub fn temp_key(file_upload_id: &Uuid) -> String {
    format!("uploads/{}", file_upload_id)
}

/// Compute the canonical object key for a publication file
#[cfg(feature = "backend")]
pub fn canonical_publication_key(doi_prefix: &str, doi_suffix: &str, extension: &str) -> String {
    format!(
        "{}/{}.{}",
        doi_prefix.to_lowercase(),
        doi_suffix.to_lowercase(),
        extension.to_lowercase()
    )
}

/// Compute the canonical object key for a frontcover file
#[cfg(feature = "backend")]
pub fn canonical_frontcover_key(doi_prefix: &str, doi_suffix: &str, extension: &str) -> String {
    format!(
        "{}/{}_frontcover.{}",
        doi_prefix.to_lowercase(),
        doi_suffix.to_lowercase(),
        extension.to_lowercase()
    )
}

/// Build the full CDN URL from domain and object key
#[cfg(feature = "backend")]
pub fn build_cdn_url(cdn_domain: &str, object_key: &str) -> String {
    // Ensure cdn_domain doesn't end with / and object_key doesn't have a leading /
    let domain = cdn_domain.trim_end_matches('/');
    let key = object_key.trim_start_matches('/');
    format!("https://{}/{}", domain, key)
}

