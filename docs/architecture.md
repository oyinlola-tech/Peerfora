# Peerfora API architecture

## Startup

`main.rs → index.rs → server.rs → app.rs → loaders/`

`main.rs` calls the index entry point. The index starts the server. The server reads typed settings, connects PostgreSQL, applies migrations, installs the versioned country snapshot and validates dependent country foreign keys, wires configured adapters, and starts the listener. `/health/ready` runs a database query; `/health` remains process liveness. The app builds the router. Loaders register routes, middleware, application state, and background jobs.

## Request flow

`client → route → middleware → validation → controller → service → repository → model → database`

The route chooses an HTTP handler. Middleware handles cross-cutting request behavior. Validators check incoming data shape and syntax. Controllers handle HTTP and call a named account use case. Services enforce product rules. Repositories own SQL and transactions. Models describe persisted records. DTOs define the public request and response shape.

## Folder map

| Folder | Responsibility | Current Peerfora use |
| --- | --- | --- |
| `configs/` | Parse environment variables into typed settings | PostgreSQL, cookie behavior, country policy, SMTP, Google, and Apple settings |
| `constants/` | Fixed application values | Health and account-auth paths, token lifetimes, cookie names, input limits |
| `controllers/` | HTTP request and response handling | Health, account flows, subject discovery, communities, profiles, and preferences |
| `database/` | PostgreSQL pool, migration startup, and pinned reference-data installation | Connects to PostgreSQL, applies SQL migrations, imports the versioned country snapshot, and validates country foreign keys before serving |
| `middlewares/` | Cross-cutting request behavior | Request logging, security response headers, and JSON body-size limits |
| `dtos/` | API wire shapes | Auth, directory, community, profile, and preference requests |
| `enums/` | Finite domain values | Account-auth, directory, community, profile failures, and provider identity |
| `errors/` | Error types and HTTP mapping | Stable domain errors and safe database/setup responses |
| `interfaces/` | Contracts at real external boundaries | `EmailSender` separates account use cases from SMTP transport |
| `integrations/` | External service adapters | SMTP implementation of `EmailSender` |
| `jobs/` | Independent background work | Remove expired auth, reset, session, and social-flow records |
| `loaders/` | Startup wiring | Routes, request logging, and the expired-token job |
| `middlewares/` | Cross-cutting request behavior | Request method/path/status logging without query strings |
| `models/` | Persisted data and shared app state | Account, directory, community, profile, database pool and adapters |
| `repositories/` | SQL reads, writes, and transactions | Account/auth, subject search, community, profile, and preference storage |
| `routes/` | HTTP method/path mapping | Health and versioned auth, directory, community, and profile routes |
| `services/<domain>/commands/` | State-changing domain use cases | Account registration/login/recovery, community membership, profile and preference updates |
| `services/<domain>/queries/` | Read-only domain use cases | Session lookup, academic subject search, community discovery, and profile retrieval |
| `utils/` | Stateless reusable helpers | Argon2 password hashing and one-time token generation/hashing |
| `validations/` | Incoming data syntax checks | Email, date format, and two-letter country-code validation |

`communities`, `academic_directory`, and `profiles` contain initial Phase Two use cases. Profiles and preferences are separate from authentication records; public and own-profile query shapes differ to protect private account data. `discussions`, `discovery`, `moderation`, `notifications`, `institutions`, `memberships`, `voting`, `reputation`, `feeds`, and `bookmarks` remain planned domains with no implemented use cases. The community repository writes audit and outbox records transactionally, but there is no outbox worker yet. The background job currently handles auth cleanup; email remains request-driven until a durable delivery queue is implemented. API JSON bodies are capped at 1 MiB. Responses receive no-store, content-type, referrer, framing, and content-security headers; HSTS is emitted only when `PEERFORA_ENV=production`.

## Account-auth flow

