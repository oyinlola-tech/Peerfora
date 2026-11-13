# Peerfora HTTP API reference

This document describes every route currently registered by the API. It is based on `src/routes/health_routes.rs`, `src/routes/account_auth_routes.rs`, the request/response DTOs, and the controller/error mappings. All API paths are relative to the API origin, for example `http://localhost:3000` locally.

## Shared request and response behavior

- Send JSON request bodies with `Content-Type: application/json`.
- Successful JSON responses use `application/json`.
- Errors created by the application use `{"message":"..."}`. Framework extractor rejections may use Axum's plain text response instead.
- A malformed JSON body returns `400`; JSON that cannot deserialize into the expected fields returns `422`; a missing/non-JSON content type returns `415`; an oversized body can return `413`.
- Missing or malformed query parameters return `400`. An unmatched path returns `404`; a known path with an unsupported method returns `405`.
- Unexpected request-time failures return `500` with `{"message":"An internal error occurred"}`. PostgreSQL connection or migration failures happen during startup; the API will not start serving routes in that case.
- No API rate limiting, request timeout, or CSRF/origin enforcement is currently implemented. Directory/community collection endpoints use a bounded `limit` (1–50) without cursor tokens; do not use this pagination shape for future high-volume feeds, posts, reports, or audit logs. No generic webhook routes are registered.

## Authentication and cookies

- Session-protected reads accept either `Authorization: Bearer <session-token>` or the `peerfora_session` cookie. If both are present, the bearer token is used.
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

**Errors:** `400 Enter a valid email address`; `401 Invalid email or password` for an unknown account, social-only account, or wrong password; `403 Confirm your email before signing in` for an unconfirmed account; `500 An internal error occurred` for unexpected failures. Extractor errors are described above.

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

**Errors:** `400 Enter a valid email address`; `503 Email delivery is not configured`; `503 Could not send the account email`; `500 An internal error occurred`. Unknown addresses still receive the same `202` response and do not trigger email delivery.

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
