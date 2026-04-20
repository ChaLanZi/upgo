CREATE TABLE fund_accounts (
    id              UUID PRIMARY KEY,
    user_id         UUID NOT NULL REFERENCES users(id),
    currency        VARCHAR(10) NOT NULL DEFAULT 'CNY',
    balance         BIGINT NOT NULL DEFAULT 0,
    frozen_balance  BIGINT NOT NULL DEFAULT 0,
    version         INTEGER NOT NULL DEFAULT 1,
    UNIQUE(user_id, currency)
);

CREATE INDEX idx_fund_accounts_user_id ON fund_accounts(user_id);
