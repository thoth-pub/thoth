ALTER TABLE file DROP CONSTRAINT IF EXISTS file_work_id_fkey;
ALTER TABLE file
  ADD CONSTRAINT file_work_id_fkey
  FOREIGN KEY (work_id) REFERENCES work(work_id);

ALTER TABLE file DROP CONSTRAINT IF EXISTS file_publication_id_fkey;
ALTER TABLE file
  ADD CONSTRAINT file_publication_id_fkey
  FOREIGN KEY (publication_id) REFERENCES publication(publication_id);

ALTER TABLE file_upload DROP CONSTRAINT IF EXISTS file_upload_work_id_fkey;
ALTER TABLE file_upload
  ADD CONSTRAINT file_upload_work_id_fkey
  FOREIGN KEY (work_id) REFERENCES work(work_id);

ALTER TABLE file_upload DROP CONSTRAINT IF EXISTS file_upload_publication_id_fkey;
ALTER TABLE file_upload
  ADD CONSTRAINT file_upload_publication_id_fkey
  FOREIGN KEY (publication_id) REFERENCES publication(publication_id);
