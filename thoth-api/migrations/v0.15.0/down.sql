DROP TABLE IF EXISTS file_upload;
DROP TABLE IF EXISTS file;
DROP TYPE IF EXISTS file_type;
DROP FUNCTION IF EXISTS file_upload_work_updated_at_with_relations();
DROP FUNCTION IF EXISTS file_work_updated_at_with_relations();

ALTER TABLE imprint
  DROP CONSTRAINT IF EXISTS imprint_storage_cfg_all_or_none,
  DROP COLUMN IF EXISTS s3_bucket,
  DROP COLUMN IF EXISTS s3_region,
  DROP COLUMN IF EXISTS cdn_domain,
  DROP COLUMN IF EXISTS cloudfront_dist_id;
