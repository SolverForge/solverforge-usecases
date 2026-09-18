mod analysis;
mod jobs;
mod meta;

use axum::{
    extract::DefaultBodyLimit,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::solver::SolverService;

pub struct AppState {
    pub solver: SolverService,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            solver: SolverService::new(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(meta::health))
        .route("/info", get(meta::info))
        .route("/demo-data", get(meta::list_demo_data))
        .route("/demo-data/{name}", get(meta::get_demo_data))
        .route("/jobs", post(jobs::create_job))
        .route("/jobs/{id}", get(jobs::get_job).delete(jobs::delete_job))
        .route("/jobs/{id}/status", get(jobs::get_job_status))
        .route("/jobs/{id}/snapshot", get(jobs::get_job_snapshot))
        .route("/jobs/{id}/analysis", get(analysis::analyze_job_snapshot))
        .route(
            "/jobs/{id}/analysis/{constraint_name}",
            get(analysis::analyze_job_constraint),
        )
        .route("/jobs/{id}/pause", post(jobs::pause_job))
        .route("/jobs/{id}/resume", post(jobs::resume_job))
        .route("/jobs/{id}/cancel", post(jobs::cancel_job))
        .route("/jobs/{id}/events", get(super::sse::events))
        .layer(DefaultBodyLimit::max(16 * 1024 * 1024))
        .with_state(state)
}

pub fn status_from_solver_error(error: solverforge::SolverManagerError) -> StatusCode {
    match error {
        solverforge::SolverManagerError::NoFreeJobSlots => StatusCode::SERVICE_UNAVAILABLE,
        solverforge::SolverManagerError::JobNotFound { .. }
        | solverforge::SolverManagerError::NoSnapshotAvailable { .. }
        | solverforge::SolverManagerError::SnapshotNotFound { .. } => StatusCode::NOT_FOUND,
        solverforge::SolverManagerError::InvalidStateTransition { .. } => StatusCode::CONFLICT,
    }
}

#[cfg(test)]
mod tests;
