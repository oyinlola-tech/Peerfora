# Database migrations

SQLx migration files live in the repository's root `migrations/` directory and are applied during API startup. The current migration is `202611120001_create_account_authentication.sql`.

Local PostgreSQL starts with `docker compose up -d postgres`. The API applies migrations on startup. Do not edit an already applied migration; add a new migration for schema changes.
