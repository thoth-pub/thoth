CREATE TYPE relation_type AS ENUM (
    'replaces',
    'has-translation',
    'has-part',
    'has-child',
    'is-replaced-by',
    'is-translation-of',
    'is-part-of',
    'is-child-of'
);

CREATE TABLE work_relation (
    work_relation_id    UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    relator_work_id     UUID NOT NULL REFERENCES work(work_id) ON DELETE CASCADE,
    related_work_id     UUID NOT NULL REFERENCES work(work_id) ON DELETE CASCADE,
    relation_type       relation_type NOT NULL,
    relation_ordinal    INTEGER NOT NULL CHECK (relation_ordinal > 0),
    created_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT work_relation_ids_check CHECK (relator_work_id != related_work_id),
    CONSTRAINT work_relation_ordinal_type_uniq UNIQUE (relation_ordinal, relator_work_id, relation_type),
    -- Two works cannot have more than one relationship.
    CONSTRAINT work_relation_relator_related_uniq UNIQUE (relator_work_id, related_work_id),
    -- Two records must exist for each relationship, one representing the 'active' relation_type
    -- (e.g. 'has-child'), and one representing the 'passive' type (e.g. 'is-child-of').
    -- Ensure that each relator/related record has a corresponding related/relator record
    -- (note we cannot verify that the relation_types themselves form a matching pair).
    CONSTRAINT work_relation_active_passive_pair
        FOREIGN KEY (relator_work_id, related_work_id)
        REFERENCES work_relation (related_work_id, relator_work_id)
        -- Allow transaction to complete before enforcing constraint
        -- (so that pairs of records can be created/updated in tandem)
        DEFERRABLE INITIALLY DEFERRED
);
SELECT diesel_manage_updated_at('work_relation');

CREATE TABLE work_relation_history (
    work_relation_history_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    work_relation_id         UUID NOT NULL REFERENCES work_relation(work_relation_id) ON DELETE CASCADE,
    account_id               UUID NOT NULL REFERENCES account(account_id),
    data                     JSONB NOT NULL,
    timestamp                TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE work
    -- Restrict the original edition-not-null constraint to non-chapter work types.
    ALTER COLUMN edition DROP NOT NULL,
    ADD CONSTRAINT work_non_chapter_has_edition CHECK
        (edition IS NOT NULL OR work_type = 'book-chapter');

-- If any chapter records exist, clear any values from existing fields
-- which are about to be newly constrained to null for chapters.
UPDATE work
    SET edition = NULL, width = NULL, height = NULL, toc = NULL, lccn = NULL, oclc = NULL
    WHERE work_type = 'book-chapter';

ALTER TABLE work
    ADD CONSTRAINT work_chapter_no_edition CHECK
        (edition IS NULL OR work_type <> 'book-chapter'),
    ADD CONSTRAINT work_chapter_no_width CHECK
        (width IS NULL OR work_type <> 'book-chapter'),
    ADD CONSTRAINT work_chapter_no_height CHECK
        (height IS NULL OR work_type <> 'book-chapter'),
    ADD CONSTRAINT work_chapter_no_toc CHECK
        (toc IS NULL OR work_type <> 'book-chapter'),
    ADD CONSTRAINT work_chapter_no_lccn CHECK
        (lccn IS NULL OR work_type <> 'book-chapter'),
    ADD CONSTRAINT work_chapter_no_oclc CHECK
        (oclc IS NULL OR work_type <> 'book-chapter');
