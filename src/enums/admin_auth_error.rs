#[derive(Debug, Clone, Copy)]
pub enum AdminAuthError {
    InvalidCredentials,
    InvalidChallenge,
    InvalidMfaCode,
    InvalidSession,
    MfaEncryptionKeyUnavailable,
    InvalidBootstrapInput,
    BootstrapAlreadyCompleted,
    PasswordTooShort,
    AuthenticationUnavailable,
    PermissionDenied,
    InvalidPagination,
    SessionNotFound,
}
