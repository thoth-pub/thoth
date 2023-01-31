ALTER TABLE work DROP CONSTRAINT work_doi_check;
ALTER TABLE work ADD CONSTRAINT work_doi_check
  CHECK (doi ~ '^https:\/\/doi\.org\/10\.\d{4,9}\/[-._\;\(\)\/:a-zA-Z0-9]+$');

ALTER TABLE reference DROP CONSTRAINT reference_doi_check;
ALTER TABLE reference ADD CONSTRAINT reference_doi_check
  CHECK (doi ~ '^https:\/\/doi\.org\/10\.\d{4,9}\/[-._\;\(\)\/:a-zA-Z0-9]+$');

ALTER TABLE institution DROP CONSTRAINT institution_institution_doi_check;
ALTER TABLE institution ADD CONSTRAINT institution_institution_doi_check
  CHECK (doi ~ '^https:\/\/doi\.org\/10\.\d{4,9}\/[-._\;\(\)\/:a-zA-Z0-9]+$');
