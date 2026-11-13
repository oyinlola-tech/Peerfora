# Contributing

Read [AGENTS.md](AGENTS.md) and [the codebase-aligned engineering specification](docs/master-engineering-specification.md) before changing code.

Keep the backend modular monolith and its Ansofra request flow. Implement one cohesive domain slice at a time. Keep API route documentation in `docs/endpoints.md` in sync with every registered route.

Use `cargo fmt --all` and `cargo clippy --all-targets -- -D warnings` for Rust changes. Follow the platform-specific instructions under `docs/platforms/`. Do not describe a scaffold as a completed feature or a successful build unless it has been built on its required toolchain.
