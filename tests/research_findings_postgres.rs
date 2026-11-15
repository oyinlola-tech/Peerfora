#[path = "research_services_postgres/mod.rs"]
mod research_services_postgres;

use axum::http::StatusCode;
use peerfora_api::{app::build_app, database::connect_database};
use research_services_postgres::{
    create_account_fixture, database_url_from_environment, test_app_state,
};
use serde_json::json;

#[path = "research_findings_postgres/support.rs"]
mod support;

use support::*;

#[tokio::test]
async fn research_findings_preserve_reviews_citations_and_public_corrections() {
    let Ok(database_url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let database = connect_database(&database_url)
        .await
        .expect("the disposable PostgreSQL database should migrate");
    let owner = create_account_fixture(&database, "find_owner").await;
    let reviewer = create_account_fixture(&database, "find_review").await;
    let outsider = create_account_fixture(&database, "find_outside").await;
    let app = build_app(test_app_state(
        database.clone(),
        database_url_from_environment(),
    ));
    let project_id = create_public_project(&app, &owner.session_token).await;
    invite_reviewer(&app, project_id, &owner.session_token, &reviewer.username).await;
    accept_project_invitation(&app, project_id, &reviewer.session_token).await;

    let invalid_source = send_json_request(
        &app,
        "POST",
        &project_sources_path(project_id),
        &owner.session_token,
        json!({
            "sourceType": "external_url",
            "title": "Unsafe source link",
            "sourceUrl": "http://example.test/source",
            "provenance": "Submitted by a project member"
        }),
    )
    .await;
    assert_eq!(invalid_source.status(), StatusCode::BAD_REQUEST);

    let source_id = create_doi_source(&app, project_id, &owner.session_token).await;
    let duplicate_source = send_json_request(
        &app,
        "POST",
        &project_sources_path(project_id),
        &owner.session_token,
        doi_source_body(),
    )
    .await;
    assert_eq!(duplicate_source.status(), StatusCode::CONFLICT);

    let finding_id = create_finding(&app, project_id, &owner.session_token, source_id).await;
    assert_finding_hidden(&app, finding_id, Some(&outsider.session_token)).await;
    assert_finding_hidden(&app, finding_id, None).await;
    assert_eq!(
        send_request(
            &app,
            "GET",
            &project_findings_path(project_id),
            Some(&outsider.session_token)
        )
        .await
        .status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        send_request(
            &app,
            "GET",
            &format!("{}/revisions", finding_path(finding_id)),
            Some(&outsider.session_token)
        )
        .await
        .status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        list_revisions(&app, finding_id, &owner.session_token)
            .await
            .len(),
        1
    );

    assert_eq!(
        submit_finding(&app, finding_id, &owner.session_token).await,
        StatusCode::NO_CONTENT
    );
    let self_review = review_finding(&app, finding_id, &owner.session_token).await;
    assert_eq!(self_review.status(), StatusCode::FORBIDDEN);
    let reviewer_publish = publish_finding(&app, finding_id, &reviewer.session_token).await;
    assert_eq!(reviewer_publish.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        review_finding(&app, finding_id, &reviewer.session_token)
            .await
            .status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        publish_finding(&app, finding_id, &owner.session_token)
            .await
            .status(),
        StatusCode::NO_CONTENT
    );

    let public_finding = find_finding(&app, finding_id, None).await;
    assert_eq!(public_finding.status(), StatusCode::OK);
    let published = response_json(public_finding).await;
    assert_eq!(published["status"], "published");
    assert_eq!(
        published["statement"],
        "The tested intervention improved access."
    );
    assert_eq!(published["citations"][0]["sourceId"], source_id.to_string());

    let correction = send_json_request(
        &app,
        "PUT",
        &finding_path(finding_id),
        &owner.session_token,
        finding_body(source_id, "The corrected intervention improved access."),
    )
    .await;
    assert_eq!(correction.status(), StatusCode::OK);
    assert_eq!(
        response_json(correction).await["status"],
        "correction_draft"
    );
    let still_published = response_json(find_finding(&app, finding_id, None).await).await;
    assert_eq!(still_published["revisionNumber"], 1);
    assert_eq!(
        still_published["statement"],
        "The tested intervention improved access."
    );

    assert_eq!(
        submit_finding(&app, finding_id, &owner.session_token).await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        review_finding(&app, finding_id, &reviewer.session_token)
            .await
            .status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        publish_finding(&app, finding_id, &owner.session_token)
            .await
            .status(),
        StatusCode::NO_CONTENT
    );
    let corrected = response_json(find_finding(&app, finding_id, None).await).await;
    assert_eq!(corrected["revisionNumber"], 2);
    assert_eq!(
        corrected["statement"],
        "The corrected intervention improved access."
    );
    let revisions = list_revisions(&app, finding_id, &owner.session_token).await;
    assert_eq!(revisions.len(), 2);
    assert_eq!(revisions[0]["reviewStatus"], "published");
    assert_eq!(revisions[1]["reviewStatus"], "superseded");

    assert_eq!(
        retract_finding(&app, finding_id, &owner.session_token)
            .await
            .status(),
        StatusCode::NO_CONTENT
    );
    assert_finding_hidden(&app, finding_id, None).await;
}
