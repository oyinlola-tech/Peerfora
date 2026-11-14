# Security policy and current controls

Peerfora is pre-launch. Do not submit real student, credential, or eligibility data to local development builds. Report suspected vulnerabilities privately to the repository owner. Do not publish exploit details before an agreed fix is available.

## Current safeguards

- Passwords are stored as Argon2 hashes; sessions and one-time tokens are opaque and stored as hashes.
- Google/Apple integration uses OpenID Connect, state, nonce and PKCE; a new social identity remains pending until birthday and eligibility checks finish. Email equality alone does not auto-link identities.
- The API does not store date of birth and does not return eligibility evidence in public account responses.
- PostgreSQL queries use parameter binding. Account status is checked in account/session/social sign-in lookups; a deferred constraint requires a password hash or a social identity.
- Academic profile data is hidden by default in the public profile projection. Private community reads require active membership.
- Administrator identities have separate passwords and sessions. Passwords use Argon2id; challenge/session tokens are stored as SHA-256 hashes. TOTP is required before an admin session is created. TOTP secrets are encrypted with AES-256-GCM under `PEERFORA_ADMIN_MFA_ENCRYPTION_KEY`.
- Admin sessions use a distinct HttpOnly, SameSite=Strict cookie, eight-hour absolute lifetime, and 20-minute idle timeout. Administrative auth challenges expire after five minutes and are single-use; accepted TOTP steps cannot be replayed.
- Credentialed CORS uses an explicit `PEERFORA_ALLOWED_ORIGINS` allowlist. Unsafe requests carrying a Peerfora session cookie require an allowed `Origin`; bearer-only clients may omit it. Student/admin registration, login, and password-reset requests have PostgreSQL-backed per-email rate windows, with email identifiers stored only as HMACs under the configured rate-limit key.
- Request processing has a configurable positive timeout (`PEERFORA_REQUEST_TIMEOUT_SECONDS`), defaulting to 30 seconds, to bound long-running handlers and database work.
- The one-time bootstrap command assigns the initial super-administrator role and rejects subsequent bootstrap attempts under a PostgreSQL advisory transaction lock.
- Administrative audit rows are append-only through an update/delete trigger under ordinary database access. A database owner or privileged operator can still bypass or alter this control; this is not tamper-proof logging.
- The admin desktop is a separate Tauri shell and does not share student credentials. Its UI remains disconnected.

## Authorization behavior currently implemented

Directory reads are public; community creation/join require an account session; public/restricted active community reads are public; private community reads require active membership; own profile/preferences require the matching account session. Admin identity, mandatory TOTP enrollment/verification, invitations, current-session lookup/logout, own-session listing/revocation, permission-gated overview, account/community administration, and audit-log reads exist. Overview reads require `admin.overview.read`; audit-log reads require `audit_logs.read`; account reads require `accounts.read`; account suspension/reinstatement require `accounts.suspend`; community metadata reads require `communities.read`; community suspension/restoration require `communities.manage`. Suspension changes authoritative lifecycle state and revokes student sessions and reset/verification tokens transactionally. Student sessions never authorize admin routes. Reports, appeals, content moderation, student verification, and evidence routes do not exist. Administrative community reads return metadata only, including for private communities. Community roles do not grant platform-wide access.

## Known security gaps

- Administrator invitations now expire after 48 hours, are single use, store only token hashes, and cannot grant the super-administrator role. Recovery codes, MFA reset, cross-administrator session revocation, and administrator role-management APIs are not implemented. Administrators can list and revoke only their own sessions. Seeded roles and permissions exist, but delegation and role-change operations are not exposed.
- Administrator password failures are locked after five failures for 15 minutes, and five MFA failures across separate challenges lock that administrator's MFA for 15 minutes. Each challenge also permits at most five failed code attempts. Unknown administrator emails use the shared request rate limit and a dummy Argon2id verification.
- Social OAuth stores the nonce and PKCE verifier as plaintext; verifier encryption and key rotation are not implemented.
- Social sign-in, MFA, and network-level abuse signals do not yet have shared request limits. Password-reset request timing can differ because known-address email delivery is synchronous. Production dependency audit is not in place. Readiness queries have a two-second timeout; the 1 MiB body cap and baseline security response headers are applied globally.
- Student session activity metadata is not updated. Granular account restrictions, temporary suspension expiry, deletion workflows, data export, verification evidence access/retention, and terms/privacy version records are not implemented.
- There is no independent security assessment. Passing local SQL smoke tests is not a security certification.

Security fixes must preserve the root Ansofra architecture, use maintained cryptographic libraries, avoid custom cryptographic algorithms, and include regression coverage. Never log passwords, tokens, MFA secrets, recovery codes, provider claims, birthdays, or private evidence. Do not test against production data or unrelated systems.
