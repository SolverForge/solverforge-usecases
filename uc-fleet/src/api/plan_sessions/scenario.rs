use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::constraints::support::{revision_diff, RevisionDiff};

use super::helpers::{
    apply_repair_event, lifecycle_to_solve_status, scenario_plan, solve_result_from_plan,
    status_from_solver_error,
};
use super::types::{
    CompareSolvesRequest, CreateSolveRequest, CreateSolveResponse, LoadScenarioRequest,
    LoadScenarioResponse, PerturbScenarioRequest, PerturbScenarioResponse, ScenarioRequest,
    ScenarioSummary, SolveResultResponse, SolveStatusResponse,
};
use crate::api::routes::AppState;

pub(super) async fn load_scenario(
    Json(request): Json<LoadScenarioRequest>,
) -> Result<Json<LoadScenarioResponse>, StatusCode> {
    let scenario_id = request.scenario.clone();
    let plan = scenario_plan(&ScenarioRequest {
        scenario_id: request.scenario,
        data: None,
    })?;
    Ok(Json(LoadScenarioResponse {
        scenario_id,
        summary: ScenarioSummary {
            vessels: plan.vessels.len(),
            docks: plan.docks.len(),
            work_packages: plan.work_packages.len(),
            planning_weeks: plan.days.iter().map(|day| day.week).max().unwrap_or(0) as usize,
        },
    }))
}

pub(super) async fn perturb_scenario(
    Json(request): Json<PerturbScenarioRequest>,
) -> Result<Json<PerturbScenarioResponse>, StatusCode> {
    // Apply every event to the named base scenario so invalid or incomplete
    // events fail here instead of silently producing an unperturbed plan.
    let mut plan = scenario_plan(&ScenarioRequest {
        scenario_id: request.base_scenario_id.clone(),
        data: None,
    })?;
    let mut changes = Vec::new();
    for event in &request.perturbations {
        apply_repair_event(&mut plan, event)?;
        changes.push(format!("{} applied", event.event_type));
    }
    Ok(Json(PerturbScenarioResponse {
        scenario_id: format!("{}_perturbed", request.base_scenario_id),
        changes,
    }))
}

pub(super) async fn create_solve(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateSolveRequest>,
) -> Result<Json<CreateSolveResponse>, StatusCode> {
    let plan = scenario_plan(&ScenarioRequest {
        scenario_id: request.scenario_id,
        data: None,
    })?;
    let solve_id = state
        .solver
        .start_job(plan)
        .map_err(status_from_solver_error)?;
    let status = state
        .solver
        .get_status(&solve_id)
        .map_err(status_from_solver_error)?;
    Ok(Json(CreateSolveResponse {
        solve_id,
        status: lifecycle_to_solve_status(crate::api::dto::lifecycle_state_label(
            status.lifecycle_state,
        )),
    }))
}

pub(super) async fn get_solve_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<SolveStatusResponse>, StatusCode> {
    let status = state
        .solver
        .get_status(&id)
        .map_err(status_from_solver_error)?;
    Ok(Json(SolveStatusResponse {
        solve_id: id,
        status: lifecycle_to_solve_status(crate::api::dto::lifecycle_state_label(
            status.lifecycle_state,
        )),
        best_score: status.best_score.map(|score| score.to_string()),
    }))
}

pub(super) async fn get_solve_result(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<SolveResultResponse>, StatusCode> {
    let snapshot = state
        .solver
        .get_snapshot(&id, None)
        .map_err(status_from_solver_error)?;
    let status = lifecycle_to_solve_status(crate::api::dto::lifecycle_state_label(
        snapshot.lifecycle_state,
    ));
    Ok(Json(solve_result_from_plan(id, status, &snapshot.solution)))
}

pub(super) async fn compare_solves(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CompareSolvesRequest>,
) -> Result<Json<RevisionDiff>, StatusCode> {
    let before = state
        .solver
        .get_snapshot(&request.before_solve_id, None)
        .map_err(status_from_solver_error)?;
    let after = state
        .solver
        .get_snapshot(&request.after_solve_id, None)
        .map_err(status_from_solver_error)?;
    Ok(Json(revision_diff(&before.solution, &after.solution)))
}
