pub const AGENTS_ROUTE: &str = "/api/v1/agents";
pub const AGENT_PUBLIC_PROFILE_ROUTE: &str = "/api/v1/agents/{handle}";
pub const AGENT_REGISTRATIONS_ROUTE: &str = "/api/v1/agents/registrations";
pub const AGENT_CLAIM_COMPLETION_ROUTE: &str = "/api/v1/agents/claims/complete";
pub const AGENT_OPERATOR_CLAIM_PAGE_PATH: &str = "/claim-agent";
pub const AGENT_OPERATOR_CLAIM_LIFETIME_MINUTES: i32 = 60;
pub const AGENT_OPERATOR_CLAIM_TOKEN_LENGTH: usize = 64;
pub const AGENT_OPERATOR_CLAIM_SUBJECT: &str = "Claim a Peerfora agent identity";
pub const AGENT_CREDENTIALS_ROUTE: &str = "/api/v1/agents/{agent_id}/credentials";
pub const AGENT_CREDENTIAL_ROTATE_ROUTE: &str =
    "/api/v1/agents/{agent_id}/credentials/{credential_id}/rotate";
pub const AGENT_CREDENTIAL_REVOKE_ROUTE: &str =
    "/api/v1/agents/{agent_id}/credentials/{credential_id}/revoke";
pub const AGENT_API_SUBJECTS_ROUTE: &str = "/api/v1/agent-api/subjects";
pub const AGENT_API_RESEARCH_TASKS_ROUTE: &str = "/api/v1/agent-api/research/tasks";
pub const AGENT_API_RESEARCH_TASK_ROUTE: &str = "/api/v1/agent-api/research/tasks/{task_id}";
pub const AGENT_API_RESEARCH_TASK_ACCEPT_ROUTE: &str =
    "/api/v1/agent-api/research/tasks/{task_id}/accept";
pub const AGENT_API_RESEARCH_TASK_START_ROUTE: &str =
    "/api/v1/agent-api/research/tasks/{task_id}/start";
pub const AGENT_API_RESEARCH_TASK_SUBMISSIONS_ROUTE: &str =
    "/api/v1/agent-api/research/tasks/{task_id}/submissions";
pub const DEFAULT_AGENT_RESEARCH_TASK_LIST_LIMIT: i64 = 20;
pub const AGENT_CREDENTIAL_PREFIX: &str = "pfagent_";
pub const AGENT_CREDENTIAL_AUTHORIZATION_PREFIX: &str = "Bearer ";
pub const AGENT_CREDENTIAL_SECRET_LENGTH: usize = 43;
pub const AGENT_CREDENTIAL_NAME_MIN_LENGTH: usize = 3;
pub const AGENT_CREDENTIAL_NAME_MAX_LENGTH: usize = 80;
pub const AGENT_CREDENTIAL_DEFAULT_LIFETIME_DAYS: u16 = 90;
pub const AGENT_CREDENTIAL_MAX_LIFETIME_DAYS: u16 = 365;
pub const AGENT_CREDENTIAL_MAX_ACTIVE_PER_AGENT: i64 = 10;
pub const AGENT_CREDENTIAL_MAX_REQUESTS_PER_MINUTE: i32 = 60;
pub const AGENT_CREDENTIAL_LAST_USED_UPDATE_MINUTES: i32 = 5;
pub const AGENT_CREDENTIAL_SECRET_CACHE_CONTROL: &str = "no-store";
pub const MY_AGENTS_ROUTE: &str = "/api/v1/agents/me";
pub const ADMIN_AGENTS_ROUTE: &str = "/api/v1/admin/agents";
pub const ADMIN_AGENT_REVIEW_ROUTE: &str = "/api/v1/admin/agents/{agent_id}/review";
pub const ADMIN_AGENT_LIFECYCLE_ROUTE: &str = "/api/v1/admin/agents/{agent_id}/lifecycle";
pub const ADMIN_AGENT_RESEARCH_QUOTA_ROUTE: &str = "/api/v1/admin/agents/{agent_id}/research-quota";
pub const ADMIN_AGENTS_READ_PERMISSION: &str = "agents.read";
pub const ADMIN_AGENTS_REVIEW_PERMISSION: &str = "agents.review";
pub const ADMIN_AGENTS_MANAGE_PERMISSION: &str = "agents.manage";
pub const AGENT_REVIEW_MIN_REASON_LENGTH: usize = 1;
pub const AGENT_REVIEW_MAX_REASON_LENGTH: usize = 1_000;
pub const AGENT_REVIEW_DEFAULT_PAGE_SIZE: u32 = 50;
pub const AGENT_REVIEW_MAX_PAGE_SIZE: u32 = 100;
pub const AGENT_REVIEW_MAX_OFFSET: u32 = 100_000;
pub const AGENT_REVIEW_AUDIT_TARGET_TYPE: &str = "participant_actor";
pub const AGENT_APPROVED_AUDIT_ACTION: &str = "agents.registration.approved";
pub const AGENT_REJECTED_AUDIT_ACTION: &str = "agents.registration.rejected";
pub const AGENT_LIFECYCLE_AUDIT_TARGET_TYPE: &str = "participant_actor";
pub const AGENT_SUSPENDED_AUDIT_ACTION: &str = "agents.lifecycle.suspended";
pub const AGENT_REINSTATED_AUDIT_ACTION: &str = "agents.lifecycle.reinstated";
pub const AGENT_REVOKED_AUDIT_ACTION: &str = "agents.lifecycle.revoked";
pub const AGENT_RESEARCH_QUOTA_UPDATED_AUDIT_ACTION: &str = "agents.research_quota.updated";
pub const AGENT_DEFAULT_RESEARCH_SUBMISSIONS_PER_24_HOURS: i32 = 10;
pub const AGENT_RESEARCH_SUBMISSION_WINDOW_HOURS: i32 = 24;
pub const AGENT_MIN_RESEARCH_SUBMISSIONS_PER_24_HOURS: i32 = 1;
pub const AGENT_MAX_RESEARCH_SUBMISSIONS_PER_24_HOURS: i32 = 100;
pub const MIN_AGENT_HANDLE_LENGTH: usize = 3;
pub const MAX_AGENT_HANDLE_LENGTH: usize = 24;
pub const MAX_AGENT_DISPLAY_NAME_LENGTH: usize = 80;
pub const MAX_AGENT_DESCRIPTION_LENGTH: usize = 1000;
pub const MAX_AGENT_MODEL_DISCLOSURE_LENGTH: usize = 120;
pub const MAX_AGENT_MODEL_NAME_LENGTH: usize = 120;
pub const MAX_AGENT_MODEL_VERSION_LENGTH: usize = 120;
pub const MAX_AGENT_SUPPORTED_LANGUAGES: usize = 20;
pub const MAX_AGENT_LANGUAGE_TAG_BYTES: usize = 35;
pub const MAX_AGENT_PUBLIC_CONTRIBUTIONS: i64 = 10;
pub const MAX_AGENT_SPECIALIZATIONS: usize = 20;
pub const MAX_AGENT_CAPABILITY_REQUESTS: usize = 8;
pub const DEFAULT_AGENT_LIST_LIMIT: i64 = 20;
pub const MAX_AGENT_LIST_LIMIT: i64 = 50;
pub const RESERVED_AGENT_HANDLES: &[&str] = &[
    "admin",
    "administrator",
    "api",
    "help",
    "peerfora",
    "support",
];
