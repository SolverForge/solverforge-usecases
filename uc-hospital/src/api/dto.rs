//! Transport DTOs that turn domain/runtime types into beginner-friendly JSON.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use solverforge::{
    HardSoftDecimalScore, ScoreAnalysis, SolverLifecycleState, SolverSnapshot,
    SolverSnapshotAnalysis, SolverStatus, SolverTelemetry, SolverTerminalReason,
};
use std::time::Duration;

use crate::domain::Plan;

/// Thin JSON wrapper around the planning solution.
///
/// We keep the plan fields flattened so the API payload reads like the domain
/// model instead of like a transport envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanDto {
    #[serde(flatten)]
    pub fields: Map<String, Value>,
    #[serde(default)]
    pub score: Option<String>,
}

/// One constraint row shown in the Analyze modal.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConstraintAnalysisDto {
    pub name: String,
    pub weight: String,
    pub score: String,
    pub match_count: usize,
}

/// Top-level analysis payload returned by `/jobs/{id}/analysis`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeResponse {
    pub score: String,
    pub constraints: Vec<ConstraintAnalysisDto>,
}

/// UI-facing telemetry summary derived from exact runtime telemetry.
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

/// Compact job summary returned by `/jobs/{id}` and `/jobs/{id}/status`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobSummaryDto {
    pub id: String,
    pub job_id: String,
    pub lifecycle_state: &'static str,
    pub terminal_reason: Option<&'static str>,
    pub checkpoint_available: bool,
    pub event_sequence: u64,
    pub snapshot_revision: Option<u64>,
    pub current_score: Option<String>,
    pub best_score: Option<String>,
    pub telemetry: TelemetryDto,
}

/// Snapshot payload returned by `/jobs/{id}/snapshot`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobSnapshotDto {
    pub id: String,
    pub job_id: String,
    pub snapshot_revision: u64,
    pub lifecycle_state: &'static str,
    pub terminal_reason: Option<&'static str>,
    pub current_score: Option<String>,
    pub best_score: Option<String>,
    pub telemetry: TelemetryDto,
    pub solution: PlanDto,
}

/// Analysis payload tied to a specific retained snapshot revision.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobAnalysisDto {
    pub id: String,
    pub job_id: String,
    pub snapshot_revision: u64,
    pub lifecycle_state: &'static str,
    pub terminal_reason: Option<&'static str>,
    pub analysis: AnalyzeResponse,
}

impl PlanDto {
    /// Captures the domain plan as a JSON object while keeping `score` explicit.
    pub fn from_plan(plan: &Plan) -> Self {
        let mut fields = plan.to_transport_fields();
        fields.remove("score");

        Self {
            fields,
            score: plan.score.map(|score| score.to_string()),
        }
    }

    /// Rebuilds the normalized domain model from the flattened JSON payload.
    pub fn to_domain(&self) -> Result<Plan, serde_json::Error> {
        Plan::from_transport_fields(self.fields.clone())
    }
}

impl TelemetryDto {
    /// Projects exact runtime telemetry into the fields the browser displays.
    pub fn from_runtime(telemetry: &SolverTelemetry) -> Self {
        Self {
            elapsed_ms: duration_millis_u64(telemetry.elapsed),
            step_count: telemetry.step_count,
            moves_generated: telemetry.moves_generated,
            moves_evaluated: telemetry.moves_evaluated,
            moves_accepted: telemetry.moves_accepted,
            score_calculations: telemetry.score_calculations,
            generation_ms: duration_millis_u64(telemetry.generation_time),
            evaluation_ms: duration_millis_u64(telemetry.evaluation_time),
            moves_per_second: moves_per_second(telemetry.moves_evaluated, telemetry.elapsed),
            acceptance_rate: acceptance_rate(telemetry.moves_accepted, telemetry.moves_evaluated),
        }
    }
}

