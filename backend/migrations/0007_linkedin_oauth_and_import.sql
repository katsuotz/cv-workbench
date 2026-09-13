CREATE TABLE linkedin_login_states (
    state_hash bytea PRIMARY KEY,
    intent text NOT NULL CHECK (intent IN ('login', 'import')),
    nonce text NOT NULL CHECK (char_length(nonce) BETWEEN 32 AND 256),
    pkce_verifier text NOT NULL CHECK (char_length(pkce_verifier) BETWEEN 43 AND 128),
    expires_at timestamptz NOT NULL,
    anonymous_session_id uuid REFERENCES anonymous_sessions(id) ON DELETE CASCADE,
    authenticated_user_id uuid REFERENCES users(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (anonymous_session_id IS NULL OR authenticated_user_id IS NULL)
);

CREATE INDEX linkedin_login_states_expires_at_idx ON linkedin_login_states(expires_at);

CREATE TABLE linkedin_pending_imports (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    data jsonb NOT NULL CHECK (octet_length(data::text) <= 1048576),
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX linkedin_pending_imports_user_idx
    ON linkedin_pending_imports(user_id, created_at DESC);
CREATE INDEX linkedin_pending_imports_expires_at_idx
    ON linkedin_pending_imports(expires_at);
