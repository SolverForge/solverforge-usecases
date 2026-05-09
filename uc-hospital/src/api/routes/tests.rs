//! Route tests for the retained-job HTTP contract.

use super::*;
use axum::body::{to_bytes, Body};
use axum::http::Request;
use tower::util::ServiceExt;

use crate::domain::{Employee, Plan};

fn empty_plan() -> PlanDto {
    PlanDto::from_plan(&Plan::new(
        vec![Employee::new(0, "Alex").with_skill("Doctor")],
        vec![],
    ))
}

fn heavy_plan() -> PlanDto {
    PlanDto::from_plan(&data::generate(DemoData::Large))
}

async fn json_body(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

async fn post_plan(app: &Router, path: &str, plan: &PlanDto) -> axum::response::Response {
    post_json_bytes(app, path, serde_json::to_vec(plan).unwrap()).await
}

async fn post_json_bytes(app: &Router, path: &str, body: Vec<u8>) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::post(path)
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn request(app: &Router, method: &str, path: &str) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn wait_for_status(
    app: &Router,
    id: &str,
    predicate: impl Fn(&serde_json::Value) -> bool,
) -> serde_json::Value {
    for _ in 0..200 {
        let response = request(app, "GET", &format!("/jobs/{id}")).await;
        if response.status() == StatusCode::OK {
            let json = json_body(response).await;
            if predicate(&json) {
                return json;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    panic!("timed out waiting for job status");
}

async fn wait_for_ok_json(app: &Router, path: &str) -> serde_json::Value {
    for _ in 0..200 {
        let response = request(app, "GET", path).await;
        if response.status() == StatusCode::OK {
            return json_body(response).await;
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    panic!("timed out waiting for successful response on {path}");
}

async fn wait_for_terminal(app: &Router, id: &str) -> serde_json::Value {
    wait_for_status(app, id, |json| {
        matches!(
            json["lifecycleState"].as_str(),
            Some("COMPLETED") | Some("CANCELLED") | Some("FAILED")
        )
    })
    .await
}

async fn cleanup_job(app: &Router, id: &str) {
    let status = request(app, "GET", &format!("/jobs/{id}")).await;
    if status.status() == StatusCode::NOT_FOUND {
        return;
    }

    let summary = json_body(status).await;
    let lifecycle = summary["lifecycleState"].as_str().unwrap_or("");
    if !matches!(lifecycle, "COMPLETED" | "CANCELLED" | "FAILED") {
        let cancel = request(app, "POST", &format!("/jobs/{id}/cancel")).await;
        assert!(
            cancel.status() == StatusCode::ACCEPTED || cancel.status() == StatusCode::CONFLICT,
            "unexpected cancel status {}",
            cancel.status()
        );
        let _ = wait_for_terminal(app, id).await;
    }

    let delete = request(app, "DELETE", &format!("/jobs/{id}")).await;
    assert!(
        delete.status() == StatusCode::NO_CONTENT || delete.status() == StatusCode::NOT_FOUND,
        "unexpected delete status {}",
        delete.status()
    );
}

#[tokio::test]
async fn stock_jobs_contract_is_exposed_without_schedule_compatibility() {
    let app = router(Arc::new(AppState::new()));

    let create = post_plan(&app, "/jobs", &empty_plan()).await;
    assert_eq!(create.status(), StatusCode::OK);
    let create_json = json_body(create).await;
    let terminal_id = create_json["id"].as_str().unwrap().to_string();
    assert!(!terminal_id.is_empty());
    assert!(create_json.get("jobId").is_none());

    let summary = wait_for_status(&app, &terminal_id, |_| true).await;
    assert_eq!(summary["id"], terminal_id);
    assert_eq!(summary["jobId"], terminal_id);
    assert!(summary.get("lifecycleState").is_some());
    assert!(summary.get("checkpointAvailable").is_some());
    assert!(summary.get("eventSequence").is_some());
    assert!(summary.get("telemetry").is_some());

    let snapshot = wait_for_ok_json(&app, &format!("/jobs/{terminal_id}/snapshot")).await;
    assert_eq!(snapshot["id"], terminal_id);
    assert_eq!(snapshot["jobId"], terminal_id);
    assert!(snapshot.get("snapshotRevision").is_some());
    assert!(snapshot.get("solution").is_some());

    let _ = wait_for_terminal(&app, &terminal_id).await;
    let analysis = wait_for_ok_json(&app, &format!("/jobs/{terminal_id}/analysis")).await;
    assert_eq!(analysis["id"], terminal_id);
    assert_eq!(analysis["jobId"], terminal_id);
    assert!(analysis.get("analysis").is_some());

    let cancel_terminal = request(&app, "POST", &format!("/jobs/{terminal_id}/cancel")).await;
    assert_eq!(cancel_terminal.status(), StatusCode::CONFLICT);

    let delete_terminal = request(&app, "DELETE", &format!("/jobs/{terminal_id}")).await;
    assert_eq!(delete_terminal.status(), StatusCode::NO_CONTENT);

    let missing_status = request(&app, "GET", &format!("/jobs/{terminal_id}/status")).await;
    assert_eq!(missing_status.status(), StatusCode::NOT_FOUND);

    let live_create = post_plan(&app, "/jobs", &heavy_plan()).await;
    assert_eq!(live_create.status(), StatusCode::OK);
    let live_id = json_body(live_create).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let _ = wait_for_status(&app, &live_id, |json| {
        json["lifecycleState"] == "SOLVING" || json["lifecycleState"] == "PAUSE_REQUESTED"
    })
    .await;
    let delete_live = request(&app, "DELETE", &format!("/jobs/{live_id}")).await;
    assert_eq!(delete_live.status(), StatusCode::CONFLICT);

    let pause = request(&app, "POST", &format!("/jobs/{live_id}/pause")).await;
    assert_eq!(pause.status(), StatusCode::ACCEPTED);
    let _ = wait_for_status(&app, &live_id, |json| json["lifecycleState"] == "PAUSED").await;

    let delete_paused = request(&app, "DELETE", &format!("/jobs/{live_id}")).await;
    assert_eq!(delete_paused.status(), StatusCode::CONFLICT);

    let resume = request(&app, "POST", &format!("/jobs/{live_id}/resume")).await;
    assert_eq!(resume.status(), StatusCode::ACCEPTED);
    let _ = wait_for_status(&app, &live_id, |json| {
        json["lifecycleState"] == "SOLVING" || json["lifecycleState"] == "PAUSE_REQUESTED"
    })
    .await;

    let cancel = request(&app, "POST", &format!("/jobs/{live_id}/cancel")).await;
    assert_eq!(cancel.status(), StatusCode::ACCEPTED);
    let _ = wait_for_terminal(&app, &live_id).await;
    let delete_cancelled = request(&app, "DELETE", &format!("/jobs/{live_id}")).await;
    assert_eq!(delete_cancelled.status(), StatusCode::NO_CONTENT);

    let mut full_ids = Vec::new();
    for _ in 0..16 {
        let response = post_plan(&app, "/jobs", &heavy_plan()).await;
        if response.status() != StatusCode::OK {
            break;
        }
        let id = json_body(response).await["id"]
            .as_str()
            .unwrap()
            .to_string();
        full_ids.push(id);
    }
    assert_eq!(
        full_ids.len(),
        16,
        "expected to occupy all 16 runtime job slots"
    );

    let full_response = post_plan(&app, "/jobs", &heavy_plan()).await;
    assert_eq!(full_response.status(), StatusCode::SERVICE_UNAVAILABLE);

    for id in full_ids {
        cleanup_job(&app, &id).await;
    }

    let old_contract = request(&app, "POST", "/schedules").await;
    assert_eq!(old_contract.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn semantically_invalid_jobs_payload_returns_bad_request_without_killing_router() {
    let app = router(Arc::new(AppState::new()));

    let invalid = post_json_bytes(&app, "/jobs", br#"{}"#.to_vec()).await;
    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);

    let valid = post_plan(&app, "/jobs", &empty_plan()).await;
    assert_eq!(valid.status(), StatusCode::OK);

    let job_id = json_body(valid).await["id"].as_str().unwrap().to_string();
    cleanup_job(&app, &job_id).await;
}
