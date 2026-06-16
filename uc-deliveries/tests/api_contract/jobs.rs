use std::time::{Duration, Instant};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use crate::support::api::{
    assert_score_text, assert_score_value, completion_plan, empty_road_network_plan,
    poll_job_state, read_body_text, read_json, test_app,
};

#[tokio::test]
async fn road_network_job_emits_a_non_empty_snapshot_when_live_tests_are_enabled() {
    if std::env::var("SOLVERFORGE_RUN_LIVE_TESTS").ok().as_deref() != Some("1") {
        return;
    }

    let app = test_app();
    let create_response = app
        .clone()
        .oneshot(
            Request::post("/jobs")
                .header("content-type", "application/json")
                .body(Body::from(completion_plan().to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);
    let created = read_json(create_response).await;
    let job_id = created["id"].as_str().expect("job id should be a string");

    let snapshot = poll_for_routed_snapshot(&app, job_id).await;
    let snapshot_revision = snapshot["snapshotRevision"]
        .as_u64()
        .expect("snapshot revision should exist");
    let solution_score = assert_score_value(&snapshot["solution"]["score"], "solution.score");
    let current_score = snapshot["currentScore"].as_str();
    let best_score = snapshot["bestScore"].as_str();
    if let Some(score) = current_score {
        assert_score_text(score, "currentScore");
    }
    if let Some(score) = best_score {
        assert_score_text(score, "bestScore");
    }
    assert!(
        current_score == Some(solution_score) || best_score == Some(solution_score),
        "solution.score should match a displayed snapshot score"
    );

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
    let status = read_json(status_response).await;
    assert!(
        status["telemetry"]["stepCount"].as_u64().unwrap_or(0) > 0,
        "solver should have taken at least one step before cancellation"
    );

    assert_routes_have_segments(&app, job_id, snapshot_revision).await;
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
    let _ = poll_job_state(&app, job_id, "CANCELLED").await;
}

#[tokio::test]
async fn empty_road_network_job_routes_return_empty_geometry() {
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
    let job_id = created["id"].as_str().expect("job id should be a string");

    let snapshot = poll_for_any_snapshot(&app, job_id).await;
    let snapshot_revision = snapshot["snapshotRevision"]
        .as_u64()
        .expect("snapshot revision should exist");

    let routes_response = app
        .clone()
        .oneshot(
            Request::get(format!(
                "/jobs/{job_id}/routes?snapshot_revision={snapshot_revision}"
            ))
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(routes_response.status(), StatusCode::OK);
    let routes_body = read_body_text(routes_response).await;
    assert_eq!(
        routes_body.matches("\"snapshotRevision\"").count(),
        1,
        "routes response should expose snapshotRevision only once: {routes_body}"
    );
    let routes: serde_json::Value =
        serde_json::from_str(&routes_body).expect("routes body should be valid JSON");
    assert_eq!(routes["snapshotRevision"].as_u64(), Some(snapshot_revision));
    assert_eq!(routes["routingMode"], "road_network");
    assert!(routes["bounds"].is_null());
    assert_eq!(
        routes["vehicles"]
            .as_array()
            .expect("routes vehicles should be an array")
            .len(),
        0
    );
}

async fn poll_for_routed_snapshot(app: &axum::Router, job_id: &str) -> serde_json::Value {
    let start = Instant::now();
    loop {
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
            let json = read_json(response).await;
            let has_assigned_stops = json["solution"]["viewState"]["preview"]["vehicles"]
                .as_array()
                .expect("preview vehicles should be an array")
                .iter()
                .any(|vehicle| vehicle["stopCount"].as_u64().unwrap_or(0) > 0);
            if has_assigned_stops {
                return json;
            }
        }
        if start.elapsed() > Duration::from_secs(6) {
            panic!("job {job_id} did not emit a routed snapshot in time");
        }
        tokio::time::sleep(Duration::from_millis(80)).await;
    }
}

async fn poll_for_any_snapshot(app: &axum::Router, job_id: &str) -> serde_json::Value {
    let start = Instant::now();
    loop {
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
            return read_json(response).await;
        }
        if start.elapsed() > Duration::from_secs(6) {
            panic!("empty job {job_id} did not emit a snapshot in time");
        }
        tokio::time::sleep(Duration::from_millis(80)).await;
    }
}

async fn assert_routes_have_segments(app: &axum::Router, job_id: &str, revision: u64) {
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
    let routes_body = read_body_text(routes_response).await;
    assert_eq!(
        routes_body.matches("\"snapshotRevision\"").count(),
        1,
        "routes response should expose snapshotRevision only on the response wrapper: {routes_body}"
    );
    let routes: serde_json::Value =
        serde_json::from_str(&routes_body).expect("routes body should be valid JSON");
    assert!(routes["vehicles"]
        .as_array()
        .expect("routes vehicles should be an array")
        .iter()
        .any(|vehicle| {
            !vehicle["segments"]
                .as_array()
                .expect("segments should be an array")
                .is_empty()
        }));
}