- Email/password registration checks terms, username, date of birth, configured country eligibility, password length, and email syntax. The date of birth is not persisted. Account creation records the country and policy version. Email confirmation is sent through the configured `EmailSender` adapter.
- Email/password login requires a confirmed email. Session tokens are opaque, generated from secure randomness, and only their SHA-256 hashes are persisted. Browser sessions use HttpOnly cookies; bearer tokens are accepted for API clients.
- Password-reset links are single-use, expire after one hour, and revoke all account sessions when used.
- Google and Apple use OpenID Connect authorization code flow with state, nonce, and PKCE. Provider issuer, signature, audience, expiry, and nonce validation are performed by the OIDC library. Provider email must be verified. Email matches do not automatically link identities.
- A first-time social identity is held as a short-lived pending signup. The browser is redirected to `PEERFORA_PUBLIC_URL/complete-social-signup`. It must POST its birthday, country, username, and terms acceptance to `/api/v1/auth/social/complete`. The API creates no account or session until those checks pass. Provider birthday claims do not bypass this page.
- Country policies are supplied as JSON in `PEERFORA_COUNTRY_POLICIES`; no country is enabled by default. A configured policy cannot lower the global 18+ floor. A policy requiring age assurance cannot pass until its provider is implemented and configured.

## Administration and access-control boundary

There are no administrator identities, MFA, RBAC, or administrative routes in the API. The admin desktop shell is not an authorization boundary and remains disconnected. Do not use ordinary student sessions for administration. The database account lifecycle status only blocks ordinary account/session/social-identity lookup for non-active accounts; no user-facing restriction or administrator enforcement workflow exists.

## Scaling and ownership

The API remains one deployable Rust modular monolith. A domain may call another domain's public service or interface, never another domain's repository or tables. Keep writes and transactions inside their owning repository/domain. PostgreSQL is the current source of truth. Add a queue, cache, replica, or separate service only when an actual workflow or operational evidence requires it.

## Endpoint reference

Every registered endpoint, with its request and response contracts, authentication, cookies, and errors, is documented in [endpoints.md](endpoints.md). Update that reference alongside route, DTO, or error changes.

The product and platform roadmap is in [master-engineering-specification.md](master-engineering-specification.md). Its opening codebase-alignment section defines how roadmap items fit the current API structure and implementation phases.

## Not implemented yet

Administrator identity/MFA/RBAC, moderation/reporting, appeals, student/institution verification, privacy requests, rate limiting, CSRF/origin enforcement, request timeouts, correlation identifiers, outbox processing, generated OpenAPI, and SDK integrations are not implemented. OAuth providers require real client credentials and registered callback URLs. Email confirmation and recovery require SMTP configuration. The social completion API exists, but the frontend birthday page is not in this backend repository. Country policy values are unresolved product-owner decisions. Detailed age-assurance integrations, eligibility-change audit workflow, user linking UI, rate limiting, account export/deletion, and full email-delivery retry infrastructure remain future work.


## Cross-platform and future boundaries

The student web, iOS, Android, student desktop, admin desktop, and widget projects are separate scaffolds under `apps/` and `widgets/`. They do not own authorization or persistence. The admin Tauri project has separate identity/configuration from the student app but is not connected to a backend admin API. The design freeze remains in effect.

There is no shared Rust SDK/UniFFI, OpenAPI generator, realtime transport, AI/recommendation service, or outbox worker. Do not create service boundaries or connect optional Redis/search/object-storage containers until a concrete implemented workflow requires them. Current auth, directory, profile and community capabilities and known omissions are summarized in [status.md](status.md).

## Current route authorization map

| Operation | Current authorization |
| --- | --- |
| Country, academic-level, and subject directory reads | Public |
| Create community | Authenticated account |
| Read active public/restricted community | Public |
| Read private community | Active member only; otherwise not found |
| Join open community / request restricted membership | Authenticated account; private join requires existing active/invited state |
| Read/update own profile and preferences | Authenticated owner |
| Read public profile | Public projection; private profiles return not found and academic affiliation is opt-in |
| Platform administration, content moderation, verification evidence | No registered endpoints |

Community owners/moderators do not have platform-wide privileges; those role operations are not implemented. Student verification status is not an administrative role. Private verification evidence does not exist in the current API.
