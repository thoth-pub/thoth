-- Make ORCID validation more permissive as the docs don't specify a strict pattern
-- Should be kept in line with Orcid::FromStr, although regex syntax differs slightly

ALTER TABLE contributor
    DROP CONSTRAINT contributor_orcid_check,
    ADD CONSTRAINT contributor_orcid_check
    CHECK (orcid ~ '^https:\/\/orcid\.org\/\d{4}-\d{4}-\d{4}-\d{3}[\dX]$');
