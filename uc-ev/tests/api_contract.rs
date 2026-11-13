//! HTTP contract tests that need no road network: catalog, demo plans, error
//! codes, and a full retained-job lifecycle on an empty plan.

use std::sync::Arc;
use std::time::Duration;

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use axum::Router;
use serde_json::{json, Value};
use solverforge_ev::api::{router, AppState};
use tower::ServiceExt;

fn app() -> Router {
    router(Arc::new(AppState::new()))
}

async fn call(app: &Router, method: Method, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let mut request = Request::builder().method(method).uri(uri);
    let body = match body {
        Some(value) => {
            request = request.header("content-type", "application/json");
            Body::from(value.to_string())
        }
        None => Body::empty(),
    };
    let response = app
        .clone()
        .oneshot(request.body(body).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, value)
}

#[tokio::test]
async fn catalog_lists_the_single_demo_plan() {
    let app = app();
    let (status, catalog) = call(&app, Method::GET, "/demo-data", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(catalog["defaultId"], "STANDARD");
    assert_eq!(catalog["availableIds"], json!(["STANDARD"]));
}

#[tokio::test]
async fn demo_plan_serializes_facts_vans_and_empty_routes() {
    let app = app();
    let (status, plan) = call(&app, Method::GET, "/demo-data/standard", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(plan["customers"].as_array().unwrap().len(), 48);
    assert_eq!(plan["chargers"].as_array().unwrap().len(), 14);
    let van = &plan["vehicles"][0];
    assert_eq!(van["batteryRangeM"], 160_000);
    assert_eq!(van["reserveChargeM"], 10_000);
    assert_eq!(van["visitOrder"], json!([]));
    assert_eq!(van["schedule"], json!([]));
    assert!(
        plan.get("problemData").is_none(),
        "transient matrices stay server-side"
    );

    let (status, _) = call(&app, Method::GET, "/demo-data/SMALL", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn malformed_plans_and_unknown_jobs_are_rejected() {
    let app = app();
    let (status, _) = call(
        &app,
        Method::POST,
        "/jobs",
        Some(json!({ "customers": "nope" })),
    )
    .await;
    assert!(status.is_client_error(), "got {status}");
    let (status, _) = call(&app, Method::GET, "/jobs/999999/routes", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn empty_plan_runs_the_retained_job_lifecycle() {
    let app = app();
    let empty = json!({ "customers": [], "chargers": [], "vehicles": [] });
    let (status, created) = call(&app, Method::POST, "/jobs", Some(empty)).await;
    assert_eq!(status, StatusCode::OK);
    let id = created["id"].as_str().unwrap().to_string();

    let (status, _) = call(&app, Method::POST, &format!("/jobs/{id}/cancel"), None).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let mut state = Value::Null;
    for _ in 0..100 {
        let (_, status) = call(&app, Method::GET, &format!("/jobs/{id}/status"), None).await;
        state = status["lifecycleState"].clone();
        if state == "CANCELLED" || state == "COMPLETED" {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(state == "CANCELLED" || state == "COMPLETED", "got {state}");

    // Routes are bound to snapshots: they exist exactly when a snapshot does.
    let (snapshot_status, snapshot) =
        call(&app, Method::GET, &format!("/jobs/{id}/snapshot"), None).await;
    let revision = snapshot["snapshotRevision"].as_u64().unwrap_or(0);
    let uri = format!("/jobs/{id}/routes?snapshot_revision={revision}");
    let (routes_status, routes) = call(&app, Method::GET, &uri, None).await;
    if snapshot_status == StatusCode::OK {
        assert_eq!(routes_status, StatusCode::OK);
        assert_eq!(routes["snapshotRevision"], revision);
        assert_eq!(routes["routes"]["vehicles"], json!([]));
    } else {
        assert_eq!(snapshot_status, StatusCode::CONFLICT);
        assert_eq!(routes_status, StatusCode::CONFLICT);
    }

    let (status, _) = call(&app, Method::DELETE, &format!("/jobs/{id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}
