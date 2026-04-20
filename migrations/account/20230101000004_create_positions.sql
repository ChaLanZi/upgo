CREATE TABLE positions (
    id              UUID PRIMARY KEY,
    user_id         UUID NOT NULL REFERENCES users(id),
    symbol          VARCHAR(20) NOT NULL,
    quantity        BIGINT NOT NULL DEFAULT 0,
    cost_price      BIGINT NOT NULL DEFAULT 0,
    current_price   BIGINT NOT NULL DEFAULT 0,
    unrealized_pnl  BIGINT NOT NULL DEFAULT 0,
    status          VARCHAR(10) NOT NULL DEFAULT 'OPEN',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version         INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX idx_positions_user_id ON positions(user_id);
CREATE INDEX idx_positions_user_symbol ON positions(user_id, symbol);
CREATE INDEX idx_positions_status ON positions(status);
