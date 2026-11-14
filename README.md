# Peerfora

Peerfora is a Rust modular monolith. The first implemented domain is account authentication, with domain ownership kept explicit so it can scale within one application before any service extraction is needed.

Platform foundations now live beside the API: Leptos web, SwiftUI iOS/iPadOS, Kotlin Compose Android, student Tauri desktop, and a separate Tauri administration desktop. Their current state and operating-system requirements are recorded in `docs/platforms/` and summarized in `docs/status.md`.

## Start locally

Requirements: Rust toolchain, Docker Compose, and a PostgreSQL container.

```sh
docker compose up -d postgres
cp .env.example .env
cargo run
```

The server listens at `127.0.0.1:3000` by default. Startup connects to PostgreSQL and applies migrations. Check HTTP liveness and database readiness with:

```sh
curl http://127.0.0.1:3000/health
curl http://127.0.0.1:3000/health/ready
```

The `/health` response reports that the HTTP process is responding. `/health/ready` checks that PostgreSQL can answer a simple query. Set `PEERFORA_HOST=0.0.0.0` to accept connections from outside the local machine. Environment variables override `.env` values.

Before running the API, set `PEERFORA_AUTH_RATE_LIMIT_HASH_KEY` in `.env` to a stable secret of at least 32 bytes. Generate a local value with `openssl rand -hex 32`. All API instances using the same database must use the same key so rate-limit identifiers remain consistent. Set `PEERFORA_ALLOWED_ORIGINS` to the comma-separated exact web origins that may call the API; it defaults to `PEERFORA_PUBLIC_URL`. `PEERFORA_REQUEST_TIMEOUT_SECONDS` sets the maximum request-processing time and defaults to 30 seconds.

Optional development containers can be started with `docker compose --profile cache --profile search --profile object-storage --profile email up -d`. These profiles are not required by the current API implementation. The object-storage profile uses S3Mock and does not represent production storage.

Email confirmation and password recovery need SMTP settings. Google and Apple sign-in need provider credentials and registered callback URLs. The API starts without these optional integrations, but their flows return a typed configuration error until configured. Country policies are configured with `PEERFORA_COUNTRY_POLICIES`; no country is enabled by default.

## Administrator access

Administrator identities are separate from student accounts. Before using administrator APIs, configure `PEERFORA_ADMIN_MFA_ENCRYPTION_KEY` with a base64-encoded 32-byte secret from deployment secret management. For local-only development, generate one with `openssl rand -base64 32` and place it in the untracked `.env` file. Provision the first administrator once against the configured PostgreSQL database:

```sh
PEERFORA_BOOTSTRAP_ADMIN_EMAIL=admin@example.test \
PEERFORA_BOOTSTRAP_ADMIN_PASSWORD='use-a-long-unique-password' \
cargo run --locked --bin provision_first_administrator
```

The migration seeds one inactive root administrator with no usable password. Run the bootstrap command once to set its email and password; it prompts locally for both and does not take credentials as command-line arguments or environment variables. The command refuses to reactivate the root. Sign in at `/api/v1/admin/auth/login`, enroll an authenticator app, then confirm its current code. A complete admin session is created only after valid MFA.

## Ansofra structure

Startup follows `main.rs → index.rs → server.rs → app.rs → loaders/`. HTTP requests flow through `routes → middlewares → validations → controllers → services → repositories → models → database`.

Services are grouped by domain. State-changing use cases belong in `services/<domain>/commands/`; read-only use cases belong in `services/<domain>/queries/`. Keep one use case per descriptively named file. Controllers handle HTTP, services enforce product rules, repositories own SQL, and models describe persisted data and shared app state.

Other folders have explicit jobs: `configs/` parses deployment settings, `constants/` holds source-level fixed values, `dtos/` defines API shapes, `enums/` defines closed domain values, `errors/` maps typed failures to safe HTTP responses, `interfaces/` defines real dependency boundaries, `integrations/` implements external adapters, `jobs/` runs background work, `utils/` holds stateless helpers, and `validations/` checks untrusted input syntax.

Read [AGENTS.md](AGENTS.md) and the codebase-aligned [master engineering specification](docs/master-engineering-specification.md) before changing this repository. Current implementation state is in [docs/status.md](docs/status.md), architecture in [docs/architecture.md](docs/architecture.md), endpoint contracts in [docs/endpoints.md](docs/endpoints.md), and database rules in [docs/database.md](docs/database.md).
