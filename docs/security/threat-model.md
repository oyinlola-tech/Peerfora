# Threat model status

The initial sensitive assets are account credentials, session and one-time tokens, student identity details, and age-eligibility outcomes. The backend stores password hashes and token hashes, does not persist date of birth, and avoids logging secret values according to the current repository rules.

The iOS scaffold includes a Keychain credential-store implementation, but no sign-in flow calls it yet. The other app and widget projects do not store credentials or fetch personalized data. Before launch, review session handling, CSRF and origin policy, rate limits, dependency advisories, device storage, deep links, and the complete data lifecycle. No independent security assessment has been completed.
