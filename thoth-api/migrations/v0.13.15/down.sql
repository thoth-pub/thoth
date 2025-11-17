ALTER TABLE contribution
    DROP CONSTRAINT contribution_contribution_ordinal_work_id_uniq,
    ADD CONSTRAINT contribution_contribution_ordinal_work_id_uniq UNIQUE (contribution_ordinal, work_id);
