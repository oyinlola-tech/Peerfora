# Peerfora HTTP API reference

This document describes every route currently registered by the API. It is based on `src/routes/health_routes.rs`, `src/routes/account_auth_routes.rs`, the request/response DTOs, and the controller/error mappings. All API paths are relative to the API origin, for example `http://localhost:3000` locally.

## Shared request and response behavior

- Send JSON request bodies with `Content-Type: application/json`.
- Successful JSON responses use `application/json`.
- Errors created by the application use `{"message":"..."}`. Framework extractor rejections may use Axum's plain text response instead.
- A malformed JSON body returns `400`; JSON that cannot deserialize into the expected fields returns `422`; a missing/non-JSON content type returns `415`; an oversized body can return `413`.
- Missing or malformed query parameters return `400`. An unmatched path returns `404`; a known path with an unsupported method returns `405`.
- Unexpected request-time failures return `500` with `{"message":"An internal error occurred"}`. PostgreSQL connection or migration failures happen during startup; the API will not start serving routes in that case.
- No API rate limiting is currently implemented.

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
| `dateOfBirth` | string | Yes | Calendar date formatted `YYYY-MM-DD`; must satisfy configured country age policy |
| `countryCode` | string | Yes | Two ASCII letters; uppercased before policy lookup |
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
| `403` | `Registration is not available in this country` | No country policy is configured for the country |
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
| `countryCode` | string | Yes | Two ASCII letters; must have a configured policy |
| `acceptedTerms` | boolean | Yes | Must be `true` |

**Success:** `200 OK`; clears the pending signup cookie, sets `peerfora_session`, and returns the same account shape as login.

**Errors:** `400 Social signup has expired. Start again.`; `400 Date of birth must use YYYY-MM-DD`; `400 Country code must be a two-letter country code`; `400 Username must be 3 to 24 letters, numbers, or underscores`; `400 Accept the terms to create an account`; `403 Registration is not available in this country`; `403 You do not meet the age requirement`; `409 Email or username is already in use`; `503 This country requires an age-check method that is not configured`; `500 An internal error occurred`.

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

Registration and social signup are disabled for countries without an explicit policy. Configure policies through `PEERFORA_COUNTRY_POLICIES`; the map key is the two-letter country code and values use this shape:

```json
{
  "NG": {
    "minimum_age": 18,
    "policy_version": "policy-version-from-product",
    "age_assurance_required": false
  }
}
```

The example is a schema illustration, not an enabled policy recommendation. When `age_assurance_required` is `true`, signup currently fails with `503` because no age-assurance integration is implemented.
