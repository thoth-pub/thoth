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
    CONSTRAINT work_relation_ordinal_type_uniq UNIQUE (relation_ordinal, relator_work_id, relation_type)
);
SELECT diesel_manage_updated_at('work_relation');

CREATE TABLE work_relation_history (
    work_relation_history_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    work_relation_id         UUID NOT NULL REFERENCES work_relation(work_relation_id) ON DELETE CASCADE,
    account_id               UUID NOT NULL REFERENCES account(account_id),
    data                     JSONB NOT NULL,
    timestamp                TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
