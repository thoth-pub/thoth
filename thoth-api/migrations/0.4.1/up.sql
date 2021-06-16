-- Improve validation of ORCID identifiers (include protocol/resource name)
-- Should be kept in line with Orcid::FromStr, although regex syntax differs slightly

ALTER TABLE contributor
    DROP CONSTRAINT contributor_orcid_check,
    ADD CONSTRAINT contributor_orcid_check
    CHECK (orcid ~* '^https:\/\/orcid\.org\/0000-000(1-[5-9]|2-[0-9]|3-[0-4])\d{3}-\d{3}[\dX]$');

-- Improve validation of DOI identifiers (add line start marker, escape periods)
-- Should be kept in line with Orcid::FromStr, although regex syntax differs slightly
-- (e.g. `;()/` need to be escaped here but not in Orcid::FromStr)

ALTER TABLE work
    DROP CONSTRAINT work_doi_check,
    ADD CONSTRAINT work_doi_check
    CHECK (doi ~* '^https:\/\/doi\.org\/10\.\d{4,9}\/[-._\;\(\)\/:a-zA-Z0-9]+$');

ALTER TABLE funder
    DROP CONSTRAINT funder_funder_doi_check,
    ADD CONSTRAINT funder_funder_doi_check
    CHECK (funder_doi ~* '^https:\/\/doi\.org\/10\.\d{4,9}\/[-._\;\(\)\/:a-zA-Z0-9]+$');
