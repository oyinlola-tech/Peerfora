# Card Peerfora architecture

## Startup path

`main.rs → index.rs → server.rs → app.rs → loaders/`

`main.rs` starts `index::start_http_server`. The index exposes the server entry point. The server loads typed settings, builds the HTTP listener, and starts serving. `app.rs` delegates setup to the loaders. Loaders register routes and middleware before the server accepts requests.

## Request path

`route → middleware → validation → controller → service → repository → model → database`

Only the controller layer knows HTTP. Only services make product decisions. Only repositories issue database queries. DTOs define API shapes; `validations/` checks untrusted data at runtime. Errors should have stable types and a consistent HTTP response. Interfaces are Rust traits for boundaries that need substitution or dependency inversion.

## Ansofra folders

| Folder | Responsibility | Current state |
| --- | --- | --- |
| `configs/` | Read environment input and convert it into typed settings | Host, port, and log filter |
| `constants/` | Reusable values fixed at runtime | Health route and service labels |
| `controllers/` | Translate HTTP requests and responses | Health controller |
| `database/` | Create, configure, and manage database connections | Reserved; no database is connected |
| `dtos/` | Request and response data contracts | Health response |
| `enums/` | Closed sets of domain states | Reserved for product states |
| `errors/` | Typed application errors and HTTP error responses | Base application error |
| `interfaces/` | Traits for explicit contracts and dependency inversion | Reserved for actual boundaries |
| `jobs/` | Scheduled or queued background work | Reserved; no jobs are running |
| `loaders/` | Prepare application components before listening | Route and middleware registration |
| `middlewares/` | Process or guard requests before controllers | Request logging |
| `models/` | Database-owned data shapes | Reserved until persistence is designed |
| `repositories/` | Database reads and writes | Reserved until persistence is designed |
| `routes/` | Map HTTP methods and paths to controllers | Health route |
| `services/` | Product and application rules | Health response assembly |
| `utils/` | Small generic helpers without product rules | Reserved until a real shared helper exists |
| `validations/` | Runtime validation of incoming data | Reserved until a request accepts input |

The folders are compiled as modules through `src/lib.rs`. An empty layer has no pretend functions; it will be populated when an agreed Card Peerfora behavior needs it.

## Domain boundaries

Follow Uley's service tree: `services/<domain>/commands/` for operations that change state and `services/<domain>/queries/` for reads. Each domain exposes its public operations from its own `mod.rs`; the root `services/mod.rs` is the service entry point. Name files for the action, such as `create_community_command.rs` or `list_communities_query.rs`.

Other layers use domain-named files, such as `routes/community_routes.rs`, `controllers/community_controller.rs`, `repositories/community_repository.rs`, `models/community.rs`, and `validations/community_validation.rs`.

Accounts, communities, discussions, discovery, moderation, and notifications are candidate domains from the PRD. Keep their rules and data ownership separate inside this one application. Cross-domain work goes through an explicit service or interface. Avoid direct cross-domain repository calls and avoid making one table the shared source of truth for multiple domains.

## CQRS

Keep commands and queries in separate service folders, as in Uley. A command applies a state change; a query returns information without changing state. Keep functions small and direct. A service can call a repository, apply product rules, and return an API-ready result. Do not move HTTP handling into the service or database queries into controllers. Use CQRS structure without adding unnecessary abstractions to simple operations.

## Data and scaling

PostgreSQL is the planned source of truth. When a product flow needs persistence, define migrations, connection settings, transaction ownership, indexes, and startup/readiness behavior. A repository owns database queries; models describe stored data. Keep each transaction within its owning domain where possible.

Start with one deployable and explicit module boundaries. Measure before adding Redis, queues, read replicas, or separate services. Introduce durable background work when a real workflow needs it. Extract a domain only when scale or operational evidence supports the added distributed-system cost.

## Naming

Use direct `verb_object` function names, such as `create_community`, `find_account_by_email`, and `approve_report`. Prefer functions that perform one clear action. Avoid abbreviations and vague names. Keep comments out of source unless they explain a non-obvious rule that the code cannot express clearly.
