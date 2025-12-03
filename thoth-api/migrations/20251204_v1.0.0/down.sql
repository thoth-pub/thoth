ALTER TABLE affiliation
    DROP CONSTRAINT affiliation_affiliation_ordinal_contribution_id_uniq;

CREATE UNIQUE INDEX affiliation_uniq_ord_in_contribution_idx ON affiliation(contribution_id, affiliation_ordinal);

ALTER TABLE contribution
    DROP CONSTRAINT contribution_contribution_ordinal_work_id_uniq,
    ADD CONSTRAINT contribution_contribution_ordinal_work_id_uniq UNIQUE (contribution_ordinal, work_id);

ALTER TABLE issue
    DROP CONSTRAINT issue_issue_ordinal_series_id_uniq;

CREATE UNIQUE INDEX issue_uniq_ord_in_series_idx ON issue(series_id, issue_ordinal);

ALTER TABLE reference
    DROP CONSTRAINT reference_reference_ordinal_work_id_uniq,
    ADD CONSTRAINT reference_reference_ordinal_work_id_uniq UNIQUE (work_id, reference_ordinal);

ALTER TABLE subject
    DROP CONSTRAINT subject_ordinal_type_uniq;

ALTER TABLE work_relation
    DROP CONSTRAINT work_relation_ordinal_type_uniq,
    ADD CONSTRAINT work_relation_ordinal_type_uniq UNIQUE (relation_ordinal, relator_work_id, relation_type);
