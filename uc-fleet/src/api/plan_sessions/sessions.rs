use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::Value;
use uuid::Uuid;

use crate::api::dto::{analysis_response, JobAnalysisDto, JobSnapshotDto, PlanDto, TelemetryDto};
use crate::constraints::support::{fleet_metrics, revision_diff};

use super::helpers::{
    apply_repair_event, lifecycle_to_solve_status, parse_revision, scenario_plan, session_record,
    solve_result_from_plan, status_from_solver_error,
};
use super::types::{
    CreatePlanSessionRequest, CreatePlanSessionResponse, PlanSessionRecord, PlanSessionRepairLink,
    PlanSessionRevision, PlanSessionStatusResponse, RepairRequest, RepairResponse,
    RevisionCompareResponse, SolveResultResponse,
};
use crate::api::routes::AppState;

pub(super) async fn create_plan_session(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreatePlanSessionRequest>,
) -> Result<Json<CreatePlanSessionResponse>, StatusCode> {
    if request
        .domain
        .as_deref()
        .is_some_and(|domain| domain != "fleet_readiness")
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let _metadata = request.metadata.as_ref();

    let plan = scenario_plan(&request.scenario)?;
    let baseline_plan = PlanDto::from_plan(&plan);
    let job_id = state
        .solver
        .start_job(plan)
        .map_err(status_from_solver_error)?;
    let status = state
        .solver
        .get_status(&job_id)
        .map_err(status_from_solver_error)?;
    let plan_session_id = format!("session_{}", Uuid::new_v4());
    let solver_profile = request
        .solver_profile
        .unwrap_or_else(|| "fleet_default_v1".to_string());

    let record = PlanSessionRecord {
        active_job_id: job_id.clone(),
        baseline_plan,
        revisions: Vec::new(),
        repair_links: Vec::new(),
        latest_repair_job_id: None,
        latest_repair_mode: None,
    };
    state
        .plan_sessions
        .write()
        .insert(plan_session_id.clone(), record);

    Ok(Json(CreatePlanSessionResponse {
        plan_session_id: plan_session_id.clone(),
        job_id,
        lifecycle_state: crate::api::dto::lifecycle_state_label(status.lifecycle_state),
        checkpoint_available: status.checkpoint_available,
        latest_revision: status.latest_snapshot_revision,
        status_url: format!("/plan-sessions/{plan_session_id}/status"),
        snapshot_url: format!("/plan-sessions/{plan_session_id}/snapshots/latest"),
        solver_profile,
    }))
}

pub(super) async fn get_plan_session_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<PlanSessionStatusResponse>, StatusCode> {
    let record = session_record(&state, &id)?;
    let status = state
        .solver
        .get_status(&record.active_job_id)
        .map_err(status_from_solver_error)?;
    Ok(Json(PlanSessionStatusResponse {
        plan_session_id: id,
        active_job_id: record.active_job_id,
        lifecycle_state: crate::api::dto::lifecycle_state_label(status.lifecycle_state),
        event_sequence: status.event_sequence,
        latest_revision: status.latest_snapshot_revision,
        checkpoint_available: status.checkpoint_available,
        current_score: status.current_score.map(|score| score.to_string()),
        best_score: status.best_score.map(|score| score.to_string()),
        telemetry: TelemetryDto::from_runtime(status.telemetry),
    }))
}

