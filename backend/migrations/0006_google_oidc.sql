ALTER TABLE users
    ALTER COLUMN password_hash DROP NOT NULL,
    ADD COLUMN display_name text CHECK (display_name IS NULL OR char_length(display_name) BETWEEN 1 AND 256);

CREATE TABLE oauth_identities (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    provider text NOT NULL CHECK (char_length(provider) BETWEEN 1 AND 32),
    subject text NOT NULL CHECK (char_length(subject) BETWEEN 1 AND 256),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (provider, subject),
    UNIQUE (provider, user_id)
);

CREATE INDEX oauth_identities_user_id_idx ON oauth_identities(user_id);

CREATE TABLE google_login_states (
    state_hash bytea PRIMARY KEY,
    nonce text NOT NULL CHECK (char_length(nonce) BETWEEN 32 AND 256),
    pkce_verifier text NOT NULL CHECK (char_length(pkce_verifier) BETWEEN 43 AND 128),
    expires_at timestamptz NOT NULL,
    anonymous_session_id uuid REFERENCES anonymous_sessions(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX google_login_states_expires_at_idx ON google_login_states(expires_at);
