ALTER TABLE work DROP CONSTRAINT work_doi_check;
ALTER TABLE work ADD CONSTRAINT work_doi_check
  CHECK (doi ~ '^https:\/\/doi\.org\/10\.\d{4,9}\/[-._\;\(\)\[\]\/:a-zA-Z0-9]+$');
