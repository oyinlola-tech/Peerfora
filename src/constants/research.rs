pub const RESEARCH_PROJECTS_ROUTE: &str = "/api/v1/research/projects";
pub const RESEARCH_PROJECT_ROUTE: &str = "/api/v1/research/projects/{project_id}";
pub const RESEARCH_PROJECT_INVITATIONS_ROUTE: &str =
    "/api/v1/research/projects/{project_id}/invitations";
pub const RESEARCH_INVITATIONS_ROUTE: &str = "/api/v1/research/invitations";
pub const RESEARCH_INVITATION_ACCEPT_ROUTE: &str =
    "/api/v1/research/projects/{project_id}/invitations/accept";
pub const RESEARCH_INVITATION_REJECT_ROUTE: &str =
    "/api/v1/research/projects/{project_id}/invitations/reject";
pub const RESEARCH_PROJECT_MEMBERSHIP_ROUTE: &str =
    "/api/v1/research/projects/{project_id}/membership";
pub const RESEARCH_PROJECT_MEMBER_ROUTE: &str =
    "/api/v1/research/projects/{project_id}/members/{username}";

pub const MIN_RESEARCH_PROJECT_TITLE_LENGTH: usize = 3;
pub const MAX_RESEARCH_PROJECT_TITLE_LENGTH: usize = 160;
pub const MAX_RESEARCH_PROJECT_ABSTRACT_LENGTH: usize = 2000;
pub const MAX_RESEARCH_PROJECT_DESCRIPTION_LENGTH: usize = 10000;
pub const MAX_RESEARCH_PROJECT_OBJECTIVES: usize = 20;
pub const MAX_RESEARCH_PROJECT_QUESTIONS: usize = 20;
pub const MAX_RESEARCH_PROJECT_SUBJECTS: usize = 20;
pub const MAX_RESEARCH_PROJECT_ITEM_LENGTH: usize = 500;
pub const DEFAULT_RESEARCH_PROJECT_LIST_LIMIT: i64 = 20;
pub const MAX_RESEARCH_PROJECT_LIST_LIMIT: i64 = 50;

pub const RESEARCH_TASKS_ROUTE: &str = "/api/v1/research/projects/{project_id}/tasks";
pub const RESEARCH_TASK_ROUTE: &str = "/api/v1/research/tasks/{task_id}";
pub const RESEARCH_TASK_QUEUE_ROUTE: &str = "/api/v1/research/tasks/{task_id}/queue";
pub const RESEARCH_TASK_ASSIGN_ROUTE: &str = "/api/v1/research/tasks/{task_id}/assign";
pub const RESEARCH_TASK_ACCEPT_ROUTE: &str = "/api/v1/research/tasks/{task_id}/accept";
pub const RESEARCH_TASK_START_ROUTE: &str = "/api/v1/research/tasks/{task_id}/start";
pub const RESEARCH_TASK_SUBMISSIONS_ROUTE: &str = "/api/v1/research/tasks/{task_id}/submissions";
pub const RESEARCH_TASK_REVIEW_ROUTE: &str = "/api/v1/research/tasks/{task_id}/review";
pub const RESEARCH_TASK_CANCEL_ROUTE: &str = "/api/v1/research/tasks/{task_id}/cancel";
pub const DEFAULT_RESEARCH_TASK_LIST_LIMIT: i64 = 20;
pub const MAX_RESEARCH_TASK_LIST_LIMIT: i64 = 50;
pub const DEFAULT_RESEARCH_TASK_RUNTIME_SECONDS: i32 = 1800;
pub const DEFAULT_RESEARCH_TASK_RESULT_BYTES: i32 = 200_000;
pub const MIN_RESEARCH_TASK_TITLE_LENGTH: usize = 3;
pub const MAX_RESEARCH_TASK_TITLE_LENGTH: usize = 160;
pub const MAX_RESEARCH_TASK_OBJECTIVE_LENGTH: usize = 3000;
pub const MAX_RESEARCH_TASK_SCOPE_LENGTH: usize = 5000;
pub const MAX_RESEARCH_TASK_RESULT_BYTES: usize = 1_000_000;
pub const MAX_RESEARCH_TASK_RUNTIME_SECONDS: i32 = 86_400;
pub const MAX_RESEARCH_TASK_ITEMS: usize = 20;
pub const MAX_RESEARCH_TASK_ITEM_LENGTH: usize = 500;
pub const MAX_RESEARCH_TASK_TOOL_LENGTH: usize = 100;
pub const MAX_RESEARCH_TASK_REVIEW_NOTES_BYTES: usize = 5000;
pub const MAX_RESEARCH_TASK_SUBMISSIONS: i16 = 100;

pub const RESEARCH_PROJECT_SOURCES_ROUTE: &str = "/api/v1/research/projects/{project_id}/sources";
pub const RESEARCH_PROJECT_FINDINGS_ROUTE: &str = "/api/v1/research/projects/{project_id}/findings";
pub const RESEARCH_FINDING_ROUTE: &str = "/api/v1/research/findings/{finding_id}";
pub const RESEARCH_FINDING_SUBMIT_ROUTE: &str = "/api/v1/research/findings/{finding_id}/submit";
pub const RESEARCH_FINDING_REVIEW_ROUTE: &str = "/api/v1/research/findings/{finding_id}/review";
pub const RESEARCH_FINDING_PUBLISH_ROUTE: &str = "/api/v1/research/findings/{finding_id}/publish";
pub const RESEARCH_FINDING_RETRACT_ROUTE: &str = "/api/v1/research/findings/{finding_id}/retract";
pub const DEFAULT_RESEARCH_FINDING_LIST_LIMIT: i64 = 20;
pub const MAX_RESEARCH_FINDING_LIST_LIMIT: i64 = 50;
pub const MAX_RESEARCH_SOURCE_TITLE_LENGTH: usize = 500;
pub const MAX_RESEARCH_SOURCE_DOI_LENGTH: usize = 255;
pub const MAX_RESEARCH_SOURCE_URL_LENGTH: usize = 2048;
pub const MAX_RESEARCH_SOURCE_PROVENANCE_LENGTH: usize = 2000;
pub const MAX_RESEARCH_SOURCE_LICENSE_LENGTH: usize = 500;
pub const MAX_RESEARCH_SOURCE_AUTHORS: usize = 50;
pub const MAX_RESEARCH_FINDING_TITLE_LENGTH: usize = 200;
pub const MAX_RESEARCH_FINDING_STATEMENT_LENGTH: usize = 10_000;
pub const MAX_RESEARCH_FINDING_DETAILS_LENGTH: usize = 20_000;
pub const MAX_RESEARCH_FINDING_REVISIONS: i16 = 100;
pub const MAX_RESEARCH_FINDING_CITATIONS: usize = 50;
pub const MAX_RESEARCH_CITATION_LOCATOR_LENGTH: usize = 500;
pub const MAX_RESEARCH_CITATION_CONTEXT_LENGTH: usize = 2_000;
pub const MAX_RESEARCH_FINDING_REVIEW_NOTES_LENGTH: usize = 5_000;