/// Small helper so the JSON surface stays integer-based.
fn duration_millis_u64(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

/// Derives whole moves per second from exact elapsed time.
fn moves_per_second(moves_evaluated: u64, elapsed: Duration) -> u64 {
    let nanos = elapsed.as_nanos();
    if nanos == 0 {
        return 0;
    }

    let rate = u128::from(moves_evaluated)
        .saturating_mul(1_000_000_000)
        .checked_div(nanos)
        .unwrap_or(0);
    u64::try_from(rate).unwrap_or(u64::MAX)
}

/// Derives acceptance as a decimal fraction for easy frontend display.
fn acceptance_rate(moves_accepted: u64, moves_evaluated: u64) -> f64 {
    if moves_evaluated == 0 {
        0.0
    } else {
        moves_accepted as f64 / moves_evaluated as f64
    }
}

impl JobSummaryDto {
    /// Converts the runtime status summary into the JSON contract used by the app.
    pub fn from_status(job_id: usize, status: &SolverStatus<HardSoftDecimalScore>) -> Self {
        Self {
            id: job_id.to_string(),
            job_id: job_id.to_string(),
            lifecycle_state: lifecycle_state_label(status.lifecycle_state),
            terminal_reason: status.terminal_reason.map(terminal_reason_label),
            checkpoint_available: status.checkpoint_available,
            event_sequence: status.event_sequence,
            snapshot_revision: status.latest_snapshot_revision,
            current_score: status.current_score.map(|score| score.to_string()),
            best_score: status.best_score.map(|score| score.to_string()),
            telemetry: TelemetryDto::from_runtime(&status.telemetry),
        }
    }
}

impl JobSnapshotDto {
    /// Converts a retained snapshot into the richer snapshot JSON payload.
    pub fn from_snapshot(snapshot: &SolverSnapshot<Plan>) -> Self {
        Self {
            id: snapshot.job_id.to_string(),
            job_id: snapshot.job_id.to_string(),
            snapshot_revision: snapshot.snapshot_revision,
            lifecycle_state: lifecycle_state_label(snapshot.lifecycle_state),
            terminal_reason: snapshot.terminal_reason.map(terminal_reason_label),
            current_score: snapshot.current_score.map(|score| score.to_string()),
            best_score: snapshot.best_score.map(|score| score.to_string()),
            telemetry: TelemetryDto::from_runtime(&snapshot.telemetry),
            solution: PlanDto::from_plan(&snapshot.solution),
        }
    }
}

impl JobAnalysisDto {
    /// Packages exact snapshot analysis together with snapshot identity metadata.
    pub fn from_snapshot_analysis(
        snapshot: &SolverSnapshotAnalysis<HardSoftDecimalScore>,
        analysis: AnalyzeResponse,
    ) -> Self {
        Self {
            id: snapshot.job_id.to_string(),
            job_id: snapshot.job_id.to_string(),
            snapshot_revision: snapshot.snapshot_revision,
            lifecycle_state: lifecycle_state_label(snapshot.lifecycle_state),
            terminal_reason: snapshot.terminal_reason.map(terminal_reason_label),
            analysis,
        }
    }
}

/// Converts SolverForge's detailed score analysis into the browser response shape.
pub fn analysis_response(analysis: &ScoreAnalysis<HardSoftDecimalScore>) -> AnalyzeResponse {
    AnalyzeResponse {
        score: analysis.score.to_string(),
        constraints: analysis
            .constraints
            .iter()
            .map(|constraint| ConstraintAnalysisDto {
                name: constraint.name.clone(),
                weight: constraint.weight.to_string(),
                score: constraint.score.to_string(),
                match_count: constraint.match_count,
            })
            .collect(),
    }
}

/// Re-exports lifecycle labels so routes and tests share one mapping.
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

/// Re-exports terminal labels so routes and tests share one mapping.
pub fn terminal_reason_label(reason: SolverTerminalReason) -> &'static str {
    match reason {
        SolverTerminalReason::Completed => "completed",
        SolverTerminalReason::TerminatedByConfig => "terminated_by_config",
        SolverTerminalReason::Cancelled => "cancelled",
        SolverTerminalReason::Failed => "failed",
    }
}

#[cfg(test)]
mod tests;
