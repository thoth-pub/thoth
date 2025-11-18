ALTER TABLE contribution
    DROP CONSTRAINT contribution_contribution_ordinal_work_id_uniq,
    ADD CONSTRAINT contribution_contribution_ordinal_work_id_uniq UNIQUE (contribution_ordinal, work_id) DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE reference
    DROP CONSTRAINT reference_reference_ordinal_work_id_uniq,
    ADD CONSTRAINT reference_reference_ordinal_work_id_uniq UNIQUE (work_id, reference_ordinal) DEFERRABLE INITIALLY IMMEDIATE;
