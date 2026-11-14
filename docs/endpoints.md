# Peerfora HTTP API reference

This document describes every route currently registered by the API. It is based on the route modules, request/response DTOs, middleware, and controller/error mappings. All API paths are relative to the API origin, for example `http://localhost:3000` locally.

## Shared request and response behavior

- Send JSON request bodies with `Content-Type: application/json`.
- Successful JSON responses use `application/json`.
- Errors created by the application use `{"message":"..."}`. Framework extractor rejections may use Axum's plain text response instead.
- A malformed JSON body returns `400`; JSON that cannot deserialize into the expected fields returns `422`; a missing/non-JSON content type returns `415`; an oversized body can return `413`.
- Missing or malformed query parameters return `400`. An unmatched path returns `404`; a known path with an unsupported method returns `405`.
- Unexpected request-time failures return `500` with `{"message":"An internal error occurred"}`. PostgreSQL connection or migration failures happen during startup; the API will not start serving routes in that case.
- Each response includes a server-generated UUID in `X-Request-Id`; caller-supplied values are replaced. Administrative audit events use this request identifier as their correlation identifier.
- Requests that exceed `PEERFORA_REQUEST_TIMEOUT_SECONDS` return `504 Gateway Timeout` with `{"message":"Request processing timed out"}`. The default is 30 seconds; configure a positive number of seconds.
- Cross-origin access is limited to the exact HTTP(S) origins in `PEERFORA_ALLOWED_ORIGINS`, a comma-separated setting. It defaults to the configured `PEERFORA_PUBLIC_URL` origin. Credentialed CORS permits only the registered methods, `Accept`, `Authorization`, and `Content-Type`; `X-Request-Id` is exposed to clients. Unsafe requests carrying a Peerfora session cookie must include an allowed `Origin`; any supplied disallowed or duplicate `Origin` is rejected with `403` and `{"message":"Request origin is not allowed"}`. Bearer-only non-browser clients may omit `Origin`.
- Authentication request limits use shared PostgreSQL fixed windows keyed by operation and an HMAC of the normalized email address, with the HMAC key supplied through `PEERFORA_AUTH_RATE_LIMIT_HASH_KEY` (at least 32 bytes). Limits are five registrations per hour, ten student logins per 15 minutes, three student password-reset requests per hour, twenty administrator logins per 15 minutes, and three administrator password-reset requests per hour. Exceeding a limit returns `429` with `{"message":"Too many requests. Wait before trying again."}`. This identifier-based control applies across API instances and does not depend on campus-wide IP addresses. Other routes do not yet have rate limits. Directory/community collection endpoints use a bounded `limit` (1–50) without cursor tokens; do not use this pagination shape for future high-volume feeds, posts, or audit logs. No generic webhook routes are registered.

## Authentication and cookies

- Session-protected reads accept either `Authorization: Bearer <session-token>` or the `peerfora_session` cookie. If both are present, the bearer token is used.
- Administration uses only the separate `peerfora_admin_session` cookie. Student bearer tokens and `peerfora_session` cookies are never considered for administration.
- Administrator session cookies are `HttpOnly; SameSite=Strict`, scoped to `/api/v1/admin`, with an eight-hour absolute lifetime and a 20-minute idle timeout. `Secure` follows the same setting as the student cookie. MFA challenges last five minutes and are single use. Five failed MFA attempts across challenges lock the administrator's MFA for 15 minutes; a successful TOTP resets that counter.
- Login and successful social sign-in set `peerfora_session` as `HttpOnly; SameSite=Lax`, path `/`, with a 30-day lifetime. `Secure` is added when `PEERFORA_COOKIE_SECURE=true` or `PEERFORA_ENV=production`.
- Social OAuth start uses `peerfora_oauth_state`, an HttpOnly, SameSite=Lax cookie scoped to `/api/v1/auth/social`, with a 10-minute lifetime.
- A new social identity gets `peerfora_social_signup`, an HttpOnly, SameSite=Lax cookie scoped to `/api/v1/auth/social/complete`, with a 30-minute lifetime. It is not an authenticated session. The account and session are created only after the birthday/eligibility submission succeeds.
- `POST /api/v1/auth/logout` does not require a token. When a session token is supplied, it revokes that session; it always clears the session cookie.

