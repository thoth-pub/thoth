CREATE TYPE event_type AS ENUM (
    'WorkCreated',
    'WorkUpdated',
    'WorkPublished'
);

CREATE TABLE webhook (
    webhook_id      UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    publisher_id    UUID NOT NULL REFERENCES publisher(publisher_id) ON DELETE CASCADE,
    endpoint        TEXT NOT NULL CHECK (endpoint ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'),
    token           TEXT CHECK (OCTET_LENGTH(token) >= 1),
    is_published    BOOLEAN NOT NULL,
    event_type      event_type NOT NULL,
    payload         TEXT CHECK (OCTET_LENGTH(payload) >= 1),
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
SELECT diesel_manage_updated_at('webhook');

CREATE INDEX idx_webhook_endpoint ON webhook (endpoint);
CREATE INDEX idx_webhook_publisher_id ON webhook (publisher_id);

CREATE TABLE webhook_history (
    webhook_history_id       UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    webhook_id               UUID NOT NULL REFERENCES webhook(webhook_id) ON DELETE CASCADE,
    account_id               UUID NOT NULL REFERENCES account(account_id),
    data                     JSONB NOT NULL,
    timestamp                TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
