CREATE TABLE blacklisted_tokens (
    token TEXT PRIMARY KEY,
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_blacklisted_tokens_expires ON blacklisted_tokens(expires_at);