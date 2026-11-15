# Web application

The student Leptos SSR and hydration project lives in `apps/web`. Five separate Leptos SSR and hydration projects also exist under `apps/research-web`, `apps/agents-web`, `apps/developers-web`, `apps/institutions-web`, and `apps/knowledge-web`. They share neutral routing and document-shell code from `apps/web-foundation`; each remains a separate Cargo package and deployable app, without a Cargo workspace. Their only page is a neutral development placeholder. They do not call the API or expose private data. Leptos settings are read through `get_configuration` and can be overridden by environment variables.

The ecosystem projects use distinct local ports in each package's Leptos metadata. Start one with `just ecosystem-web research-web`, replacing `research-web` with `agents-web`, `developers-web`, `institutions-web`, or `knowledge-web`. The recipe builds shared neutral CSS from the existing Tailwind tooling and starts that app with `cargo-leptos watch`; it does not install packages. SSR responses from the five new projects are marked `private, no-store`.

Native SSR `cargo check` passed for all five ecosystem packages and the student web package. WebAssembly hydration was not compiled because only the native Rust target is installed. `cargo-leptos` and `just` are unavailable, so the Leptos watch recipes and full production asset/server build have not been executed.

The student web project contains the route placeholders listed in the master specification plus `/complete-social-signup`, Tailwind CSS tooling, a PWA manifest, robots rules, and neutral tokens imported from `packages/design-tokens`.

Run `just setup-web`, then `just web`. CSS can also be built with `just web-css`. `apps/web/.env.example` names the public API base URL setting; API calls are not wired into the pages yet. SSR responses are marked `private, no-store`. A sitemap is intentionally absent until a canonical public origin and public community visibility rules are defined.
