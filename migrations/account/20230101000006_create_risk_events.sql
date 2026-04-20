CREATE TABLE risk_events (
    id          UUID PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES users(id),
    rule_name   VARCHAR(100) NOT NULL,
    condition   TEXT NOT NULL,
    action      VARCHAR(20) NOT NULL,
    detail      TEXT NOT NULL DEFAULT '',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_risk_events_user_id ON risk_events(user_id);
CREATE INDEX idx_risk_events_created_at ON risk_events(created_at);
