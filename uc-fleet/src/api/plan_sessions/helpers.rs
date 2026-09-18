use axum::http::StatusCode;
use serde_json::Value;

use crate::constraints::support::{
    apply_dock_outage, apply_parts_delay, apply_readiness_floor, apply_technician_shortage_window,
    explanation_summary, fleet_metrics, inspection_schedule_assignments, resource_utilization,
    seed_technician_shortage_repair, training_schedule_assignments, weekly_readiness,
    work_package_assignments,
};
use crate::data::{
    baseline_plan, delayed_parts_plan, raised_readiness_plan, technician_shortage_solution_plan,
};
use crate::domain::Plan;

use crate::api::routes::AppState;

use super::types::{
    PlanSessionRecord, RepairEvent, ScenarioRequest, SchedulePayload, SolveResultResponse,
};

pub(super) fn session_record(state: &AppState, id: &str) -> Result<PlanSessionRecord, StatusCode> {
    state
        .plan_sessions
        .read()
        .get(id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)
}

pub(super) fn scenario_plan(request: &ScenarioRequest) -> Result<Plan, StatusCode> {
    if let Some(data) = &request.data {
        if !data.fields.is_empty() {
            return data.to_domain().map_err(|_| StatusCode::BAD_REQUEST);
        }
    }

    match request.scenario_id.as_str() {
        "baseline" | "scenario_fleet_alpha" | "BASELINE" => Ok(baseline_plan()),
        "technician_shortage" | "TECHNICIAN_SHORTAGE" => Ok(technician_shortage_solution_plan()),
        "delayed_parts" | "DELAYED_PARTS" => Ok(delayed_parts_plan()),
        "raised_readiness_floor" | "RAISED_READINESS_FLOOR" => Ok(raised_readiness_plan()),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

pub(super) fn apply_repair_event(plan: &mut Plan, event: &RepairEvent) -> Result<(), StatusCode> {
    // Every disruption must carry the fields its handler needs. Rejecting an
    // incomplete event here keeps a repair from silently re-solving the
    // unmodified plan and reporting a no-op as a successful repair.
    match event.event_type.as_str() {
        "technician_unavailable" | "technician_capacity_drop" | "resource_capacity_changed" => {
            let pool_id = required_string(&event.payload, &["pool_id", "skill_pool_id"])?;
            let delta = required_int(&event.payload, &["delta"])?;
            let start_day = required_int(&event.payload, &["start_day"])?;
            let end_day = required_int(&event.payload, &["end_day"])?;
            apply_technician_shortage_window(plan, &pool_id, delta, start_day, end_day);
            seed_technician_shortage_repair(plan, &pool_id, start_day, end_day);
            Ok(())
        }
        "parts_delay" | "delayed_delivery" => {
            let delivery_id = required_string(&event.payload, &["delivery_id"])?;
            let new_arrival_day =
                required_int(&event.payload, &["new_arrival_day", "arrival_day"])?;
            apply_parts_delay(plan, &delivery_id, new_arrival_day);
            Ok(())
        }
        "readiness_floor_changed" | "policy_override" => {
            let value = required_int(&event.payload, &["value", "min_ready_overall_per_week"])?;
            apply_readiness_floor(plan, value);
            Ok(())
        }
        "dock_outage" => {
            let dock_id = required_string(&event.payload, &["dock_id"])?;
            let start_day = required_int(&event.payload, &["start_day"])?;
            let end_day = required_int(&event.payload, &["end_day"])?;
            apply_dock_outage(plan, &dock_id, start_day, end_day);
            Ok(())
        }
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

fn required_string(payload: &Value, keys: &[&str]) -> Result<String, StatusCode> {
    keys.iter()
        .find_map(|key| string_payload(payload, key))
        .ok_or(StatusCode::BAD_REQUEST)
}

fn required_int(payload: &Value, keys: &[&str]) -> Result<i32, StatusCode> {
    keys.iter()
        .find_map(|key| int_payload(payload, key))
        .ok_or(StatusCode::BAD_REQUEST)
}

pub(super) fn solve_result_from_plan(
    solve_id: String,
    status: &'static str,
    plan: &Plan,
) -> SolveResultResponse {
    SolveResultResponse {
        solve_id,
        status,
        metrics: fleet_metrics(plan),
        schedule: SchedulePayload {
            work_package_assignments: work_package_assignments(plan),
            inspection_assignments: inspection_schedule_assignments(plan),
            training_assignments: training_schedule_assignments(plan),
            weekly_readiness: weekly_readiness(plan),
        },
        resource_utilization: resource_utilization(plan),
        explanations: explanation_summary(plan),
    }
}

pub(super) fn lifecycle_to_solve_status(lifecycle_state: &'static str) -> &'static str {
    match lifecycle_state {
        "SOLVING" | "PAUSE_REQUESTED" => "running",
        "PAUSED" => "paused",
        "COMPLETED" => "completed",
        "CANCELLED" => "cancelled",
        "FAILED" => "failed",
        _ => "pending",
    }
}

pub(super) fn parse_revision(revision: &str) -> Result<Option<u64>, StatusCode> {
    if revision == "latest" {
        Ok(None)
    } else {
        revision
            .parse::<u64>()
            .map(Some)
            .map_err(|_| StatusCode::BAD_REQUEST)
    }
}

pub(super) fn string_payload(payload: &Value, key: &str) -> Option<String> {
    payload.get(key)?.as_str().map(ToString::to_string)
}

pub(super) fn int_payload(payload: &Value, key: &str) -> Option<i32> {
    payload
        .get(key)?
        .as_i64()
        .and_then(|value| i32::try_from(value).ok())
}

pub(super) fn status_from_solver_error(error: solverforge::SolverManagerError) -> StatusCode {
    match error {
        solverforge::SolverManagerError::NoFreeJobSlots => StatusCode::SERVICE_UNAVAILABLE,
        solverforge::SolverManagerError::JobNotFound { .. } => StatusCode::NOT_FOUND,
        solverforge::SolverManagerError::InvalidStateTransition { .. } => StatusCode::CONFLICT,
        solverforge::SolverManagerError::NoSnapshotAvailable { .. } => StatusCode::CONFLICT,
        solverforge::SolverManagerError::SnapshotNotFound { .. } => StatusCode::NOT_FOUND,
    }
}
