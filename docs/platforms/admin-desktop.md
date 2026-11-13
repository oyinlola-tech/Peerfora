# Administration desktop application

The separate admin shell is in `apps/admin-desktop`. It has its own product identifier, Tauri configuration, window label, frontend build directory, and capability file. Placeholder route paths match the master specification.

No administrative token is created or accepted by this shell. Admin authentication, MFA, expiry, dedicated API client, secure token storage, backend authorization, and signed updates remain unimplemented. Build prerequisites match [the student desktop app](desktop.md).
