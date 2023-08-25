-- Reinstate earlier version of ORCID validation

ALTER TABLE contributor
    DROP CONSTRAINT contributor_orcid_check,
    ADD CONSTRAINT contributor_orcid_check
    CHECK (orcid ~ '^https:\/\/orcid\.org\/0000-000(1-[5-9]|2-[0-9]|3-[0-4])\d{3}-\d{3}[\dX]$');
