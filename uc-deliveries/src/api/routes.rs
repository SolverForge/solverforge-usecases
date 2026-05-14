//! HTTP routes for the deliveries tutorial app.
//!
//! Each handler follows the same beginner-friendly shape:
//! decode request -> prepare the domain model if needed -> call the retained
//! solver facade -> encode a DTO for the browser.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::dto::{
    analysis_response, DeliveryInsertionCandidateDto, DeliveryInsertionRequestDto,
    DeliveryInsertionResponseDto, JobAnalysisDto, JobRoutesDto, JobSnapshotDto, JobSummaryDto,
    PlanDto,
};
use super::errors::{parse_job_id, status_from_routing_error, status_from_solver_error};
use super::sse;
use crate::data::{generate, DemoData};
use crate::domain::{build_routes_snapshot, prepare_plan, rank_delivery_insertions};
use crate::solver::SolverService;

/// Shared application state stored once inside Axum.
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

/// Registers the public HTTP surface used by the browser and tests.
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
        .route("/jobs/{id}/routes", get(get_routes))
        .route("/jobs/{id}/pause", post(pause_job))
        .route("/jobs/{id}/resume", post(resume_job))
        .route("/jobs/{id}/cancel", post(cancel_job))
        .route("/jobs/{id}/events", get(sse::events))
        .route(
            "/recommendations/delivery-insertions",
            post(recommend_delivery_insertions),
        )
        .with_state(state)
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

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

async fn info() -> Json<InfoResponse> {
    Json(InfoResponse {
        name: "SolverForge Deliveries",
        version: env!("CARGO_PKG_VERSION"),
        solver_engine: "SolverForge",
    })
}

/// Lists the deterministic demo datasets accepted by `/demo-data/{id}`.
async fn list_demo_data() -> Json<Vec<&'static str>> {
    Json(vec![
        DemoData::Philadelphia.id(),
        DemoData::Hartford.id(),
        DemoData::Firenze.id(),
    ])
}

/// Materializes one demo plan and sends it through the same DTO as snapshots.
async fn get_demo_data(Path(id): Path<String>) -> Result<Json<PlanDto>, StatusCode> {
    let demo = id.parse::<DemoData>().map_err(|_| StatusCode::NOT_FOUND)?;
    let plan = generate(demo);
    Ok(Json(PlanDto::from_plan(&plan)))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateJobResponse {
    id: String,
}

async fn create_job(
    State(state): State<Arc<AppState>>,
    Json(dto): Json<PlanDto>,
) -> Result<Json<CreateJobResponse>, StatusCode> {
    let mut plan = dto.to_domain().map_err(|_| StatusCode::BAD_REQUEST)?;
    // Route matrices and shadow variables must be ready before SolverForge
    // starts construction, because the list-variable hooks read them directly.
    prepare_plan(&mut plan)
        .await
        .map_err(status_from_routing_error)?;
    let id = state
        .solver
        .start_job(plan)
        .map_err(status_from_solver_error)?;
    Ok(Json(CreateJobResponse { id }))
}

/// Returns the retained-job summary without requiring a snapshot payload.
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

/// Stock alias used by the shared SolverForge UI job-status helpers.
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

/// Builds route geometry for the exact retained snapshot the browser is viewing.
async fn get_routes(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<SnapshotQuery>,
) -> Result<Json<JobRoutesDto>, StatusCode> {
    let job_id = parse_job_id(&id)?;
    let mut snapshot = state
        .solver
        .get_snapshot(&id, query.snapshot_revision)
        .map_err(status_from_solver_error)?;
    if snapshot
        .solution
        .vehicles
        .iter()
        .any(|vehicle| vehicle.prepared_routing.is_none())
    {
        // Older snapshots can be reconstructed from transport data. If the
        // transient routing cache is absent, rebuild it before drawing routes.
        prepare_plan(&mut snapshot.solution)
            .await
            .map_err(status_from_routing_error)?;
    }
    let routes = build_routes_snapshot(&snapshot.solution)
        .await
        .map_err(status_from_routing_error)?;
    Ok(Json(JobRoutesDto::new(
        job_id,
        snapshot.snapshot_revision,
        routes,
    )))
}

/// Requests a runtime-managed pause at the next safe solver point.
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

/// Cancels a live or paused retained job without deleting its final snapshot.
async fn cancel_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state.solver.cancel(&id).map_err(status_from_solver_error)?;
    Ok(StatusCode::ACCEPTED)
}

/// Deletes a terminal retained job and its cached SSE bootstrap state.
async fn delete_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state.solver.delete(&id).map_err(status_from_solver_error)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Ranks candidate vehicle/position insertions for one delivery.
async fn recommend_delivery_insertions(
    Json(request): Json<DeliveryInsertionRequestDto>,
) -> Result<Json<DeliveryInsertionResponseDto>, StatusCode> {
    let mut plan = request
        .plan
        .to_domain()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    if request.delivery_id >= plan.deliveries.len() {
        return Err(StatusCode::BAD_REQUEST);
    }
    // Candidate scoring uses the same prepared data as real solving so the
    // modal preview matches the constraints and route metrics.
    prepare_plan(&mut plan)
        .await
        .map_err(status_from_routing_error)?;
    let candidates = rank_delivery_insertions(
        &plan,
        request.delivery_id,
        request.limit.unwrap_or(8).min(24),
    )
    .await
    .map_err(status_from_routing_error)?
    .into_iter()
    .map(DeliveryInsertionCandidateDto::from_candidate)
    .collect();
    Ok(Json(DeliveryInsertionResponseDto {
        delivery_id: request.delivery_id,
        candidates,
    }))
}
