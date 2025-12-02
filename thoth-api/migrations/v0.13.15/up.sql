ALTER TABLE affiliation
    ADD CONSTRAINT affiliation_affiliation_ordinal_contribution_id_uniq UNIQUE (contribution_id, affiliation_ordinal) DEFERRABLE INITIALLY IMMEDIATE;

DROP INDEX IF EXISTS affiliation_uniq_ord_in_contribution_idx;

ALTER TABLE contribution
    DROP CONSTRAINT contribution_contribution_ordinal_work_id_uniq,
    ADD CONSTRAINT contribution_contribution_ordinal_work_id_uniq UNIQUE (work_id, contribution_ordinal) DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE issue
    ADD CONSTRAINT issue_issue_ordinal_series_id_uniq UNIQUE (series_id, issue_ordinal) DEFERRABLE INITIALLY IMMEDIATE;

DROP INDEX IF EXISTS issue_uniq_ord_in_series_idx;

ALTER TABLE reference
    DROP CONSTRAINT reference_reference_ordinal_work_id_uniq,
    ADD CONSTRAINT reference_reference_ordinal_work_id_uniq UNIQUE (work_id, reference_ordinal) DEFERRABLE INITIALLY IMMEDIATE;

-- There were previously no database constraints on subject ordinals, so multiple subjects
-- of the same type could have the same ordinal. We want to enforce a stricter hierarchy,
-- which requires renumbering existing duplicates. Keep existing ordering where ordinals
-- are distinctive, otherwise renumber them based on the order in which they were created.
-- Note that records created prior to the introduction of `created_at` in v0.2.11 may have
-- identical default values for the creation timestamp. Therefore, we perform a backup
-- sort on the system column `ctid`; although this value is subject to change and
-- should not be relied upon, it should give a suitable rough ordering here.
-- !!! This is irreversible
UPDATE subject
    SET subject_ordinal = s.rownum
    FROM (
        SELECT
            subject_id,
            row_number() OVER (PARTITION BY work_id,subject_type ORDER BY subject_ordinal,created_at,ctid) AS rownum
        FROM subject
    ) s
    WHERE subject.subject_id = s.subject_id;

ALTER TABLE subject
    ADD CONSTRAINT subject_ordinal_type_uniq UNIQUE (work_id, subject_ordinal, subject_type) DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE work_relation
    DROP CONSTRAINT work_relation_ordinal_type_uniq,
    ADD CONSTRAINT work_relation_ordinal_type_uniq UNIQUE (relator_work_id, relation_ordinal, relation_type) DEFERRABLE INITIALLY IMMEDIATE;
