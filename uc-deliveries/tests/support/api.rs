use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use solverforge_deliveries::api;
use solverforge_deliveries::data::{generate, DemoData};
use solverforge_deliveries::domain::{Plan, RoutingMode};
use tower::ServiceExt;
use tower_http::services::ServeDir;

pub fn test_app() -> axum::Router {
    let state = Arc::new(api::AppState::new());
    api::router(state)
        .merge(solverforge_ui::routes())
        .fallback_service(ServeDir::new("static"))
}

pub fn small_plan() -> serde_json::Value {
    let mut plan = generate(DemoData::Hartford);
    plan.deliveries.truncate(8);
    plan.vehicles.truncate(3);
    plan.normalize();
    plan.routing_mode = RoutingMode::StraightLine;
    serde_json::to_value(plan.refreshed_for_transport()).expect("plan should serialize")
}

pub fn bigger_plan() -> serde_json::Value {
    let mut plan = generate(DemoData::Philadelphia);
    plan.routing_mode = RoutingMode::StraightLine;
    serde_json::to_value(plan.refreshed_for_transport()).expect("plan should serialize")
}

pub fn completion_plan() -> serde_json::Value {
    let mut plan = generate(DemoData::Hartford);
    plan.deliveries.truncate(6);
    plan.vehicles.truncate(2);
    plan.normalize();
    plan.routing_mode = RoutingMode::StraightLine;
    serde_json::to_value(plan.refreshed_for_transport()).expect("plan should serialize")
}

pub fn empty_road_network_plan() -> serde_json::Value {
    let mut plan = Plan::new("Empty Road Network", Vec::new(), Vec::new());
    plan.routing_mode = RoutingMode::RoadNetwork;
    serde_json::to_value(plan.refreshed_for_transport()).expect("plan should serialize")
}

pub fn small_road_network_plan() -> serde_json::Value {
    let mut plan = generate(DemoData::Hartford);
    plan.deliveries.truncate(4);
    plan.vehicles.truncate(2);
    plan.normalize();
    plan.routing_mode = RoutingMode::RoadNetwork;
    serde_json::to_value(plan.refreshed_for_transport()).expect("plan should serialize")
}

pub async fn read_json(response: axum::response::Response) -> serde_json::Value {
    let body = read_body_text(response).await;
    serde_json::from_str(&body).expect("body should be valid JSON")
}

pub async fn read_body_text(response: axum::response::Response) -> String {
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body should collect")
        .to_bytes();
    String::from_utf8(body.to_vec()).expect("body should be valid UTF-8")
}

pub fn assert_score_value<'a>(value: &'a serde_json::Value, label: &str) -> &'a str {
    let score = value
        .as_str()
        .unwrap_or_else(|| panic!("{label} should be a display string, got {value:?}"));
    assert_score_text(score, label);
    score
}

pub fn assert_score_text(score: &str, label: &str) {
    assert!(
        score.contains("hard/") && score.ends_with("soft") && !score.trim_start().starts_with('{'),
        "{label} should use SolverForge display-score format, got {score:?}"
    );
}

pub async fn poll_job_state(app: &axum::Router, job_id: &str, wanted: &str) -> serde_json::Value {
    let start = Instant::now();
    loop {
        let response = app
            .clone()
            .oneshot(
                Request::get(format!("/jobs/{job_id}"))
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(response.status(), StatusCode::OK);
        let json = read_json(response).await;
        if json["lifecycleState"] == wanted {
            return json;
        }
        if start.elapsed() > Duration::from_secs(6) {
            panic!("job {job_id} did not reach {wanted} in time; last state={json:?}");
        }
        tokio::time::sleep(Duration::from_millis(80)).await;
    }
}
