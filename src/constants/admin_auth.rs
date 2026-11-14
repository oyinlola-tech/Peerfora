pub const ADMIN_AUTH_LOGIN_ROUTE: &str = "/api/v1/admin/auth/login";
pub const ADMIN_AUTH_MFA_ENROLL_ROUTE: &str = "/api/v1/admin/auth/mfa/enroll";
pub const ADMIN_AUTH_MFA_CONFIRM_ROUTE: &str = "/api/v1/admin/auth/mfa/confirm";
pub const ADMIN_AUTH_MFA_VERIFY_ROUTE: &str = "/api/v1/admin/auth/mfa/verify";
pub const ADMIN_AUTH_MFA_RECOVERY_CODES_REGENERATE_ROUTE: &str =
    "/api/v1/admin/auth/mfa/recovery-codes/regenerate";
pub const ADMIN_RECOVERY_CODES_NO_STORE_CACHE_CONTROL: &str = "no-store";
pub const ADMIN_RECOVERY_CODE_COUNT: usize = 10;
pub const ADMIN_AUTH_ME_ROUTE: &str = "/api/v1/admin/auth/me";
pub const ADMIN_AUTH_LOGOUT_ROUTE: &str = "/api/v1/admin/auth/logout";
pub const ADMIN_AUTH_PASSWORD_CHANGE_ROUTE: &str = "/api/v1/admin/auth/password/change";
pub const ADMIN_AUTH_PASSWORD_RESET_REQUEST_ROUTE: &str =
    "/api/v1/admin/auth/password-reset/request";
pub const ADMIN_AUTH_PASSWORD_RESET_CONFIRM_ROUTE: &str =
    "/api/v1/admin/auth/password-reset/confirm";
pub const ADMIN_PASSWORD_RESET_PAGE_PATH: &str = "/admin/reset-password";
pub const ADMIN_PASSWORD_RESET_LIFETIME_MINUTES: i64 = 30;
pub const ADMIN_PASSWORD_RESET_EMAIL_SUBJECT: &str = "Reset your Peerfora administrator password";
pub const ADMIN_PASSWORD_RESET_REQUESTED_MESSAGE: &str =
    "If the administrator account exists, password reset instructions have been sent";
pub const ADMIN_AUTH_NO_REFERRER_POLICY: &str = "no-referrer";
pub const ADMIN_AUDIT_LOGS_ROUTE: &str = "/api/v1/admin/audit-logs";
pub const ADMIN_OVERVIEW_ROUTE: &str = "/api/v1/admin/overview";
pub const ADMIN_OVERVIEW_READ_PERMISSION: &str = "admin.overview.read";
pub const ADMIN_SESSIONS_ROUTE: &str = "/api/v1/admin/sessions";
pub const ADMIN_SESSION_BY_ID_ROUTE: &str = "/api/v1/admin/sessions/{id}";
pub const ADMIN_AUDIT_LOGS_READ_PERMISSION: &str = "audit_logs.read";
pub const ADMIN_ACCOUNTS_ROUTE: &str = "/api/v1/admin/accounts";
pub const ADMIN_ACCOUNT_BY_ID_ROUTE: &str = "/api/v1/admin/accounts/{id}";
pub const ADMIN_ACCOUNTS_READ_PERMISSION: &str = "accounts.read";
pub const ADMIN_ACCOUNT_SUSPEND_PERMISSION: &str = "accounts.suspend";
pub const ADMIN_ACCOUNT_SUSPEND_ROUTE: &str = "/api/v1/admin/accounts/{id}/suspend";
pub const ADMIN_ACCOUNT_REINSTATE_ROUTE: &str = "/api/v1/admin/accounts/{id}/reinstate";
pub const ADMIN_ACCOUNT_SESSIONS_REVOKE_ROUTE: &str = "/api/v1/admin/accounts/{id}/sessions/revoke";
pub const ADMIN_ACCOUNT_SESSIONS_REVOKE_PERMISSION: &str = "accounts.sessions.revoke";
pub const ADMIN_ACCOUNT_SESSIONS_REVOKED_AUDIT_ACTION: &str = "account.sessions_revoked";
pub const ADMIN_COMMUNITIES_ROUTE: &str = "/api/v1/admin/communities";
pub const ADMIN_COMMUNITY_BY_ID_ROUTE: &str = "/api/v1/admin/communities/{id}";
pub const ADMIN_COMMUNITY_SUSPEND_ROUTE: &str = "/api/v1/admin/communities/{id}/suspend";
pub const ADMIN_COMMUNITY_RESTORE_ROUTE: &str = "/api/v1/admin/communities/{id}/restore";
pub const ADMIN_COMMUNITIES_READ_PERMISSION: &str = "communities.read";
pub const ADMIN_COMMUNITIES_MANAGE_PERMISSION: &str = "communities.manage";
pub const ADMIN_COMMUNITIES_DEFAULT_PAGE_SIZE: u32 = 50;
pub const ADMIN_COMMUNITIES_MAX_PAGE_SIZE: u32 = 100;
pub const ADMIN_COMMUNITIES_MAX_OFFSET: u32 = 100_000;
pub const ADMIN_ACCOUNTS_DEFAULT_PAGE_SIZE: u32 = 50;
pub const ADMIN_ACCOUNTS_MAX_PAGE_SIZE: u32 = 100;
pub const ADMIN_ACCOUNTS_MAX_OFFSET: u32 = 100_000;
pub const ADMIN_INVITATIONS_ROUTE: &str = "/api/v1/admin/invitations";
pub const ADMIN_INVITATION_ACCEPT_ROUTE: &str = "/api/v1/admin/invitations/accept";
pub const ADMIN_INVITATION_ACCEPT_PAGE_PATH: &str = "/admin/accept-invitation";
pub const ADMIN_INVITATIONS_MANAGE_PERMISSION: &str = "admin.accounts.manage";
pub const ADMIN_INVITATION_LIFETIME_HOURS: i64 = 48;
pub const ADMIN_PASSWORD_MAXIMUM_LENGTH: usize = 128;
pub const ADMIN_INVITATION_EMAIL_SUBJECT: &str = "Invitation to administer Peerfora";

