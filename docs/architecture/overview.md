# Architecture overview

Peerfora starts as one Rust modular monolith. Its current deployed unit is the Axum API crate at the repository root. The request path and layer responsibilities are documented in [the backend architecture](../architecture.md).

Client applications are separate projects under `apps/`. They call the API through public client configuration; they do not own authorization decisions or database access. Shared static foundations live under `packages/`.

Domain extraction, queues, caches, search, AI, and real-time services remain future decisions. Add them when an implemented workflow requires them and document the boundary before splitting deployment units.
