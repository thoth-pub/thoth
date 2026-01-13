ALTER TABLE imprint
  ADD COLUMN s3_bucket          TEXT,
  ADD COLUMN s3_region          TEXT,
  ADD COLUMN cdn_domain         TEXT,
  ADD COLUMN cloudfront_dist_id TEXT,
  ADD COLUMN aws_access_key_id  TEXT,
  ADD COLUMN aws_secret_access_key TEXT;

ALTER TABLE imprint
  ADD CONSTRAINT imprint_storage_cfg_all_or_none
  CHECK (
    (
      s3_bucket          IS NULL AND
      s3_region          IS NULL AND
      cdn_domain         IS NULL AND
      cloudfront_dist_id IS NULL AND
      aws_access_key_id  IS NULL AND
      aws_secret_access_key IS NULL
    )
    OR
    (
      s3_bucket          IS NOT NULL AND
      s3_region          IS NOT NULL AND
      cdn_domain         IS NOT NULL AND
      cloudfront_dist_id IS NOT NULL AND
      (
        (aws_access_key_id IS NULL AND aws_secret_access_key IS NULL)
        OR
        (aws_access_key_id IS NOT NULL AND aws_secret_access_key IS NOT NULL)
      )
    )
  );


CREATE TYPE file_type AS ENUM ('publication', 'frontcover');

CREATE TABLE file (
  file_id        UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  file_type      file_type NOT NULL,
  work_id        UUID REFERENCES work (work_id),
  publication_id UUID REFERENCES publication (publication_id),
  object_key     TEXT NOT NULL,
  cdn_url        TEXT NOT NULL,
  mime_type      TEXT NOT NULL,
  bytes          BIGINT NOT NULL,
  sha256         TEXT NOT NULL,
  created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE file
  ADD CONSTRAINT file_type_check
  CHECK (
    (file_type = 'frontcover' AND work_id IS NOT NULL AND publication_id IS NULL) OR
    (file_type = 'publication' AND publication_id IS NOT NULL AND work_id IS NULL)
  );

CREATE UNIQUE INDEX file_frontcover_work_unique_idx
  ON file (work_id)
  WHERE file_type = 'frontcover';

CREATE UNIQUE INDEX file_publication_unique_idx
  ON file (publication_id)
  WHERE file_type = 'publication';

CREATE UNIQUE INDEX file_object_key_unique_idx
  ON file (object_key);

SELECT diesel_manage_updated_at('file');

CREATE TABLE file_upload (
  file_upload_id     UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  file_type          file_type NOT NULL,
  work_id            UUID REFERENCES work (work_id),
  publication_id     UUID REFERENCES publication (publication_id),
  declared_mime_type TEXT NOT NULL,
  declared_extension TEXT NOT NULL,
  declared_sha256    TEXT NOT NULL,
  created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE file_upload
  ADD CONSTRAINT file_upload_type_check
  CHECK (
    (file_type = 'frontcover' AND work_id IS NOT NULL AND publication_id IS NULL) OR
    (file_type = 'publication' AND publication_id IS NOT NULL AND work_id IS NULL)
  );

CREATE INDEX file_upload_work_idx
  ON file_upload (work_id)
  WHERE file_type = 'frontcover';

CREATE INDEX file_upload_publication_idx
  ON file_upload (publication_id)
  WHERE file_type = 'publication';

SELECT diesel_manage_updated_at('file_upload');

