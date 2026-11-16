use axum::http::StatusCode;
use serde_json::{Value, json};

use super::{
    send_agent_request,
    support::{response_json, send_json_request},
};

pub async fn verify_agent_submission_quota_is_enforced(
    app: &axum::Router,
    project_id: &str,
    agent_id: &str,
    credential: &str,
    operator_session: &str,
) {
    let mut task_ids = Vec::new();
    for task_number in 1..=2 {
        let task = send_json_request(
            app,
            "POST",
            &format!("/api/v1/research/projects/{project_id}/tasks"),
            operator_session,
            json!({
                "title":format!("Concurrent quota task {task_number}"),
                "objective":"Verify the agent submission limit across tasks.",
                "scope":"Submit only a synthetic fixture result.",
                "expectedDeliverables":["A short summary"]
            }),
        )
        .await;
        assert_eq!(task.status(), StatusCode::CREATED);
        let task: Value = response_json(task).await;
        let task_id = task["id"].as_str().expect("quota task ID").to_owned();
        for action in ["queue", "assign-agent"] {
            let body = if action == "assign-agent" {
                json!({"agentId":agent_id})
            } else {
                json!({})
            };
            assert_eq!(
                send_json_request(
                    app,
                    "POST",
                    &format!("/api/v1/research/tasks/{task_id}/{action}"),
                    operator_session,
                    body,
                )
                .await
                .status(),
                StatusCode::NO_CONTENT
            );
        }
        for action in ["accept", "start"] {
            assert_eq!(
                send_agent_request(
                    app,
                    "POST",
                    &format!("/api/v1/agent-api/research/tasks/{task_id}/{action}"),
                    credential,
                    None,
                )
                .await
                .status(),
                StatusCode::NO_CONTENT
            );
        }
        task_ids.push(task_id);
    }

    let first_task_path = format!(
        "/api/v1/agent-api/research/tasks/{}/submissions",
        task_ids[0]
    );
    let second_task_path = format!(
        "/api/v1/agent-api/research/tasks/{}/submissions",
        task_ids[1]
    );
    let first_submission = async {
        send_agent_request(
            app,
            "POST",
            &first_task_path,
            credential,
            Some(json!({
                "result":"A concurrent quota fixture submission.",
                "sources":[{"title":"Quota fixture source", "url":"https://research.example.org/quota-fixture"}]
            })),
        )
        .await
    };
    let second_submission = async {
        send_agent_request(
            app,
            "POST",
            &second_task_path,
            credential,
            Some(json!({
                "result":"A concurrent quota fixture submission.",
                "sources":[{"title":"Quota fixture source", "url":"https://research.example.org/quota-fixture"}]
            })),
        )
        .await
    };
    let (first_submission, second_submission) = tokio::join!(first_submission, second_submission);
    let statuses = [first_submission.status(), second_submission.status()];
    assert!(
        statuses.contains(&StatusCode::CREATED)
            && statuses.contains(&StatusCode::TOO_MANY_REQUESTS),
        "only one concurrent submission may consume the remaining quota: {statuses:?}"
    );
}
