FROM rust:1.90-bookworm AS build

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home peerfora

COPY --from=build /build/target/release/peerfora-api /usr/local/bin/peerfora-api

USER 10001:10001
EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/peerfora-api"]
