# Client SDK architecture

No shared Rust client SDK, API contracts crate, or UniFFI bindings are implemented yet. Current apps keep their platform entry points isolated. Add the SDK only after the API contract and cross-platform authentication model are stable enough to share.