pub(super) async fn get_plan_session_result(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<SolveResultResponse>, StatusCode> {
    let record = session_record(&state, &id)?;
    let snapshot = state
        .solver
        .get_snapshot(&record.active_job_id, None)
        .map_err(status_from_solver_error)?;
    let status = lifecycle_to_solve_status(crate::api::dto::lifecycle_state_label(
        snapshot.lifecycle_state,
    ));
    Ok(Json(solve_result_from_plan(
        record.active_job_id,
        status,
        &snapshot.solution,
    )))
}

pub(super) async fn get_plan_session_snapshot(
    State(state): State<Arc<AppState>>,
    Path((id, revision)): Path<(String, String)>,
) -> Result<Json<JobSnapshotDto>, StatusCode> {
    let record = session_record(&state, &id)?;
    let snapshot = state
        .solver
        .get_snapshot(&record.active_job_id, parse_revision(&revision)?)
        .map_err(status_from_solver_error)?;
    Ok(Json(JobSnapshotDto::from_snapshot(&snapshot)))
}

pub(super) async fn get_plan_session_analysis(
    State(state): State<Arc<AppState>>,
    Path((id, revision)): Path<(String, String)>,
) -> Result<Json<JobAnalysisDto>, StatusCode> {
    let record = session_record(&state, &id)?;
    let snapshot_analysis = state
        .solver
        .analyze_snapshot(&record.active_job_id, parse_revision(&revision)?)
        .map_err(status_from_solver_error)?;
    let analysis = analysis_response(&snapshot_analysis.analysis);
    Ok(Json(JobAnalysisDto::from_snapshot_analysis(
        &snapshot_analysis,
        analysis,
    )))
}

pub(super) async fn repair_plan_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(request): Json<RepairRequest>,
) -> Result<Json<RepairResponse>, StatusCode> {
    let record = session_record(&state, &id)?;
    let _repair_policy = request.repair_policy.as_ref();
    let requested_baseline_revision = request
        .baseline_revision
        .as_ref()
        .map(revision_value_to_string)
        .transpose()?;
    let snapshot = state
        .solver
        .get_snapshot(
            &record.active_job_id,
            requested_baseline_revision
                .as_deref()
                .map(parse_revision)
                .transpose()?
                .flatten(),
        )
        .map_err(status_from_solver_error)?;
    let baseline_revision = snapshot.snapshot_revision;
    let repair_from_plan = PlanDto::from_plan(&snapshot.solution);
    let mut repaired_plan = snapshot.solution;
    apply_repair_event(&mut repaired_plan, &request.event)?;

    let repair_job_id = state
        .solver
        .start_job(repaired_plan)
        .map_err(status_from_solver_error)?;
    let status = state
        .solver
        .get_status(&repair_job_id)
        .map_err(status_from_solver_error)?;

    {
        let mut sessions = state.plan_sessions.write();
        let record = sessions.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
        record.active_job_id = repair_job_id.clone();
        record.revisions.push(PlanSessionRevision {
            revision: baseline_revision.to_string(),
            job_id: snapshot.job_id.to_string(),
            snapshot_revision: Some(baseline_revision),
            plan: repair_from_plan,
        });
        record.repair_links.push(PlanSessionRepairLink {
            from_revision: baseline_revision.to_string(),
            to_job_id: repair_job_id.clone(),
        });
        record.latest_repair_job_id = Some(repair_job_id.clone());
        record.latest_repair_mode = Some("derived_resolve_with_lineage");
    }

    Ok(Json(RepairResponse {
        plan_session_id: id.clone(),
        baseline_revision: Some(baseline_revision),
        repair_job_id,
        repair_mode: "derived_resolve_with_lineage",
        lifecycle_state: crate::api::dto::lifecycle_state_label(status.lifecycle_state),
        compare_url: format!(
            "/plan-sessions/{id}/revisions/{}/compare/next",
            baseline_revision
        ),
        status_url: format!("/plan-sessions/{id}/status"),
    }))
}

fn revision_value_to_string(value: &Value) -> Result<String, StatusCode> {
    if let Some(revision) = value.as_u64() {
        Ok(revision.to_string())
    } else if let Some(revision) = value.as_str() {
        Ok(revision.to_string())
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

pub(super) async fn compare_plan_session_revisions(
    State(state): State<Arc<AppState>>,
    Path((id, from_revision, to_revision)): Path<(String, String, String)>,
) -> Result<Json<RevisionCompareResponse>, StatusCode> {
    let record = session_record(&state, &id)?;
    let before = resolve_session_plan(&state, &record, &from_revision)?;
    let after = resolve_compare_target(&state, &record, &from_revision, &to_revision)?;
    let metrics = fleet_metrics(&after);
    let diff = revision_diff(&before, &after);
    let requires_approval = metrics.readiness_shortfall_units > 0
        || diff.moved_assignments > 5
        || !diff.affected_constraints.is_empty();
    let approval_reason = requires_approval.then(|| {
        if metrics.readiness_shortfall_units > 0 {
            "Readiness floor pressure remains after repair".to_string()
        } else if diff.moved_assignments > 5 {
            "More than five assignments moved during repair".to_string()
        } else {
            format!(
                "Affected constraints: {}",
                diff.affected_constraints.join(", ")
            )
        }
    });

    Ok(Json(RevisionCompareResponse {
        plan_session_id: id,
        from_revision,
        to_revision,
        metrics,
        diff,
        requires_approval,
        approval_reason,
    }))
}

fn resolve_session_plan(
    state: &AppState,
    record: &PlanSessionRecord,
    revision: &str,
) -> Result<crate::domain::Plan, StatusCode> {
    if revision == "baseline" {
        return record
            .baseline_plan
            .to_domain()
            .map_err(|_| StatusCode::CONFLICT);
    }
    if let Some(stored) = record
        .revisions
        .iter()
        .rev()
        .find(|stored| stored.revision == revision)
    {
        return stored.plan.to_domain().map_err(|_| StatusCode::CONFLICT);
    }
    let snapshot = state
        .solver
        .get_snapshot(&record.active_job_id, parse_revision(revision)?)
        .map_err(status_from_solver_error)?;
    Ok(snapshot.solution)
}

fn resolve_compare_target(
    state: &AppState,
    record: &PlanSessionRecord,
    from_revision: &str,
    to_revision: &str,
) -> Result<crate::domain::Plan, StatusCode> {
    let job_id = if to_revision == "next" {
        record
            .repair_links
            .iter()
            .rev()
            .find(|link| link.from_revision == from_revision)
            .map(|link| link.to_job_id.as_str())
            .ok_or(StatusCode::NOT_FOUND)?
    } else {
        record.active_job_id.as_str()
    };
    let snapshot = state
        .solver
        .get_snapshot(
            job_id,
            if to_revision == "next" || to_revision == "latest" {
                None
            } else {
                parse_revision(to_revision)?
            },
        )
        .map_err(status_from_solver_error)?;
    Ok(snapshot.solution)
}
