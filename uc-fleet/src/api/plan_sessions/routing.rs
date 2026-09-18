use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use super::{scenario, sessions};
use crate::api::routes::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/scenarios/load", post(scenario::load_scenario))
        .route("/scenarios/perturb", post(scenario::perturb_scenario))
        .route("/solves", post(scenario::create_solve))
        .route("/solves/compare", post(scenario::compare_solves))
        .route("/solves/{id}/status", get(scenario::get_solve_status))
        .route("/solves/{id}/result", get(scenario::get_solve_result))
        .route("/plan-sessions", post(sessions::create_plan_session))
        .route(
            "/plan-sessions/{id}/status",
            get(sessions::get_plan_session_status),
        )
        .route(
            "/plan-sessions/{id}/result",
            get(sessions::get_plan_session_result),
        )
        .route(
            "/plan-sessions/{id}/snapshots/{revision}",
            get(sessions::get_plan_session_snapshot),
        )
        .route(
            "/plan-sessions/{id}/snapshots/{revision}/analysis",
            get(sessions::get_plan_session_analysis),
        )
        .route(
            "/plan-sessions/{id}/repair",
            post(sessions::repair_plan_session),
        )
        .route(
            "/plan-sessions/{id}/revisions/{from_revision}/compare/{to_revision}",
            get(sessions::compare_plan_session_revisions),
        )
}
