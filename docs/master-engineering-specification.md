# PEERFORA — MASTER ENGINEERING IMPLEMENTATION SPECIFICATION

## Codebase alignment for this repository

This is the operating layer for applying the full specification below to the current Peerfora checkout. Read this section together with the repository's `AGENTS.md` before making changes.

### Instruction priority

Apply instructions in this order:

1. System and developer instructions and the user's latest direct request.
2. The user's explicit repository rules in `AGENTS.md` and prior direct decisions that still apply.
3. Product requirements that this user-provided specification explicitly says it updates.
4. The current Peerfora PRD for all other confirmed product requirements.
5. The remaining parts of this document as a phased product roadmap and platform target.
6. Existing implementation and documentation as evidence of current behavior, not proof that every roadmap requirement is complete.

If a requirement is missing, ambiguous, or conflicts with a higher-priority instruction, do not guess. Record the unresolved point and implement only the independently clear part of the requested work. The roadmap describes the desired direction; it is not blanket authorization to implement the entire platform in a single change.

### Current repository baseline

- The product name is **Peerfora**. Use Peerfora in product-facing text, documentation, logs, and generated messages.
- This checkout contains a Rust/Axum API crate named `peerfora-api`, PostgreSQL migrations, account/authentication flows, and initial web, iOS, Android, student desktop, administration desktop, and widget foundations. It is not yet the full multi-platform product described by the roadmap; the new clients are uncompiled placeholders.
- The current backend uses the Ansofra startup flow `main.rs → index.rs → server.rs → app.rs → loaders/` and layered request flow `routes → middlewares → validations → controllers → services → repositories → models → database`.
- Account services follow the existing Uley-style `src/services/<domain>/commands/` and `queries/` layout, with descriptively named use cases. Preserve these boundaries when implementing features.
- Do not move the backend into `services/api`, split it into new crates, introduce a Cargo workspace, or replace its layering just to match a proposed directory tree. Make such an architectural change only for a concrete requirement, after inspecting dependencies and current user changes, and document the decision.
- The directory tree, service list, platform applications, widgets, and integrations in the specification are target architecture and roadmap. The user has explicitly authorized initial scaffolds for web, iOS, Android, student desktop, administration desktop, and widgets; keep them minimal until their real features are requested. This does not authorize placeholder backend logic, unrelated crates, production integrations, or final UI design.
- Keep `docs/endpoints.md` as the authoritative catalog of every registered HTTP route in this checkout. For each route document method/path, inputs, auth, cookies, success status and response, handler errors, and framework-level request errors. Update it in the same change as route, DTO, auth, or error behavior. Do not create a competing endpoint catalog.
- Keep `docs/architecture.md` as the current backend architecture map. The specification's proposed nested docs tree is a future organization option; do not duplicate or relocate current documents unless requested or needed for a real documentation change.
- The application's current error envelope is `{"message":"..."}`. This is a documented consistent format for the current API. A switch to RFC 9457 problem details requires a planned compatibility change across handlers, clients, endpoint docs, and API contract.
- `/health` currently checks only that the HTTP server responds. Local optional Compose profiles exist for Redis, Meilisearch, S3Mock, and Mailpit, but the API does not use them. Readiness checks, OpenAPI generation, durable queues, production object storage, live client integrations, and live widget endpoints remain roadmap items.

### Peerfora code-writing rules

- Keep source comments out unless a non-obvious security or correctness rule cannot be expressed in code or a clear name.
- Keep functions and files focused. Use one named service command/query per file; split long or unrelated implementations. Prefer complete, self-explanatory function names and readable line lengths.
- Put stateless reusable helpers such as password and one-time-token handling in `src/utils/`. Keep product policy in domain services, SQL in repositories, and HTTP concerns in controllers/routes.
- Define finite providers and failure kinds in `src/enums/`; map public error messages and HTTP statuses in `src/errors/`. Do not put ad hoc public errors in controllers, services, or repositories.
- Put fixed source-level values in `src/constants/`; deployment-specific values belong in typed config. Define interfaces only at real boundaries and inject the implementation, as with the current `EmailSender`/SMTP adapter.
- Use idiomatic, understandable Rust. Do not add a dependency, abstraction, or layer solely because it appears in the roadmap. Add it when a requested feature has a concrete use and verify compatibility with the repository's Rust version and dependency graph.
- Preserve user changes. Do not commit, push, publish, deploy, provision paid resources, or perform destructive repository operations unless the user explicitly authorizes that action.
- Add meaningful tests for implemented behavior when the task and environment permit. Run only checks that are relevant and authorized for the task, report their actual results, and identify checks blocked by unavailable toolchains or dependencies. Never describe uncompiled or untested code as verified.

### Current documentation ownership

The current repository maintains `README.md`, `AGENTS.md`, `SECURITY.md`, `docs/architecture.md`, `docs/endpoints.md`, `docs/database.md`, and `docs/status.md` as its primary engineering documents. This current-repository rule supersedes the legacy nested documentation tree later in this preserved roadmap. Keep the master specification, ADRs, source product requirements, licenses, and genuine platform build instructions.

### How to apply the roadmap

Before implementation, inspect `git status`, read `AGENTS.md`, the relevant architecture and endpoint documentation, the relevant source files, and the applicable PRD material. Implement only the requested phase or vertical slice. Keep future platform requirements visible as planned work, not as code that appears complete. Update implementation status and limitations whenever the change affects what the repository actually supports.

---

## Full product and platform roadmap

The specification below preserves the supplied master prompt. Apply it through the codebase-alignment rules above; its future platform and infrastructure goals are not all immediate implementation tasks.

---

## 1. Your role

Act as a principal software architect, senior Rust engineer, native mobile engineer, DevOps engineer, database architect, and application security engineer.

You are responsible for establishing and implementing the technical foundation of Peerfora, a global student community and academic discussion platform inspired by Reddit.

Work directly inside the existing Peerfora monorepo.

This is a real software engineering implementation task, not a conceptual architecture exercise. Create the necessary code, configuration, migrations, tests, documentation, infrastructure definitions, and application scaffolding.

Do not merely describe what should be implemented. Implement it.

---

## 2. Product overview

**Product name:** Peerfora

**Product positioning:** A global community platform connecting students across institutions, academic disciplines, educational levels, and countries.

**Core vision:** Different schools. One community.

Peerfora must support:

- Secondary-school students who meet the minimum age requirement.
- Undergraduate students.
- Master's students.
- PhD students.
- Vocational and technical students.
- Professional learners.
- Independent learners.
- Researchers.
- Approved educators and institutional representatives.

Students must not be restricted to interacting with people from their own institutions.

For example, a Computer Science student in Nigeria should be able to participate in the same course community as students in Canada, Germany, India, Japan, or the United States.

Users must also be able to create school-specific communities, general interest communities, and communities for academic disciplines.

Peerfora is a social network with a strong educational identity, not a conventional learning management system.

### Source of truth

Locate and read the existing Peerfora Product Requirements Document, including the document previously titled `Global_Student_Community_PRD_v1`.

Use that document as the primary source for business requirements.

If the PRD is available inside the repository, preserve it and create a requirements traceability matrix linking its requirements to implementation modules, API endpoints, database migrations, and tests.

If the PRD is unavailable, implement the explicitly described requirements in this prompt and document missing or unconfirmed requirements. Do not invent details and present them as approved requirements.

Where this prompt explicitly updates the PRD, follow this prompt.

---

## 3. Critical implementation constraints

### 3.1 Monorepo requirement

