set shell := ["bash", "-cu"]

setup:
    cp -n .env.example .env
    docker compose up -d postgres

dev: setup
    cargo run

api:
    cargo run

worker:
    @echo "No standalone worker is implemented yet."

web:
    cd apps/web && pnpm run build:css && cargo leptos watch

web-css:
    cd apps/web && pnpm run build:css

admin:
    cd apps/admin-desktop/src-tauri && cargo tauri dev

desktop:
    cd apps/desktop/src-tauri && cargo tauri dev

migrate:
    sqlx migrate run --source migrations

lint:
    cargo clippy --all-targets -- -D warnings

test:
    cargo test

fmt:
    cargo fmt --all -- --check

check:
    cargo check

openapi:
    @echo "OpenAPI generation is not configured yet. docs/endpoints.md remains authoritative."

generate-bindings:
    @echo "Shared SDK and UniFFI bindings are scheduled for a later implementation phase."

setup-web:
    cargo install cargo-leptos --locked
    rustup target add wasm32-unknown-unknown
    pnpm install --dir apps/web

setup-android:
    @echo "Install Android Studio, Android SDK API 37, Build Tools 36, and JDK 17."

android:
    cd apps/android && ./gradlew :app:assembleDebug

ios-project:
    cd apps/ios && xcodegen generate --spec project.yml

setup-ios:
    @echo "Requires macOS, Xcode, and XcodeGen. See docs/platforms/ios.md."

setup-desktop:
    cargo install tauri-cli --locked
    rustup target add wasm32-unknown-unknown
    cargo install trunk --locked
