CREATE TYPE event_type AS ENUM (
    'WorkCreated',
    'WorkUpdated',
    'WorkPublished'
);

CREATE TABLE webhook (
    webhook_id      UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    endpoint        TEXT NOT NULL CHECK (endpoint ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'),
    token           TEXT CHECK (OCTET_LENGTH(token) >= 1),
    is_published    BOOLEAN NOT NULL,
    event_type      event_type NOT NULL,
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
SELECT diesel_manage_updated_at('webhook');

CREATE INDEX idx_webhook_endpoint ON webhook (endpoint);

CREATE TABLE publisher_webhook (
    webhook_id      UUID NOT NULL REFERENCES webhook(webhook_id) ON DELETE CASCADE,
    publisher_id    UUID NOT NULL REFERENCES publisher(publisher_id) ON DELETE CASCADE,
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (webhook_id, publisher_id)
);
SELECT diesel_manage_updated_at('publisher_webhook');

CREATE INDEX idx_publisher_webhook_webhook_id ON publisher_webhook (webhook_id);
