ALTER TABLE contribution
    ADD COLUMN institution TEXT CHECK (octet_length(institution) >= 1);

-- Migrate affiliation information back into contribution table as far as possible
-- before dropping affiliation table. Where a contribution has multiple affiliations,
-- combine the institution names into a single semicolon-separated string.
UPDATE contribution
    SET institution = subquery.institutions
    FROM (
        SELECT affiliation.contribution_id, string_agg(institution_name, '; ') AS institutions
        FROM institution, affiliation
        WHERE affiliation.institution_id = institution.institution_id
        GROUP BY affiliation.contribution_id
        ) AS subquery
    WHERE contribution.contribution_id = subquery.contribution_id;

ALTER TABLE institution_history RENAME COLUMN institution_history_id TO funder_history_id;
ALTER TABLE institution_history RENAME COLUMN institution_id TO funder_id;

ALTER TABLE institution_history RENAME TO funder_history;

ALTER TABLE institution RENAME COLUMN institution_id TO funder_id;
ALTER TABLE institution RENAME COLUMN institution_name TO funder_name;
ALTER TABLE institution RENAME COLUMN institution_doi TO funder_doi;

ALTER TABLE institution
    DROP COLUMN ror,
    DROP COLUMN country_code;

ALTER TABLE institution RENAME TO funder;

ALTER TABLE funding RENAME COLUMN institution_id TO funder_id;

DROP TYPE IF EXISTS country_code;

DROP TABLE affiliation_history;
DROP TABLE affiliation;
