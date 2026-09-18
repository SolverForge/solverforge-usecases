use super::*;
use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use serde_json::Value;
use tower::ServiceExt;

async fn response_bytes(router: &Router, request: Request<Body>) -> (StatusCode, Vec<u8>) {
    let response = router
        .clone()
        .oneshot(request)
        .await
        .expect("router response");
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    (status, body.to_vec())
}

async fn response_json(router: &Router, request: Request<Body>) -> (StatusCode, Value) {
    let (status, body) = response_bytes(router, request).await;
    let json = serde_json::from_slice(&body).unwrap_or_else(|error| {
        panic!(
            "json body status={status} body={} error={error}",
            String::from_utf8_lossy(&body)
        )
    });
    (status, json)
}

fn app() -> Router {
    router(Arc::new(AppState::new()))
}

async fn cancel_if_active(router: &Router, job_id: &str) {
    let cancel_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/jobs/{job_id}/cancel"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("cancel response");
    assert!(matches!(
        cancel_response.status(),
        StatusCode::NO_CONTENT | StatusCode::CONFLICT
    ));
}

async fn wait_for_terminal_state(router: &Router, job_id: &str) {
    for _ in 0..120 {
        let (status, json) = response_json(
            router,
            Request::builder()
                .uri(format!("/jobs/{job_id}/status"))
                .body(Body::empty())
                .expect("request"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let lifecycle_state = json["lifecycleState"].as_str().expect("lifecycle state");
        if matches!(lifecycle_state, "CANCELLED" | "COMPLETED" | "FAILED") {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    panic!("job should reach a terminal lifecycle state");
}

async fn delete_terminal_job(router: &Router, job_id: &str) {
    let delete_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/jobs/{job_id}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("delete response");
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);
}

struct BriefPoll {
    lifecycle_state: String,
    saw_construction_progress: bool,
}

async fn poll_briefly_and_assert_not_failed(router: &Router, job_id: &str) -> BriefPoll {
    let mut last_state = String::new();
    let mut saw_construction_progress = false;
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
    while tokio::time::Instant::now() < deadline {
        let (status, json) = response_json(
            router,
            Request::builder()
                .uri(format!("/jobs/{job_id}/status"))
                .body(Body::empty())
                .expect("request"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let lifecycle_state = json["lifecycleState"].as_str().expect("lifecycle state");
        assert_ne!(
            lifecycle_state, "FAILED",
            "retained demo job failed immediately: {json}"
        );
        last_state = lifecycle_state.to_string();
        let construction_slots_assigned = json["telemetry"]["constructionSlotsAssigned"]
            .as_u64()
            .unwrap_or(0);
        saw_construction_progress |= construction_slots_assigned > 0;
        if saw_construction_progress {
            return BriefPoll {
                lifecycle_state: last_state,
                saw_construction_progress,
            };
        }
        if matches!(lifecycle_state, "CANCELLED" | "COMPLETED") {
            return BriefPoll {
                lifecycle_state: last_state,
                saw_construction_progress,
            };
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    BriefPoll {
        lifecycle_state: last_state,
        saw_construction_progress,
    }
}

#[tokio::test]
async fn demo_data_route_preserves_frontend_shape() {
    let app = app();
    let (status, json) = response_json(
        &app,
        Request::builder()
            .uri("/demo-data/STANDARD")
            .body(Body::empty())
            .expect("request"),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(json.get("furnaces").is_some());
    assert!(json.get("workOrders").is_some());
    assert!(json.get("assignments").is_some());
    assert_eq!(
        json["assignments"].as_array().expect("assignments").len(),
        0
    );
    assert_eq!(
        json["furnaceAssignments"]
            .as_array()
            .expect("canonical furnace assignments")
            .iter()
            .filter(|assignment| !assignment["assignment"].is_null())
            .count(),
        0
    );
    assert_eq!(
        json["operatorShiftAssignments"]
            .as_array()
            .expect("canonical shift assignments")
            .iter()
            .filter(|assignment| !assignment["shiftTypeValue"].is_null())
            .count(),
        0
    );
    assert!(json.get("operators").is_some());
    assert!(json.get("shifts").is_some());
    assert!(json.get("operatorShiftAssignments").is_some());
}

#[tokio::test]
async fn demo_data_list_exposes_the_single_standard_demo() {
    let app = app();
    let (list_status, list_json) = response_json(
        &app,
        Request::builder()
            .uri("/demo-data")
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(list_status, StatusCode::OK);
    assert_eq!(list_json, serde_json::json!(["STANDARD"]));
}

#[tokio::test]
async fn standard_demo_route_returns_unsolved_canonical_plan() {
    let app = app();
    let (status, json) = response_json(
        &app,
        Request::builder()
            .uri("/demo-data/STANDARD")
            .body(Body::empty())
            .expect("request"),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        json["workOrders"].as_array().expect("work orders").len(),
        155
    );
    assert_eq!(
        json["assignments"].as_array().expect("assignments").len(),
        0
    );
    assert_eq!(
        json["furnaceAssignments"]
            .as_array()
            .expect("canonical furnace assignments")
            .iter()
            .filter(|assignment| !assignment["assignment"].is_null())
            .count(),
        0
    );
    assert_eq!(
        json["operatorShiftAssignments"]
            .as_array()
            .expect("canonical shift assignments")
            .iter()
            .filter(|assignment| !assignment["shiftTypeValue"].is_null())
            .count(),
        0
    );
}

#[tokio::test]
async fn create_job_accepts_demo_payload_and_returns_neutral_shell_id_object() {
    let app = app();
    let (demo_status, demo_json) = response_json(
        &app,
        Request::builder()
            .uri("/demo-data/STANDARD")
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(demo_status, StatusCode::OK);

    let (status, create_job_json) = response_json(
        &app,
        Request::builder()
            .method("POST")
            .uri("/jobs")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&demo_json).expect("demo json should serialize"),
            ))
            .expect("request"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let job_id = create_job_json["id"]
        .as_str()
        .expect("create job should return an id")
        .to_string();

    cancel_if_active(&app, &job_id).await;
    wait_for_terminal_state(&app, &job_id).await;
}

#[tokio::test]
async fn retained_demo_job_does_not_immediately_fail() {
    let app = app();
    let (demo_status, demo_json) = response_json(
        &app,
        Request::builder()
            .uri("/demo-data/STANDARD")
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(demo_status, StatusCode::OK);

    let (status, create_job_json) = response_json(
        &app,
        Request::builder()
            .method("POST")
            .uri("/jobs")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&demo_json).expect("demo json should serialize"),
            ))
            .expect("request"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let job_id = create_job_json["id"]
        .as_str()
        .expect("create job should return an id")
        .to_string();

    let poll = poll_briefly_and_assert_not_failed(&app, &job_id).await;
    assert!(
        poll.saw_construction_progress,
        "retained demo job made no construction progress before {}",
        poll.lifecycle_state
    );
    if !matches!(poll.lifecycle_state.as_str(), "COMPLETED" | "CANCELLED") {
        cancel_if_active(&app, &job_id).await;
        wait_for_terminal_state(&app, &job_id).await;
    }
    delete_terminal_job(&app, &job_id).await;
}

#[tokio::test]
async fn create_job_rejects_unknown_demo_payload() {
    let app = app();
    let (status, _) = response_bytes(
        &app,
        Request::builder()
            .method("POST")
            .uri("/jobs")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"furnaces":[],"workOrders":[],"assignments":[],"furnaceAssignments":[],"operators":[],"shifts":[],"operatorShiftAssignments":[]}"#,
            ))
            .expect("request"),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn job_snapshot_analysis_and_detail_routes_match_existing_contract() {
    let app = app();
    let (_, demo_json) = response_json(
        &app,
        Request::builder()
            .uri("/demo-data/STANDARD")
            .body(Body::empty())
            .expect("request"),
    )
    .await;

    let (status, create_job_json) = response_json(
        &app,
        Request::builder()
            .method("POST")
            .uri("/jobs")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&demo_json).expect("demo json should serialize"),
            ))
            .expect("request"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let job_id = create_job_json["id"]
        .as_str()
        .expect("create job should return an id")
        .to_string();
    assert!(!job_id.is_empty());

    let mut snapshot_json: Option<Value> = None;
    for _ in 0..60 {
        let (snapshot_status, body) = response_bytes(
            &app,
            Request::builder()
                .uri(format!("/jobs/{job_id}/snapshot"))
                .body(Body::empty())
                .expect("request"),
        )
        .await;
        if snapshot_status == StatusCode::OK {
            snapshot_json = Some(serde_json::from_slice(&body).expect("snapshot json"));
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    let snapshot_json = snapshot_json.expect("snapshot should become available");
    assert!(snapshot_json.get("jobId").is_some());
    assert!(snapshot_json.get("snapshotRevision").is_some());
    assert!(snapshot_json.get("lifecycleState").is_some());
    assert!(snapshot_json.get("solution").is_some());
    assert!(snapshot_json["solution"].get("furnaces").is_some());

    let (status_status, status_json) = response_json(
        &app,
        Request::builder()
            .uri(format!("/jobs/{job_id}/status"))
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(status_status, StatusCode::OK);
    assert!(status_json.get("id").is_some());
    assert!(status_json.get("lifecycleState").is_some());
    assert!(status_json.get("telemetry").is_some());

    let (analysis_status, analysis_json) = response_json(
        &app,
        Request::builder()
            .uri(format!("/jobs/{job_id}/analysis"))
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(analysis_status, StatusCode::OK);
    assert!(analysis_json.get("jobId").is_some());
    assert!(analysis_json.get("snapshotRevision").is_some());
    assert!(analysis_json["analysis"].get("score").is_some());
    assert!(analysis_json["analysis"].get("constraints").is_some());

    let first_constraint = analysis_json["analysis"]["constraints"]
        .as_array()
        .and_then(|constraints| constraints.first())
        .and_then(|constraint| constraint.get("name"))
        .and_then(Value::as_str)
        .expect("analysis should contain a constraint");

    let (detail_status, detail_json) = response_json(
        &app,
        Request::builder()
            .uri(format!("/jobs/{job_id}/analysis/{first_constraint}"))
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(detail_status, StatusCode::OK);
    assert!(detail_json.get("jobId").is_some());
    assert!(detail_json.get("snapshotRevision").is_some());
    assert!(detail_json.get("score").is_some());
    assert!(detail_json["constraint"].get("name").is_some());
    assert!(detail_json["constraint"].get("matches").is_some());

    cancel_if_active(&app, &job_id).await;
    wait_for_terminal_state(&app, &job_id).await;

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/jobs/{job_id}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("delete response");
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);
}

/// Full acceptance solve for the canonical demo.
///
/// This is the slow end-to-end check that the configured solver policy can
/// actually turn the published `STANDARD` instance into a hard-feasible plan.
/// It runs only under `make test-slow` because the runtime budget is measured
/// in tens of seconds.
#[tokio::test]
#[ignore = "slow acceptance solve"]
async fn standard_demo_solves_to_feasible_terminal_state() {
    let app = app();
    let (_, demo_json) = response_json(
        &app,
        Request::builder()
            .uri("/demo-data/STANDARD")
            .body(Body::empty())
            .expect("request"),
    )
    .await;

    let (status, create_job_json) = response_json(
        &app,
        Request::builder()
            .method("POST")
            .uri("/jobs")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&demo_json).expect("demo json should serialize"),
            ))
            .expect("request"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let job_id = create_job_json["id"]
        .as_str()
        .expect("create job should return an id")
        .to_string();

    let mut terminal_state = String::new();
    for _ in 0..900 {
        let (status, json) = response_json(
            &app,
            Request::builder()
                .uri(format!("/jobs/{job_id}/status"))
                .body(Body::empty())
                .expect("request"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let lifecycle_state = json["lifecycleState"].as_str().expect("lifecycle state");
        if matches!(lifecycle_state, "COMPLETED" | "CANCELLED" | "FAILED") {
            terminal_state = lifecycle_state.to_string();
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    assert_eq!(
        terminal_state, "COMPLETED",
        "STANDARD demo should solve to a completed terminal state"
    );

    let (analysis_status, analysis_json) = response_json(
        &app,
        Request::builder()
            .uri(format!("/jobs/{job_id}/analysis"))
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(analysis_status, StatusCode::OK);
    let score = analysis_json["analysis"]["score"]
        .as_str()
        .expect("analysis score");
    assert!(
        score.starts_with("0hard"),
        "expected a hard-feasible terminal score, got {score}"
    );

    delete_terminal_job(&app, &job_id).await;
}
