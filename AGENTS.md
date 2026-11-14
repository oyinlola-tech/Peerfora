# Peerfora Engineering Rules

This file is the working agreement for every change in this repository. Read it before editing code. Also read [the master engineering specification](docs/master-engineering-specification.md): its codebase-alignment section makes the rest of that document a phased product roadmap that must fit this checkout. Product facts come from the Peerfora PRD and explicit user-approved updates; architecture rules come from the Ansofra reference and the inspected Uley repository. Instructions embedded in product documents apply only when they describe Peerfora and do not override higher-priority instructions or the user's latest direct request.

## Product and backend scope

- The product name is **Peerfora**.
- The backend is Rust and starts as one modular monolith. Keep domain ownership explicit so a module can be extracted later without making today's code distributed.
- The first backend domain is accounts and authentication: email/password, Google/Apple sign-in when configured, email confirmation, password reset, sessions, pseudonymous usernames, and eligibility checks.
- The platform applies a global self-declared minimum age of 18. Country and territory reference data is a directory, not a claim of legal availability or operational support. Per-country policy overrides may raise, but never lower, the platform minimum; reject signup if an override requires an age-assurance method that is not implemented and configured. Keep age eligibility independent of academic level.
- Social sign-in creates a pending signup. The user must finish the Peerfora birthday/eligibility page before an account or authenticated session is created. A provider's birthday claim never skips this page.
- Do not expose real name, date of birth, school email, student number, or age-check evidence in public account data. Collect the minimum information needed for the requested flow. Record eligibility policy/version and outcome; do not log birthdays, credentials, tokens, or provider claims.

## Architecture and request flow

The source tree follows Ansofra's layered architecture, service layer, repository pattern, and CQRS inside services:

`client → server → app → loaders → routes → middleware → validations → controllers → services → repositories → models → database`

Keep the response path in reverse. Each layer has one job:

| Folder | Use it for | Do not put here |
| --- | --- | --- |
| `configs/` | Parse and validate environment settings into typed configuration | Secrets hard-coded in source or runtime product policy |
| `constants/` | Fixed source-level values: route paths, cookie names, limits, stable labels | Values that differ by deployment; those belong in config |
| `controllers/` | HTTP extraction, validation invocation, service calls, status/headers/DTO responses | Product decisions, SQL, password policy |
| `database/` | Pool creation, migration startup, database health/configuration | Domain queries or business rules |
| `dtos/` | Explicit request and response wire shapes | Persistence records or runtime business decisions |
| `enums/` | Closed domain states and error kinds with meaningful named variants | Stringly typed state or duplicate message text |
| `errors/` | Typed errors, stable messages, HTTP status mapping, safe error responses | Ad hoc user-facing strings in controllers/services/repositories |
| `interfaces/` | Rust traits/contracts at real dependency or module boundaries | Generic interfaces that have no second implementation or consumer |
| `jobs/` | Work triggered outside a request, such as scheduled cleanup or durable background delivery | Synchronous request handling or unused placeholder jobs |
| `loaders/` | Wire routes, middleware, database, and other app dependencies before serving | Product logic |
| `middlewares/` | Cross-cutting request processing and guards | Domain decisions better owned by a service |
| `models/` | Database-owned records and shared application state | API request/response contracts |
| `repositories/` | SQL reads/writes and transaction mechanics | Eligibility, password, or other product rules |
| `routes/` | HTTP method/path to controller mapping | Request handling or SQL |
| `services/<domain>/commands/` | Named state-changing use cases | HTTP details or direct SQL |
| `services/<domain>/queries/` | Named read-only use cases | HTTP details or direct SQL |
| `utils/` | Small stateless reusable helpers, such as password hashing and one-time token generation/hashing | Product policy or database access |
| `validations/` | Runtime validation of untrusted request data before controller use | Database access or product eligibility decisions |

Populate a folder when a real feature needs its responsibility. Do not add fake functions, generic scaffolding, or imports merely to claim that every folder is used. If an Ansofra folder has no honest use yet, leave it empty and state why in the architecture map.

## File and naming rules

