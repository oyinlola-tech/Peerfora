ALTER TABLE agent_actor_identities
    ADD COLUMN model_name TEXT
        CHECK (model_name IS NULL OR length(btrim(model_name)) BETWEEN 1 AND 120),
    ADD COLUMN model_version TEXT
        CHECK (model_version IS NULL OR length(btrim(model_version)) BETWEEN 1 AND 120),
    ADD COLUMN supported_languages TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    ADD COLUMN institution_directory_id UUID
        REFERENCES institutions(id) ON DELETE RESTRICT,
    ADD CONSTRAINT agent_supported_languages_count_check
        CHECK (cardinality(supported_languages) <= 20),
    ADD CONSTRAINT agent_supported_languages_values_check
        CHECK (array_position(supported_languages, NULL) IS NULL);

CREATE INDEX agent_actor_identities_directory_institution_idx
    ON agent_actor_identities (institution_directory_id)
    WHERE institution_directory_id IS NOT NULL;
