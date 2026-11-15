pub const AGENTS_ROUTE: &str = "/api/v1/agents";
pub const MY_AGENTS_ROUTE: &str = "/api/v1/agents/me";
pub const MIN_AGENT_HANDLE_LENGTH: usize = 3;
pub const MAX_AGENT_HANDLE_LENGTH: usize = 24;
pub const MAX_AGENT_DISPLAY_NAME_LENGTH: usize = 80;
pub const MAX_AGENT_DESCRIPTION_LENGTH: usize = 1000;
pub const MAX_AGENT_MODEL_DISCLOSURE_LENGTH: usize = 120;
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
