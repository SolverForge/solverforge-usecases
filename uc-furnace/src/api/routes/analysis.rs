use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use solverforge::{ConstraintAnalysis, HardSoftScore, ScoreAnalysis};
use std::sync::Arc;

use super::super::dto::{
    status_lifecycle_fields, ConstraintAnalysisDto, ConstraintMatchDto, ConstraintSummaryDto,
    DetailedScoreAnalysisDto, JobAnalysisDto, JobConstraintAnalysisDto,
};
use super::jobs::SnapshotRevisionQuery;
use super::{status_from_solver_error, AppState};

pub(super) async fn analyze_job_snapshot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<SnapshotRevisionQuery>,
) -> Result<Json<JobAnalysisDto>, StatusCode> {
    let snapshot = state
        .solver
        .get_snapshot(&id, query.snapshot_revision)
        .map_err(status_from_solver_error)?;
    let status = state
        .solver
        .get_status(&id)
        .map_err(status_from_solver_error)?;
    let (lifecycle_state, terminal_reason) = status_lifecycle_fields(&status);

    let snapshot_analysis = state
        .solver
        .analyze_snapshot(&id, Some(snapshot.snapshot_revision))
        .map_err(status_from_solver_error)?;

    Ok(Json(JobAnalysisDto {
        job_id: snapshot.job_id.to_string(),
        snapshot_revision: snapshot.snapshot_revision,
        lifecycle_state,
        terminal_reason,
        analysis: analysis_to_dto(&snapshot_analysis.analysis),
    }))
}

pub(super) async fn analyze_job_constraint(
    State(state): State<Arc<AppState>>,
    Path((id, constraint_name)): Path<(String, String)>,
    Query(query): Query<SnapshotRevisionQuery>,
) -> Result<Json<JobConstraintAnalysisDto>, StatusCode> {
    let snapshot = state
        .solver
        .get_snapshot(&id, query.snapshot_revision)
        .map_err(status_from_solver_error)?;
    let status = state
        .solver
        .get_status(&id)
        .map_err(status_from_solver_error)?;
    let (lifecycle_state, terminal_reason) = status_lifecycle_fields(&status);

    let snapshot_analysis = state
        .solver
        .analyze_snapshot(&id, Some(snapshot.snapshot_revision))
        .map_err(status_from_solver_error)?;
    let analysis = snapshot_analysis
        .analysis
        .constraints
        .iter()
        .find(|analysis| analysis.name == constraint_name)
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(JobConstraintAnalysisDto {
        job_id: snapshot.job_id.to_string(),
        snapshot_revision: snapshot.snapshot_revision,
        lifecycle_state,
        terminal_reason,
        score: snapshot_analysis.analysis.score.to_string(),
        constraint: ConstraintAnalysisDto {
            name: analysis.name.clone(),
            constraint_type: constraint_type(&analysis.name),
            weight: analysis.weight.to_string(),
            score: analysis.score.to_string(),
            match_count: analysis.match_count,
            matches: Vec::<ConstraintMatchDto>::new(),
        },
    }))
}

fn analysis_to_dto(analysis: &ScoreAnalysis<HardSoftScore>) -> DetailedScoreAnalysisDto {
    DetailedScoreAnalysisDto {
        score: analysis.score.to_string(),
        constraints: analysis.constraints.iter().map(constraint_to_dto).collect(),
    }
}

fn constraint_to_dto(analysis: &ConstraintAnalysis<HardSoftScore>) -> ConstraintSummaryDto {
    ConstraintSummaryDto {
        name: analysis.name.clone(),
        constraint_type: constraint_type(&analysis.name),
        weight: analysis.weight.to_string(),
        score: analysis.score.to_string(),
        match_count: analysis.match_count,
    }
}

fn constraint_type(name: &str) -> &'static str {
    match name {
        "expressLateness"
        | "urgentLateness"
        | "standardLateness"
        | "thermalChangeover"
        | "earlyStartPressure"
        | "nightStartPreference"
        | "overtime"
        | "shiftLoadBalance" => "soft",
        _ => "hard",
    }
}
