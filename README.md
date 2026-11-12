# Card Peerfora

Card Peerfora is starting as one Rust modular monolith: one application process with clear ownership boundaries that can scale before any module needs to become a separate service.

## Start the API

```sh
cargo run
```

The server listens at `127.0.0.1:3000` by default. Check it with:

```sh
curl http://127.0.0.1:3000/health
```

Copy `.env.example` to `.env` to change local settings. `PEERFORA_HOST`, `PEERFORA_PORT`, and `PEERFORA_LOG` are read by `configs/settings.rs`. Existing process environment variables take precedence. Set `PEERFORA_HOST=0.0.0.0` when the server must accept connections from outside the local machine.

The health response confirms only that the HTTP process is responding. It does not claim a database or other external service is connected.

## Ansofra layers

The Rust folder names follow the live Uley conventions and the Ansofra architecture reference. The request path is:

`client → server → app → loaders → routes → middlewares → validations → controllers → services → repositories → models → database`

The response returns through the same layers. `dtos` define request and response shapes; `interfaces` define Rust traits and contracts; `enums` define closed sets of domain states; `constants` contain fixed values; `errors` centralize application errors; `utils` hold small generic helpers; and `jobs` hold background work. Each layer has a top-level module under `src/` and is declared by `src/lib.rs`.

Use names that state the action, such as `create_community`, `find_account_by_email`, and `show_health`. Keep HTTP handling in controllers, rules in services, and database queries in repositories. Keep comments out of source unless a non-obvious rule truly needs explanation.

## Modular monolith boundaries

Services are organized by domain, with `commands/` for changes and `queries/` for reads, following Uley's service layout. Routes, controllers, repositories, models, and validations use domain-named files at their layer. A domain must use another domain's public service or interface; it must not reach into another domain's repository or tables.

PostgreSQL is the planned source of truth, but database connection, migrations, and product persistence are not implemented yet. The health endpoint is the only live endpoint. Empty layer modules are reserved for their Ansofra responsibility and will receive real code when a product requirement needs them; they do not contain fake implementations.

See [`docs/architecture.md`](docs/architecture.md) for the folder map and scaling rules.
