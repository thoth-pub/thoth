ALTER TABLE file DROP CONSTRAINT IF EXISTS file_type_check;
ALTER TABLE file_upload DROP CONSTRAINT IF EXISTS file_upload_type_check;

ALTER TABLE file
  ADD CONSTRAINT file_type_check
  CHECK (
    (file_type = 'frontcover' AND work_id IS NOT NULL AND publication_id IS NULL AND additional_resource_id IS NULL AND work_featured_video_id IS NULL) OR
    (file_type = 'publication' AND publication_id IS NOT NULL AND work_id IS NULL AND additional_resource_id IS NULL AND work_featured_video_id IS NULL) OR
    (file_type = 'additional_resource' AND additional_resource_id IS NOT NULL AND work_id IS NULL AND publication_id IS NULL AND work_featured_video_id IS NULL) OR
    (file_type = 'work_featured_video' AND work_featured_video_id IS NOT NULL AND work_id IS NULL AND publication_id IS NULL AND additional_resource_id IS NULL) OR
    (file_type = 'accessibility_report' AND publication_id IS NOT NULL AND work_id IS NULL AND additional_resource_id IS NULL AND work_featured_video_id IS NULL)
  );

ALTER TABLE file_upload
  ADD CONSTRAINT file_upload_type_check
  CHECK (
    (file_type = 'frontcover' AND work_id IS NOT NULL AND publication_id IS NULL AND additional_resource_id IS NULL AND work_featured_video_id IS NULL) OR
    (file_type = 'publication' AND publication_id IS NOT NULL AND work_id IS NULL AND additional_resource_id IS NULL AND work_featured_video_id IS NULL) OR
    (file_type = 'additional_resource' AND additional_resource_id IS NOT NULL AND work_id IS NULL AND publication_id IS NULL AND work_featured_video_id IS NULL) OR
    (file_type = 'work_featured_video' AND work_featured_video_id IS NOT NULL AND work_id IS NULL AND publication_id IS NULL AND additional_resource_id IS NULL) OR
    (file_type = 'accessibility_report' AND publication_id IS NOT NULL AND work_id IS NULL AND additional_resource_id IS NULL AND work_featured_video_id IS NULL)
  );