All applications, backend services, shared libraries, infrastructure, widgets, tests, and documentation must exist inside one Peerfora monorepo.

Do not create independent Git repositories inside the monorepo.

Each application must nevertheless have its own build configuration, release process, versioning strategy, environment configuration, and platform-specific dependencies.

### 3.2 Frontend design freeze

A product designer is currently being selected.

The final product design, branding guidelines, colors, typography, spacing, user journeys, and interface layouts will be provided later.

Therefore:

- Initialize every frontend application.
- Configure its framework and dependencies.
- Create application entry points.
- Establish the folder architecture.
- Configure routing and navigation foundations.
- Initialize API clients and integration boundaries.
- Configure accessibility foundations.
- Establish localization infrastructure.
- Configure light and dark appearance support.
- Establish design-token interfaces.
- Configure testing and build pipelines.
- Create minimal development-only placeholder screens.
- Create widget extension foundations and minimal development previews.
- Prepare native platform integrations.

Do not:

- Design the final home feed.
- Create a complete Reddit-style interface.
- Invent a final brand identity.
- Choose permanent brand colors.
- Design production-ready onboarding screens.
- Build final authentication interfaces.
- Create final community or discussion screens.
- Design the final administration dashboard.
- Create elaborate animations.
- Generate final logos.
- Introduce arbitrary UI component libraries.
- Replicate Reddit's visual identity.
- Treat placeholder UI as an approved design.

Only implement the minimum presentation layer needed to confirm that each frontend application initializes, compiles, launches, and can eventually communicate with the backend.

Keep placeholder presentation code isolated and replaceable.

Backend implementation must not be blocked by the absence of final product designs.

---

## 4. Initial repository inspection

Before writing code:

1. Inspect the complete repository structure.
2. Read all existing README, AGENTS.md, architecture, and project instruction files.
3. Locate the Peerfora PRD.
4. Inspect Git status.
5. Identify existing applications and dependencies.
6. Identify partially implemented functionality.
7. Detect duplicated modules and conflicting configurations.
8. Review existing database migrations.
9. Review existing environment configuration.
10. Determine which platform toolchains are available.
11. Document the current implementation state.
12. Create an implementation plan and requirements matrix.

Do not overwrite working implementations unnecessarily.

Preserve existing user changes.

Do not use destructive Git commands, delete unrelated files, reset branches, force-push, or discard uncommitted changes.

Do not commit, push, publish, or deploy unless explicitly authorized.

---

## 5. Mandatory technology stack

### 5.1 Core backend

Use:

- Rust, stable toolchain.
- Latest suitable stable Rust edition.
- Axum for HTTP APIs.
- Tokio for asynchronous execution.
- Hyper and Tower through compatible framework integrations.
- tower-http for HTTP middleware.
- Serde for serialization and deserialization.
- SQLx for PostgreSQL access.
- PostgreSQL as the authoritative relational database.
- Redis for caching, rate limiting, and suitable distributed coordination.
- Apalis for background job processing.
- Meilisearch for advanced search.
- pgvector for future semantic search and recommendations.
- Utoipa for OpenAPI documentation.
- Swagger UI for development API exploration.
- Reqwest with Rustls where appropriate for external HTTP integrations.
- Tracing for structured logging.
- OpenTelemetry for metrics and distributed tracing.
- Argon2id for password hashing.
- UUID or an appropriately documented sortable identifier strategy.
- thiserror for typed domain and application errors.
- anyhow where appropriate for executable startup and operational errors.
- AWS SDK for Rust or suitable S3-compatible integrations for object storage.

Select compatible stable dependency versions using current official documentation.

Pin the Rust toolchain, commit the relevant lockfiles, and document compatibility requirements.

Do not blindly install incompatible latest versions.

### 5.2 Web frontend

Use:

- Leptos.
- Rust.
- WebAssembly.
- Server-side rendering.
- Hydration.
- cargo-leptos.
- Leptos Router.
- Tailwind CSS.
- Semantic HTML.
- Accessible ARIA patterns.
- Progressive Web App foundations.
- Service worker infrastructure where appropriate.
- Playwright for future end-to-end testing.

The public website must be capable of rendering public communities and discussions as indexable HTML.

Configure the application as an independently deployable web frontend within the monorepo.

The Leptos server must consume the central Peerfora backend API. It must not become a second independent implementation of Peerfora's business logic.

### 5.3 iOS and iPadOS

Use:

- Swift.
- SwiftUI.
- Rust client SDK.
- Mozilla UniFFI for Rust-to-Swift bindings.
- Swift concurrency.
- NavigationStack.
- TabView.
- SF Symbols.
- Apple's native Liquid Glass design system.
- WidgetKit.
- App Intents.
- UserNotifications.
- APNs.
- BackgroundTasks.
- Keychain.
- URLSession where native networking is appropriate.
- SQLite for suitable local persistence.
- XCTest and XCUITest.

The application must support both iPhone and iPad.

### 5.4 Android

Use:

- Kotlin.
- Jetpack Compose.
- Material 3.
- Material 3 Adaptive.
- Android Gradle Plugin.
- Gradle Kotlin DSL.
- Gradle version catalogs.
- Rust client SDK.
- Mozilla UniFFI for Rust-to-Kotlin bindings.
- Kotlin Coroutines.
- Kotlin Flow.
- Navigation Compose.
- Room and SQLite.
- DataStore.
- Android Keystore-backed credential storage.
- WorkManager.
- Firebase Cloud Messaging.
- Jetpack Glance for widgets.
- JUnit and Compose UI testing.

The application must support Android phones, tablets, and foldables.

### 5.5 Student desktop

Use:

- Tauri 2.
- Rust.
- Leptos in client-side rendering mode.
- WebAssembly.
- Trunk or another officially supported compatible build pipeline.
- Tailwind CSS.
- SQLite.
- Tauri capabilities and permissions.
- OS credential storage or Stronghold where appropriate.
- Tauri notification integrations.
- Signed application updates.

Support Windows, macOS, and Linux.

Do not embed an SSR server as the desktop frontend.

### 5.6 Administration desktop

Use:

- Tauri 2.
- Rust.
- Leptos CSR.
- WebAssembly.
- Tailwind CSS.
- Dedicated administration API client.
- OS-backed secure credential storage.
- Signed application updates.
- Restricted Tauri permissions.

The administration desktop application must be completely separate from the student desktop application at the application, build, identity, and authorization levels.

Sharing the same monorepo does not mean sharing privileged access.

### 5.7 Artificial intelligence and data engineering

Use Rust-first tooling:

- Candle for supported model inference.
- Burn for suitable machine learning training and inference.
- SmartCore where appropriate for classical machine learning.
- Polars for data processing and analytics.
- ndarray for numerical operations when necessary.
- Ort for supported ONNX inference.
- pgvector for vector storage and retrieval.
- FastEmbed where suitable for embeddings.

Do not add every machine learning library to the core API binary.

Isolate optional AI dependencies in dedicated crates or workers behind suitable feature flags.

Do not download large AI models or require GPU infrastructure to run the ordinary development environment.

### 5.8 Infrastructure

Use:

- Docker.
- Docker Compose.
- Terraform.
- GitHub Actions.
- S3-compatible object storage.
- PostgreSQL.
- Redis.
- Meilisearch.
- OpenTelemetry-compatible observability.

Keep infrastructure cloud-ready without unnecessarily coupling development to a particular cloud provider.

Do not provision paid cloud resources without authorization.

---

## 6. Monorepo architecture

