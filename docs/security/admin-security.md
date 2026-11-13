# Administration security status

The administration desktop is a separate shell with its own application identifier and a minimal Tauri capability set. It has no administrative authentication, MFA, API client, token storage, role enforcement, or audit-log integration yet. The desktop shell is not a security boundary; the backend must authorize every administrative operation.