## Endpoint index

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/health` | Report that the HTTP service is responding |
| `GET` | `/health/ready` | Confirm PostgreSQL can execute a readiness query |
| `POST` | `/api/v1/auth/register` | Create an email/password account and send confirmation email |
| `POST` | `/api/v1/auth/login` | Verify credentials and create a session |
| `GET` | `/api/v1/auth/verify-email?token=...` | Confirm an email address |
| `POST` | `/api/v1/auth/password-reset/request` | Request a password reset email |
| `POST` | `/api/v1/auth/password-reset/confirm` | Set a new password using a reset token |
| `GET` | `/api/v1/auth/social/{provider}/start` | Start Google or Apple sign-in |
| `GET` | `/api/v1/auth/social/{provider}/callback` | Complete provider sign-in or create pending signup |
| `POST` | `/api/v1/auth/social/complete` | Complete social signup after birthday and eligibility checks |
| `GET` | `/api/v1/auth/me` | Return the current signed-in account |
| `POST` | `/api/v1/auth/logout` | Revoke the current session and clear its cookie |
| `GET` | `/api/v1/countries` | Search active ISO country and territory reference data |
| `GET` | `/api/v1/academic-levels` | List canonical academic levels and aliases |
| `GET` | `/api/v1/subjects` | Search the seeded global academic subject directory |
| `GET` | `/api/v1/communities` | List public and restricted active communities |
| `POST` | `/api/v1/communities` | Create a community and become its owner |
| `GET` | `/api/v1/communities/{slug}` | Retrieve a discoverable community if visibility permits |
| `PUT` | `/api/v1/communities/{slug}/membership` | Join a community or request membership |
| `DELETE` | `/api/v1/communities/{slug}/membership` | Leave a community |
| `GET` | `/api/v1/profiles/me` | Retrieve the authenticated user's private profile view |
| `PATCH` | `/api/v1/profiles/me` | Update the authenticated user's profile |
| `GET` | `/api/v1/profiles/{username}` | Retrieve a public profile |
| `GET` | `/api/v1/me/preferences` | Retrieve private account preferences |
| `PATCH` | `/api/v1/me/preferences` | Update private account preferences |
| `POST` | `/api/v1/admin/auth/login` | Verify administrator password and begin mandatory MFA |
| `POST` | `/api/v1/admin/auth/password/change` | Change the authenticated administrator password and revoke sessions |
| `POST` | `/api/v1/admin/auth/password-reset/request` | Send administrator password reset instructions when the email is configured |
| `POST` | `/api/v1/admin/auth/password-reset/confirm` | Reset an administrator password using a one-use token |
| `POST` | `/api/v1/admin/invitations` | Invite an administrator with `admin.accounts.manage` permission |
| `POST` | `/api/v1/admin/invitations/accept` | Activate an administrator identity using a one-use invitation token |
| `GET` | `/api/v1/admin/roles` | List active system and custom roles with `admin.roles.read` |
| `POST` | `/api/v1/admin/roles` | Create a custom role with `admin.roles.create` |
| `GET` | `/api/v1/admin/roles/{id}` | Read a role and its permissions with `admin.roles.read` |
| `PATCH` | `/api/v1/admin/roles/{id}` | Update a custom role with `admin.roles.update` |
| `DELETE` | `/api/v1/admin/roles/{id}` | Deactivate and logically delete an unused custom role with `admin.roles.delete` |
| `GET` | `/api/v1/admin/permissions` | Read permission catalog and assignment metadata with `admin.permissions.read` |
| `GET` | `/api/v1/admin/roles/{id}/permissions` | Read a role's permission set with `admin.permissions.read` |
| `PUT` | `/api/v1/admin/roles/{id}/permissions` | Replace a custom role's permissions with `admin.permissions.assign` |
| `GET` | `/api/v1/admin/administrators/{id}/roles` | List an administrator's roles with `admin.roles.read` |
| `PUT` | `/api/v1/admin/administrators/{id}/roles` | Replace another administrator's roles with `admin.roles.assign` |
| `GET` | `/api/v1/admin/administrators/{id}/permissions` | Read effective permissions with `admin.roles.read` |
| `DELETE` | `/api/v1/admin/administrators/{id}/sessions` | Revoke another administrator's sessions with `admin.accounts.manage` |
| `POST` | `/api/v1/admin/administrators/{id}/mfa/reset` | Reset another administrator's MFA after recent reauthentication with `admin.accounts.manage` |
| `POST` | `/api/v1/admin/auth/mfa/enroll` | Begin initial TOTP enrollment |
| `POST` | `/api/v1/admin/auth/mfa/confirm` | Confirm TOTP enrollment and create an admin session |
| `POST` | `/api/v1/admin/auth/mfa/verify` | Complete administrator login MFA |
| `POST` | `/api/v1/admin/auth/mfa/recovery-codes/regenerate` | Replace unused MFA recovery codes after recent TOTP verification |
| `GET` | `/api/v1/admin/auth/me` | Return the authenticated administrator and effective roles/permissions |
| `POST` | `/api/v1/admin/auth/logout` | Revoke the current administrator session |
| `GET` | `/api/v1/admin/sessions` | List the current administrator's recent sessions |
| `DELETE` | `/api/v1/admin/sessions/{id}` | Revoke one of the current administrator's sessions |
| `GET` | `/api/v1/admin/overview` | Read stored account, institution, and community counts with `admin.overview.read` permission |
| `GET` | `/api/v1/admin/audit-logs` | Read administrative audit events with `audit_logs.read` permission |
| `GET` | `/api/v1/admin/accounts` | Search accounts with `accounts.read` permission |
| `GET` | `/api/v1/admin/accounts/{id}` | Read an account administration summary with `accounts.read` permission |
| `POST` | `/api/v1/admin/accounts/{id}/suspend` | Suspend an account and revoke its sessions with `accounts.suspend` permission |
| `POST` | `/api/v1/admin/accounts/{id}/reinstate` | Reinstate a suspended account with `accounts.suspend` permission |
| `POST` | `/api/v1/admin/accounts/{id}/sessions/revoke` | Revoke all student sessions with `accounts.sessions.revoke` permission |
| `GET` | `/api/v1/admin/communities` | Search administrative community metadata with `communities.read` permission |
| `GET` | `/api/v1/admin/communities/{id}` | Retrieve community metadata with `communities.read` permission |
| `POST` | `/api/v1/admin/communities/{id}/suspend` | Suspend a community with `communities.manage` permission |
| `POST` | `/api/v1/admin/communities/{id}/restore` | Restore a suspended community with `communities.manage` permission |

## `GET /health`

No authentication, query parameters, or request body are required.

**Success:** `200 OK`

```json
{
  "status": "ok",
  "service": "peerfora-api"
}
```

This checks only that the HTTP process responds. It does not check PostgreSQL, email, or OAuth connectivity.

**Errors:** No route-specific application errors. A non-matching method/path is handled by Axum.

## `GET /health/ready`

No authentication, query parameters, or request body are required. This endpoint runs `SELECT 1` against the configured PostgreSQL pool. It does not claim that SMTP, OAuth providers, or background jobs are ready.

**Success:** `200 OK`

```json
{"status":"ready","service":"peerfora-api"}
```

**Errors:** `503 Service Unavailable` with `{"message":"Database is unavailable"}` when PostgreSQL cannot answer the query. The response does not expose connection details.

## `POST /api/v1/auth/register`

Creates an account, stores the country policy version, creates a 24-hour email confirmation token, and sends the confirmation email. The date of birth is used for the eligibility decision and is not stored. Email confirmation requires SMTP configuration.

**Authentication:** None.

**Request JSON:**

| Field | Type | Required | Rules |
| --- | --- | --- | --- |
| `email` | string | Yes | Valid email address, at most 254 bytes |
| `password` | string | Yes | 12–128 bytes |
| `username` | string | Yes | 3–24 ASCII letters, digits, or underscores; stored lowercase |
| `displayName` | string or `null` | No | Stored as provided; no additional rule is currently enforced |
| `dateOfBirth` | string | Yes | Calendar date formatted `YYYY-MM-DD`; must satisfy the global 18+ minimum and any stricter configured country policy |
| `countryCode` | string | Yes | ISO 3166-1 alpha-2 country code from the active country directory; uppercased before lookup |
| `acceptedTerms` | boolean | Yes | Must be `true` |

Example:

```json
{
  "email": "student@example.edu",
  "password": "a-long-password-value",
  "username": "peer_learner",
  "displayName": "Learner",
  "dateOfBirth": "2002-05-12",
  "countryCode": "NG",
  "acceptedTerms": true
}
```

**Success:** `202 Accepted`

```json
{"message":"Check your email to confirm your account"}
```

**Errors:**

| Status | Error message | Cause |
| --- | --- | --- |
| `400` | `Enter a valid email address` | Email is malformed or exceeds the registration limit |
| `400` | `Password must be between 12 and 128 bytes` | Password is outside the allowed byte length |
| `400` | `Username must be 3 to 24 letters, numbers, or underscores` | Username violates its length or character rules |
| `400` | `Accept the terms to create an account` | `acceptedTerms` is false |
| `400` | `Date of birth must use YYYY-MM-DD` | Date is malformed or not a calendar date |
| `400` | `Country code must be a two-letter country code` | Country code is not two ASCII letters |
| `400` | `Country code is not in the country directory` | Country code is syntactically valid but absent or inactive in the reference directory |
| `429` | `Too many requests. Wait before trying again.` | More than five registration requests for this email in one hour |
| `403` | `You do not meet the age requirement` | User is younger than the configured minimum age |
| `409` | `Email or username is already in use` | Email or username conflicts with an existing account |
| `503` | `This country requires an age-check method that is not configured` | Country policy requires age assurance, but no method is configured |
| `503` | `Email delivery is not configured` | SMTP is not configured |
| `503` | `Could not send the account email` | SMTP could not send the confirmation message |
| `500` | `An internal error occurred` | Unexpected persistence or processing failure |

If email delivery fails after account and verification-token creation, this implementation returns the email error while leaving the unverified account and token in the database. There is currently no resend-confirmation endpoint.

## `POST /api/v1/auth/login`

Checks the password, requires email confirmation, and creates a 30-day session.

**Authentication:** None.

**Request JSON:**

| Field | Type | Required | Rules |
| --- | --- | --- | --- |
| `email` | string | Yes | Valid email address |
| `password` | string | Yes | Compared with the stored password hash |

**Success:** `200 OK`, sets the session cookie described above.

```json
{
  "account": {
    "id": "<uuid>",
    "email": "student@example.edu",
    "username": "peer_learner",
    "display_name": "Learner",
    "email_verified": true
  }
}
```

`display_name` is `null` when not supplied. The response does not include password hashes, birth date, or eligibility evidence.

**Errors:** `400 Enter a valid email address`; `401 Invalid email or password` for an unknown account, social-only account, or wrong password; `403 Confirm your email before signing in` for an unconfirmed account; `429 Too many requests. Wait before trying again.` after ten attempts for this email within 15 minutes; `500 An internal error occurred` for unexpected failures. Extractor errors are described above.

## `GET /api/v1/auth/verify-email`

Consumes a single-use email confirmation token. The confirmation link is sent by email and includes the token as a query parameter.

**Authentication:** None.

**Query parameters:** `token` (string, required).

**Success:** `200 OK`, with `Cache-Control: no-store` and `Referrer-Policy: no-referrer` headers.

```json
{"message":"Email confirmed. You can now sign in."}
```

**Errors:** `400 Verification link is invalid or expired`; `500 An internal error occurred` for unexpected failures; `400` for missing/malformed query data.

## `POST /api/v1/auth/password-reset/request`

Creates a single-use reset token valid for one hour and emails it when the account exists. The response does not reveal whether the email is registered.

**Authentication:** None.

**Request JSON:** `{"email":"student@example.edu"}`. Email must be valid and no longer than 254 bytes.

**Success:** `202 Accepted`

```json
{"message":"If the account exists, password reset instructions have been sent"}
```

**Errors:** `400 Enter a valid email address`; `429 Too many requests. Wait before trying again.` after three requests for this email within one hour; `503 Email delivery is not configured`; `503 Could not send the account email`; `500 An internal error occurred`. Unknown addresses still receive the same `202` response and do not trigger email delivery.

## `POST /api/v1/auth/password-reset/confirm`

Consumes a reset token, stores a new password hash, and revokes all existing sessions for that account.

**Authentication:** None.

**Request JSON:**

| Field | Type | Required | Rules |
| --- | --- | --- | --- |
| `token` | string | Yes | Single-use token from the reset email |
| `newPassword` | string | Yes | 12–128 bytes |

**Success:** `200 OK`

```json
{"message":"Password updated. Sign in again with your new password."}
```

**Errors:** `400 Password must be between 12 and 128 bytes`; `400 Password reset link is invalid or expired`; `500 An internal error occurred`.

## `GET /api/v1/auth/social/{provider}/start`

Starts OpenID Connect authorization-code sign-in for a configured provider.

**Authentication:** None.

**Path parameter:** `provider` must be `google` or `apple`.

**Success:** `303 See Other` redirect to the provider's authorization page. Sets the short-lived OAuth state cookie. The callback URL registered with that provider must exactly match its configured `PEERFORA_<PROVIDER>_CALLBACK_URL`.

**Errors:** `400 Social sign-in provider is not supported`; `503 Social sign-in provider is not configured`; `503 Could not connect to the identity provider`; `500 An internal error occurred`.

## `GET /api/v1/auth/social/{provider}/callback`

Handles the provider redirect, validates the OAuth attempt and identity token, and either signs in an existing linked identity or starts pending signup.

**Authentication:** No existing account session is required. The OAuth state cookie set by the start endpoint is required.

**Path parameter:** `provider` must be `google` or `apple` and must match the provider used to start the flow.

**Query parameters:**

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `code` | string | Yes | Authorization code from the provider |
| `state` | string | Yes | State value paired with the OAuth state cookie |

**Success:**

- Existing linked identity: `303 See Other` to the Peerfora public URL root and sets `peerfora_session`.
- New identity: `303 See Other` to `${PEERFORA_PUBLIC_URL}/complete-social-signup`; sets the pending signup cookie. The account and authenticated session do not yet exist.
- On a successful callback, the OAuth state cookie is cleared.
- If the verified provider email belongs to an existing Peerfora account that is not linked to this identity, the endpoint returns an error and does not link automatically.

**Errors:** `400 Social sign-in provider is not supported`; `401 Social sign-in request is invalid or expired`; `403 The identity provider did not confirm your email`; `409 Sign in with your existing method before linking another provider`; `503 Social sign-in provider is not configured`; `503 Could not connect to the identity provider`; `500 An internal error occurred`.

## `POST /api/v1/auth/social/complete`

Completes a first-time social signup after the user returns from the Peerfora birthday/eligibility page. It creates the account, links the provider identity, and creates a session only after validation and eligibility succeed. The date of birth is not stored.

**Authentication:** Must include the `peerfora_social_signup` cookie from the callback. This cookie is scoped to this endpoint and expires after 30 minutes.

**Request JSON:**

| Field | Type | Required | Rules |
| --- | --- | --- | --- |
| `username` | string | Yes | 3–24 ASCII letters, digits, or underscores |
| `dateOfBirth` | string | Yes | Calendar date in `YYYY-MM-DD` format |
| `countryCode` | string | Yes | Two ASCII letters; must exist in the active country directory |
| `acceptedTerms` | boolean | Yes | Must be `true` |

**Success:** `200 OK`; clears the pending signup cookie, sets `peerfora_session`, and returns the same account shape as login.

**Errors:** `400 Social signup has expired. Start again.`; `400 Date of birth must use YYYY-MM-DD`; `400 Country code must be a two-letter country code`; `400 Username must be 3 to 24 letters, numbers, or underscores`; `400 Accept the terms to create an account`; `400 Country code is not in the country directory`; `403 You do not meet the age requirement`; `409 Email or username is already in use`; `503 This country requires an age-check method that is not configured`; `500 An internal error occurred`.

## `GET /api/v1/auth/me`

Returns the signed-in account.

**Authentication:** `Authorization: Bearer <session-token>` or `peerfora_session` cookie.

**Success:** `200 OK`, with the account JSON shape documented under login.

**Errors:** `401 Sign in to continue` when the token is missing, invalid, or expired; `500 An internal error occurred` for unexpected failures.

## `POST /api/v1/auth/logout`

Revokes the provided session when one is present and clears the `peerfora_session` cookie.

**Authentication:** Optional. Accepts bearer token or session cookie.

**Success:** `204 No Content`, with a `Set-Cookie` header expiring `peerfora_session` immediately. Calling logout without a token is still successful.

**Errors:** `500 An internal error occurred` if session revocation fails.

## Country policy configuration

All countries and territories in the active ISO 3166-1 directory use the global minimum age of 18 by default. Configure only stricter jurisdiction overrides through `PEERFORA_COUNTRY_POLICIES`; the map key is the two-letter country code and values use this shape:

```json
{
  "NG": {
    "minimum_age": 18,
    "policy_version": "policy-version-from-product",
    "age_assurance_required": false
  }
}
```

The example is a schema illustration, not an enabled policy recommendation. A country policy may raise the global minimum but cannot lower it. When `age_assurance_required` is `true`, signup currently fails with `503` because no age-assurance integration is implemented. A directory entry does not establish legal availability or service support in that territory.

## `GET /api/v1/countries`

Lists or searches active ISO 3166-1 countries and territories, including alpha-2 and alpha-3 codes, display names, and official names where available. This is reference data and does not declare product availability. No authentication is required.

**Query parameters:** `q` is optional and searches codes, names, official names, and configured aliases; `limit` is optional, defaults to 50, and is clamped to 1–250.

**Success:** `200 OK` with a JSON array of objects containing `countryCode`, `alpha3` (ISO alpha-3), `displayName`, and nullable `officialName`.

**Errors:** `400 Search term cannot exceed 100 characters` when `q` is longer than 100 characters; `500 An internal error occurred`.

## `GET /api/v1/academic-levels`

Lists all active canonical academic levels and their accepted aliases. Age eligibility is independent from this taxonomy. No authentication, query parameters, or request body are required.

**Success:** `200 OK` with a JSON array of objects containing `id`, `displayName`, `displayOrder`, and `aliases`. The `high_school` alias resolves to canonical `secondary_school`.

**Errors:** `500 An internal error occurred`.

## `GET /api/v1/subjects`

Searches active global academic subjects and their aliases. No authentication is required.

**Query parameters:** `q` is optional and may contain at most 100 characters. `limit` is optional and is clamped to 1–50; the default is 20. Omitting `q` lists the seeded subjects up to the limit. Results are ordered by display name and ID. This initial route does not yet provide cursor pagination.

**Success:** `200 OK`, JSON array:

```json
[
  {
    "id": "<uuid>",
    "slug": "data-structures",
    "display_name": "Data Structures",
    "description": null,
    "discipline_name": "Computer Science"
  }
]
```

**Errors:** `400 Search term cannot exceed 100 characters`; `400` for a malformed `limit`; `500 An internal error occurred` for a database query failure.

## `GET /api/v1/communities`

Lists active public and restricted communities. Private communities are excluded. No authentication is required.

**Query parameters:** `limit` is optional, clamped to 1–50, and defaults to 20. Results are ordered newest first with ID as a stable tie-breaker. Cursor pagination and community filtering are not implemented yet.

**Success:** `200 OK`, JSON array of community response objects. Fields are `id`, `slug`, `displayName`, `description`, `category`, `institutionId`, `courseId`, `disciplineId`, `visibility`, `membershipPolicy`, `lifecycleStatus`, and `createdAt`.

**Errors:** `400` for a malformed `limit`; `500 An internal error occurred` for a database query failure.

## `POST /api/v1/communities`

Creates a community and atomically records its creator as the owner and an active member. The creator must have a valid authenticated session. Community names support Unicode. A URL-safe slug is generated from the name when `slug` is omitted; a slug can be supplied for names without ASCII characters. Private communities always use approval-required membership.

**Request JSON:**

| Field | Type | Required | Rules |
| --- | --- | --- | --- |
| `displayName` | string | Yes | 3–80 characters after trimming |
| `slug` | string or `null` | No | Normalized to lowercase ASCII letters/digits separated by hyphens; 3–50 characters; reserved paths are rejected |
| `description` | string | No | Defaults to empty; at most 2000 characters |
| `category` | string | No | Defaults to `academic_subject`; values: `academic_subject`, `institution`, `institution_course`, `academic_discipline`, `study_group`, `scholarship`, `internship`, `career`, `student_life`, `general`, `other` |
| `visibility` | string | No | Defaults to `public`; `public`, `restricted`, or `private` |
| `membershipPolicy` | string | No | Defaults to `open`; `open` or `approval_required`; private always requires approval |
| `primaryLanguageTag` | string | No | Defaults to `en`; basic BCP-47-shaped ASCII tag, up to 35 characters |
| `institutionId` | UUID or `null` | No | Must reference an active directory institution when supplied |
| `courseId` | UUID or `null` | No | Must reference an active course; its institution must match `institutionId` |
| `disciplineId` | UUID or `null` | No | Must reference an active discipline when supplied |
| `subjectIds` | UUID array | No | Defaults to empty; up to 20 unique active subjects |

Category references: `academic_subject` requires at least one subject; `institution` requires `institutionId`; `institution_course` requires `courseId`; and `academic_discipline` requires `disciplineId`. A course association also requires the matching institution ID.

**Success:** `201 Created`, with the community response shape documented for `GET /api/v1/communities`.

**Errors:** `400` for invalid name, slug, language tag, category references, description length, subject count/duplicates, or unavailable directory records; `401 Sign in to continue`; `409 That community name is already in use`; `422` for JSON fields that cannot be deserialized; `500 An internal error occurred`.

## `GET /api/v1/communities/{slug}`

Returns an active public or restricted community without authentication. A private community is returned only to an active member; otherwise it is indistinguishable from a missing community and returns `404`.

**Success:** `200 OK`, with the community response shape documented above.

**Errors:** `401 Sign in to continue` when a supplied bearer token or session cookie is invalid; `404 Community was not found`; `500 An internal error occurred`.

## `PUT /api/v1/communities/{slug}/membership`

Requires an authenticated account. Joins an open public/restricted community or records a membership request when approval is required. Private communities can only be joined when an invitation membership already exists; uninvited callers receive `404` to avoid revealing the private community. Repeating an existing active join does not create another membership. Banned accounts cannot rejoin.

**Success:** `200 OK` for active membership; `202 Accepted` for an approval request. Response fields are `communityId`, `membershipStatus`, `communityRole`, and `joinedAt`.

**Errors:** `401 Sign in to continue`; `403 You cannot join this community`; `404 Community was not found` for an unknown community or an uninvited attempt to join a private community; `500 An internal error occurred`.

Membership approval and rejection endpoints are not implemented yet, so approval-required requests cannot yet be reviewed through this API.

## `DELETE /api/v1/communities/{slug}/membership`

Requires an authenticated account. A community owner cannot leave until ownership transfer is implemented.

**Success:** `204 No Content`.

**Errors:** `401 Sign in to continue`; `404 Community was not found` when the community or membership does not exist; `409 Transfer community ownership before leaving`; `500 An internal error occurred`.

## `GET /api/v1/profiles/me`

Returns the authenticated user's own profile, including profile visibility and directory identifiers. Private account fields such as email, birthday, eligibility evidence, and student verification are never included.

**Authentication:** Bearer session token or `peerfora_session` cookie.

**Success:** `200 OK`, with `accountId`, `username`, `displayName`, `biography`, `avatarReference`, `bannerReference`, `academicLevelId`, `institutionId`, `disciplineId`, and `profileVisibility`.

**Errors:** `401 Sign in to continue`; `404 Profile not found`; `500 An internal error occurred`.

## `PATCH /api/v1/profiles/me`

Updates the authenticated user's own profile. Omitted fields are retained. This endpoint currently does not support clearing an existing nullable field by sending `null`.

**Authentication:** Bearer session token or `peerfora_session` cookie.

**Request JSON:** All fields are optional: `displayName` (string, max 120 characters), `biography` (string, max 500 characters), `avatarReference` and `bannerReference` (references up to 2048 bytes), `academicLevelId` (canonical directory ID or supported alias such as `high_school`), `institutionId` and `disciplineId` (UUIDs), `profileVisibility` (`public` or `private`), and `showAcademicAffiliation` (boolean, defaults to false). Directory references are enforced by database foreign keys. Academic level, institution, and discipline are omitted from the public profile unless the owner explicitly enables `showAcademicAffiliation`.

**Success:** `200 OK`, with the updated own-profile shape above.

**Errors:** `400 Biography cannot exceed 500 characters`; `400 Display name cannot exceed 120 characters`; `400 Profile visibility must be public or private`; `400 Profile reference cannot exceed 2048 bytes`; `400 Selected academic directory reference was not found`; `401 Sign in to continue`; `404 Profile not found`; `500 An internal error occurred`.

## `GET /api/v1/profiles/{username}`

Returns only profiles whose owner selected public visibility. Private profiles and unknown usernames both return the same not-found response. No email, date of birth, eligibility information, account ID, or student verification data is exposed.

**Authentication:** None.

**Success:** `200 OK`, with `username`, `displayName`, `biography`, `avatarReference`, `bannerReference`, `academicLevelId`, `institutionName`, `disciplineName`, `showAcademicAffiliation`, and `createdAt`. Academic affiliation values are `null` unless the owner opted in.

**Errors:** `404 Profile not found`; `500 An internal error occurred`.

## `GET /api/v1/me/preferences`

Returns private preferences for the authenticated account.

**Authentication:** Bearer session token or `peerfora_session` cookie.

**Success:** `200 OK`, with `languageTag`, `countryCode`, `notificationPreferences`, `contentPreferences`, and `personalizationEnabled`.

**Errors:** `401 Sign in to continue`; `404 Profile not found`; `500 An internal error occurred`.

## `PATCH /api/v1/me/preferences`

Updates preferences for the authenticated account. Omitted fields are retained. Notification and content preferences are stored as JSON objects/values and are not yet interpreted by notification or feed behavior.

**Authentication:** Bearer session token or `peerfora_session` cookie.

**Request JSON:** All fields are optional: `languageTag` (ASCII tag matching the BCP 47 language-tag structure, up to 35 characters (subtag registry membership is not checked)), `countryCode` (two-letter code, uppercased), `notificationPreferences` (JSON value), `contentPreferences` (JSON value), and `personalizationEnabled` (boolean).

**Success:** `200 OK`, with the preferences response above.

**Errors:** `400 Country code must contain two letters`; `400 Language tag is invalid or too long`; `401 Sign in to continue`; `404 Profile not found`; `500 An internal error occurred`.

## `POST /api/v1/admin/auth/login`

Checks the separate administrator email/password credentials. Correct credentials return a five-minute, single-use challenge and the next required MFA step. A new administrator must enroll TOTP before any admin session can be issued. Existing administrators must verify TOTP.

**Authentication:** None.

**Request JSON:** `email` and `password`, both strings.

**Success:** `200 OK`, JSON fields `challengeToken`, `nextStep` (`enroll_mfa` or `verify_mfa`), and `expiresInSeconds` (`300`). No session cookie is issued at this step.

**Errors:** `401 Invalid administrator credentials` for invalid, suspended, revoked, or temporarily locked credentials; `429 Too many requests. Wait before trying again.` after twenty attempts for this email within 15 minutes; `500 An internal error occurred` for unexpected persistence failures. Five failed password attempts also apply a 15-minute per-administrator lock.

## `POST /api/v1/admin/invitations`

Creates a 48-hour single-use invitation, stores only the SHA-256 token hash, and sends the raw token in an email to the invited address. Reissuing an invitation revokes the previous pending invitation for that address. Email delivery failure revokes the newly created invitation and records the failure. The selected active role is bound to the invitation and assigned transactionally at acceptance. A Super Admin can invite another Super Admin; this requires recent authentication with the same active administrator session. Other administrators may invite only roles whose effective permissions are within their own current delegation scope. The invitation email uses the role's current display name.

**Authentication:** Active `peerfora_admin_session` cookie after MFA and the `admin.accounts.manage` permission. The seeded permission is assigned only to `super_administrator`.

**Request JSON:** `email` (valid email address) and `roleKey` (stable key of an active system or custom role the inviter is permitted to delegate).

**Success:** `202 Accepted`, no response body.

**Errors:** `400 Enter a valid email address`; `400 Administrator role is invalid`; `400 One or more permissions are unknown, unavailable, or outside your delegation scope`; `401 Administrator session is invalid`; `403 Administrator permission is required` or `403 Recent administrator authentication is required for this operation`; `409 An administrator identity already uses this email`; `503 Email delivery is not configured` or `503 Could not send the account email`; `500 An internal error occurred`.

## `POST /api/v1/admin/invitations/accept`

Consumes a valid, unexpired administrator invitation and transactionally creates the administrator identity, password hash, invited role assignment, consumed invitation state, and audit record. It does not issue an administrative session. The new administrator must sign in and complete mandatory TOTP enrollment before accessing administration.

**Authentication:** None. Possession of the one-use invitation token is required.

**Request JSON:** `token` (invitation token from the email) and `password` (16–128 bytes).

**Success:** `201 Created`, JSON fields `administratorId`, `assignedRole`, and `nextStep` (`sign_in_and_enroll_mfa`).

**Errors:** `400 Administrator invitation is invalid or expired`; `400 Administrator password must contain 16 to 128 bytes`; `409 An administrator identity already uses this email`; `500 An internal error occurred`.

## `POST /api/v1/admin/auth/mfa/enroll`

Starts TOTP enrollment for the pending enrollment challenge. The response contains an `otpauth://` provisioning URI; treat it as a secret and do not log or retain it. The URI contains the TOTP secret and is returned only for setup. TOTP secrets are encrypted with AES-256-GCM at rest using `PEERFORA_ADMIN_MFA_ENCRYPTION_KEY`, which must be a base64-encoded 32-byte deployment secret.

