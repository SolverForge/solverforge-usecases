//! HTTP routes for the hospital example.
//!
//! These handlers stay intentionally small: each one should read like
//! "decode request -> call `SolverService` -> encode response".

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::dto::{analysis_response, JobAnalysisDto, JobSnapshotDto, JobSummaryDto, PlanDto};
use super::sse;
use crate::data::{self, DemoData};
use crate::solver::SolverService;

/// Shared application state stored inside Axum.
pub struct AppState {
    pub solver: SolverService,
}

impl AppState {
    /// Builds the shared runtime facade once for the whole router.
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

/// Registers the full public HTTP surface of the example app.
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/info", get(info))
        .route("/demo-data", get(list_demo_data))
        .route("/demo-data/{id}", get(get_demo_data))
        .route("/jobs", post(create_job))
        .route("/jobs/{id}", get(get_job).delete(delete_job))
        .route("/jobs/{id}/status", get(get_job_status))
        .route("/jobs/{id}/snapshot", get(get_snapshot))
        .route("/jobs/{id}/analysis", get(analyze_by_id))
        .route("/jobs/{id}/pause", post(pause_job))
        .route("/jobs/{id}/resume", post(resume_job))
        .route("/jobs/{id}/cancel", post(cancel_job))
        .route("/jobs/{id}/events", get(sse::events))
        .with_state(state)
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

/// Liveness probe used by demos and container platforms.
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "UP" })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InfoResponse {
    name: &'static str,
    version: &'static str,
    solver_engine: &'static str,
}

/// Tiny self-description endpoint for the UI and quick manual checks.
async fn info() -> Json<InfoResponse> {
    Json(InfoResponse {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
        solver_engine: "SolverForge",
    })
}

/// Lists the demo ids accepted by `/demo-data/{id}`.
async fn list_demo_data() -> Json<Vec<&'static str>> {
    Json(data::list_demo_data())
}

/// Materializes one demo dataset and returns it as a `PlanDto`.
async fn get_demo_data(Path(id): Path<String>) -> Result<Json<PlanDto>, StatusCode> {
    let demo = id.parse::<DemoData>().map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(PlanDto::from_plan(&data::generate(demo))))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateJobResponse {
    id: String,
}

/// Starts a retained solve for the posted plan payload.
async fn create_job(
    State(state): State<Arc<AppState>>,
    Json(dto): Json<PlanDto>,
) -> Result<Json<CreateJobResponse>, StatusCode> {
    let plan = dto.to_domain().map_err(|_| StatusCode::BAD_REQUEST)?;
    let id = state
        .solver
        .start_job(plan)
        .map_err(status_from_solver_error)?;
    Ok(Json(CreateJobResponse { id }))
}

/// Returns the current retained-job summary.
async fn get_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<JobSummaryDto>, StatusCode> {
    let job_id = parse_job_id(&id)?;
    let status = state
        .solver
        .get_status(&id)
        .map_err(status_from_solver_error)?;
    Ok(Json(JobSummaryDto::from_status(job_id, &status)))
}

/// Alias route kept for the stock job-summary URL shape.
async fn get_job_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<JobSummaryDto>, StatusCode> {
    get_job(State(state), Path(id)).await
}

#[derive(Debug, Default, Deserialize)]
struct SnapshotQuery {
    snapshot_revision: Option<u64>,
}

/// Fetches either the latest retained snapshot or an exact revision.
async fn get_snapshot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<SnapshotQuery>,
) -> Result<Json<JobSnapshotDto>, StatusCode> {
    let snapshot = state
        .solver
        .get_snapshot(&id, query.snapshot_revision)
        .map_err(status_from_solver_error)?;
    Ok(Json(JobSnapshotDto::from_snapshot(&snapshot)))
}

/// Runs exact score analysis against a retained snapshot revision.
async fn analyze_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<SnapshotQuery>,
) -> Result<Json<JobAnalysisDto>, StatusCode> {
    let snapshot_analysis = state
        .solver
        .analyze_snapshot(&id, query.snapshot_revision)
        .map_err(status_from_solver_error)?;
    let analysis = analysis_response(&snapshot_analysis.analysis);
    Ok(Json(JobAnalysisDto::from_snapshot_analysis(
        &snapshot_analysis,
        analysis,
    )))
}

/// Requests that the runtime pause the job at the next exact safe point.
async fn pause_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state.solver.pause(&id).map_err(status_from_solver_error)?;
    Ok(StatusCode::ACCEPTED)
}

/// Resumes a paused retained job.
async fn resume_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state.solver.resume(&id).map_err(status_from_solver_error)?;
    Ok(StatusCode::ACCEPTED)
}

/// Cancels a live or paused retained job.
async fn cancel_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state.solver.cancel(&id).map_err(status_from_solver_error)?;
    Ok(StatusCode::ACCEPTED)
}

/// Deletes a terminal retained job and its cached SSE state.
async fn delete_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state.solver.delete(&id).map_err(status_from_solver_error)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Parses the path segment into the numeric runtime job id.
fn parse_job_id(id: &str) -> Result<usize, StatusCode> {
    id.parse::<usize>().map_err(|_| StatusCode::NOT_FOUND)
}

/// Maps stock runtime errors onto the HTTP semantics the UI expects.
fn status_from_solver_error(error: solverforge::SolverManagerError) -> StatusCode {
    match error {
        solverforge::SolverManagerError::NoFreeJobSlots => StatusCode::SERVICE_UNAVAILABLE,
        solverforge::SolverManagerError::JobNotFound { .. } => StatusCode::NOT_FOUND,
        solverforge::SolverManagerError::InvalidStateTransition { .. } => StatusCode::CONFLICT,
        solverforge::SolverManagerError::NoSnapshotAvailable { .. } => StatusCode::CONFLICT,
        solverforge::SolverManagerError::SnapshotNotFound { .. } => StatusCode::NOT_FOUND,
    }
}

#[cfg(test)]
mod tests;
