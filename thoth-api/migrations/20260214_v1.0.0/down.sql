DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON public.work_featured_video;
DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON public.book_review;
DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON public.endorsement;
DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON public.award;
DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON public.additional_resource;

DROP INDEX IF EXISTS file_upload_work_featured_video_idx;
DROP INDEX IF EXISTS file_upload_additional_resource_idx;
DROP INDEX IF EXISTS file_work_featured_video_unique_idx;
DROP INDEX IF EXISTS file_additional_resource_unique_idx;

ALTER TABLE file DROP CONSTRAINT IF EXISTS file_type_check;
ALTER TABLE file_upload DROP CONSTRAINT IF EXISTS file_upload_type_check;

ALTER TABLE file
  DROP COLUMN IF EXISTS work_featured_video_id,
  DROP COLUMN IF EXISTS additional_resource_id;

ALTER TABLE file_upload
  DROP COLUMN IF EXISTS work_featured_video_id,
  DROP COLUMN IF EXISTS additional_resource_id;

ALTER TABLE file
  ADD CONSTRAINT file_type_check
  CHECK (
    (file_type = 'frontcover' AND work_id IS NOT NULL AND publication_id IS NULL) OR
    (file_type = 'publication' AND publication_id IS NOT NULL AND work_id IS NULL)
  );

ALTER TABLE file_upload
  ADD CONSTRAINT file_upload_type_check
  CHECK (
    (file_type = 'frontcover' AND work_id IS NOT NULL AND publication_id IS NULL) OR
    (file_type = 'publication' AND publication_id IS NOT NULL AND work_id IS NULL)
  );

DROP TABLE IF EXISTS work_featured_video_history;
DROP TABLE IF EXISTS book_review_history;
DROP TABLE IF EXISTS endorsement_history;
DROP TABLE IF EXISTS award_history;
DROP TABLE IF EXISTS additional_resource_history;

DROP TABLE IF EXISTS work_featured_video;
DROP TABLE IF EXISTS book_review;
DROP TABLE IF EXISTS endorsement;
DROP TABLE IF EXISTS award;
DROP TABLE IF EXISTS additional_resource;

DROP TYPE IF EXISTS resource_type;

ALTER TABLE work
  DROP COLUMN IF EXISTS resources_description;
