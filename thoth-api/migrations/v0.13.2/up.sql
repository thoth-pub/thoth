CREATE TYPE event_type AS ENUM (
    'work-created',
    'work-updated',
    'work-published',
);

CREATE TABLE event (
    event_id        UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_type      event_type NOT NULL,
    work_id         UUID NOT NULL REFERENCES work(work_id) ON DELETE CASCADE,
    is_published    BOOLEAN NOT NULL DEFAULT False,
    event_timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
)