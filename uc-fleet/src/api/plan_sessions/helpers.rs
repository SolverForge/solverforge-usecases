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
    let _event_id = event.event_id.as_deref();
    let _occurred_at = event.occurred_at.as_deref();
    match event.event_type.as_str() {
        "technician_unavailable" | "technician_capacity_drop" | "resource_capacity_changed" => {
            let pool_id = string_payload(&event.payload, "pool_id")
                .or_else(|| string_payload(&event.payload, "skill_pool_id"))
                .unwrap_or_else(|| "ELEC".to_string());
            let delta = int_payload(&event.payload, "delta").unwrap_or(-1);
            let start_day = int_payload(&event.payload, "start_day").unwrap_or(15);
            let end_day = int_payload(&event.payload, "end_day").unwrap_or(21);
            apply_technician_shortage_window(plan, &pool_id, delta, start_day, end_day);
            seed_technician_shortage_repair(plan, &pool_id, start_day, end_day);
            Ok(())
        }
        "parts_delay" | "delayed_delivery" => {
            let delivery_id = string_payload(&event.payload, "delivery_id")
                .unwrap_or_else(|| "DELIV-01".to_string());
            let new_arrival_day = int_payload(&event.payload, "new_arrival_day")
                .or_else(|| int_payload(&event.payload, "arrival_day"))
                .or_else(|| int_payload(&event.payload, "delay_days").map(|delay| 19 + delay))
                .unwrap_or(29);
            apply_parts_delay(plan, &delivery_id, new_arrival_day);
            Ok(())
        }
        "readiness_floor_changed" | "policy_override" => {
            let value = int_payload(&event.payload, "value")
                .or_else(|| int_payload(&event.payload, "min_ready_overall_per_week"))
                .unwrap_or(9);
            apply_readiness_floor(plan, value);
            Ok(())
        }
        "dock_outage" => {
            let dock_id =
                string_payload(&event.payload, "dock_id").unwrap_or_else(|| "D2".to_string());
            let start_day = int_payload(&event.payload, "start_day").unwrap_or(22);
            let end_day = int_payload(&event.payload, "end_day").unwrap_or(31);
            apply_dock_outage(plan, &dock_id, start_day, end_day);
            Ok(())
        }
        _ => Err(StatusCode::BAD_REQUEST),
    }
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
