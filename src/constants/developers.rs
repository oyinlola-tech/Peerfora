pub const DEVELOPER_PROFILE_ROUTE: &str = "/api/v1/developers/me";
pub const DEVELOPER_APPLICATIONS_ROUTE: &str = "/api/v1/developers/applications";
pub const DEVELOPER_APPLICATION_SUBMIT_ROUTE: &str =
    "/api/v1/developers/applications/{application_id}/submit";
pub const ADMIN_DEVELOPER_APPLICATIONS_ROUTE: &str = "/api/v1/admin/developers/applications";
pub const ADMIN_DEVELOPER_APPLICATION_REVIEW_ROUTE: &str =
    "/api/v1/admin/developers/applications/{application_id}/review";
pub const ADMIN_DEVELOPER_APPLICATIONS_READ_PERMISSION: &str = "developers.applications.read";
pub const ADMIN_DEVELOPER_APPLICATIONS_REVIEW_PERMISSION: &str = "developers.applications.review";
pub const DEVELOPER_APPLICATION_DEFAULT_PAGE_SIZE: u32 = 50;
pub const DEVELOPER_APPLICATION_MAX_PAGE_SIZE: u32 = 100;
pub const DEVELOPER_APPLICATION_MAX_OFFSET: u32 = 100_000;
pub const DEVELOPER_PROFILE_MIN_NAME_LENGTH: usize = 3;
pub const DEVELOPER_PROFILE_MAX_NAME_LENGTH: usize = 80;
pub const DEVELOPER_PROFILE_MAX_DESCRIPTION_LENGTH: usize = 1_000;
pub const DEVELOPER_APPLICATION_MIN_NAME_LENGTH: usize = 3;
pub const DEVELOPER_APPLICATION_MAX_NAME_LENGTH: usize = 100;
pub const DEVELOPER_APPLICATION_MAX_DESCRIPTION_LENGTH: usize = 2_000;
pub const DEVELOPER_APPLICATION_MAX_SCOPES: usize = 8;
pub const DEVELOPER_APPLICATION_MIN_REVIEW_REASON_LENGTH: usize = 1;
pub const DEVELOPER_APPLICATION_MAX_REVIEW_REASON_LENGTH: usize = 1_000;
pub const DEVELOPER_APPLICATION_AUDIT_TARGET_TYPE: &str = "developer_application";
pub const DEVELOPER_APPLICATION_APPROVED_AUDIT_ACTION: &str = "developers.application.approved";
pub const DEVELOPER_APPLICATION_REJECTED_AUDIT_ACTION: &str = "developers.application.rejected";

pub const DEVELOPER_APPLICATION_CREDENTIALS_ROUTE: &str =
    "/api/v1/developers/applications/{application_id}/credentials";
pub const DEVELOPER_APPLICATION_CREDENTIAL_REVOKE_ROUTE: &str =
    "/api/v1/developers/applications/{application_id}/credentials/{credential_id}/revoke";
pub const DEVELOPER_APPLICATION_CREDENTIAL_ROTATE_ROUTE: &str =
    "/api/v1/developers/applications/{application_id}/credentials/{credential_id}/rotate";
pub const ADMIN_DEVELOPER_APPLICATION_SUSPEND_ROUTE: &str =
    "/api/v1/admin/developers/applications/{application_id}/suspend";
pub const ADMIN_DEVELOPER_CREDENTIAL_REVOKE_ROUTE: &str =
    "/api/v1/admin/developers/applications/{application_id}/credentials/{credential_id}/revoke";
pub const DEVELOPER_API_SUBJECTS_ROUTE: &str = "/api/v1/developer-api/subjects";
pub const DEVELOPER_CREDENTIAL_PREFIX: &str = "pfdev_";
pub const DEVELOPER_CREDENTIAL_AUTHORIZATION_PREFIX: &str = "Bearer ";
pub const DEVELOPER_CREDENTIAL_NAME_MIN_LENGTH: usize = 3;
pub const DEVELOPER_CREDENTIAL_NAME_MAX_LENGTH: usize = 80;
pub const DEVELOPER_CREDENTIAL_DEFAULT_LIFETIME_DAYS: u16 = 90;
pub const DEVELOPER_CREDENTIAL_MAX_LIFETIME_DAYS: u16 = 365;
pub const DEVELOPER_CREDENTIAL_MAX_ACTIVE_PER_APPLICATION: i64 = 10;
pub const DEVELOPER_CREDENTIAL_MAX_REQUESTS_PER_MINUTE: i32 = 60;
pub const DEVELOPER_CREDENTIAL_LAST_USED_UPDATE_MINUTES: i64 = 5;
pub const DEVELOPER_APPLICATION_SUSPEND_PERMISSION: &str = "developers.applications.suspend";
pub const DEVELOPER_CREDENTIAL_REVOKE_PERMISSION: &str = "developers.credentials.revoke";
pub const DEVELOPER_APPLICATION_SUSPENDED_AUDIT_ACTION: &str = "developers.application.suspended";
pub const DEVELOPER_CREDENTIAL_AUDIT_TARGET_TYPE: &str = "developer_application_credential";
pub const DEVELOPER_CREDENTIAL_REVOKED_AUDIT_ACTION: &str = "developers.credential.revoked";
pub const DEVELOPER_CREDENTIAL_SECRET_CACHE_CONTROL: &str = "no-store";
pub const DEVELOPER_CREDENTIAL_SECRET_LENGTH: usize = 43;
pub const DEVELOPER_SCOPE_ACADEMIC_DIRECTORY_READ: &str = "academic.directory.read";