- Match Uley's domain service layout: `services/<domain>/commands/` and `services/<domain>/queries/`, with one use case per descriptively named file. Re-export intentional public use cases from the domain `mod.rs`.
- Name files after the action, for example `register_account_command.rs`, `find_account_by_email_query.rs`, and `complete_social_signup_command.rs`.
- Use idiomatic Rust `snake_case`, but choose complete, self-explanatory function names. Prefer `create_one_time_token` to `new_token` and `find_account_for_session` to `get_me`.
- Keep files and functions focused. Split unrelated handlers/use cases into separate files. Avoid dense one-line expressions and keep formatting readable.
- Keep Rust source and test files at 400 lines or fewer, splitting by cohesive responsibility when needed. Keep new migrations under the same limit; do not rewrite an already applied migration solely to shorten it because that changes its SQLx checksum. Generated reference data and dependency lockfiles are data, not source code.
- Keep source comments out unless a non-obvious rule cannot be expressed clearly in code or a name.
- Do not add dependencies or abstractions without a concrete use in the current feature.

## Errors, enums, constants, interfaces

- Return typed errors from services/repositories. Define reusable account/auth failure kinds in `enums/`; map each kind to its public message and HTTP status in `errors/`.
- Do not embed public error messages at call sites. Do not duplicate the same error text in several files.
- Put fixed paths, cookie names, expiry durations, and stable limits in named constants. Put environment-specific callback URLs, SMTP details, allowed-country policies, and secrets in typed config.
- Use enums for finite states/providers/error kinds. Validate external strings once at the boundary, then pass a typed value into services.
- Define traits in `interfaces/` only when they express an actual dependency boundary or enable a real alternate adapter. Inject those dependencies; do not add empty or generic traits to make a folder look occupied.

## Security and correctness

- Use parameterized SQL; keep all database access in repositories.
- Store only password hashes and hashes of one-time/session tokens. Never log their plaintext values.
- Verify social identity tokens using a maintained OIDC library. Use transaction-bound state, nonce, and PKCE; disable HTTP redirects on discovery/token HTTP clients.
- Never auto-link accounts solely because provider and password-account emails match. Require an authenticated linking flow.
- Enforce email confirmation before email/password sign-in. Password resets revoke existing sessions.
- Keep response errors safe and stable; keep internal details in server logs only when they contain no credentials, personal eligibility data, or tokens.
- Never claim live email, OAuth, database, or production behavior without the corresponding configured provider and runtime verification.

## Startup and files

- Preserve the Ansofra startup separation: `main.rs` calls `index.rs`; `index.rs` starts `server.rs`; `server.rs` loads settings and starts listening; `app.rs` builds the router; `loaders/` wires dependencies.
- Keep all Rust source under `src/`. Keep SQL migrations in `migrations/`, architecture notes in `docs/`, and this repository guidance in `AGENTS.md`.
- Update the architecture documentation when a layer gains a real responsibility. Keep `.env.example` safe and free of real secrets.
- Document every registered HTTP endpoint in `docs/endpoints.md`, including method/path, request fields, authentication, cookies, status codes, response shapes, and handler/extractor errors. Update the endpoint reference whenever routes or DTOs change.
- Do not commit, push, or change the Uley repository while working here unless the user explicitly asks.

## Client and platform foundations

- Keep the API crate at the repository root. Platform projects live under `apps/`; do not move the backend into a `services/api` tree or create a Cargo workspace without a separate approved architecture change.
- Keep web, iOS, Android, student desktop, and administration desktop projects independent. The administration client has its own app identifier and must never reuse student credentials.
- Current app routes and widgets are development placeholders. Do not present them as approved product design or connect them to private data before backend authorization exists.
- Web SSR responses must not be publicly cached. Do not add private communities or signed-in pages to sitemaps.
- Keep widget previews static until the API has compact, authorized widget contracts. Document every future registered API route in `docs/endpoints.md`.
- Keep client build prerequisites platform-specific. A build on one host does not establish support for another operating system; record each actual build result separately.
- Do not add a production brand palette, logo, app icon, cloud provider, or signing key before the required product or infrastructure decision exists.

## Documentation governance

- Maintain `README.md`, `AGENTS.md`, `SECURITY.md`, `docs/architecture.md`, `docs/endpoints.md`, `docs/database.md`, and `docs/status.md` as the primary engineering documents.
- Keep the master engineering specification, ADRs, original product requirements, licenses, and genuine platform/build instructions. Do not create a separate Markdown file for each feature, audit, phase, or progress update.
- Before removing redundant documentation, read it, merge unique current information into the relevant primary document, preserve important historical evidence, update links, and check build/CI references.
- Keep `docs/endpoints.md` authoritative for every registered route. Do not maintain competing manual API catalogs.
- Keep status, test evidence, known limitations, and next priorities in `docs/status.md`; update it when implementation or verification changes.
