use serde::Serialize;
use solverforge::{SolverLifecycleState, SolverTelemetry, SolverTerminalReason};
use std::time::Duration;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryDto {
    pub elapsed_ms: u64,
    pub step_count: u64,
    pub moves_generated: u64,
    pub moves_evaluated: u64,
    pub moves_accepted: u64,
    pub score_calculations: u64,
    pub generation_ms: u64,
    pub evaluation_ms: u64,
    pub moves_per_second: u64,
    pub acceptance_rate: f64,
}

impl TelemetryDto {
    pub fn from_runtime(telemetry: &SolverTelemetry) -> Self {
        Self {
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
}

pub fn lifecycle_state_label(state: SolverLifecycleState) -> &'static str {
    match state {
        SolverLifecycleState::Solving => "SOLVING",
        SolverLifecycleState::PauseRequested => "PAUSE_REQUESTED",
        SolverLifecycleState::Paused => "PAUSED",
        SolverLifecycleState::Completed => "COMPLETED",
        SolverLifecycleState::Cancelled => "CANCELLED",
        SolverLifecycleState::Failed => "FAILED",
    }
}

pub fn terminal_reason_label(reason: SolverTerminalReason) -> &'static str {
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
