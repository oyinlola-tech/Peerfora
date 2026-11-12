# Peerfora

Peerfora is a Rust modular monolith. The first implemented domain is account authentication, with domain ownership kept explicit so it can scale within one application before any service extraction is needed.

## Start locally

Requirements: Rust toolchain, Docker Compose, and a PostgreSQL container.

```sh
docker compose up -d postgres
cp .env.example .env
cargo run
```

The server listens at `127.0.0.1:3000` by default. Startup connects to PostgreSQL and applies migrations. Check the HTTP process with:

```sh
curl http://127.0.0.1:3000/health
```

The health response reports that the HTTP process is responding; it is not a database health check. Set `PEERFORA_HOST=0.0.0.0` to accept connections from outside the local machine. Environment variables override `.env` values.

Email confirmation and password recovery need SMTP settings. Google and Apple sign-in need provider credentials and registered callback URLs. The API starts without these optional integrations, but their flows return a typed configuration error until configured. Country policies are configured with `PEERFORA_COUNTRY_POLICIES`; no country is enabled by default.

## Ansofra structure

Startup follows `main.rs → index.rs → server.rs → app.rs → loaders/`. HTTP requests flow through `routes → middlewares → validations → controllers → services → repositories → models → database`.

Services are grouped by domain. State-changing use cases belong in `services/<domain>/commands/`; read-only use cases belong in `services/<domain>/queries/`. Keep one use case per descriptively named file. Controllers handle HTTP, services enforce product rules, repositories own SQL, and models describe persisted data and shared app state.

Other folders have explicit jobs: `configs/` parses deployment settings, `constants/` holds source-level fixed values, `dtos/` defines API shapes, `enums/` defines closed domain values, `errors/` maps typed failures to safe HTTP responses, `interfaces/` defines real dependency boundaries, `integrations/` implements external adapters, `jobs/` runs background work, `utils/` holds stateless helpers, and `validations/` checks untrusted input syntax.

Read [AGENTS.md](AGENTS.md) before changing this repository. It contains the full working rules and folder responsibilities. See [docs/architecture.md](docs/architecture.md) for current flows and [docs/endpoints.md](docs/endpoints.md) for the full API endpoint reference.
