DO $database_safety_check$
BEGIN
    IF current_database() !~* '(^|_)test($|_)' THEN
        RAISE EXCEPTION 'this smoke test may run only in a test database';
    END IF;
END
$database_safety_check$;

BEGIN;

DO $global_directory_test$
DECLARE
    active_country_count INTEGER;
    canonical_level_count INTEGER;
    test_account_id UUID;
BEGIN
    SELECT count(*) INTO active_country_count
    FROM countries
    WHERE is_active = TRUE AND source = 'iso-codes' AND source_version = '4.20.1-1';

    IF active_country_count <> 249 THEN
        RAISE EXCEPTION 'expected all 249 ISO country and territory records, found %',
            active_country_count;
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM countries
        WHERE country_code = 'NG' AND alpha_3 = 'NGA' AND display_name = 'Nigeria'
    ) OR NOT EXISTS (
        SELECT 1 FROM countries
        WHERE country_code = 'CA' AND alpha_3 = 'CAN' AND display_name = 'Canada'
    ) OR NOT EXISTS (
        SELECT 1 FROM countries
        WHERE country_code = 'JP' AND alpha_3 = 'JPN' AND display_name = 'Japan'
    ) OR NOT EXISTS (
        SELECT 1 FROM countries
        WHERE country_code = 'BR' AND alpha_3 = 'BRA' AND display_name = 'Brazil'
    ) OR NOT EXISTS (
        SELECT 1 FROM countries
        WHERE country_code = 'AU' AND alpha_3 = 'AUS' AND display_name = 'Australia'
    ) OR NOT EXISTS (
        SELECT 1 FROM countries
        WHERE country_code = 'DE' AND alpha_3 = 'DEU' AND display_name = 'Germany'
    ) THEN
        RAISE EXCEPTION 'country snapshot is missing one or more regional fixtures';
    END IF;

    SELECT count(*) INTO canonical_level_count
    FROM academic_levels
    WHERE is_active = TRUE AND id IN (
        'secondary_school', 'foundation', 'vocational', 'technical', 'diploma',
        'associate_degree', 'undergraduate', 'masters', 'doctoral', 'postdoctoral',
        'professional_qualification', 'independent_learner'
    );

    IF canonical_level_count <> 12 THEN
        RAISE EXCEPTION 'canonical international education taxonomy is incomplete';
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM academic_level_aliases
        WHERE lower(alias) = 'high_school' AND academic_level_id = 'secondary_school'
    ) THEN
        RAISE EXCEPTION 'high_school alias does not map to secondary_school';
    END IF;

    IF EXISTS (
        SELECT 1 FROM academic_levels
        WHERE id IN ('secondary', 'professional', 'independent') AND is_active = TRUE
    ) THEN
        RAISE EXCEPTION 'legacy ambiguous academic-level keys remain active';
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM academic_qualification_types
        WHERE id = 'higher_national_diploma' AND academic_level_id = 'diploma'
    ) OR NOT EXISTS (
        SELECT 1 FROM academic_qualification_types
        WHERE id = 'doctor_of_education' AND academic_level_id = 'doctoral'
    ) THEN
        RAISE EXCEPTION 'qualification types are not separated from academic levels';
    END IF;

    INSERT INTO accounts (
        email, username, display_name, password_hash, eligibility_country_code,
        age_policy_version, terms_accepted_at
    ) VALUES (
        'directory-suggestion-smoke@example.test', 'directory_suggestion_smoke',
        'Directory Suggestion Smoke', 'test-password-hash', 'NG', 'test-policy', now()
    ) RETURNING id INTO test_account_id;

    INSERT INTO directory_suggestions (
        suggestion_type, proposed_name, parent_name, submitted_by, details
    ) VALUES (
        'subject', 'Community Energy Systems', 'Engineering', test_account_id,
        'A user-suggested subject that is not in the initial seed.'
    );

    IF NOT EXISTS (
        SELECT 1 FROM directory_suggestions
        WHERE submitted_by = test_account_id
            AND suggestion_type = 'subject'
            AND proposed_name = 'Community Energy Systems'
            AND review_status = 'pending'
    ) THEN
        RAISE EXCEPTION 'unlisted directory options cannot be submitted for review';
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM account_eligibility_assessments
        WHERE account_id = test_account_id
            AND policy_version = 'test-policy'
            AND assessment_method = 'self_declared'
            AND eligible = TRUE
    ) THEN
        RAISE EXCEPTION 'initial eligibility assessment evidence is missing';
    END IF;

    BEGIN
        INSERT INTO accounts (
            email, username, eligibility_country_code, age_policy_version, terms_accepted_at
        ) VALUES (
            'social-only-smoke@example.test', 'social_only_smoke', 'NG', 'test-policy', now()
        ) RETURNING id INTO test_account_id;
        INSERT INTO account_social_identities (account_id, provider, provider_subject)
        VALUES (test_account_id, 'google', 'social-only-subject');
        SET CONSTRAINTS ALL IMMEDIATE;
        SET CONSTRAINTS ALL DEFERRED;
    END;

    BEGIN
        INSERT INTO accounts (
            email, username, eligibility_country_code, age_policy_version, terms_accepted_at
        ) VALUES (
            'no-auth-method-smoke@example.test', 'no_auth_method_smoke', 'NG', 'test-policy', now()
        );
        SET CONSTRAINTS ALL IMMEDIATE;
        RAISE EXCEPTION 'account without an authentication method was accepted';
    EXCEPTION WHEN check_violation THEN
        SET CONSTRAINTS ALL DEFERRED;
    END;

    BEGIN
        INSERT INTO accounts (
            email, username, password_hash, eligibility_country_code, age_policy_version, terms_accepted_at
        ) VALUES (
            'DIRECTORY-SUGGESTION-SMOKE@example.test', 'duplicate_email_case_smoke',
            'test-password-hash', 'NG', 'test-policy', now()
        );
        RAISE EXCEPTION 'case-insensitive email uniqueness was not enforced';
    EXCEPTION WHEN unique_violation THEN
        NULL;
    END;

    INSERT INTO institution_types (id, display_name, source)
    VALUES ('community_learning_lab', 'Community Learning Lab', 'directory-review-smoke');

    INSERT INTO institutions (
        slug, display_name, country_code, institution_type, record_source
    ) VALUES (
        'community-learning-lab', 'Community Learning Lab', 'NG',
        'community_learning_lab', 'directory-extensibility-smoke'
    );

    IF NOT EXISTS (
        SELECT 1 FROM institutions
        WHERE slug = 'community-learning-lab'
            AND institution_type = 'community_learning_lab'
    ) THEN
        RAISE EXCEPTION 'a new institution type requires a schema change';
    END IF;
END
$global_directory_test$;

ROLLBACK;
