use serde::Serialize;
use std::time::Duration;

use solverforge::{
    HardMediumSoftScore, SolverEventMetadata, SolverLifecycleState, SolverSnapshot, SolverStatus,
    SolverTelemetry, SolverTerminalReason,
};

use crate::api::PlanDto;
use crate::domain::Plan;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TelemetryPayload {
    elapsed_ms: u64,
    step_count: u64,
    moves_generated: u64,
    moves_evaluated: u64,
    moves_accepted: u64,
    score_calculations: u64,
    generation_ms: u64,
    evaluation_ms: u64,
    moves_per_second: u64,
    acceptance_rate: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JobEventPayload {
    id: String,
    job_id: String,
    event_type: &'static str,
    event_sequence: u64,
    lifecycle_state: &'static str,
    terminal_reason: Option<&'static str>,
    telemetry: TelemetryPayload,
    current_score: Option<String>,
    best_score: Option<String>,
    snapshot_revision: Option<u64>,
    solution: Option<PlanDto>,
    error: Option<String>,
}

pub(super) fn status_event_payload(
    job_id: usize,
    event_type: &'static str,
    status: &SolverStatus<HardMediumSoftScore>,
) -> String {
    serialize_payload(JobEventPayload {
        id: job_id.to_string(),
        job_id: job_id.to_string(),
        event_type,
        event_sequence: status.event_sequence,
        lifecycle_state: lifecycle_state_label(status.lifecycle_state),
        terminal_reason: status.terminal_reason.map(terminal_reason_label),
        telemetry: telemetry_payload(&status.telemetry),
        current_score: status.current_score.map(|score| score.to_string()),
        best_score: status.best_score.map(|score| score.to_string()),
        snapshot_revision: status.latest_snapshot_revision,
        solution: None,
        error: None,
    })
}

pub(super) fn snapshot_status_event_payload(
    job_id: usize,
    event_type: &'static str,
    status: &SolverStatus<HardMediumSoftScore>,
    snapshot: &SolverSnapshot<Plan>,
) -> String {
    serialize_payload(JobEventPayload {
        id: job_id.to_string(),
        job_id: job_id.to_string(),
        event_type,
        event_sequence: status.event_sequence,
        lifecycle_state: lifecycle_state_label(status.lifecycle_state),
        terminal_reason: status.terminal_reason.map(terminal_reason_label),
        telemetry: telemetry_payload(&status.telemetry),
        current_score: status
            .current_score
            .or(snapshot.current_score)
            .map(|score| score.to_string()),
        best_score: status
            .best_score
            .or(snapshot.best_score)
            .map(|score| score.to_string()),
        snapshot_revision: Some(snapshot.snapshot_revision),
        solution: Some(PlanDto::from_plan(&snapshot.solution)),
        error: None,
    })
}

pub(super) fn event_payload(
    job_id: usize,
    event_type: &'static str,
    metadata: &SolverEventMetadata<HardMediumSoftScore>,
    solution: Option<&Plan>,
    error: Option<&str>,
) -> String {
    serialize_payload(JobEventPayload {
        id: job_id.to_string(),
        job_id: job_id.to_string(),
        event_type,
        event_sequence: metadata.event_sequence,
        lifecycle_state: lifecycle_state_label(metadata.lifecycle_state),
        terminal_reason: metadata.terminal_reason.map(terminal_reason_label),
        telemetry: telemetry_payload(&metadata.telemetry),
        current_score: metadata.current_score.map(|score| score.to_string()),
        best_score: metadata.best_score.map(|score| score.to_string()),
        snapshot_revision: metadata.snapshot_revision,
        solution: solution.map(PlanDto::from_plan),
        error: error.map(ToOwned::to_owned),
    })
}

pub(super) fn bootstrap_event_type(state: SolverLifecycleState) -> &'static str {
    match state {
        SolverLifecycleState::Solving => "progress",
        SolverLifecycleState::PauseRequested => "pause_requested",
        SolverLifecycleState::Paused => "paused",
        SolverLifecycleState::Completed => "completed",
        SolverLifecycleState::Cancelled => "cancelled",
        SolverLifecycleState::Failed => "failed",
    }
}

pub(super) fn bootstrap_snapshot_event_type(state: SolverLifecycleState) -> &'static str {
    match state {
        SolverLifecycleState::Solving => "best_solution",
        other => bootstrap_event_type(other),
    }
}

fn serialize_payload(payload: JobEventPayload) -> String {
    serde_json::to_string(&payload).expect("failed to serialize solver lifecycle payload")
}

fn telemetry_payload(telemetry: &SolverTelemetry) -> TelemetryPayload {
    TelemetryPayload {
        elapsed_ms: duration_to_millis(telemetry.elapsed),
        step_count: telemetry.step_count,
        moves_generated: telemetry.moves_generated,
        moves_evaluated: telemetry.moves_evaluated,
        moves_accepted: telemetry.moves_accepted,
        score_calculations: telemetry.score_calculations,
        generation_ms: duration_to_millis(telemetry.generation_time),
        evaluation_ms: duration_to_millis(telemetry.evaluation_time),
        moves_per_second: whole_units_per_second(telemetry.moves_evaluated, telemetry.elapsed),
        acceptance_rate: derive_acceptance_rate(
            telemetry.moves_accepted,
            telemetry.moves_evaluated,
        ),
    }
}

fn lifecycle_state_label(state: SolverLifecycleState) -> &'static str {
    match state {
        SolverLifecycleState::Solving => "SOLVING",
        SolverLifecycleState::PauseRequested => "PAUSE_REQUESTED",
        SolverLifecycleState::Paused => "PAUSED",
        SolverLifecycleState::Completed => "COMPLETED",
        SolverLifecycleState::Cancelled => "CANCELLED",
        SolverLifecycleState::Failed => "FAILED",
    }
}

fn terminal_reason_label(reason: SolverTerminalReason) -> &'static str {
    match reason {
        SolverTerminalReason::Completed => "completed",
        SolverTerminalReason::TerminatedByConfig => "terminated_by_config",
        SolverTerminalReason::Cancelled => "cancelled",
        SolverTerminalReason::Failed => "failed",
    }
}

fn duration_to_millis(duration: Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}

fn whole_units_per_second(count: u64, elapsed: Duration) -> u64 {
    let nanos = elapsed.as_nanos();
    if nanos == 0 {
        0
    } else {
        let per_second = u128::from(count)
            .saturating_mul(1_000_000_000)
            .checked_div(nanos)
            .unwrap_or(0);
        per_second.min(u128::from(u64::MAX)) as u64
    }
}

fn derive_acceptance_rate(moves_accepted: u64, moves_evaluated: u64) -> f64 {
    if moves_evaluated == 0 {
        0.0
    } else {
        moves_accepted as f64 / moves_evaluated as f64
    }
}