**Authentication:** A valid `challengeToken` from administrator login in the JSON body. This token only permits MFA enrollment and cannot authorize admin operations.

**Success:** `200 OK`, JSON field `provisioningUri`.

**Errors:** `401 Administrator challenge is invalid or expired`; `503 Administrator multifactor authentication is unavailable` when the encryption key is absent or invalid; `500 An internal error occurred` for unexpected processing failures.

## `POST /api/v1/admin/auth/mfa/confirm`

Confirms first-time authenticator enrollment. A valid TOTP code atomically activates MFA, consumes the enrollment challenge, stores the accepted TOTP step, creates ten single-use recovery codes, records an audit event, and creates the administrator session. Recovery-code plaintext is returned once and cannot be retrieved again.

**Authentication:** JSON fields `challengeToken` and six-digit `code` from the authenticator application.

**Success:** `200 OK`, sets `peerfora_admin_session` and returns `administratorId`, `email`, `roles`, effective `permissions`, and `recoveryCodes` (ten opaque strings; returned only in this enrollment response).

**Errors:** `401 Administrator challenge is invalid or expired`; `401 Administrator verification failed` for an invalid, already-used, or replayed code; `503 Administrator multifactor authentication is unavailable`; `500 An internal error occurred`.

## `POST /api/v1/admin/auth/mfa/verify`

