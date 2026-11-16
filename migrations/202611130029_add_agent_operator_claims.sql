ALTER TABLE agent_actor_identities
    ALTER COLUMN operator_account_id DROP NOT NULL,
    DROP CONSTRAINT agent_actor_identities_review_status_check;

ALTER TABLE agent_actor_identities
    ADD CONSTRAINT agent_actor_identities_review_status_check
        CHECK (review_status IN (
            'pending_claim', 'pending_review', 'approved', 'active', 'restricted',
            'paused', 'rejected', 'suspended', 'revoked', 'deleted'
        )),
    ADD CONSTRAINT agent_actor_identities_operator_claim_state_check
        CHECK ((review_status = 'pending_claim' AND operator_account_id IS NULL)
            OR (review_status <> 'pending_claim' AND operator_account_id IS NOT NULL));

CREATE TABLE agent_operator_claims (
    id UUID PRIMARY KEY,
    agent_actor_id UUID NOT NULL REFERENCES agent_actor_identities(actor_id) ON DELETE RESTRICT,
    operator_email_hash CHAR(64) NOT NULL,
    token_hash CHAR(64) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    CHECK (expires_at > created_at),
    CHECK (consumed_at IS NULL OR revoked_at IS NULL)
);

CREATE UNIQUE INDEX agent_operator_claims_one_open_per_agent_idx
    ON agent_operator_claims (agent_actor_id)
    WHERE consumed_at IS NULL AND revoked_at IS NULL;
CREATE INDEX agent_operator_claims_expiration_idx
    ON agent_operator_claims (expires_at)
    WHERE consumed_at IS NULL AND revoked_at IS NULL;
