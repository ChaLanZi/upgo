CREATE TABLE users (
    id          UUID PRIMARY KEY,
    email       VARCHAR(255) NOT NULL UNIQUE,
    phone       VARCHAR(20),
    nickname    VARCHAR(100) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    kyc_status  VARCHAR(20) NOT NULL DEFAULT 'NONE',
    account_status VARCHAR(20) NOT NULL DEFAULT 'PENDING_VERIFICATION',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version     INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_account_status ON users(account_status);
