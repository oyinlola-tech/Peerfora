# Backend architecture

The implemented backend uses the existing startup path `main.rs → index.rs → server.rs → app.rs → loaders/` and layered request flow `routes → middleware → validation → controllers → services → repositories → models → database`.

The current first domain is account authentication. See [the live backend architecture map](../architecture.md), [endpoint catalog](../endpoints.md), and [database migration notes](../database/migrations.md). Frontend scaffolding does not change API behavior.
