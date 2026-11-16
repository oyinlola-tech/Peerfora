CREATE TABLE agent_machine_credentials (
    id UUID PRIMARY KEY,
    agent_actor_id UUID NOT NULL REFERENCES agent_actor_identities(actor_id) ON DELETE RESTRICT,
    token_hash CHAR(64) NOT NULL UNIQUE,
    name TEXT NOT NULL CHECK (length(btrim(name)) BETWEEN 3 AND 80),
    environment TEXT NOT NULL CHECK (environment IN ('sandbox', 'production')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    UNIQUE (id, agent_actor_id),
    CHECK (expires_at > created_at),
    CHECK (revoked_at IS NULL OR revoked_at >= created_at)
);

CREATE INDEX agent_machine_credentials_agent_created_idx
    ON agent_machine_credentials (agent_actor_id, created_at DESC, id DESC);
CREATE INDEX agent_machine_credentials_expiry_idx
    ON agent_machine_credentials (expires_at)
    WHERE revoked_at IS NULL;

CREATE TABLE agent_machine_credential_capabilities (
    credential_id UUID NOT NULL,
    agent_actor_id UUID NOT NULL,
    capability_key TEXT NOT NULL,
    PRIMARY KEY (credential_id, capability_key),
    FOREIGN KEY (credential_id, agent_actor_id)
        REFERENCES agent_machine_credentials(id, agent_actor_id) ON DELETE RESTRICT,
    FOREIGN KEY (agent_actor_id, capability_key)
        REFERENCES agent_capability_requests(agent_actor_id, capability_key) ON DELETE RESTRICT
);

CREATE TABLE agent_machine_credential_rate_limits (
    credential_id UUID PRIMARY KEY
        REFERENCES agent_machine_credentials(id) ON DELETE RESTRICT,
    window_started_at TIMESTAMPTZ NOT NULL,
    request_count INTEGER NOT NULL CHECK (request_count > 0)
);