Completes login for an administrator with enrolled TOTP. A valid TOTP code consumes its challenge and TOTP step transactionally. A valid recovery code is also accepted once; consuming the code, challenge and creating the admin-only session happen transactionally. Recovery codes are stored as SHA-256 hashes and successful recovery is audited.

**Authentication:** JSON fields `challengeToken` and `code` (six-digit current TOTP or an unused recovery code).

**Success:** `200 OK`, sets `peerfora_admin_session` and returns `administratorId`, `email`, `roles`, and effective `permissions`.

**Errors:** `401 Administrator challenge is invalid or expired`; `401 Administrator verification failed` for an invalid, replayed, or previously used code; `503 Administrator multifactor authentication is unavailable`; `500 An internal error occurred`.

## `POST /api/v1/admin/auth/mfa/recovery-codes/regenerate`

Replaces the authenticated administrator's unused recovery codes. Existing unused codes are revoked transactionally; previously consumed codes remain recorded as used. New plaintext codes are returned once, while only hashes are stored. The response uses `Cache-Control: no-store`.

**Authentication:** Active `peerfora_admin_session` cookie issued within the previous 15 minutes.

**Request JSON:** `code` (current six-digit authenticator-app TOTP code). Recovery codes cannot authorize their own replacement.

**Success:** `200 OK`, JSON field `recoveryCodes` containing ten one-use strings.