Establish a structure similar to the following, adapting it to existing repository conventions where necessary:

```text
peerfora/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── rustfmt.toml
├── deny.toml
├── README.md
├── AGENTS.md
├── SECURITY.md
├── CONTRIBUTING.md
├── .env.example
├── .gitignore
├── justfile
│
├── apps/
│   ├── web/
│   ├── ios/
│   │   ├── Peerfora/
│   │   ├── PeerforaWidgets/
│   │   ├── PeerforaTests/
│   │   └── PeerforaUITests/
│   ├── android/
│   │   ├── app/
│   │   ├── core/
│   │   ├── widgets/
│   │   └── gradle/
│   ├── desktop/
│   │   ├── frontend/
│   │   └── src-tauri/
│   └── admin-desktop/
│       ├── frontend/
│       └── src-tauri/
│
├── services/
│   ├── api/
│   ├── worker/
│   └── ai-worker/
│
├── crates/
│   ├── domain/
│   ├── application/
│   ├── infrastructure/
│   ├── database/
│   ├── configuration/
│   ├── security/
│   ├── observability/
│   ├── api-contracts/
│   ├── client-sdk/
│   ├── mobile-ffi/
│   ├── realtime/
│   ├── search/
│   ├── feed-ranking/
│   ├── moderation/
│   ├── analytics/
│   └── testing/
│
├── widgets/
│   ├── macos/
│   ├── windows/
│   └── linux/
│       ├── gnome/
│       └── kde/
│
├── packages/
│   ├── design-tokens/
│   ├── localization/
│   └── platform-assets/
│
├── contracts/
│   ├── openapi/
│   ├── events/
│   └── schemas/
│
├── migrations/
├── tests/
│   ├── integration/
│   ├── contract/
│   ├── security/
│   └── load/
│
├── infrastructure/
│   ├── docker/
│   ├── terraform/
│   └── observability/
│
├── scripts/
├── tools/
│
├── docs/
│   ├── product/
│   ├── architecture/
│   ├── adr/
│   ├── api/
│   ├── database/
│   ├── security/
│   ├── platforms/
│   ├── deployment/
│   └── progress/
│
└── .github/
    └── workflows/
```

Do not create empty crates merely to match this tree.

Create meaningful modules and consolidate closely related responsibilities when that produces a simpler, maintainable architecture.

The architecture should support a large project without excessive abstraction or unnecessary microservices.

Use a Cargo workspace for compatible Rust applications and libraries.

Keep platform-specific build configurations isolated so ordinary backend builds do not require Xcode, Android Studio, WebAssembly tooling, or desktop system libraries.

---

## 7. Backend architectural requirements

Implement the backend as a modular monolith with independently executable background workers.

Use principles from:

- Domain-driven design.
- Clean Architecture.
- Hexagonal Architecture.
- Explicit dependency boundaries.
- Pragmatic CQRS for operations that benefit from distinct command and query handling.

Do not use CQRS as an excuse to create separate databases or unnecessary distributed services.

Organize domain modules around:

```text
domain/
├── identity/
├── users/
├── institutions/
├── education/
├── communities/
├── memberships/
├── posts/
├── comments/
├── votes/
├── reputation/
├── feeds/
├── search/
├── bookmarks/
├── notifications/
├── messaging/
├── moderation/
├── reports/
├── verification/
├── opportunities/
├── subscriptions/
├── advertising/
├── payments/
├── analytics/
├── administration/
├── widgets/
└── feature_flags/
```

Keep:

- Domain rules independent of Axum.
- HTTP handlers thin.
- Database queries outside HTTP handlers.
- Transactions explicit.
- Infrastructure replaceable through well-defined interfaces.
- Authorization checks close to protected operations.
- External provider integrations behind interfaces.
- Business logic independent of UI implementations.

Use asynchronous interfaces where they provide genuine benefits.

Avoid unnecessary trait abstractions, excessive cloning, deep inheritance-like abstractions, and unbounded asynchronous concurrency.

Use `unsafe` Rust only when unavoidable, isolated, documented, reviewed, and tested.

---

## 8. Identity and age eligibility

Implement account registration, authentication, account recovery, and session management.

The initial product policy is an 18+ launch.

Users aged 16–17 must not be admitted until a separately approved minor-safety release is implemented.

### Registration requirements

Support:

- Username.
- Email address.
- Password.
- Date-of-birth or approved age-assurance flow.
- Country or region.
- Academic level.
- Optional institution.
- Optional course or academic discipline.
- Language preferences.
- Terms acceptance.
- Privacy-policy acceptance.

Do not publicly expose date of birth, email addresses, verification evidence, precise location, or legal identity.

Implement:

- Server-side age eligibility enforcement.
- Configurable jurisdiction-specific eligibility rules.
- Email verification.
- Secure password hashing.
- Password reset.
- Session revocation.
- Session expiration.
- Device and session management.
- Optional user MFA architecture.
- Mandatory administrator MFA.
- Account suspension.
- Account deletion.
- Personal data export.

Age eligibility must not rely solely on frontend validation.

Self-declared age must not be represented as independently verified age.

Prepare integration boundaries for future privacy-preserving age assurance without implementing biometric collection or identity-document retention unnecessarily.

---

## 9. Global academic directory

Implement a normalized educational directory covering:

- Countries.
- Regions.
- Institutions.
- Institution aliases.
- Educational systems.
- Academic levels.
- Academic disciplines.
- Fields of study.
- Global subject topics.
- Institution-specific courses.
- Course codes.
- Course aliases.
- Languages.
- Qualification types.

Academic levels must support:

- Secondary education.
- Vocational and technical education.
- Foundation programs.
- Undergraduate degrees.
- Master's degrees.
- Doctoral degrees.
- Professional qualifications.
- Independent learning.

Do not assume that every country uses the same educational system.

Do not assume that identical course codes refer to identical courses across universities.

Distinguish global subject concepts from institution-specific course offerings.

For example, a global Data Structures community should remain discoverable independently of a university's internal course code.

Support aliases, alternate spellings, multilingual names, duplicate detection, and administrator-approved merge workflows.

Do not falsely present an institution as officially verified merely because a student creates a community using its name.

---

## 10. Community system

Implement one flexible community system.

Community categories should support:

- Global subjects and courses.
- Schools and universities.
- Academic departments.
- Research topics.
- Study groups.
- Educational levels.
- Scholarships.
- Internships.
- Student life.
- Career development.
- General interests.
- Other policy-compliant user-created communities.

A community must be able to represent a subject without requiring an institution.

Support:

- Community name and unique identifier.
- Slug.
- Description.
- Community category.
- Optional institution relationship.
- Optional academic subject relationship.
- Community rules.
- Post flairs.
- Member flairs.
- Moderators.
- Ownership.
- Public, restricted, and private visibility.
- Membership.
- Membership requests where appropriate.
- Member bans.
- Community muting.
- Community reports.
- Community archiving.
- Community transfers.
- Community discovery.
- Related communities.
- Community statistics.
- Moderation configuration.

Use stable identifiers internally rather than depending on mutable display names.

Implement naming collision rules and safeguards against impersonation.

Community owners must not receive platform-wide administration privileges.

Private community information must not leak through search, recommendations, notifications, aggregate counts, media URLs, or widget responses.

---

## 11. Posts and discussions

Implement the core Reddit-inspired discussion model.

Support approved PRD post types, including:

- Text posts.
- Questions.
- Link posts.
- Images.
- Image galleries.
- Polls.
- Academic resources.
- Research discussions.
- Announcements.
- Opportunity posts.

