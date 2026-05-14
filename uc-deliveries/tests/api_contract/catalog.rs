use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use crate::support::api::{read_json, small_plan, test_app};

#[tokio::test]
async fn demo_data_and_static_assets_match_current_contract() {
    let app = test_app();

    let demo_response = app
        .clone()
        .oneshot(Request::get("/demo-data").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(demo_response.status(), StatusCode::OK);
    let demos = read_json(demo_response).await;
    assert_eq!(
        demos,
        serde_json::json!(["PHILADELPHIA", "HARTFORD", "FIRENZE"])
    );

    let demo_plan_response = app
        .clone()
        .oneshot(
            Request::get("/demo-data/PHILADELPHIA")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(demo_plan_response.status(), StatusCode::OK);
    let demo_plan = read_json(demo_plan_response).await;
    assert_eq!(demo_plan["name"], "Philadelphia");
    assert_eq!(demo_plan["routingMode"], "road_network");
    assert!(!demo_plan["viewState"]["preview"]["vehicles"]
        .as_array()
        .expect("preview vehicles should exist")
        .is_empty());

    let asset_response = app
        .clone()
        .oneshot(Request::get("/sf/sf.js").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(asset_response.status(), StatusCode::OK);

    let no_legacy_response = app
        .clone()
        .oneshot(Request::get("/webjars/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(no_legacy_response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn recommendations_endpoint_returns_ranked_preview_plans() {
    let app = test_app();
    let mut plan = small_plan();
    if let Some(order) = plan["vehicles"][0]["deliveryOrder"].as_array_mut() {
        order.clear();
    }

    let response = app
        .clone()
        .oneshot(
            Request::post("/recommendations/delivery-insertions")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "plan": plan,
                        "deliveryId": 0,
                        "limit": 5
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    let candidates = json["candidates"]
        .as_array()
        .expect("candidates should be an array");
    assert!(!candidates.is_empty());
    assert!(candidates[0]["previewPlan"]["viewState"]["preview"].is_object());
}
