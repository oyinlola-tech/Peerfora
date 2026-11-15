use axum::{Router, routing::get};

use crate::{
    constants::research::{
        RESEARCH_INVITATION_ACCEPT_ROUTE, RESEARCH_INVITATION_REJECT_ROUTE,
        RESEARCH_INVITATIONS_ROUTE, RESEARCH_PROJECT_INVITATIONS_ROUTE,
        RESEARCH_PROJECT_MEMBER_ROUTE, RESEARCH_PROJECT_MEMBERSHIP_ROUTE, RESEARCH_PROJECT_ROUTE,
        RESEARCH_PROJECTS_ROUTE, RESEARCH_TASK_ACCEPT_ROUTE, RESEARCH_TASK_ASSIGN_ROUTE,
        RESEARCH_TASK_CANCEL_ROUTE, RESEARCH_TASK_QUEUE_ROUTE, RESEARCH_TASK_REVIEW_ROUTE,
        RESEARCH_TASK_ROUTE, RESEARCH_TASK_START_ROUTE, RESEARCH_TASK_SUBMISSIONS_ROUTE,
        RESEARCH_TASKS_ROUTE,
    },
    controllers::{research_project_controller, research_task_controller},
    models::app_state::AppState,
};

pub fn build_routes() -> Router<AppState> {
    Router::new()
        .route(
            RESEARCH_PROJECTS_ROUTE,
            get(research_project_controller::list_research_projects)
                .post(research_project_controller::create_research_project),
        )
        .route(
            RESEARCH_PROJECT_ROUTE,
            get(research_project_controller::find_research_project)
                .patch(research_project_controller::update_research_project)
                .delete(research_project_controller::archive_research_project),
        )
        .route(
            RESEARCH_PROJECT_INVITATIONS_ROUTE,
            axum::routing::post(research_project_controller::invite_research_project_member),
        )
        .route(
            RESEARCH_INVITATION_ACCEPT_ROUTE,
            axum::routing::post(research_project_controller::accept_research_project_invitation),
        )
        .route(
            RESEARCH_INVITATION_REJECT_ROUTE,
            axum::routing::post(research_project_controller::reject_research_project_invitation),
        )
        .route(
            RESEARCH_INVITATIONS_ROUTE,
            get(research_project_controller::list_my_research_invitations),
        )
        .route(
            RESEARCH_PROJECT_MEMBERSHIP_ROUTE,
            axum::routing::delete(research_project_controller::leave_research_project),
        )
        .route(
            RESEARCH_PROJECT_MEMBER_ROUTE,
            axum::routing::delete(research_project_controller::remove_research_project_member),
        )
        .route(
            RESEARCH_TASKS_ROUTE,
            get(research_task_controller::list_research_tasks)
                .post(research_task_controller::create_research_task),
        )
        .route(
            RESEARCH_TASK_ROUTE,
            get(research_task_controller::find_research_task)
                .put(research_task_controller::update_research_task),
        )
        .route(
            RESEARCH_TASK_QUEUE_ROUTE,
            axum::routing::post(research_task_controller::queue_research_task),
        )
        .route(
            RESEARCH_TASK_ASSIGN_ROUTE,
            axum::routing::post(research_task_controller::assign_research_task),
        )
        .route(
            RESEARCH_TASK_ACCEPT_ROUTE,
            axum::routing::post(research_task_controller::accept_research_task_assignment),
        )
        .route(
            RESEARCH_TASK_START_ROUTE,
            axum::routing::post(research_task_controller::start_research_task),
        )
        .route(
            RESEARCH_TASK_SUBMISSIONS_ROUTE,
            axum::routing::post(research_task_controller::submit_research_task_result),
        )
        .route(
            RESEARCH_TASK_REVIEW_ROUTE,
            axum::routing::post(research_task_controller::review_research_task_result),
        )
        .route(
            RESEARCH_TASK_CANCEL_ROUTE,
            axum::routing::post(research_task_controller::cancel_research_task),
        )
}
