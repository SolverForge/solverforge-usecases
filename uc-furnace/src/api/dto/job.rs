use serde::Serialize;
use solverforge::{
    HardSoftScore, SelectorTelemetry, SolverLifecycleState, SolverSnapshot, SolverStatus,
    SolverTelemetry, SolverTerminalReason,
};
use std::time::Duration;

use crate::domain::Plan;

use super::plan::PlanDto;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryDto {
    pub elapsed_ms: u64,
    pub step_count: u64,
    pub moves_generated: u64,
    pub moves_evaluated: u64,
    pub moves_accepted: u64,
    pub moves_applied: u64,
    pub moves_not_doable: u64,
    pub moves_acceptor_rejected: u64,
    pub moves_forager_ignored: u64,
    pub moves_hard_improving: u64,
    pub moves_hard_neutral: u64,
    pub moves_hard_worse: u64,
    pub conflict_repair_provider_generated: u64,
    pub conflict_repair_duplicate_filtered: u64,
    pub conflict_repair_illegal_filtered: u64,
    pub conflict_repair_not_doable_filtered: u64,
    pub conflict_repair_hard_improving: u64,
    pub conflict_repair_exposed: u64,
    pub score_calculations: u64,
    pub construction_slots_assigned: u64,
    pub construction_slots_kept: u64,
    pub construction_slots_no_doable: u64,
    pub generation_time_ms: u64,
    pub evaluation_time_ms: u64,
    pub moves_per_second: u64,
    pub acceptance_rate: f64,
    pub selectors: Vec<SelectorTelemetryDto>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SelectorTelemetryDto {
    pub selector_index: usize,
    pub selector_label: String,
    pub moves_generated: u64,
    pub moves_evaluated: u64,
    pub moves_accepted: u64,
    pub moves_applied: u64,
    pub moves_not_doable: u64,
    pub moves_acceptor_rejected: u64,
    pub moves_forager_ignored: u64,
    pub moves_hard_improving: u64,
    pub moves_hard_neutral: u64,
    pub moves_hard_worse: u64,
    pub conflict_repair_provider_generated: u64,
    pub conflict_repair_duplicate_filtered: u64,
    pub conflict_repair_illegal_filtered: u64,
    pub conflict_repair_not_doable_filtered: u64,
    pub conflict_repair_hard_improving: u64,
    pub conflict_repair_exposed: u64,
    pub generation_time_ms: u64,
    pub evaluation_time_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobSummaryDto {
    pub id: String,
    pub lifecycle_state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_reason: Option<&'static str>,
    pub checkpoint_available: bool,
    pub event_sequence: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_revision: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_score: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_score: Option<String>,
    pub telemetry: TelemetryDto,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobSnapshotDto {
    pub job_id: String,
    pub snapshot_revision: u64,
    pub lifecycle_state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_reason: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_score: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_score: Option<String>,
    pub telemetry: TelemetryDto,
    pub solution: PlanDto,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConstraintMatchDto {
    pub score: String,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConstraintSummaryDto {
    pub name: String,
    #[serde(rename = "type")]
    pub constraint_type: &'static str,
    pub weight: String,
    pub score: String,
    pub match_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailedScoreAnalysisDto {
    pub score: String,
    pub constraints: Vec<ConstraintSummaryDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobAnalysisDto {
    pub job_id: String,
    pub snapshot_revision: u64,
    pub lifecycle_state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_reason: Option<&'static str>,
    pub analysis: DetailedScoreAnalysisDto,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConstraintAnalysisDto {
    pub name: String,
    #[serde(rename = "type")]
    pub constraint_type: &'static str,
    pub weight: String,
    pub score: String,
    pub match_count: usize,
    pub matches: Vec<ConstraintMatchDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobConstraintAnalysisDto {
    pub job_id: String,
    pub snapshot_revision: u64,
    pub lifecycle_state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_reason: Option<&'static str>,
    pub score: String,
    pub constraint: ConstraintAnalysisDto,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobLifecycleEventDto {
    pub event_type: &'static str,
    pub job_id: String,
    pub event_sequence: u64,
    pub lifecycle_state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_reason: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_revision: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_score: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_score: Option<String>,
    pub telemetry: TelemetryDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solution: Option<PlanDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl JobLifecycleEventDto {
    pub fn is_terminal(&self) -> bool {
        matches!(self.event_type, "completed" | "cancelled" | "failed")
    }
}

pub fn status_to_dto(status: SolverStatus<HardSoftScore>) -> JobSummaryDto {
    let (lifecycle_state, terminal_reason) = status_lifecycle_fields(&status);
    JobSummaryDto {
        id: status.job_id.to_string(),
        lifecycle_state,
        terminal_reason,
        checkpoint_available: status.checkpoint_available,
        event_sequence: status.event_sequence,
        snapshot_revision: status.latest_snapshot_revision,
        current_score: format_score(status.current_score),
        best_score: format_score(status.best_score),
        telemetry: telemetry_to_dto(status.telemetry),
    }
}

pub fn snapshot_to_dto(
    snapshot: SolverSnapshot<Plan>,
    status: &SolverStatus<HardSoftScore>,
) -> JobSnapshotDto {
    let (lifecycle_state, terminal_reason) = status_lifecycle_fields(status);
    JobSnapshotDto {
        job_id: snapshot.job_id.to_string(),
        snapshot_revision: snapshot.snapshot_revision,
        lifecycle_state,
        terminal_reason,
        current_score: format_score(snapshot.current_score),
        best_score: format_score(snapshot.best_score),
        telemetry: telemetry_to_dto(snapshot.telemetry),
        solution: PlanDto::from_plan(&snapshot.solution),
    }
}

pub fn status_lifecycle_fields(
    status: &SolverStatus<HardSoftScore>,
) -> (&'static str, Option<&'static str>) {
    (
        lifecycle_state_label(status.lifecycle_state),
        status.terminal_reason.map(terminal_reason_label),
    )
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

pub fn telemetry_to_dto(telemetry: SolverTelemetry) -> TelemetryDto {
    TelemetryDto {
        elapsed_ms: duration_to_millis(telemetry.elapsed),
        step_count: telemetry.step_count,
        moves_generated: telemetry.moves_generated,
        moves_evaluated: telemetry.moves_evaluated,
        moves_accepted: telemetry.moves_accepted,
        moves_applied: telemetry.moves_applied,
        moves_not_doable: telemetry.moves_not_doable,
        moves_acceptor_rejected: telemetry.moves_acceptor_rejected,
        moves_forager_ignored: telemetry.moves_forager_ignored,
        moves_hard_improving: telemetry.moves_hard_improving,
        moves_hard_neutral: telemetry.moves_hard_neutral,
        moves_hard_worse: telemetry.moves_hard_worse,
        conflict_repair_provider_generated: telemetry.conflict_repair_provider_generated,
        conflict_repair_duplicate_filtered: telemetry.conflict_repair_duplicate_filtered,
        conflict_repair_illegal_filtered: telemetry.conflict_repair_illegal_filtered,
        conflict_repair_not_doable_filtered: telemetry.conflict_repair_not_doable_filtered,
        conflict_repair_hard_improving: telemetry.conflict_repair_hard_improving,
        conflict_repair_exposed: telemetry.conflict_repair_exposed,
        score_calculations: telemetry.score_calculations,
        construction_slots_assigned: telemetry.construction_slots_assigned,
        construction_slots_kept: telemetry.construction_slots_kept,
        construction_slots_no_doable: telemetry.construction_slots_no_doable,
        generation_time_ms: duration_to_millis(telemetry.generation_time),
        evaluation_time_ms: duration_to_millis(telemetry.evaluation_time),
        moves_per_second: whole_units_per_second(telemetry.moves_evaluated, telemetry.elapsed),
        acceptance_rate: derive_acceptance_rate(
            telemetry.moves_accepted,
            telemetry.moves_evaluated,
        ),
        selectors: telemetry
            .selector_telemetry
            .into_iter()
            .map(selector_telemetry_to_dto)
            .collect(),
    }
}

fn selector_telemetry_to_dto(telemetry: SelectorTelemetry) -> SelectorTelemetryDto {
    SelectorTelemetryDto {
        selector_index: telemetry.selector_index,
        selector_label: telemetry.selector_label,
        moves_generated: telemetry.moves_generated,
        moves_evaluated: telemetry.moves_evaluated,
        moves_accepted: telemetry.moves_accepted,
        moves_applied: telemetry.moves_applied,
        moves_not_doable: telemetry.moves_not_doable,
        moves_acceptor_rejected: telemetry.moves_acceptor_rejected,
        moves_forager_ignored: telemetry.moves_forager_ignored,
        moves_hard_improving: telemetry.moves_hard_improving,
        moves_hard_neutral: telemetry.moves_hard_neutral,
        moves_hard_worse: telemetry.moves_hard_worse,
        conflict_repair_provider_generated: telemetry.conflict_repair_provider_generated,
        conflict_repair_duplicate_filtered: telemetry.conflict_repair_duplicate_filtered,
        conflict_repair_illegal_filtered: telemetry.conflict_repair_illegal_filtered,
        conflict_repair_not_doable_filtered: telemetry.conflict_repair_not_doable_filtered,
        conflict_repair_hard_improving: telemetry.conflict_repair_hard_improving,
        conflict_repair_exposed: telemetry.conflict_repair_exposed,
        generation_time_ms: duration_to_millis(telemetry.generation_time),
        evaluation_time_ms: duration_to_millis(telemetry.evaluation_time),
    }
}

pub fn format_score(score: Option<HardSoftScore>) -> Option<String> {
    score.map(|value| value.to_string())
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
