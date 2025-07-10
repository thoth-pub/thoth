DROP TABLE IF EXISTS webhook_history;
DROP TRIGGER set_updated_at ON webhook;
DROP TABLE IF EXISTS webhook;
DROP TYPE IF EXISTS event_type;
