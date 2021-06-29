ALTER TABLE contribution
    ADD COLUMN contribution_ordinal INTEGER;

-- As a default, set the `contribution_ordinal` for existing records to reflect
-- the order in which they were added (within separate groups for each work).
-- We should be able to find this by sorting on the `created_at` timestamp, however,
-- records created prior to the introduction of `created_at` in v0.2.11 may have
-- identical default values for this field. Therefore, we perform a secondary
-- sort on the system column `ctid`; although this value is subject to change and
-- should not be relied upon, it should give a suitable rough ordering here.
UPDATE contribution
    SET contribution_ordinal = c.rownum
    FROM (
        SELECT
            contribution_id,
            row_number() OVER (PARTITION BY work_id ORDER BY created_at,ctid) AS rownum
        FROM contribution
    ) c
    WHERE contribution.contribution_id = c.contribution_id;

ALTER TABLE contribution
    ALTER COLUMN contribution_ordinal SET NOT NULL,
    ADD CONSTRAINT contribution_contribution_ordinal_check CHECK (contribution_ordinal > 0),
    ADD CONSTRAINT contribution_contribution_ordinal_work_id_uniq UNIQUE (contribution_ordinal, work_id);