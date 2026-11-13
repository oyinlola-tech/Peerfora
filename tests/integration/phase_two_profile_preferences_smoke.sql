DO $database_safety_check$
BEGIN
    IF current_database() !~* '(^|_)test($|_)' THEN
        RAISE EXCEPTION 'this smoke test may run only in a test database';
    END IF;
END
$database_safety_check$;

BEGIN;

DO $profile_preferences_test$
DECLARE
    test_account_id UUID;
    profile_count INTEGER;
    preference_count INTEGER;
    affiliation_is_hidden BOOLEAN;
BEGIN
    INSERT INTO accounts (
        email, username, display_name, password_hash, eligibility_country_code,
        age_policy_version, terms_accepted_at
    ) VALUES (
        'profile-preferences-smoke@example.test', 'profile_preferences_smoke',
        'Profile Smoke', 'test-password-hash', 'NG', 'test-policy', now()
    ) RETURNING id INTO test_account_id;

    SELECT count(*) INTO profile_count
    FROM user_profiles WHERE user_profiles.account_id = test_account_id;
    SELECT count(*) INTO preference_count
    FROM account_preferences WHERE account_preferences.account_id = test_account_id;

    IF profile_count <> 1 OR preference_count <> 1 THEN
        RAISE EXCEPTION 'new account did not receive default profile and preferences';
    END IF;

    UPDATE user_profiles SET profile_visibility = 'private'
    WHERE user_profiles.account_id = test_account_id;

    IF EXISTS (
        SELECT 1 FROM user_profiles
        WHERE user_profiles.account_id = test_account_id
            AND user_profiles.profile_visibility = 'public'
    ) THEN
        RAISE EXCEPTION 'private profile could be selected as public';
    END IF;

    UPDATE user_profiles
    SET profile_visibility = 'public', academic_level_id = 'secondary_school'
    WHERE user_profiles.account_id = test_account_id;

    SELECT academic_level_id IS NULL INTO affiliation_is_hidden
    FROM (
        SELECT CASE WHEN show_academic_affiliation THEN academic_level_id END AS academic_level_id
        FROM user_profiles WHERE account_id = test_account_id
    ) AS public_profile_projection;

    IF affiliation_is_hidden IS DISTINCT FROM true THEN
        RAISE EXCEPTION 'academic affiliation is visible without owner opt-in';
    END IF;

    UPDATE user_profiles SET show_academic_affiliation = true
    WHERE user_profiles.account_id = test_account_id;

    SELECT academic_level_id IS NOT NULL INTO affiliation_is_hidden
    FROM (
        SELECT CASE WHEN show_academic_affiliation THEN academic_level_id END AS academic_level_id
        FROM user_profiles WHERE account_id = test_account_id
    ) AS public_profile_projection;

    IF affiliation_is_hidden IS DISTINCT FROM true THEN
        RAISE EXCEPTION 'academic affiliation opt-in was not applied';
    END IF;

    UPDATE account_preferences
    SET language_tag = 'en-GB', personalization_enabled = false
    WHERE account_preferences.account_id = test_account_id;

    IF NOT EXISTS (
        SELECT 1 FROM account_preferences
        WHERE account_preferences.account_id = test_account_id
            AND language_tag = 'en-GB'
            AND personalization_enabled = false
    ) THEN
        RAISE EXCEPTION 'account preferences did not persist';
    END IF;
END
$profile_preferences_test$;

ROLLBACK;
