-- Drop file_upload table
DROP TABLE IF EXISTS file_upload;

-- Drop file table
DROP TABLE IF EXISTS file;

-- Drop file_type enum
DROP TYPE IF EXISTS file_type;

-- Remove storage configuration columns from imprint table
ALTER TABLE imprint
  DROP CONSTRAINT IF EXISTS imprint_storage_cfg_all_or_none,
  DROP COLUMN IF EXISTS s3_bucket,
  DROP COLUMN IF EXISTS s3_region,
  DROP COLUMN IF EXISTS cdn_domain,
  DROP COLUMN IF EXISTS cloudfront_dist_id;