**Errors:** `401 Administrator session is invalid`; `401 Administrator verification failed` for an invalid or replayed TOTP code; `403 Recent administrator authentication is required for this operation`; `503 Administrator multifactor authentication is unavailable`; `500 An internal error occurred`. Shared JSON extractor errors apply.

## `GET /api/v1/admin/auth/me`

Returns the current active administrator identity, roles, and effective permissions.

**Authentication:** `peerfora_admin_session` cookie issued only after successful TOTP verification. Student sessions and bearer tokens are rejected.

**Success:** `200 OK`, JSON fields `administratorId`, `email`, `roles`, and `permissions`.

**Errors:** `401 Administrator session is invalid`; `503 Administrator authentication is unavailable`; `500 An internal error occurred`.

## `POST /api/v1/admin/auth/logout`

Revokes the current administrator session and clears its cookie. The revocation and audit event are written transactionally.

**Authentication:** `peerfora_admin_session` cookie.

**Success:** `204 No Content`, clears `peerfora_admin_session`.

**Errors:** `401 Administrator session is invalid`; `503 Administrator authentication is unavailable`; `500 An internal error occurred`.

## `POST /api/v1/admin/auth/password/change`

Changes the authenticated administrator's password after verifying the current password. The new password must contain 16–128 bytes and differ from the current password. On success, the transaction stores the Argon2id hash, revokes all active administrator sessions, consumes pending MFA challenges, and appends an audit event. The response clears `peerfora_admin_session`; the administrator must sign in again and complete MFA. A failed current-password check increments the administrator login-failure counter and is audited.

**Authentication:** Active `peerfora_admin_session` cookie after MFA.

**Request JSON:** `currentPassword` and `newPassword` (strings; each password must meet the 16–128-byte policy, and the new value must differ).

**Success:** `204 No Content`, clears `peerfora_admin_session`.

**Errors:** `400 Administrator password must contain 16 to 128 bytes`; `400 New administrator password must differ from the current password`; `401 Administrator session is invalid` or `401 Invalid administrator credentials`; `409 Administrator credentials changed; sign in again` when another request changed the password concurrently; `413` oversized body; `415` missing/invalid JSON content type; `422` malformed or incomplete request fields; `500 An internal error occurred`.

