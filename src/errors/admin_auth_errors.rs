use axum::http::StatusCode;

use crate::enums::admin_auth_error::AdminAuthError;

pub fn get_admin_auth_error_response(error: AdminAuthError) -> (StatusCode, &'static str) {
    match error {
        AdminAuthError::InvalidCredentials => (
            StatusCode::UNAUTHORIZED,
            "Invalid administrator credentials",
        ),
        AdminAuthError::InvalidChallenge => (
            StatusCode::UNAUTHORIZED,
            "Administrator challenge is invalid or expired",
        ),
        AdminAuthError::InvalidMfaCode => (
            StatusCode::UNAUTHORIZED,
            "Administrator verification failed",
        ),
        AdminAuthError::InvalidSession => {
            (StatusCode::UNAUTHORIZED, "Administrator session is invalid")
        }
        AdminAuthError::MfaEncryptionKeyUnavailable => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Administrator multifactor authentication is unavailable",
        ),
        AdminAuthError::InvalidBootstrapInput => (
            StatusCode::BAD_REQUEST,
            "Administrator bootstrap input is invalid",
        ),
        AdminAuthError::BootstrapAlreadyCompleted => (
            StatusCode::CONFLICT,
            "Administrator bootstrap has already been completed",
        ),
        AdminAuthError::AuthenticationUnavailable => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Administrator authentication is unavailable",
        ),
        AdminAuthError::PermissionDenied => (
            StatusCode::FORBIDDEN,
            "Administrator permission is required",
        ),
        AdminAuthError::InvalidPagination => {
            (StatusCode::BAD_REQUEST, "Pagination parameters are invalid")
        }
        AdminAuthError::SessionNotFound => {
            (StatusCode::NOT_FOUND, "Administrator session was not found")
        }
        AdminAuthError::InvalidAccountFilter => (
            StatusCode::BAD_REQUEST,
            "Account search filters are invalid",
        ),
        AdminAuthError::AccountNotFound => (StatusCode::NOT_FOUND, "Account was not found"),
        AdminAuthError::InvalidInvitation => (
            StatusCode::BAD_REQUEST,
            "Administrator invitation is invalid or expired",
        ),
        AdminAuthError::InvitationEmailAlreadyInUse => (
            StatusCode::CONFLICT,
            "An administrator identity already uses this email",
        ),
        AdminAuthError::InvalidAdministratorRole => {
            (StatusCode::BAD_REQUEST, "Administrator role is invalid")
        }
        AdminAuthError::InvalidAdministratorPasswordLength => (
            StatusCode::BAD_REQUEST,
            "Administrator password must contain 16 to 128 bytes",
        ),
        AdminAuthError::AdministratorPasswordUnchanged => (
            StatusCode::BAD_REQUEST,
            "New administrator password must differ from the current password",
        ),
        AdminAuthError::AdministratorCredentialChanged => (
            StatusCode::CONFLICT,
            "Administrator credentials changed; sign in again",
        ),
        AdminAuthError::InvalidAdministratorPasswordResetToken => (
            StatusCode::BAD_REQUEST,
            "Administrator password reset token is invalid or expired",
        ),
        AdminAuthError::InvalidEnforcementReason => (
            StatusCode::BAD_REQUEST,
            "Account enforcement reason is invalid",
        ),
        AdminAuthError::AccountStateConflict => (
            StatusCode::CONFLICT,
            "Account is not in a state that allows this action",
        ),
        AdminAuthError::InvalidCommunityFilter => (
            StatusCode::BAD_REQUEST,
            "Community search filters are invalid",
        ),
        AdminAuthError::CommunityNotFound => (StatusCode::NOT_FOUND, "Community was not found"),
        AdminAuthError::CommunityStateConflict => (
            StatusCode::CONFLICT,
            "Community is not in a state that allows this action",
        ),
        AdminAuthError::AccountOwnsCommunities => (
            StatusCode::CONFLICT,
            "Community ownership must be resolved before suspending this account",
        ),
        AdminAuthError::InvalidCommunityEnforcementReason => (
            StatusCode::BAD_REQUEST,
            "Community enforcement reason is invalid",
        ),
        AdminAuthError::AdministratorEmailAlreadyInUse => (
            StatusCode::CONFLICT,
            "An administrator identity already uses this email",
        ),
        AdminAuthError::InvalidRoleKey => (
            StatusCode::BAD_REQUEST,
            "Role key must use lowercase letters, numbers, and underscores",
        ),
        AdminAuthError::RoleNotFound => {
            (StatusCode::NOT_FOUND, "Administrative role was not found")
        }
        AdminAuthError::SystemRoleProtected => (
            StatusCode::FORBIDDEN,
            "System roles cannot be changed through this operation",
        ),
        AdminAuthError::RoleInUse => (
            StatusCode::CONFLICT,
            "Role is assigned to an administrator or pending invitation and cannot be deleted",
        ),
        AdminAuthError::InvalidPermissionSelection => (
            StatusCode::BAD_REQUEST,
            "One or more permissions are unknown, unavailable, or outside your delegation scope",
        ),
        AdminAuthError::AdministratorNotFound => {
            (StatusCode::NOT_FOUND, "Administrator was not found")
        }
        AdminAuthError::SeededAdministratorProtected => (
            StatusCode::FORBIDDEN,
            "The original seeded administrator is protected",
        ),
        AdminAuthError::EmptyRoleUpdate => (
            StatusCode::BAD_REQUEST,
            "At least one role field must be provided",
        ),
        AdminAuthError::RoleKeyAlreadyExists => (
            StatusCode::CONFLICT,
            "An administrative role already uses this key",
        ),
        AdminAuthError::InvalidRoleDetails => (
            StatusCode::BAD_REQUEST,
            "Administrative role details are invalid",
        ),
        AdminAuthError::InvalidRoleAssignment => (
            StatusCode::BAD_REQUEST,
            "Administrator role assignments are invalid",
        ),
        AdminAuthError::RecentAdministratorAuthenticationRequired => (
            StatusCode::FORBIDDEN,
            "Recent administrator authentication is required for this operation",
        ),
        AdminAuthError::AdministratorStateConflict => (
            StatusCode::CONFLICT,
            "Administrator is not in a state that allows this action",
        ),
        AdminAuthError::AdministratorSelfManagement => (
            StatusCode::CONFLICT,
            "Administrators cannot suspend or delete their own identity",
        ),
        AdminAuthError::AdministratorSelfMfaReset => (
            StatusCode::CONFLICT,
            "Administrators cannot reset their own multifactor authentication",
        ),
        AdminAuthError::AdministratorSelfSessionRevocation => (
            StatusCode::CONFLICT,
            "Use the current-session endpoint to revoke your own administrator session",
        ),
        AdminAuthError::LastSuperAdministratorProtected => (
            StatusCode::CONFLICT,
            "At least one active multifactor-enabled Super Admin must remain",
        ),
        AdminAuthError::InvalidAdministratorStatus => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "An internal error occurred",
        ),
    }
}