Prepare video and other expensive media features according to the PRD release phases rather than prematurely enabling them.

Implement:

- Post creation.
- Post editing.
- Post deletion.
- Draft persistence where required.
- Post flair.
- Spoiler markers.
- Content warnings.
- Crossposting.
- Pinned posts.
- Locked posts.
- Archived posts.
- Saved posts.
- Reporting.
- Sharing through stable URLs.
- Helpful-answer recognition.
- Post moderation status.
- Post revision metadata.

Implement nested comments with:

- Replies.
- Editing.
- Deletion.
- Voting.
- Mentions.
- Comment sorting.
- Moderator actions.
- Helpful-answer designation.
- Thread pagination.
- Maximum safe depth and lazy loading.
- Abuse prevention.

Avoid recursively loading entire unbounded discussion trees.

Preserve discussion structure when individual comments are deleted, while respecting privacy and legal deletion requirements.

Sanitize user-generated rich text and prevent stored cross-site scripting.

---

## 12. Voting, reputation, feeds, and recommendations

Implement:

- Upvotes.
- Downvotes.
- Removing votes.
- Post scores.
- Comment scores.
- User reputation.
- Community-specific contribution statistics.
- Unique-vote constraints.
- Transaction-safe vote changes.
- Vote manipulation detection foundations.
- Rate limits.
- Anti-brigading safeguards.
- Spam prevention.

Reputation must not be directly purchasable.

Do not allow paid subscriptions or advertising payments to alter organic voting outcomes.

Implement feed types:

- Home.
- Following or joined communities.
- Popular.
- Latest.
- Top.
- Trending.
- Community-specific feeds.
- Academic subject feeds.
- Institution-specific feeds.
- Saved content.

Start with deterministic, explainable Rust ranking algorithms.

Ranking may consider:

- Recency.
- Vote score.
- Discussion quality.
- Engagement.
- Community membership.
- Explicit user interests.
- Academic relevance.
- Moderation status.
- Spam signals.
- Content diversity.

Never use private messages or sensitive verification documents as recommendation inputs.

Use cursor-based pagination.

Keep ranking configurable and versioned.

Implement machine learning recommendations only behind approved feature flags after relevant data and evaluation infrastructure exist.

---

## 13. Search and discovery

Implement PostgreSQL search as the initial reliable search capability.

Prepare Meilisearch integration for advanced search without making core posting and commenting depend on search-engine availability.

Support searching:

- Communities.
- Courses.
- Subjects.
- Schools.
- Universities.
- Posts.
- Comments where permitted.
- Public user profiles.
- Scholarships.
- Internships.
- Academic resources.

Support:

- Search filters.
- Typographical tolerance where available.
- Search aliases.
- Language-aware indexing.
- Relevance ranking.
- Safe result snippets.
- Pagination.
- Duplicate detection.
- Index rebuilding.

Use a durable outbox and background workers for search-index synchronization.

Revalidate authorization against authoritative backend data where necessary.

Private or removed content must be excluded from unauthorized search results.

Prepare semantic search using pgvector, but do not require it for the launch MVP.

---

## 14. Real-time communication and notifications

Implement a Rust-based real-time architecture using Axum WebSockets where appropriate.

Support:

- New comment notifications.
- Reply notifications.
- Mentions.
- Community announcements.
- Moderation notifications.
- Verification updates.
- Subscription notifications.
- Security notifications.
- Opportunity alerts.
- In-app inbox notifications.

Use:

- Authenticated WebSocket connections.
- Appropriate Redis coordination for multiple API instances.
- Connection limits.
- Heartbeats.
- Reconnection.
- Backpressure.
- Authorization checks for subscriptions.
- Durable notification storage.
- Idempotent delivery.
- User notification preferences.

Use APNs for Apple devices and Firebase Cloud Messaging for Android.

Abstract notification providers.

Prepare direct messaging architecture according to the PRD, including message requests, blocking, reporting, privacy settings, and abuse protections.

Keep messaging disabled until its safety requirements and tests are satisfied.

Do not assume end-to-end encryption is implemented unless an actual reviewed implementation exists.

---

## 15. Media storage

Use S3-compatible object storage.

Implement:

- Presigned upload flows.
- File size restrictions.
- Allowed file types.
- Content-type and file-signature validation.
- Upload ownership checks.
- Media metadata.
- Upload status.
- Quarantine.
- Malware-scanning integration boundaries.
- Image thumbnail generation.
- Removal of unnecessary image metadata.
- Appropriate private access controls.
- Signed downloads for restricted content.
- Deletion and orphan cleanup.

Do not serve arbitrary user-uploaded executable files.

Do not make private community media globally public.

Keep media processing out of latency-sensitive HTTP handlers.

---

## 16. Moderation, trust, and safety

Moderation is a core launch requirement.

Implement:

- User reports.
- Post reports.
- Comment reports.
- Community reports.
- Message reports when messaging is enabled.
- Moderation queues.
- Report assignment.
- Report prioritization.
- Evidence handling.
- Moderator decisions.
- Temporary suspensions.
- Permanent bans.
- Community-specific bans.
- Post and comment removals.
- Appeals.
- Administrative escalation.
- Moderation audit history.
- Policy violation categories.
- Spam controls.
- Automated moderation rules.
- Human review workflows.

Support policy enforcement for:

- Harassment.
- Bullying.
- Discrimination.
- Threats.
- Doxxing.
- Sexual exploitation.
- Impersonation.
- Scams.
- Academic cheating services.
- Illegal content.
- Copyright infringement.
- Malicious links.
- Coordinated abuse.

Do not build an unrestricted adult-content community feature.

AI-generated moderation assessments may assist reviewers, but serious enforcement decisions must have appropriate review, escalation, and appeal mechanisms.

Restrict access to sensitive moderation evidence.

Do not include sensitive evidence in ordinary logs.

---

## 17. Administration roles and permissions

Implement scoped RBAC with fine-grained permission checks.

Support roles defined by the PRD, including:

- Regular user.
- Verified student.
- Community moderator.
- Community owner.
- Approved educator.
- Institutional representative.
- Support agent.
- Verification reviewer.
- Trust and safety reviewer.
- Senior moderation administrator.
- Academic directory administrator.
- Finance administrator.
- Advertising administrator.
- Platform administrator.
- Super administrator.

Distinguish identity attributes and verification status from actual administrative privileges.

A verified student is not an administrator.

A community owner cannot suspend users platform-wide.

A finance administrator cannot automatically access private messages or moderation evidence.

Implement:

- Roles.
- Permissions.
- Role assignments.
- Permission scopes.
- Community-level permissions.
- Platform-level permissions.
- Administrative session management.
- Mandatory MFA.
- Step-up authentication.
- Sensitive operation approval.
- Revocation.
- Least-privilege access.
- Administrative audit logging.
- Break-glass access procedures.

Prevent privilege escalation through direct API calls.

Never rely on hidden frontend buttons to enforce permissions.

Super administrator provisioning must use a secure bootstrap command or controlled operational process.

Do not hardcode administrator passwords.

Do not create default production administrator credentials.

---

## 18. Administration backend modules

Implement dedicated administration APIs supporting:

### Overview

- Registered users.
- Active users.
- Growth.
- New communities.
- Posts and comments.
- Report queues.
- Moderation trends.
- Verification queues.
- Subscription metrics.
- System health.

### User administration

- Account lookup.
- Account status.
- Verification state.
- Restricted account actions.
- Suspension and reinstatement.
- Appeals.
- Session revocation.
- Security history.

