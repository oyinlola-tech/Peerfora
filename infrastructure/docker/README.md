# Local and production containers

The root Compose file starts PostgreSQL by default. Optional local profiles
are available for cache (`cache`), search (`search`), S3 API development
(`object-storage`), and captured email (`email`). Start only the services
needed for the current feature.

The S3Mock profile implements only part of the S3 API and is for
development/integration work. It is not a production object store.
Meilisearch, Redis, and Mailpit are not yet wired into the API.

`api.Dockerfile` builds the existing API in a Rust stage and runs it as a
non-root Debian user. No production deployment or cloud infrastructure is
provisioned here.