## `POST /api/v1/admin/auth/password-reset/request`

Requests a one-use administrator password reset email. The API returns the same status and message for known and unknown addresses. Reset delivery is attempted only when an email sender is configured. The token is stored as a hash, expires after 30 minutes, and replaces prior unused reset tokens. Failed email delivery triggers token revocation and a generic server log entry. This route does not create a session or alter MFA.

**Authentication:** None.

**Request JSON:** `email` (valid email address, at most 254 bytes).

**Success:** `202 Accepted`, `{"message":"If the administrator account exists, password reset instructions have been sent"}` with `Cache-Control: no-store` and `Referrer-Policy: no-referrer`.

**Errors:** `400 Enter a valid email address`; `429 Too many requests. Wait before trying again.` after three requests for this email within one hour; `413` oversized body; `415` missing/invalid JSON content type; `422` malformed or incomplete request fields; `500 An internal error occurred`. SMTP delivery failures retain the generic `202` response. Delivery is synchronous when configured; timing equalization is not implemented yet.

## `POST /api/v1/admin/auth/password-reset/confirm`

Consumes a valid, unused reset token and sets a new Argon2id password hash. In one transaction it revokes all active administrator sessions, consumes pending MFA challenges, revokes other unused reset tokens, and appends an audit record. The administrator must sign in again and complete mandatory MFA; the reset does not reset, disable, or replace MFA.

**Authentication:** None.

**Request JSON:** `token` and `newPassword` (strings; new password must contain 16–128 bytes).

**Success:** `204 No Content`, with `Cache-Control: no-store` and `Referrer-Policy: no-referrer`. No session cookie is issued.

**Errors:** `400 Administrator password must contain 16 to 128 bytes`; `400 Administrator password reset token is invalid or expired`; `413` oversized body; `415` missing/invalid JSON content type; `422` malformed or incomplete request fields; `500 An internal error occurred`.

## `GET /api/v1/admin/sessions`

Returns up to 50 recent sessions belonging to the authenticated administrator, including revoked and expired entries. Session tokens are never returned.

**Authentication:** Active `peerfora_admin_session` cookie after MFA.

**Success:** `200 OK`, JSON field `items`; each item contains `id`, `createdAt`, `lastActivityAt`, `expiresAt`, and nullable `revokedAt`.

**Errors:** `401 Administrator session is invalid`; `500 An internal error occurred`.

## `DELETE /api/v1/admin/sessions/{id}`

Revokes a session owned by the current administrator and records the action. Administrators cannot use this endpoint to revoke another administrator's session.

**Authentication:** Active `peerfora_admin_session` cookie after MFA.

**Path parameter:** `id` (UUID of a session owned by the authenticated administrator).

**Success:** `204 No Content`.

**Errors:** `401 Administrator session is invalid`; `404 Administrator session was not found`; `500 An internal error occurred`.

## `GET /api/v1/admin/overview`

Returns aggregate counts from current PostgreSQL records. It does not return personal data or metrics for features that have no backing tables.

**Authentication:** Active `peerfora_admin_session` cookie after MFA and the `admin.overview.read` permission.

**Success:** `200 OK`, JSON fields `registeredAccounts`, `activeAccounts`, `pendingEmailVerification`, `suspendedAccounts`, `totalInstitutions`, `totalCommunities`, and `activeCommunities`.

Metric definitions: `registeredAccounts` counts all retained account rows, including lifecycle states; `activeAccounts` and `suspendedAccounts` use the current account lifecycle state; `pendingEmailVerification` counts active accounts without `email_verified_at`; `totalInstitutions` counts institution directory rows without implying institutional verification; `totalCommunities` counts all community rows; and `activeCommunities` counts rows with `lifecycle_status = active`. These are cumulative/current counts, not daily or monthly active users. Date, country, academic-level, and community-category filters are not implemented.

**Errors:** `401 Administrator session is invalid`; `403 Administrator permission is required`; `503 Administrator authentication is unavailable`; `500 An internal error occurred`.

## `GET /api/v1/admin/audit-logs`

Returns audit records in reverse chronological order. Access requires an active MFA-complete administrator session with the `audit_logs.read` permission. Reading the log itself creates an audit event. Permission-denied reads are also audited.

**Authentication:** `peerfora_admin_session` cookie.

**Query parameters:** `limit` (optional integer, default 50, range 1–100) and `offset` (optional integer, default 0, maximum 100000).

**Success:** `200 OK`, JSON fields `items`, `limit`, and `offset`. Each item includes actor, action, target, result, reason, correlation ID, and timestamp. Records are append-only under application and ordinary database access; a database owner can still alter schema or bypass triggers.

**Errors:** `401 Administrator session is invalid`; `403 Administrator permission is required`; `400 Pagination parameters are invalid`; `503 Administrator authentication is unavailable`; `500 An internal error occurred`.

## `GET /api/v1/admin/accounts`

Searches and filters retained account records. Search matches a case-insensitive email or username prefix. Filters are exact; the country code is normalized to uppercase. The response intentionally excludes passwords, session tokens, date of birth, and eligibility evidence.

**Authentication:** Active `peerfora_admin_session` cookie after MFA and the `accounts.read` permission.

**Query parameters:** `search` (optional, 1–128 characters), `status` (`active`, `suspended`, `pending_deletion`, or `deleted`), `countryCode` (optional two-letter code), `academicLevelId` (optional canonical level), `limit` (default 50, range 1–100), and `offset` (default 0, maximum 100000).

**Success:** `200 OK`, JSON fields `items`, `totalCount`, `limit`, and `offset`. Each item contains `id`, `email`, `username`, `status`, `emailVerified`, `countryCode`, nullable `academicLevelId`, and `createdAt`.

**Errors:** `400 Account search filters are invalid`; `400 Pagination parameters are invalid`; `401 Administrator session is invalid`; `403 Administrator permission is required`; `503 Administrator authentication is unavailable`; `500 An internal error occurred`.

## `GET /api/v1/admin/accounts/{id}`

Returns the same minimal account administration summary as one item in account search. It does not return profile biography, credentials, date of birth, tokens, or verification evidence.

**Authentication:** Active `peerfora_admin_session` cookie after MFA and the `accounts.read` permission.

**Path parameter:** `id` (account UUID).

**Success:** `200 OK`, an account summary object with the fields listed for account search.

**Errors:** `400` for a malformed UUID path; `401 Administrator session is invalid`; `403 Administrator permission is required`; `404 Account was not found`; `503 Administrator authentication is unavailable`; `500 An internal error occurred`.

## `POST /api/v1/admin/accounts/{id}/suspend`

Permanently suspends an active account until an authorized reinstatement. In one PostgreSQL transaction it updates the account lifecycle status, deletes active student sessions and password-reset/email-verification tokens, and records the administrator action. The account's existing session cookie becomes unusable immediately. Temporary suspensions and automatic expiry are not implemented.

**Authentication:** Active `peerfora_admin_session` cookie after MFA and the `accounts.suspend` permission.

**Path parameter:** `id` (account UUID).

**Request JSON:** `reasonCode`, one of `policy_violation`, `user_safety`, `account_compromise`, `legal_requirement`, or `other`. Free-form notes are not stored in the audit record.

**Success:** `204 No Content`.

**Errors:** `400 Account enforcement reason is invalid`; `400` for a malformed UUID path; `401 Administrator session is invalid`; `403 Administrator permission is required`; `404 Account was not found`; `409 Account is not in a state that allows this action`; `500 An internal error occurred`.

## `POST /api/v1/admin/accounts/{id}/sessions/revoke`

Revokes every current session belonging to the account. The operation takes a row lock on the account so an in-flight session creation is serialized with revocation. It records the administrator, reason, target, and request correlation ID in the append-only audit log in the same transaction. A later user login can create a new session.

**Authentication:** Active `peerfora_admin_session` cookie after MFA and the `accounts.sessions.revoke` permission. The permission is assigned to Super Administrators, Platform Administrators, and Trust and Safety Leads by the current migration.

