CREATE TABLE position_histories (
    id              UUID PRIMARY KEY,
    position_id     UUID NOT NULL REFERENCES positions(id),
    user_id         UUID NOT NULL REFERENCES users(id),
    symbol          VARCHAR(20) NOT NULL,
    change_type     VARCHAR(10) NOT NULL,
    change_quantity BIGINT NOT NULL,
    quantity_after  BIGINT NOT NULL,
    price           BIGINT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_position_histories_position_id ON position_histories(position_id);
CREATE INDEX idx_position_histories_user_id ON position_histories(user_id);
