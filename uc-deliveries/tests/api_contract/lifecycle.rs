use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use crate::support::api::{bigger_plan, poll_job_state, read_json, test_app};

#[tokio::test]
async fn jobs_lifecycle_snapshot_analysis_routes_and_delete_work() {
    let app = test_app();

    let create_response = app
        .clone()
        .oneshot(
            Request::post("/jobs")
                .header("content-type", "application/json")
                .body(Body::from(bigger_plan().to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);
    let created = read_json(create_response).await;
    let job_id = created["id"].as_str().expect("job id should be a string");

    let status_response = app
        .clone()
        .oneshot(
            Request::get(format!("/jobs/{job_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(status_response.status(), StatusCode::OK);

    let pause_response = app
        .clone()
        .oneshot(
            Request::post(format!("/jobs/{job_id}/pause"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(pause_response.status(), StatusCode::ACCEPTED);
    let paused = poll_job_state(&app, job_id, "PAUSED").await;
    assert!(paused["snapshotRevision"].is_number() || paused["snapshotRevision"].is_null());

    let resume_response = app
        .clone()
        .oneshot(
            Request::post(format!("/jobs/{job_id}/resume"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resume_response.status(), StatusCode::ACCEPTED);

    let cancel_response = app
        .clone()
        .oneshot(
            Request::post(format!("/jobs/{job_id}/cancel"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cancel_response.status(), StatusCode::ACCEPTED);

    let terminal = loop {
        let json = poll_job_state(&app, job_id, "CANCELLED").await;
        if json["snapshotRevision"].is_number() {
            break json;
        }
    };
    let snapshot_revision = terminal["snapshotRevision"]
        .as_u64()
        .expect("snapshot revision should exist");

    assert_snapshot_analysis_and_routes(&app, job_id, snapshot_revision).await;
    let delete_response = app
        .clone()
        .oneshot(
            Request::delete(format!("/jobs/{job_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);
}

async fn assert_snapshot_analysis_and_routes(app: &axum::Router, job_id: &str, revision: u64) {
    let snapshot_response = app
        .clone()
        .oneshot(
            Request::get(format!(
                "/jobs/{job_id}/snapshot?snapshot_revision={revision}"
            ))
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(snapshot_response.status(), StatusCode::OK);

    let analysis_response = app
        .clone()
        .oneshot(
            Request::get(format!(
                "/jobs/{job_id}/analysis?snapshot_revision={revision}"
            ))
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(analysis_response.status(), StatusCode::OK);

    let routes_response = app
        .clone()
        .oneshot(
            Request::get(format!(
                "/jobs/{job_id}/routes?snapshot_revision={revision}"
            ))
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(routes_response.status(), StatusCode::OK);
    let routes = read_json(routes_response).await;
    assert!(routes["vehicles"]
        .as_array()
        .expect("routes vehicles should be an array")
        .iter()
        .all(|vehicle| vehicle["segments"].is_array()));
}
