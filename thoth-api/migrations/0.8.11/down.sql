ALTER TABLE work DROP CONSTRAINT work_place_check;
ALTER TABLE work ADD CONSTRAINT work_reference_check1 CHECK (octet_length(reference) >= 1);

ALTER TABLE institution RENAME CONSTRAINT institution_pkey TO funder_pkey;
ALTER INDEX institution_doi_uniq_idx RENAME TO funder_doi_uniq_idx;
ALTER TABLE institution RENAME CONSTRAINT institution_funder_doi_check TO funder_funder_doi_check;
ALTER TABLE institution RENAME CONSTRAINT institution_institution_name_check TO funder_funder_name_check;