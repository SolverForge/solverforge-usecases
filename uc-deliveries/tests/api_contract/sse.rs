use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tokio_stream::StreamExt;
use tower::ServiceExt;

use crate::support::api::{empty_road_network_plan, read_json, small_road_network_plan, test_app};

#[tokio::test]
async fn sse_endpoint_bootstraps_typed_events() {
    let app = test_app();

    let create_response = app
        .clone()
        .oneshot(
            Request::post("/jobs")
                .header("content-type", "application/json")
                .body(Body::from(empty_road_network_plan().to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);
    let created = read_json(create_response).await;
    let job_id = created["id"].as_str().expect("job id should be present");

    let events_response = app
        .clone()
        .oneshot(
            Request::get(format!("/jobs/{job_id}/events"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(events_response.status(), StatusCode::OK);
    let mut stream = events_response.into_body().into_data_stream();
    let first_chunk = stream
        .next()
        .await
        .expect("sse stream should yield a bootstrap chunk")
        .expect("bootstrap chunk should be valid");
    let text = String::from_utf8(first_chunk.to_vec()).expect("chunk should be UTF-8");
    assert!(text.contains("\"eventType\""));
    assert!(text.contains("\"lifecycleState\""));

    let _ = app
        .clone()
        .oneshot(
            Request::post(format!("/jobs/{job_id}/cancel"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
}

#[tokio::test]
async fn road_network_job_routes_work_when_live_tests_are_enabled() {
    if std::env::var("SOLVERFORGE_RUN_LIVE_TESTS").ok().as_deref() != Some("1") {
        return;
    }

    let app = test_app();
    let create_response = app
        .clone()
        .oneshot(
            Request::post("/jobs")
                .header("content-type", "application/json")
                .body(Body::from(small_road_network_plan().to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);
    let created = read_json(create_response).await;
    let job_id = created["id"].as_str().unwrap();

    let snapshot_response = loop {
        let response = app
            .clone()
            .oneshot(
                Request::get(format!("/jobs/{job_id}/snapshot"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        if response.status() == StatusCode::OK {
            break response;
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
    };
    let snapshot = read_json(snapshot_response).await;
    let revision = snapshot["snapshotRevision"].as_u64().unwrap();

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
}