### Community administration

- Community lookup.
- Community status.
- Ownership.
- Moderator management.
- Duplicate communities.
- Impersonation.
- Reports.
- Policy enforcement.
- Appeals.

### Academic directory

- Institutions.
- Academic disciplines.
- Courses.
- Aliases.
- Duplicate records.
- Verification workflows.
- Merge operations.

### Trust and safety

- Incident queues.
- Report assignment.
- Enforcement.
- Appeals.
- Escalations.
- Restricted evidence.
- Audit history.

### Finance and monetization

- Subscription management.
- Payment events.
- Refund requests.
- Payment reconciliation.
- Revenue reporting.
- Advertising transactions.
- Approved payouts where applicable.

### Platform operations

- Feature flags.
- Configuration.
- Worker status.
- Health checks.
- Operational metrics.
- System events.
- Administrative access logs.

Administration API endpoints must be independently protected and require appropriate permission scopes.

The admin desktop application must consume these APIs and never connect directly to PostgreSQL.

---

## 19. Monetization architecture

Peerfora's core participation must remain free.

Prepare monetization modules for:

### Peerfora Plus

Optional subscriptions may provide approved benefits such as:

- Reduced advertising or an ad-free experience.
- Additional customization.
- Advanced organization tools.
- Additional saved-content organization.
- Approved AI productivity tools.
- Enhanced convenience features.

Do not paywall basic posting, commenting, voting, community membership, ordinary search, or access to essential academic discussions.

### Advertising

Prepare:

- Advertiser accounts.
- Campaigns.
- Creative submissions.
- Review workflows.
- Campaign status.
- Budget management.
- Placement configuration.
- Impression tracking.
- Click tracking.
- Fraud controls.
- Clearly labeled sponsored content.

Do not implement sensitive-data advertising or sell students' educational records.

### Sponsored opportunities

Support reviewed sponsored:

- Scholarships.
- Internships.
- Graduate programs.
- Academic events.
- Educational opportunities.

Do not allow payments to bypass scam prevention or opportunity verification.

### Institutional services

Prepare approved institutional plans for:

- Verified institution profiles.
- Official announcements.
- Institution community management.
- Approved aggregate analytics.
- Opportunity publishing.

Institutional customers must not gain unrestricted access to individual students' private activity.

### Payments

Create a payment-provider abstraction.

Support:

- Products.
- Plans.
- Prices.
- Currencies.
- Regional pricing.
- Subscription entitlements.
- Checkout sessions.
- Payment webhooks.
- Payment verification.
- Refunds.
- Billing history.
- Invoice metadata.
- Reconciliation.
- Idempotency.
- Subscription expiration.

Store monetary amounts using integer minor units with explicit currency handling.

Do not store raw payment-card details.

Do not invent pricing that has not been approved.

Do not select a permanent payment provider without a documented decision.

Keep production charging disabled until authorized.

Prepare native Apple and Google billing integration boundaries for applicable digital products and subscriptions. Follow the relevant storefront and regional policies rather than assuming one universal payment rule.

---

## 20. API architecture

Expose versioned REST APIs.

Use a base path such as:

```text
/api/v1/
```

Define consistent namespaces:

```text
/auth
/users
/sessions
/institutions
/subjects
/courses
/communities
/posts
/comments
/votes
/feeds
/search
/bookmarks
/notifications
/messages
/reports
/verification
/opportunities
/subscriptions
/payments
/widgets
/admin
```

Implement:

- OpenAPI specifications.
- Typed request and response DTOs.
- Consistent error responses.
- Validation errors.
- Request IDs.
- Pagination.
- Authentication documentation.
- Authorization documentation.
- Idempotency keys for suitable write operations.
- Correct HTTP status codes.
- Deprecation policies.
- Backward-compatibility checks.
- WebSocket event documentation.

Use RFC 9457-compatible problem details or another explicitly documented consistent error format.

Generate and validate OpenAPI artifacts in CI.

Do not expose internal database entities directly as public response models.

Never expose password hashes, secrets, internal moderation evidence, or restricted verification records.

---

## 21. Database requirements

Use PostgreSQL with SQLx migrations.

Create normalized schemas supporting the approved product domains.

Include appropriate:

- Primary keys.
- Foreign keys.
- Unique constraints.
- Check constraints.
- Composite indexes.
- Partial indexes.
- Transaction boundaries.
- Concurrency controls.
- Data lifecycle policies.
- Referential integrity.

Important entities include:

```text
users
user_profiles
user_credentials
user_sessions
user_preferences
user_verifications
roles
permissions
role_assignments
institutions
institution_aliases
academic_levels
subjects
subject_aliases
courses
communities
community_members
community_moderators
community_rules
community_bans
post_flairs
posts
post_revisions
comments
post_votes
comment_votes
reputation_events
bookmarks
notifications
notification_preferences
device_tokens
media_assets
reports
moderation_cases
moderation_actions
appeals
admin_audit_logs
opportunities
subscription_plans
subscription_prices
subscriptions
entitlements
payment_events
advertisers
advertising_campaigns
outbox_events
feature_flags
```

Adapt entity names and normalization to the PRD and actual implementation.

Implement constraints preventing duplicate votes, duplicate memberships, invalid relationships, and unauthorized state transitions.

Use separate tables or another referentially sound design for different vote targets.

Do not implement generic polymorphic relationships that silently sacrifice database integrity.

Use database transactions for related writes.

Use migration tooling that can safely initialize a fresh database.

Document rollback and recovery procedures.

Provide development seed data that is clearly fictional.

Never seed production with demo users or fake academic institutions.

---

## 22. Background workers

Use Tokio and Apalis.

Prefer a durable queue backend with documented operational behavior.

Implement or prepare jobs for:

- Email delivery.
- Email verification.
- Password recovery.
- Push notifications.
- Search indexing.
- Media processing.
- Feed materialization where appropriate.
- Trending calculations.
- Reputation reconciliation.
- Moderation assistance.
- Payment-event processing.
- Subscription expiration.
- Data export.
- Account deletion.
- Analytics aggregation.
- Orphan media cleanup.

Implement:

- Retry policies.
- Exponential backoff.
- Job timeouts.
- Idempotency.
- Concurrency limits.
- Failed-job handling.
- Job observability.
- Graceful shutdown.
- Recovery after worker crashes.

Do not assume exactly-once delivery.

Use a transactional outbox for reliable publication of events originating from database transactions.

---

## 23. Shared Rust client SDK

Create a reusable Rust client SDK inside the monorepo.

The SDK should support:

- API request and response models.
- HTTP client configuration.
- Authentication interfaces.
- Error handling.
- Pagination.
- Safe input validation.
- Request cancellation where supported.
- Retry policies for safe operations.
- WebSocket event models.
- Synchronization interfaces.
- Offline operation foundations.
- API compatibility management.

Separate platform-neutral models from native-only networking and storage dependencies.

Ensure WebAssembly-compatible code does not depend on unsupported native libraries.

Use platform-specific adapters when necessary.

The SDK must not:

- Contain production secrets.
- Include administrator credentials.
- Trust client-side authorization.
- Implement authoritative payment decisions.
- Contain backend database access.
- Override server-side business rules.

Expose mobile bindings through a dedicated UniFFI crate.

Generate Swift and Kotlin bindings through reproducible scripts.

Keep generated binding versions aligned with their Rust library.

Use explicit error types and safe FFI boundaries.

Avoid exposing unnecessarily complex Rust internals across FFI.

