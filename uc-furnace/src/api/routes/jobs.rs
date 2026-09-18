use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use solverforge::HardSoftScore;
use std::sync::Arc;

use super::super::dto::{snapshot_to_dto, status_to_dto, JobSnapshotDto, JobSummaryDto, PlanDto};
use super::{status_from_solver_error, AppState};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CreateJobResponse {
    id: String,
}

pub(super) async fn create_job(
    State(state): State<Arc<AppState>>,
    Json(request): Json<PlanDto>,
) -> Result<Json<CreateJobResponse>, StatusCode> {
    let plan = request.to_plan().map_err(|_| StatusCode::BAD_REQUEST)?;
    let id = state
        .solver
        .start_job(plan)
        .map_err(status_from_solver_error)?;
    Ok(Json(CreateJobResponse { id }))
}

pub(super) async fn get_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<JobSummaryDto>, StatusCode> {
    let status = state
        .solver
        .get_status(&id)
        .map_err(status_from_solver_error)?;
    let snapshot_revision = status.latest_snapshot_revision;
    let mut dto = status_to_dto(status);
    let score = analyzed_snapshot_score(&state, &id, snapshot_revision)
        .map_err(status_from_solver_error)?;
    set_display_score(&mut dto.current_score, &mut dto.best_score, score);
    Ok(Json(dto))
}

pub(super) async fn get_job_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<JobSummaryDto>, StatusCode> {
    get_job(State(state), Path(id)).await
}

#[derive(Debug, Deserialize)]
pub(super) struct SnapshotRevisionQuery {
    pub snapshot_revision: Option<u64>,
}

pub(super) async fn get_job_snapshot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<SnapshotRevisionQuery>,
) -> Result<Json<JobSnapshotDto>, StatusCode> {
    let snapshot = state
        .solver
        .get_snapshot(&id, query.snapshot_revision)
        .map_err(status_from_solver_error)?;
    let status = state
        .solver
        .get_status(&id)
        .map_err(status_from_solver_error)?;
    let snapshot_revision = snapshot.snapshot_revision;
    let mut dto = snapshot_to_dto(snapshot, &status);
    let score = analyzed_snapshot_score(&state, &id, Some(snapshot_revision))
        .map_err(status_from_solver_error)?;
    set_display_score(&mut dto.current_score, &mut dto.best_score, score);
    Ok(Json(dto))
}

pub(super) async fn pause_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state.solver.pause(&id).map_err(status_from_solver_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn resume_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state.solver.resume(&id).map_err(status_from_solver_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn cancel_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state.solver.cancel(&id).map_err(status_from_solver_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn delete_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state.solver.delete(&id).map_err(status_from_solver_error)?;
    Ok(StatusCode::NO_CONTENT)
}

fn analyzed_snapshot_score(
    state: &Arc<AppState>,
    id: &str,
    snapshot_revision: Option<u64>,
) -> Result<Option<HardSoftScore>, solverforge::SolverManagerError> {
    match snapshot_revision {
        Some(revision) => state
            .solver
            .analyze_snapshot(id, Some(revision))
            .map(|snapshot_analysis| Some(snapshot_analysis.analysis.score)),
        None => Ok(None),
    }
}

fn set_display_score(
    current_score: &mut Option<String>,
    best_score: &mut Option<String>,
    score: Option<HardSoftScore>,
) {
    match score {
        Some(score) => {
            let score = score.to_string();
            *current_score = Some(score.clone());
            *best_score = Some(score);
        }
        None => {
            *current_score = None;
            *best_score = None;
        }
    }
}
