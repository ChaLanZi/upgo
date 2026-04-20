CREATE TABLE account_deletions (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID NOT NULL UNIQUE,
    confirmation_code   VARCHAR(6) NOT NULL,
    requested_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    soft_deleted_at     TIMESTAMPTZ,
    permanent_delete_at TIMESTAMPTZ,
    cancelled           BOOLEAN NOT NULL DEFAULT false
);

CREATE INDEX idx_account_deletions_user_id ON account_deletions(user_id);
CREATE INDEX idx_account_deletions_permanent_delete ON account_deletions(permanent_delete_at)
    WHERE permanent_delete_at IS NOT NULL AND cancelled = false;