---

## 24. Web frontend initialization

Initialize `apps/web`.

Configure:

- Leptos SSR and hydration.
- cargo-leptos.
- Leptos Router.
- Tailwind CSS.
- Static assets.
- API environment configuration.
- Server-side and browser-side API access boundaries.
- Request-scoped authentication architecture.
- Error boundaries.
- Localization.
- Accessibility.
- SEO metadata infrastructure.
- Sitemap and robots configuration.
- PWA foundations.
- Testing.

Prepare route definitions for:

```text
/
/login
/register
/communities
/communities/:slug
/posts/:id
/search
/profile/:username
/inbox
/settings
/saved
/opportunities
```

Use minimal placeholders.

Do not implement final screen layouts.

Initialize design tokens without assigning an unapproved final design system.

Do not accidentally cache personalized or authenticated SSR responses as public content.

Do not expose private communities through SEO metadata or sitemaps.

Provide development scripts and a successful SSR/hydration smoke test where supported.

---

## 25. iOS and iPadOS initialization

Initialize `apps/ios`.

Create a native SwiftUI application with proper Xcode project or reproducible project-generation configuration.

Create foundations for:

- Application lifecycle.
- App configuration.
- SwiftUI navigation.
- Deep links.
- Authentication state.
- Secure credential access.
- API client.
- UniFFI integration.
- Local storage.
- Notification permissions.
- Background task registration.
- Localization.
- Accessibility.
- iPhone and iPad adaptive layouts.
- Widget extensions.
- Unit tests.
- UI tests.

### Liquid Glass requirement

Use Apple's documented Liquid Glass behavior on supported operating systems.

Use native SwiftUI components wherever suitable so standard system controls adopt the appropriate platform appearance.

Prepare for:

- Native tab bars.
- Navigation bars.
- Toolbars.
- Search.
- Sheets.
- Menus.
- Floating controls.
- Adaptive iPad navigation.
- System appearance changes.
- Reduced transparency.
- Increased contrast.
- Dynamic Type.
- VoiceOver.

Do not manually reproduce Liquid Glass with arbitrary blur effects.

Use custom glass APIs only where justified by future approved designs.

Gate OS-specific APIs using appropriate availability checks.

Provide suitable fallback behavior for supported older operating systems.

Do not invent the final iOS interface.

Create a minimal native placeholder that confirms successful application initialization.

### Build limitations

If the current environment does not have macOS and Xcode:

- Create the platform source and configuration where feasible.
- Create macOS CI build workflows.
- Document required Xcode and SDK versions.
- Document device and simulator testing requirements.
- Do not claim successful iOS compilation, signing, or runtime testing.
- Clearly distinguish untested scaffolding from verified application functionality.

---

## 26. Android initialization

Initialize `apps/android`.

Create a native Kotlin and Jetpack Compose project.

Configure:

- Gradle wrapper.
- Kotlin DSL.
- Version catalog.
- Android Gradle Plugin.
- Compatible Java toolchain.
- Jetpack Compose.
- Material 3.
- Navigation Compose.
- Material 3 Adaptive.
- Kotlin Coroutines.
- Kotlin Flow.
- Dependency injection where justified.
- API client.
- UniFFI integration.
- Room.
- DataStore.
- Keystore-backed credentials.
- WorkManager.
- FCM integration boundaries.
- Jetpack Glance.
- Unit tests.
- Instrumented tests.

Prepare an adaptive application architecture supporting:

- Phones.
- Tablets.
- Foldables.
- Landscape.
- Resizable windows.
- Dynamic colors.
- Edge-to-edge layouts.
- Predictive back.
- Screen readers.
- Font scaling.

Use native Android conventions rather than replicating iOS.

Create only minimal placeholder screens and navigation foundations.

Do not implement final screen layouts.

Do not require live Firebase credentials for ordinary local builds.

Document emulator, device, Java, NDK, and Rust cross-compilation requirements.

---

## 27. Student desktop initialization

Initialize `apps/desktop` using Tauri 2 and Leptos CSR.

Configure:

- Tauri application manifest.
- Rust application core.
- Leptos CSR.
- WASM asset build.
- Tailwind CSS.
- Development server.
- Production frontend asset directory.
- Application windows.
- Application identity.
- API configuration.
- Secure credential storage abstraction.
- SQLite abstraction.
- Deep-link infrastructure.
- Native notifications.
- Keyboard-shortcut infrastructure.
- System-tray integration foundation.
- Update configuration.
- Platform packaging.
- Unit tests.
- Build workflows.

Create minimal placeholder application windows.

Support independently building for:

- Windows.
- macOS.
- Linux.

Do not assume all platform installers can be generated from one Linux machine.

Use platform-appropriate CI runners.

Apply restrictive Content Security Policy and Tauri capabilities.

Do not expose arbitrary filesystem or shell execution capabilities.

Do not load untrusted remote pages into privileged Tauri windows.

Prepare signed updates without creating or exposing production signing keys.

---

## 28. Administration desktop initialization

Initialize `apps/admin-desktop`.

This must have:

- A unique application identifier.
- Separate build output.
- Separate application configuration.
- Separate administrative authentication.
- Separate token storage namespace.
- Dedicated admin API client.
- Restricted Tauri permissions.
- Separate release configuration.
- Mandatory MFA integration boundaries.
- Administrative session expiration handling.
- Secure update foundations.

Prepare route definitions for:

```text
/overview
/users
/communities
/institutions
/courses
/reports
/moderation
/verification
/appeals
/subscriptions
/finance
/advertising
/opportunities
/analytics
/support
/system
/audit-logs
/configuration
```

Create only minimal route placeholders.

Do not implement the final dashboard, charts, tables, or visual analytics experience until the product designer provides approved designs.

Do not reuse public student authentication tokens as administrator tokens.

Do not embed backend secrets.

Administrative API permissions must remain enforced even if the desktop client is modified.

---

## 29. Device widgets

Widgets are first-class platform integrations.

Initialize widget foundations for supported platforms without building final visual designs.

### Widget categories

Prepare support for:

1. Trending discussions.
2. Followed academic courses.
3. Community activity.
4. New discussion replies.
5. Scholarships.
6. Internships.
7. Saved discussions.
8. Academic opportunities.
9. User reputation.
10. Quick access to community discovery.

Use fictional sample content for development previews.

### Apple widgets

Use:

- SwiftUI.
- WidgetKit.
- App Intents.
- Timeline providers.
- Widget configuration.
- Supported widget families.
- App Groups where needed for app-to-widget data exchange.
- Deep links.

Prepare iPhone and iPad widget extensions.

Support appropriate Home Screen, Lock Screen, Today View, and StandBy contexts where available.

Prepare a native macOS WidgetKit companion or supported widget extension arrangement for desktop and Notification Center integration.

Follow Apple's documented widget appearance modes, including supported Liquid Glass rendering behaviors.

Do not force glass backgrounds that conflict with system widget rendering.

### Android widgets

Use:

- Jetpack Glance.
- AppWidget infrastructure.
- WorkManager where appropriate.
- Native widget update mechanisms.
- Safe deep links.
- Responsive widget sizes.

Prepare widget manifests and example widget receivers.

### Windows widgets

Use:

- Windows App SDK.
- A compatible native widget provider implementation.
- C# where appropriate for supported platform integration.
- Adaptive Cards.
- Required Windows package registration.

Target the Windows Widgets Board where the platform supports third-party providers.

Do not claim arbitrary desktop-widget support.

### Linux widgets

