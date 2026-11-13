# Web application

The Leptos SSR and hydration project lives in `apps/web`. It contains the route placeholders listed in the master specification plus `/complete-social-signup`, Tailwind CSS tooling, a PWA manifest, robots rules, and neutral tokens imported from `packages/design-tokens`.

Run `just setup-web`, then `just web`. CSS can also be built with `just web-css`. `apps/web/.env.example` names the public API base URL setting; API calls are not wired into the pages yet. SSR responses are marked `private, no-store`. A sitemap is intentionally absent until a canonical public origin and public community visibility rules are defined.
