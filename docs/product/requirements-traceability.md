# Requirements traceability

| Requirement area | Current evidence | Status |
| --- | --- | --- |
| Rust modular-monolith backend | Root `peerfora-api` crate and Ansofra layer map | Implemented foundation; moving to a Cargo workspace/`services/api` conflicts with current codebase-alignment rules and needs a separate decision |
| Authentication first | Account registration, sessions, password recovery, email confirmation, and social auth code | Implemented slice; providers need configuration |
| 18+ launch eligibility | Country policy plus global minimum-age floor in `src/services/accounts/commands/account_eligibility.rs` | Implemented; tests added but not executable until Cargo can resolve the locked dependency graph |
| Birthday page after social sign-in | Pending social signup and completion endpoint | Backend flow present; frontend completion page not implemented |
| Student web app | `apps/web` Leptos routes and placeholder pages | Scaffolded; build not verified |
| iOS/iPadOS app | `apps/ios/project.yml` and SwiftUI entry point | Scaffolded; requires macOS/Xcode |
| Android app | `apps/android` Compose project | Scaffolded; build not verified |
| Student and admin desktop | Separate Tauri projects and identifiers | Scaffolded; build not verified |
| Widgets | Platform folders and development previews | Partial foundation; Windows provider and all live data flows absent |
| Communities, discussions, search, notifications | PRD domains | Not implemented |
| Payments, moderation admin, AI, realtime | Later roadmap phases | Not implemented |

The supplied PRD attachment is titled `Global_Student_Community_PRD_v1.pdf` and describes Student Commons as a working name; the user's explicit product identity decision is Peerfora. The attachment is not currently stored in the repository. This matrix records repository evidence, not launch readiness. See `docs/progress/repository-audit.md` for the full audit and `docs/progress/next-implementation-phase.md` for continuation tasks.
