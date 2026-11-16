# Test foundation

The API has Rust unit tests and PostgreSQL-backed HTTP integration tests. Use a new disposable PostgreSQL database and set `PEERFORA_TEST_DATABASE_URL`, then run `cargo test --locked -- --test-threads=1`. Some tests return early when that variable is absent, so a test run without it does not verify PostgreSQL behavior. The API CI workflow provisions PostgreSQL 17 and runs the serialized suite, formatting, Clippy, and build checks. Client build workflows do not by themselves verify product behavior.

Phase 3.6 adds password-reset race and confirmation-form tests, but these current changes have not been run as Rust/HTTP tests locally. The base GitHub API workflow passed its serialized PostgreSQL suite. Use a fresh disposable database: several suites make bootstrap and aggregate assumptions.
