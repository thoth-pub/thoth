ALTER TABLE work DROP CONSTRAINT work_reference_check1;
ALTER TABLE work ADD CONSTRAINT work_place_check CHECK (octet_length(place) >= 1);

ALTER TABLE institution RENAME CONSTRAINT funder_pkey TO institution_pkey;
ALTER TABLE institution RENAME CONSTRAINT funder_doi_uniq_idx TO institution_doi_uniq_idx;
ALTER TABLE institution RENAME CONSTRAINT funder_funder_doi_check TO institution_institution_doi_check;
ALTER TABLE institution RENAME CONSTRAINT funder_funder_name_check TO institution_institution_name_check;