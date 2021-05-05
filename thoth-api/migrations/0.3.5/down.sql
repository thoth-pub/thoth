-- Convert Issue table to use composite key instead of single primary key

ALTER TABLE issue_history
    ADD COLUMN series_id UUID,
    ADD COLUMN work_id UUID;

UPDATE issue_history
    SET series_id = issue.series_id,
        work_id = issue.work_id
    FROM issue
    WHERE issue_history.issue_id = issue.issue_id;

ALTER TABLE issue_history
    DROP COLUMN issue_id,
    ALTER COLUMN series_id SET NOT NULL,
    ALTER COLUMN work_id SET NOT NULL;

ALTER TABLE issue
    DROP COLUMN issue_id,
    ADD PRIMARY KEY (series_id, work_id),
    -- Remove the manually-added constraint which will now be enforced by the composite key
    DROP CONSTRAINT issue_series_id_work_id_uniq;

ALTER TABLE issue_history
    ADD CONSTRAINT issue_history_series_id_work_id_fkey
    FOREIGN KEY (series_id, work_id)
    REFERENCES issue(series_id, work_id)
    ON DELETE CASCADE;

-- Convert Contribution table to use composite key instead of single primary key

ALTER TABLE contribution_history
    ADD COLUMN work_id UUID,
    ADD COLUMN contributor_id UUID,
    ADD COLUMN contribution_type contribution_type;

UPDATE contribution_history
    SET work_id = contribution.work_id,
        contributor_id = contribution.contributor_id,
        contribution_type = contribution.contribution_type
    FROM contribution
    WHERE contribution_history.contribution_id = contribution.contribution_id;

ALTER TABLE contribution_history
    DROP COLUMN contribution_id,
    ALTER COLUMN work_id SET NOT NULL,
    ALTER COLUMN contributor_id SET NOT NULL,
    ALTER COLUMN contribution_type SET NOT NULL;

ALTER TABLE contribution
    DROP COLUMN contribution_id,
    ADD PRIMARY KEY (work_id, contributor_id, contribution_type),
    -- Remove the manually-added constraint which will now be enforced by the composite key
    DROP CONSTRAINT contribution_work_id_contributor_id_contribution_type_uniq;

ALTER TABLE contribution_history
    ADD CONSTRAINT contribution_history_work_id_contributor_id_contribution_t_fkey
    FOREIGN KEY (work_id, contributor_id, contribution_type)
    REFERENCES contribution(work_id, contributor_id, contribution_type)
    ON DELETE CASCADE;