**Path parameter:** `id` (account UUID).

**Request JSON:** `reasonCode`, one of `policy_violation`, `user_safety`, `account_compromise`, `legal_requirement`, or `other`.

**Success:** `200 OK`, JSON field `revokedSessionCount` (number of sessions deleted; zero when none existed).

**Errors:** `400 Account enforcement reason is invalid`; `400` for a malformed UUID path; `401 Administrator session is invalid`; `403 Administrator permission is required`; `404 Account was not found`; `500 An internal error occurred`.

## `GET /api/v1/admin/communities`

Searches active, suspended, and archived communities using metadata only. Private community descriptions, rules, posts, and member identities are not returned. Search is a bounded case-insensitive prefix match over community name and slug.

**Authentication:** Active `peerfora_admin_session` cookie after MFA and the `communities.read` permission.

**Query parameters:** `search` (optional, 1–128 characters), `category` (existing community category), `status` (`active`, `suspended`, or `archived`), `limit` (default 50, range 1–100), and `offset` (default 0, maximum 100000).

**Success:** `200 OK`, JSON fields `items`, `totalCount`, `limit`, and `offset`. Each item includes `id`, `slug`, `displayName`, `category`, `visibility`, `lifecycleStatus`, `ownerAccountId`, active `memberCount`, and `createdAt`.

**Errors:** `400 Community search filters are invalid`; `400 Pagination parameters are invalid`; `401 Administrator session is invalid`; `403 Administrator permission is required`; `503 Administrator authentication is unavailable`; `500 An internal error occurred`.

## `GET /api/v1/admin/communities/{id}`

Returns the same administrative metadata item as community search. It does not return private community content, rules, posts, or member identities.

**Authentication:** Active `peerfora_admin_session` cookie after MFA and the `communities.read` permission.

**Path parameter:** `id` (community UUID).

**Success:** `200 OK`, a community metadata item with the fields listed for community search.

**Errors:** `400` for a malformed UUID path; `401 Administrator session is invalid`; `403 Administrator permission is required`; `404 Community was not found`; `503 Administrator authentication is unavailable`; `500 An internal error occurred`.

## `POST /api/v1/admin/communities/{id}/suspend`

Changes an active community to suspended and appends an audit record in one transaction. Suspended communities are omitted from public discovery and cannot be joined. The owner and active owner membership remain intact. No private community content is accessed by this operation.

**Authentication:** Active `peerfora_admin_session` cookie after MFA and the `communities.manage` permission.

**Path parameter:** `id` (community UUID).

**Request JSON:** `reasonCode`, one of `policy_violation`, `user_safety`, `legal_requirement`, or `other`.

**Success:** `204 No Content`.

**Errors:** `400 Community enforcement reason is invalid`; `400` for a malformed UUID path; `401 Administrator session is invalid`; `403 Administrator permission is required`; `404 Community was not found`; `409 Community is not in a state that allows this action`; `500 An internal error occurred`.

## `POST /api/v1/admin/communities/{id}/restore`

Changes a suspended community back to active and appends an audit record transactionally. Archived communities cannot be restored through this endpoint.

**Authentication:** Active `peerfora_admin_session` cookie after MFA and the `communities.manage` permission.

**Path parameter:** `id` (community UUID).

**Request JSON:** `reasonCode`, one of `policy_violation`, `user_safety`, `legal_requirement`, or `other`.

**Success:** `204 No Content`.

**Errors:** `400 Community enforcement reason is invalid`; `400` for a malformed UUID path; `401 Administrator session is invalid`; `403 Administrator permission is required`; `404 Community was not found`; `409 Community is not in a state that allows this action`; `500 An internal error occurred`.

## `POST /api/v1/admin/accounts/{id}/reinstate`

Changes a suspended account back to active and appends an audit record transactionally. Previously revoked student sessions are not restored; the account must sign in again.

**Authentication:** Active `peerfora_admin_session` cookie after MFA and the `accounts.suspend` permission.

**Path parameter:** `id` (account UUID).

**Request JSON:** `reasonCode`, one of `policy_violation`, `user_safety`, `account_compromise`, `legal_requirement`, or `other`.

**Success:** `204 No Content`.

**Errors:** `400 Account enforcement reason is invalid`; `400` for a malformed UUID path; `401 Administrator session is invalid`; `403 Administrator permission is required`; `404 Account was not found`; `409 Account is not in a state that allows this action`; `500 An internal error occurred`.

## Administrative roles and permissions

All endpoints below require an active administrator session after mandatory MFA. Effective permissions are resolved from active role assignments at request time; disabling a role removes its permissions immediately. Permission-denied requests are recorded in the audit log.

### `GET /api/v1/admin/roles`

Lists active system and custom roles. Each item contains `id` (stable UUID), `roleKey` (stable authorization key), `displayName`, `description`, `isSystemRole`, `isActive`, and effective `permissions` assigned to that role.

**Permission:** `admin.roles.read`.

**Success:** `200 OK`, JSON object with `items`.

**Errors:** `401 Administrator session is invalid`; `403 Administrator permission is required`; `503 Administrator authentication is unavailable`; `500 An internal error occurred`.

### `POST /api/v1/admin/roles`

Creates a custom role and its initial permission set in one transaction. `roleKey` must be 3–64 lowercase ASCII letters, digits, or underscores and start with a letter. `displayName` must be 3–80 characters; `description` must be 1–500 characters. `permissionKeys` may contain up to 100 distinct recognized permissions. Only catalog entries marked assignable can be selected, and every selected permission must already be in the actor's effective permissions.

**Permission:** `admin.roles.create`.

**Request JSON:** `roleKey`, `displayName`, `description`, and `permissionKeys` (array of permission-key strings).

**Success:** `201 Created`, the role item described above.

**Errors:** `400 Administrative role details are invalid`; `400 Role key must use lowercase letters, numbers, and underscores`; `400 One or more permissions are unknown, unavailable, or outside your delegation scope`; `401 Administrator session is invalid`; `403 Administrator permission is required`; `409 An administrative role already uses this key`; `500 An internal error occurred`.

### `GET /api/v1/admin/roles/{id}`

Returns one active role and its current permission set.

**Permission:** `admin.roles.read`.

**Path parameter:** `id` (role UUID).

**Success:** `200 OK`, a role item.

**Errors:** `400` for malformed UUID; `401 Administrator session is invalid`; `403 Administrator permission is required`; `404 Administrative role was not found`; `500 An internal error occurred`.

### `PATCH /api/v1/admin/roles/{id}`

Updates custom role metadata or active state. At least one supported field must be present. `displayName` and `description` follow the same length rules as role creation. System roles, including the reserved Super Admin role, cannot be changed.

**Permission:** `admin.roles.update`.

**Request JSON:** Any of `displayName`, `description`, and `isActive`.

**Success:** `200 OK`, the updated role item.

**Errors:** `400 Administrative role details are invalid`; `400 At least one role field must be provided`; `401 Administrator session is invalid`; `403 Administrator permission is required` or `403 System roles cannot be changed through this operation`; `404 Administrative role was not found`; `500 An internal error occurred`.

### `DELETE /api/v1/admin/roles/{id}`

Logically deletes an unused custom role by disabling it and recording `deletedAt`. Role records and their permission history remain stored. A role assigned to an administrator or an unexpired pending invitation cannot be deleted. System roles cannot be deleted.

**Permission:** `admin.roles.delete`.

**Success:** `204 No Content`.

**Errors:** `400` for malformed UUID; `401 Administrator session is invalid`; `403 Administrator permission is required` or `403 System roles cannot be changed through this operation`; `404 Administrative role was not found`; `409 Role is assigned to an administrator or pending invitation and cannot be deleted`; `500 An internal error occurred`.

### `GET /api/v1/admin/permissions`

