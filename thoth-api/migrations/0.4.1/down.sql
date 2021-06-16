-- Reinstate earlier versions of ORCID and DOI validation

ALTER TABLE contributor
    DROP CONSTRAINT contributor_orcid_check,
    ADD CONSTRAINT contributor_orcid_check
    CHECK (orcid ~* '0000-000(1-[5-9]|2-[0-9]|3-[0-4])\d{3}-\d{3}[\dX]');

ALTER TABLE work
    DROP CONSTRAINT work_doi_check,
    ADD CONSTRAINT work_doi_check
    CHECK (doi ~* 'https:\/\/doi.org\/10.\d{4,9}\/[-._\;\(\)\/:a-zA-Z0-9]+$');

ALTER TABLE funder
    DROP CONSTRAINT funder_funder_doi_check,
    ADD CONSTRAINT funder_funder_doi_check
    CHECK (funder_doi ~* 'https:\/\/doi.org\/10.\d{4,9}\/[-._\;\(\)\/:a-zA-Z0-9]+$');
