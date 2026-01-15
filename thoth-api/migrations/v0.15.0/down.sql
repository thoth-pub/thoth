DROP TABLE IF EXISTS file_upload;
DROP TABLE IF EXISTS file;
DROP TYPE IF EXISTS file_type;

ALTER TABLE imprint
  DROP CONSTRAINT IF EXISTS imprint_storage_cfg_all_or_none,
  DROP COLUMN IF EXISTS aws_access_key_id,
  DROP COLUMN IF EXISTS aws_secret_access_key,
  DROP COLUMN IF EXISTS s3_bucket,
  DROP COLUMN IF EXISTS s3_region,
  DROP COLUMN IF EXISTS cdn_domain,
  DROP COLUMN IF EXISTS cloudfront_dist_id;

