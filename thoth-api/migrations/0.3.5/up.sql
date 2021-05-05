-- Convert Issue table to use single primary key instead of composite key

ALTER TABLE issue
    ADD COLUMN issue_id UUID NOT NULL DEFAULT uuid_generate_v4();

ALTER TABLE issue_history
    ADD COLUMN issue_id UUID;

UPDATE issue_history
    SET issue_id = issue.issue_id
    FROM issue
    WHERE issue_history.series_id = issue.series_id
    AND issue_history.work_id = issue.work_id;

ALTER TABLE issue_history
    DROP COLUMN series_id,
    DROP COLUMN work_id,
    ALTER COLUMN issue_id SET NOT NULL;

ALTER TABLE issue
    DROP CONSTRAINT issue_pkey,
    ADD PRIMARY KEY (issue_id),
    -- Retain the data constraint originally enforced by the composite key
    ADD CONSTRAINT issue_series_id_work_id_uniq UNIQUE (series_id, work_id);

ALTER TABLE issue_history
    ADD CONSTRAINT issue_history_issue_id_fkey
    FOREIGN KEY (issue_id)
    REFERENCES issue(issue_id)
    ON DELETE CASCADE;

-- Convert Contribution table to use single primary key instead of composite key

ALTER TABLE contribution
    ADD COLUMN contribution_id UUID NOT NULL DEFAULT uuid_generate_v4();

ALTER TABLE contribution_history
    ADD COLUMN contribution_id UUID;

UPDATE contribution_history
    SET contribution_id = contribution.contribution_id
    FROM contribution
    WHERE contribution_history.work_id = contribution.work_id
    AND contribution_history.contributor_id = contribution.contributor_id
    AND contribution_history.contribution_type = contribution.contribution_type;

ALTER TABLE contribution_history
    DROP COLUMN work_id,
    DROP COLUMN contributor_id,
    DROP COLUMN contribution_type,
    ALTER COLUMN contribution_id SET NOT NULL;

ALTER TABLE contribution
    DROP CONSTRAINT contribution_pkey,
    ADD PRIMARY KEY (contribution_id),
    -- Retain the data constraint originally enforced by the composite key
    ADD CONSTRAINT contribution_work_id_contributor_id_contribution_type_uniq UNIQUE (work_id, contributor_id, contribution_type);

ALTER TABLE contribution_history
    ADD CONSTRAINT contribution_history_contribution_id_fkey
    FOREIGN KEY (contribution_id)
    REFERENCES contribution(contribution_id)
    ON DELETE CASCADE;