pub const ADMIN_SESSION_COOKIE_NAME: &str = "peerfora_admin_session";
pub const ADMIN_SESSION_COOKIE_PATH: &str = "/api/v1/admin";
pub const ADMIN_SESSION_LIFETIME_HOURS: i64 = 8;
pub const ADMIN_SESSION_IDLE_TIMEOUT_MINUTES: i64 = 20;
pub const ADMIN_CHALLENGE_LIFETIME_MINUTES: i64 = 5;
pub const ADMIN_MFA_FAILED_ATTEMPT_LIMIT: i16 = 5;
pub const ADMIN_MFA_LOCK_MINUTES: i64 = 15;
pub const ADMIN_PASSWORD_MINIMUM_LENGTH: usize = 16;
pub const ADMIN_LOGIN_FAILED_ATTEMPT_LIMIT: i16 = 5;
pub const ADMIN_LOGIN_LOCK_MINUTES: i64 = 15;
pub const SEEDED_ROOT_ADMINISTRATOR_ID: &str = "00000000-0000-0000-0000-000000000001";
pub const SUPER_ADMINISTRATOR_ROLE_KEY: &str = "super_administrator";
pub const ADMIN_ROLES_ROUTE: &str = "/api/v1/admin/roles";
pub const ADMIN_ROLE_BY_ID_ROUTE: &str = "/api/v1/admin/roles/{id}";
pub const ADMIN_PERMISSIONS_ROUTE: &str = "/api/v1/admin/permissions";
pub const ADMIN_ROLE_PERMISSIONS_ROUTE: &str = "/api/v1/admin/roles/{id}/permissions";
pub const ADMIN_ADMINISTRATOR_ROLES_ROUTE: &str = "/api/v1/admin/administrators/{id}/roles";
pub const ADMIN_ADMINISTRATOR_PERMISSIONS_ROUTE: &str =
    "/api/v1/admin/administrators/{id}/permissions";
pub const ADMINISTRATOR_SUSPEND_ROUTE: &str = "/api/v1/admin/administrators/{id}/suspend";
pub const ADMINISTRATOR_REINSTATE_ROUTE: &str = "/api/v1/admin/administrators/{id}/reinstate";
pub const ADMINISTRATOR_DELETE_ROUTE: &str = "/api/v1/admin/administrators/{id}";
pub const ADMINISTRATOR_SESSIONS_REVOKE_ROUTE: &str = "/api/v1/admin/administrators/{id}/sessions";
pub const ADMINISTRATOR_MFA_RESET_ROUTE: &str = "/api/v1/admin/administrators/{id}/mfa/reset";
pub const ADMINISTRATORS_MANAGE_PERMISSION: &str = "admin.accounts.manage";
pub const ADMINISTRATOR_ROLE_REPLACEMENT_REASON: &str =
    "roles replaced by authorized administrator";
pub const DELETED_ADMINISTRATOR_EMAIL_DOMAIN: &str = "invalid.peerfora";
pub const ADMINISTRATOR_SUSPENDED_AUDIT_ACTION: &str = "administrator.suspended";
pub const ADMINISTRATOR_REINSTATED_AUDIT_ACTION: &str = "administrator.reinstated";
pub const ADMINISTRATOR_DELETED_AUDIT_ACTION: &str = "administrator.deleted";
pub const ADMINISTRATOR_SESSIONS_REVOKED_AUDIT_ACTION: &str = "administrator.sessions_revoked";
pub const ADMINISTRATOR_RECOVERY_CODES_REGENERATED_AUDIT_ACTION: &str =
    "administrator.mfa_recovery_codes_regenerated";
pub const ADMINISTRATOR_MFA_VERIFICATION_FAILED_AUDIT_ACTION: &str =
    "administrator.mfa_verification_failed";
pub const ADMINISTRATOR_PASSWORD_CHANGED_AUDIT_ACTION: &str = "administrator.password_changed";
pub const ADMINISTRATOR_PASSWORD_CHANGE_FAILED_AUDIT_ACTION: &str =
    "administrator.password_change_failed";
pub const ADMINISTRATOR_PASSWORD_RESET_COMPLETED_AUDIT_ACTION: &str =
    "administrator.password_reset_completed";
pub const ADMINISTRATOR_MFA_RESET_AUDIT_ACTION: &str = "administrator.mfa_reset";
pub const ADMIN_ROLES_READ_PERMISSION: &str = "admin.roles.read";
pub const ADMIN_ROLES_CREATE_PERMISSION: &str = "admin.roles.create";
pub const ADMIN_ROLES_UPDATE_PERMISSION: &str = "admin.roles.update";
pub const ADMIN_ROLES_DELETE_PERMISSION: &str = "admin.roles.delete";
pub const ADMIN_ROLES_ASSIGN_PERMISSION: &str = "admin.roles.assign";
pub const ADMIN_PERMISSIONS_READ_PERMISSION: &str = "admin.permissions.read";
pub const ADMIN_PERMISSIONS_ASSIGN_PERMISSION: &str = "admin.permissions.assign";
pub const ADMIN_ROLE_MANAGEMENT_ADVISORY_LOCK_ID: i64 = 734125092;
pub const ADMIN_SENSITIVE_OPERATION_RECENT_AUTH_MINUTES: i64 = 15;