Returns the known permission catalog with `permissionKey`, `description`, `isAssignable`, and `isSensitive`. Catalog entries for deferred capabilities are not assignable and cannot authorize an unimplemented operation.

**Permission:** `admin.permissions.read`.

**Success:** `200 OK`, JSON object with `items`.

**Errors:** `401 Administrator session is invalid`; `403 Administrator permission is required`; `500 An internal error occurred`.

### `GET /api/v1/admin/roles/{id}/permissions`

Returns the role item and its current permission set.

**Permission:** `admin.permissions.read`.

**Success:** `200 OK`, a role item.

**Errors:** `400` for malformed UUID; `401 Administrator session is invalid`; `403 Administrator permission is required`; `404 Administrative role was not found`; `500 An internal error occurred`.

### `PUT /api/v1/admin/roles/{id}/permissions`

Replaces a custom role's permission set transactionally. The `permissionKeys` array uses only catalog entries marked assignable, and every permission must be within the acting administrator's effective permission set. System-role permissions are reserved and immutable through this API.

**Permission:** `admin.permissions.assign`.

**Request JSON:** `permissionKeys` (array of up to 100 distinct permission-key strings; an empty array removes all permissions).

**Success:** `200 OK`, the updated role item.

**Errors:** `400 One or more permissions are unknown, unavailable, or outside your delegation scope`; `401 Administrator session is invalid`; `403 Administrator permission is required` or `403 System roles cannot be changed through this operation`; `404 Administrative role was not found`; `500 An internal error occurred`.

### `GET /api/v1/admin/administrators/{id}/roles`

Lists system and custom role assignments for the target administrator.

**Permission:** `admin.roles.read`.

**Success:** `200 OK`, JSON object with `items` containing role items.

**Errors:** `400` for malformed UUID; `401 Administrator session is invalid`; `403 Administrator permission is required`; `404 Administrator was not found`; `500 An internal error occurred`.

## Administrator identity lifecycle

The following state-changing operations require the actor's active administrative session, the `admin.accounts.manage` permission, and an MFA-authenticated session created within the last 15 minutes. Each operation locks administrative authorization state, checks the actor's current permission again inside the transaction, records a reason and correlation ID in the append-only audit log, and commits lifecycle changes with credential/session revocation.

### `POST /api/v1/admin/administrators/{id}/suspend`

Suspends an active, non-seeded administrator. This prevents new login, revokes all administrative sessions and pending MFA challenges, and cancels invitations created by that administrator. At least one other active, MFA-enabled Super Admin must remain. Administrators cannot suspend themselves.

**Request JSON:** `reason` (`security_concern`, `policy_violation`, `role_change`, `departure`, or `other`).

**Success:** `204 No Content`.

**Errors:** `400` malformed JSON or UUID; `401 Administrator session is invalid`; `403 Administrator permission is required`, `403 The original seeded administrator is protected`, or `403 Recent administrator authentication is required for this operation`; `404 Administrator was not found`; `409` for an invalid lifecycle transition, self-management, or removal of the last usable Super Admin; `413` oversized body; `415` missing/invalid JSON content type; `422` missing or unrecognized reason; `500 An internal error occurred`.

### `POST /api/v1/admin/administrators/{id}/reinstate`

Reinstates a suspended, non-seeded administrator. Existing roles and credentials are retained; revoked sessions remain revoked, so the administrator must sign in and complete MFA again.

**Request JSON:** `reason` (`security_concern`, `policy_violation`, `role_change`, `departure`, or `other`).

**Success:** `204 No Content`.

**Errors:** `400` malformed JSON or UUID; `401 Administrator session is invalid`; `403 Administrator permission is required`, `403 The original seeded administrator is protected`, or `403 Recent administrator authentication is required for this operation`; `404 Administrator was not found`; `409` for an invalid lifecycle transition or self-management; `413` oversized body; `415` missing/invalid JSON content type; `422` missing or unrecognized reason; `500 An internal error occurred`.

### `DELETE /api/v1/admin/administrators/{id}`

Logically deletes an active or suspended, non-seeded administrator. The API replaces the email with an `@invalid.peerfora` address, clears the password hash and MFA secret, revokes sessions, challenges, invitations, and role assignments, and retains the administrator row and audit references. Physical deletion is not performed.

**Request JSON:** `reason` (`security_concern`, `policy_violation`, `role_change`, `departure`, or `other`).

**Success:** `204 No Content`.

**Errors:** `400` malformed JSON or UUID; `401 Administrator session is invalid`; `403 Administrator permission is required`, `403 The original seeded administrator is protected`, or `403 Recent administrator authentication is required for this operation`; `404 Administrator was not found`; `409` for an invalid lifecycle transition, self-management, or removal of the last usable Super Admin; `413` oversized body; `415` missing/invalid JSON content type; `422` missing or unrecognized reason; `500 An internal error occurred`.

### `DELETE /api/v1/admin/administrators/{id}/sessions`

Revokes all active sessions and consumes outstanding MFA challenges for a different administrator, then records the reason and request correlation ID in the audit log. The target administrator row is locked to serialize this change with MFA login session creation.

**Authentication:** Active administrator session with `admin.accounts.manage` permission. The session must have been created within the previous 15 minutes.

**Request JSON:** `reason` (`security_concern`, `policy_violation`, `role_change`, `departure`, or `other`).

**Success:** `204 No Content`.

**Errors:** `400` for malformed JSON or UUID; `401 Administrator session is invalid`; `403 Administrator permission is required` or recent administrator authentication is missing; `404 Administrator was not found`; `409 Use the current-session endpoint to revoke your own administrator session`; `413` oversized body; `415` missing/invalid JSON content type; `422` missing or unrecognized reason; `500 An internal error occurred`.

### `POST /api/v1/admin/administrators/{id}/mfa/reset`

Resets MFA for a different active administrator. The endpoint requires `admin.accounts.manage` and an actor session created within the past 15 minutes. It clears the target's TOTP credential and lockout, revokes active sessions, consumes pending MFA challenges, revokes unused recovery codes, and records the actor, reason, target, and correlation ID in one transaction. The target must sign in with their existing password and enroll a new authenticator before receiving an administrative session. This operation does not change the target's password or grant a session.

**Request JSON:** `reason` (`security_concern`, `policy_violation`, `role_change`, `departure`, or `other`). Self-reset is rejected; use an independently authorized administrator.

**Success:** `204 No Content`.

**Errors:** `400` malformed JSON or UUID; `401 Administrator session is invalid`; `403 Administrator permission is required` or recent administrator authentication is missing; `404 Administrator was not found`; `409 Administrators cannot reset their own multifactor authentication`; `413` oversized body; `415` missing/invalid JSON content type; `422` missing or unrecognized reason; `500 An internal error occurred`.

### `PUT /api/v1/admin/administrators/{id}/roles`

Replaces the target administrator's assigned roles in a single transaction. At most 50 distinct active role IDs may be submitted. Permissions granted by the selected roles must be within the actor's current effective permissions. The migration-seeded root administrator cannot be modified.

**Permission:** `admin.roles.assign`.

**Request JSON:** `roleIds` (array of role UUIDs; an empty array removes all role assignments from an unprotected administrator).

**Success:** `200 OK`, JSON object with the updated role `items`.

**Errors:** `400 Administrator role assignments are invalid`; `400 One or more permissions are unknown, unavailable, or outside your delegation scope`; `400` for malformed UUID; `401 Administrator session is invalid`; `403 Administrator permission is required` or `403 The original seeded administrator is protected`; `404 Administrator or one of its roles was not found`; `500 An internal error occurred`.

### `GET /api/v1/admin/administrators/{id}/permissions`

Returns effective permission keys from the target administrator's active roles.

**Permission:** `admin.roles.read`.

**Success:** `200 OK`, JSON fields `administratorId` and sorted `permissionKeys`.

**Errors:** `400` for malformed UUID; `401 Administrator session is invalid`; `403 Administrator permission is required`; `404 Administrator was not found`; `500 An internal error occurred`.
