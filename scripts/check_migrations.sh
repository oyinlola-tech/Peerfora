#!/usr/bin/env bash
#
# Apply every SQL migration in filename order to a disposable PostgreSQL
# database and assert the resulting schema. This is an independent check that
# catches migration syntax errors, missing migrations, and schema drift without
# requiring the Rust build, complementing the integration-test job.
set -euo pipefail

root_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
database_url="${PEERFORA_MIGRATION_DATABASE_URL:-${DATABASE_URL:-}}"

if [[ -z "$database_url" ]]; then
    printf 'No database URL provided; set PEERFORA_MIGRATION_DATABASE_URL or DATABASE_URL.\n' >&2
    exit 1
fi

# Wait for the database to accept connections before applying migrations.
for _ in $(seq 1 60); do
    if psql "$database_url" -c 'SELECT 1' >/dev/null 2>&1; then
        break
    fi
    sleep 1
done

if ! psql "$database_url" -c 'SELECT 1' >/dev/null 2>&1; then
    printf 'Database is not reachable at the configured URL.\n' >&2
    exit 1
fi

shopt -s nullglob
migration_files=("$root_directory"/migrations/*.sql)
if [[ ${#migration_files[@]} -eq 0 ]]; then
    printf 'No migration files found in %s.\n' "$root_directory/migrations" >&2
    exit 1
fi

# Apply migrations in filename (chronological) order. ON_ERROR_STOP aborts the
# first failing statement so a broken migration fails the job immediately.
for migration_file in "${migration_files[@]}"; do
    printf 'Applying %s\n' "$(basename "$migration_file")"
    psql "$database_url" -v ON_ERROR_STOP=1 -f "$migration_file"
done

# Assert the Phase Four schema and the moderation-report target CHECK exist and
# are internally consistent. Any missing object raises an exception, which makes
# psql exit non-zero and fails the job.
psql "$database_url" -v ON_ERROR_STOP=1 <<'SQL'
DO $$
DECLARE
    required_table TEXT;
    missing_tables TEXT := '';
    check_constraint_count INTEGER;
BEGIN
    FOR required_table IN
        SELECT unnest(ARRAY[
            'notifications', 'notification_preferences', 'device_push_tokens',
            'conversations', 'conversation_members', 'user_blocks', 'messages',
            'message_read_states', 'media_objects', 'media_upload_sessions',
            'media_processing_jobs', 'email_delivery_outbox', 'realtime_event_cursors',
            'outbox_events', 'moderation_reports'
        ])
    LOOP
        IF NOT EXISTS (
            SELECT 1 FROM information_schema.tables
            WHERE table_schema = 'public' AND table_name = required_table
        ) THEN
            missing_tables := CASE
                WHEN missing_tables = '' THEN required_table
                ELSE missing_tables || ', ' || required_table
            END;
        END IF;
    END LOOP;
    IF missing_tables <> '' THEN
        RAISE EXCEPTION 'Missing tables: %', missing_tables;
    END IF;

    PERFORM 1 FROM pg_constraint WHERE conname = 'moderation_reports_check';
    IF NOT FOUND THEN
        RAISE EXCEPTION 'moderation_reports_check constraint is missing';
    END IF;

    -- The target CHECK must require exactly one target column to be non-null.
    SELECT count(*) INTO check_constraint_count
    FROM pg_get_constraintdef(
        (SELECT oid FROM pg_constraint WHERE conname = 'moderation_reports_check')
    ) AS definition
    WHERE definition ILIKE '%num_nonnulls%= 1%';
    IF check_constraint_count = 0 THEN
        RAISE EXCEPTION 'moderation_reports_check does not enforce exactly one target';
    END IF;
END
$$;
SQL

printf 'All migrations applied and schema assertions passed.\n'