Prepare platform-specific integrations.

Use:

- GNOME-compatible GJS where appropriate.
- KDE Plasma widget technology, including QML where appropriate.

Do not assume Linux has one universal widget framework.

### Widget safety and privacy

Widgets must:

- Respect community membership.
- Respect blocked users.
- Respect moderation status.
- Respect privacy settings.
- Avoid exposing private messages by default.
- Avoid exposing sensitive academic or identity information.
- Use lightweight cached responses.
- Handle offline states.
- Use safe deep links.
- Support OS-controlled refresh scheduling.
- Avoid unrestricted background polling.

Do not assume widgets can update continuously.

### Widget backend endpoints

Implement or prepare:

```text
GET /api/v1/widgets/trending
GET /api/v1/widgets/communities
GET /api/v1/widgets/opportunities
GET /api/v1/widgets/notifications
GET /api/v1/widgets/saved
GET /api/v1/widgets/reputation
```

Use compact response DTOs designed specifically for widget consumption.

Enforce authorization independently of the widget application.

---

## 30. AI and recommendation foundations

Create isolated Rust AI and recommendation modules.

Prepare capabilities for:

- Content classification.
- Spam detection.
- Moderation assistance.
- Duplicate question detection.
- Semantic search.
- Discussion summaries.
- Community recommendations.
- Feed personalization.
- Language detection.
- Translation-provider integration.
- Academic topic classification.
- Opportunity recommendations.

Use Burn, Candle, Ort, SmartCore, Polars, and pgvector only where appropriate.

Do not make all of them mandatory runtime dependencies.

Use feature flags to isolate heavy or experimental functionality.

Start with deterministic recommendation algorithms.

Document:

- Model versions.
- Model licenses.
- Training-data provenance where applicable.
- Evaluation metrics.
- Privacy constraints.
- Inference resource requirements.
- Human review expectations.

Do not silently transmit private student content to external AI providers.

Do not use student data for model training without an approved lawful basis, privacy review, and appropriate user disclosures.

Treat generated summaries and classification results as potentially incorrect.

AI processing must not block ordinary discussion creation unless an explicit safety requirement justifies it.

---

## 31. Internationalization and accessibility

Peerfora is global.

Prepare:

- Unicode support.
- UTF-8 handling.
- Language preferences.
- Localized strings.
- Localized dates.
- Locale-sensitive formatting.
- UTC storage for timestamps.
- User time zones.
- Right-to-left language support.
- Country-aware configuration.
- Currency formatting.
- Regional policies.

Avoid hardcoded English user-facing strings where practical.

Use appropriate localization systems for each application, with shared translation keys or generated resources where practical.

Maintain equivalent meaning across platform-specific translations.

Accessibility foundations must support:

- Keyboard navigation.
- Screen readers.
- Scalable text.
- Contrast requirements.
- Reduced motion.
- Reduced transparency where supported.
- Semantic controls.
- Meaningful labels.
- Focus management.

Do not depend exclusively on color to communicate status.

The final accessibility audit must occur once actual product designs are implemented.

---

## 32. Security requirements

Apply secure-by-default engineering throughout the repository.

Implement or configure:

- TLS-ready production deployment.
- Secure cookies.
- CSRF protection where cookie-based authentication requires it.
- Strict CORS policies.
- Security headers.
- Content Security Policy.
- HSTS where appropriate.
- Request body limits.
- Request timeouts.
- Rate limiting.
- Trusted proxy configuration.
- Input validation.
- Output sanitization.
- Parameterized SQL queries.
- Secret redaction.
- SSRF protections for external URL fetching.
- File upload restrictions.
- Session revocation.
- Token rotation where appropriate.
- Authentication audience separation.
- Administrative MFA.
- Fine-grained authorization.
- Audit logging.
- Dependency vulnerability scanning.
- Credential and secret scanning.

Use secure OS credential storage for native applications.

Do not store long-lived authentication secrets in plaintext application settings or logs.

Use separate development, testing, staging, and production configurations.

Never commit actual `.env` files containing secrets.

Store only placeholders in `.env.example`.

Document all secret requirements.

Threat-model the following:

- Account takeover.
- Privilege escalation.
- Insecure direct object references.
- Private community data leakage.
- Vote manipulation.
- Spam and automated abuse.
- Malicious file uploads.
- Unauthorized administration actions.
- Payment webhook forgery.
- Duplicate payment processing.
- WebSocket subscription abuse.
- Search index privacy leakage.
- Supply-chain compromise.
- Malicious desktop IPC calls.

---

## 33. Privacy and compliance

Implement privacy foundations supporting:

- Data minimization.
- Consent records where required.
- Privacy-policy version tracking.
- Terms version tracking.
- Account deletion.
- Data export.
- Data retention.
- Restricted administrative access.
- Verification data separation.
- Audit logging.
- Appropriate deletion propagation.
- Regional configuration.

Consider relevant frameworks including GDPR, applicable Nigerian data protection requirements, and other jurisdiction-specific obligations.

Do not claim legal compliance merely because technical controls exist.

Document legal and operational decisions requiring qualified review.

Keep any future 16–17 age expansion disabled until a dedicated safety and compliance assessment is approved.

---

## 34. Observability and performance

Use Tracing and OpenTelemetry.

Provide:

- Structured logs.
- Correlation IDs.
- HTTP request metrics.
- Request latency.
- Error rates.
- Database connection-pool metrics.
- Background job metrics.
- Queue latency.
- Search health.
- WebSocket connection metrics.
- Notification delivery metrics.
- Resource usage.
- Readiness and liveness endpoints.

Implement:

```text
GET /health/live
GET /health/ready
```

Expose operational metrics through appropriately restricted infrastructure.

Do not expose sensitive internal metrics publicly.

Use bounded concurrency.

Avoid unbounded queues.

Use connection pooling.

Prevent N+1 queries.

Use efficient pagination.

Review query plans for important feed, search, membership, and moderation operations.

Use caching only where correctness and invalidation behavior are understood.

Document performance assumptions.

Do not claim million-user capacity without workload-specific testing.

---

## 35. Infrastructure and developer experience

Create Docker Compose development services for:

- PostgreSQL.
- Redis.
- Meilisearch.
- S3-compatible local object storage.
- Local email testing.
- Optional observability components.

Use a lightweight default development profile.

Keep expensive components, AI models, and observability stacks optional.

Provide reproducible scripts or a justfile supporting commands such as:

```text
just setup
just dev
just api
just worker
just web
just desktop
just admin
just migrate
just test
just lint
just fmt
just check
just openapi
just generate-bindings
```

Separate platform setup commands:

```text
just setup-web
just setup-android
just setup-ios
just setup-desktop
```

Document operating system prerequisites.

Do not automatically install global packages or modify the developer's operating system without authorization.

Provide instructions for memory-constrained development machines.

Support limiting Cargo build concurrency.

Use Docker multi-stage builds.

Run production containers with non-root users where practical.

Prepare Terraform modules for production infrastructure without provisioning resources.

---

## 36. Testing requirements

Implement meaningful tests.

### Rust backend tests

Include:

- Domain unit tests.
- Service tests.
- Repository integration tests.
- API integration tests.
- Authorization tests.
- Database constraint tests.
- Migration tests.
- Concurrent voting tests.
- Pagination tests.
- Payment webhook verification tests.
- Background job idempotency tests.
- Privacy tests.
- Search authorization tests.
- Widget response authorization tests.

Use real PostgreSQL for database integration tests where relevant.

