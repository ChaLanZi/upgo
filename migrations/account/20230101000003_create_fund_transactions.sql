CREATE TABLE fund_transactions (
    id              UUID PRIMARY KEY,
    user_id         UUID NOT NULL REFERENCES users(id),
    account_id      UUID NOT NULL REFERENCES fund_accounts(id),
    type            VARCHAR(20) NOT NULL,
    amount          BIGINT NOT NULL,
    balance_before  BIGINT NOT NULL,
    balance_after   BIGINT NOT NULL,
    order_id        VARCHAR(100),
    remark          TEXT NOT NULL DEFAULT '',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_fund_transactions_user_id ON fund_transactions(user_id);
CREATE INDEX idx_fund_transactions_created_at ON fund_transactions(created_at);
