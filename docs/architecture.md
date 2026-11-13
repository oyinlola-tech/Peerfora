# Peerfora API architecture

## Startup

`main.rs → index.rs → server.rs → app.rs → loaders/`

`main.rs` calls the index entry point. The index starts the server. The server reads typed settings, connects PostgreSQL, applies migrations, wires configured adapters, and starts the listener. The app builds the router. Loaders register routes, middleware, application state, and background jobs.

## Request flow

`client → route → middleware → validation → controller → service → repository → model → database`

The route chooses an HTTP handler. Middleware handles cross-cutting request behavior. Validators check incoming data shape and syntax. Controllers handle HTTP and call a named account use case. Services enforce product rules. Repositories own SQL and transactions. Models describe persisted records. DTOs define the public request and response shape.

## Folder map

| Folder | Responsibility | Current Peerfora use |
| --- | --- | --- |
| `configs/` | Parse environment variables into typed settings | PostgreSQL, cookie behavior, country policy, SMTP, Google, and Apple settings |
| `constants/` | Fixed application values | Health and account-auth paths, token lifetimes, cookie names, input limits |
| `controllers/` | HTTP request and response handling | Health, account registration, sessions, recovery, and social sign-in |
| `database/` | PostgreSQL pool and migration startup | Connects to PostgreSQL and applies SQL migrations |
| `dtos/` | API wire shapes | Health, registration, login, reset, and account responses |
| `enums/` | Finite domain values | Account-auth failures and Google/Apple provider identity |
| `errors/` | Error types and HTTP mapping | Stable auth errors and safe database/setup responses |
| `interfaces/` | Contracts at real external boundaries | `EmailSender` separates account use cases from SMTP transport |
| `integrations/` | External service adapters | SMTP implementation of `EmailSender` |
| `jobs/` | Independent background work | Remove expired auth, reset, session, and social-flow records |
| `loaders/` | Startup wiring | Routes, request logging, and the expired-token job |
| `middlewares/` | Cross-cutting request behavior | Request method/path/status logging without query strings |
| `models/` | Persisted data and shared app state | Account, database pool, policy, email adapter, and provider configuration |
| `repositories/` | SQL reads, writes, and transactions | Account, session, token, OAuth attempt, and social identity storage |
| `routes/` | HTTP method/path mapping | Health and versioned account-auth routes |
| `services/<domain>/commands/` | State-changing account use cases | Registration, login, confirmation, reset, session, and social commands |
| `services/<domain>/queries/` | Read-only account use cases | Find the signed-in account for a session |
| `utils/` | Stateless reusable helpers | Argon2 password hashing and one-time token generation/hashing |
| `validations/` | Incoming data syntax checks | Email, date format, and two-letter country-code validation |

`communities`, `discussions`, `discovery`, `moderation`, `notifications`, `institutions`, `academic_directory`, `memberships`, `voting`, `reputation`, `feeds`, and `bookmarks` are future product domains. Their command/query directories are structural placeholders with no implemented use cases. Add behavior only with the corresponding feature. The background job is used for cleanup; email remains request-driven until a durable delivery queue is required.

## Account-auth flow

- Email/password registration checks terms, username, date of birth, configured country eligibility, password length, and email syntax. The date of birth is not persisted. Account creation records the country and policy version. Email confirmation is sent through the configured `EmailSender` adapter.
- Email/password login requires a confirmed email. Session tokens are opaque, generated from secure randomness, and only their SHA-256 hashes are persisted. Browser sessions use HttpOnly cookies; bearer tokens are accepted for API clients.
- Password-reset links are single-use, expire after one hour, and revoke all account sessions when used.
- Google and Apple use OpenID Connect authorization code flow with state, nonce, and PKCE. Provider issuer, signature, audience, expiry, and nonce validation are performed by the OIDC library. Provider email must be verified. Email matches do not automatically link identities.
- A first-time social identity is held as a short-lived pending signup. The browser is redirected to `PEERFORA_PUBLIC_URL/complete-social-signup`. It must POST its birthday, country, username, and terms acceptance to `/api/v1/auth/social/complete`. The API creates no account or session until those checks pass. Provider birthday claims do not bypass this page.
- Country policies are supplied as JSON in `PEERFORA_COUNTRY_POLICIES`; no country is enabled by default. A policy requiring age assurance cannot pass until its provider is implemented and configured.

## Scaling and ownership

The API remains one deployable Rust modular monolith. A domain may call another domain's public service or interface, never another domain's repository or tables. Keep writes and transactions inside their owning repository/domain. PostgreSQL is the current source of truth. Add a queue, cache, replica, or separate service only when an actual workflow or operational evidence requires it.

## Endpoint reference

Every registered endpoint, with its request and response contracts, authentication, cookies, and errors, is documented in [endpoints.md](endpoints.md). Update that reference alongside route, DTO, or error changes.

The product and platform roadmap is in [master-engineering-specification.md](master-engineering-specification.md). Its opening codebase-alignment section defines how roadmap items fit the current API structure and implementation phases.

## Not implemented yet

OAuth providers require real client credentials and registered callback URLs. Email confirmation and recovery require SMTP configuration. The social completion API exists, but the frontend birthday page is not in this backend repository. Country policy values are unresolved product-owner decisions. Detailed age-assurance integrations, eligibility-change audit workflow, user linking UI, rate limiting, account export/deletion, and full email-delivery retry infrastructure remain future work.