Do not use SQLite as a misleading replacement for PostgreSQL-specific behavior.

### Security regression tests

Specifically test:

- Unauthorized administrative access.
- Cross-account resource access.
- Private community leakage.
- Duplicate voting.
- Vote races.
- Authentication failures.
- Rate limiting.
- Expired sessions.
- Revoked sessions.
- Invalid payment signatures.
- Duplicate webhooks.
- Underage registration.
- Restricted media access.
- Unauthorized WebSocket subscriptions.
- Unauthorized widget data access.

### Frontend initialization tests

At this stage, test only:

- Application startup.
- Compilation.
- Routing foundations.
- Configuration parsing.
- API-client initialization.
- Placeholder rendering.
- Widget extension initialization.
- Platform capability configuration.

Do not create visual regression baselines for placeholder designs.

### Quality gates

Run applicable checks:

```bash
cargo fmt --all -- --check
cargo clippy
cargo test
cargo check
cargo audit
cargo deny check
```

Adapt workspace package selection and target flags appropriately.

Do not run a broad all-features build if mutually exclusive features or platform-specific dependencies make it invalid.

Record the actual commands executed and their results.

---

## 37. CI/CD

Create GitHub Actions workflows with path-based change detection where suitable.

Provide separate jobs for:

- Backend.
- Shared Rust crates.
- Web.
- Android.
- iOS.
- Student desktop.
- Administration desktop.
- Widget integrations.
- Database migration checks.
- API contract validation.
- Security scanning.

Use:

- Linux runners for compatible Rust and web builds.
- macOS runners for Apple builds and tests.
- Windows runners for Windows desktop and widget builds.
- Android SDK and NDK-compatible runners for Android.

Do not expose secrets to untrusted pull-request workflows.

Pin critical third-party GitHub Actions to reviewed versions or commit SHAs.

Do not automatically deploy production builds.

Prepare release pipelines requiring appropriate approvals and signing credentials.

A workflow must not report success for a platform that it silently skipped.

---

## 38. Required documentation

For the current Peerfora checkout, maintain these primary engineering documents:

```text
README.md
AGENTS.md
SECURITY.md
CONTRIBUTING.md
docs/architecture.md
docs/endpoints.md
docs/database.md
docs/status.md
```

Preserve architecture decision records, the original PRD and master specification, licenses, genuine deployment instructions, and platform-specific build instructions. Consolidate phase audits, feature status, database notes, security notes, and API conventions into the primary documents rather than creating a separate Markdown file for every feature or implementation phase.

Document actual behavior, major decisions, dependency/security assumptions, build/test commands, unresolved requirements, and platform limits. Keep `docs/endpoints.md` authoritative for registered routes. Do not describe planned features as implemented.

## 39. Implementation sequence

Follow this sequence while preserving the existing repository.

### Phase 0: Repository assessment

Inspect the repository, locate the PRD, establish the implementation matrix, and document constraints.

### Phase 1: Engineering foundation

Implement the Cargo workspace, configuration, error handling, logging, infrastructure, database connection, migrations, health endpoints, testing foundations, and local development scripts.

### Phase 2: Core backend

Implement authentication, users, institutions, academic directory, communities, memberships, posts, comments, voting, reputation, bookmarks, and feeds.

### Phase 3: Safety and administration

Implement authorization, administrative identity, mandatory administrative MFA, reports, moderation, appeals, administrative APIs, and audit logs.

### Phase 4: Platform integrations

Implement search, notifications, background workers, media, payment abstractions, monetization foundations, and widget APIs.

Respect PRD release priorities and feature gates.

### Phase 5: Shared SDK

Implement API contracts, Rust SDK, UniFFI bindings, and platform integration interfaces.

### Phase 6: Frontend initialization

Initialize the Leptos web application, SwiftUI iOS/iPadOS application, Kotlin Compose Android application, student Tauri desktop application, and administration Tauri desktop application.

Keep all presentation minimal and temporary.

### Phase 7: Widgets

Initialize Apple, Android, macOS, Windows, and Linux widget foundations.

Implement only functional development previews and required integration infrastructure.

### Phase 8: Quality assurance

Run available builds and tests, audit security boundaries, verify migrations, validate API contracts, and document unsupported platform checks.

### Phase 9: Final implementation report

Produce a detailed factual engineering report.

Do not treat the final report as a substitute for implementing code.

---

## 40. Definition of done

The requested implementation is complete only to the extent that it can be verified.

Required outcomes include:

- One organized Peerfora monorepo.
- A reproducible Rust workspace.
- An independently buildable Axum backend.
- PostgreSQL schema and migrations.
- Core student community APIs.
- Authentication and authorization.
- Moderation and administration foundations.
- Background job infrastructure.
- Search integration.
- OpenAPI documentation.
- A shared Rust SDK.
- UniFFI integration foundations.
- An initialized Leptos website.
- An initialized SwiftUI iOS/iPadOS application.
- An initialized Jetpack Compose Android application.
- An initialized student desktop application.
- An initialized administration desktop application.
- Widget foundations.
- Docker Compose development infrastructure.
- Terraform foundations.
- CI workflows.
- Security documentation.
- Meaningful automated tests.
- Requirements traceability.
- Accurate implementation status.

All frontends must remain ready for future approved product designs without requiring replacement of their underlying architecture.

A placeholder screen is not a completed feature.

An endpoint returning hardcoded data is not a completed backend implementation.

A generated interface without an actual provider integration is not a completed integration.

A source file that has never been compiled is not verified working software.

---

## 41. Final reporting requirements

At the conclusion of the implementation, provide:

1. Executive implementation summary.
2. Final monorepo directory structure.
3. Completed backend modules.
4. Implemented API endpoints.
5. Database schema and migration summary.
6. Frontend initialization status for every platform.
7. Widget initialization status for every platform.
8. SDK and binding status.
9. Security controls implemented.
10. Tests executed and actual results.
11. Build commands and actual results.
12. Remaining errors.
13. Missing dependencies.
14. Platform-specific verification limitations.
15. Features deferred by the PRD.
16. Requirements awaiting the product designer.
17. Production readiness assessment.
18. Exact next development priorities.

Classify every major requirement as one of:

- Implemented and verified.
- Implemented but not verified.
- Partially implemented.
- Scaffolded.
- Deferred by product requirements.
- Blocked by environment or external dependencies.
- Not implemented.

Never falsely claim completion.

If the implementation is too large to complete within the current execution budget, prioritize foundational and core backend functionality, preserve a working repository, and maintain a detailed continuation document.

Continue from the actual implementation state rather than restarting the project.

Do not silently abandon unfinished requirements.

---

## 42. Engineering conduct

Throughout implementation:

- Prefer maintainable, readable code.
- Use official documentation.
- Select compatible dependencies.
- Avoid unnecessary framework duplication.
- Avoid premature microservices.
- Avoid unnecessary Python, Node.js, or other backend runtimes.
- Allow native languages where platform requirements justify them.
- Avoid hardcoded secrets.
- Avoid insecure development shortcuts that can reach production.
- Avoid placeholder backend implementations presented as production-ready.
- Avoid inventing business requirements.
- Avoid unapproved frontend designs.
- Preserve user changes.
- Maintain a working repository.

The objective is to establish Peerfora as a secure, scalable, internationally accessible, Rust-first student community platform with genuinely native mobile experiences, dedicated desktop applications, a separate administration application, and operating-system widgets.

Begin by inspecting the existing monorepo and the PRD. Then execute the implementation phases in order, validating each completed component before proceeding.
