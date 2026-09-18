use solverforge::{HardSoftScore, SolverEvent, SolverEventMetadata};

use crate::api::{
    format_score, lifecycle_state_label, telemetry_to_dto, terminal_reason_label,
    JobLifecycleEventDto, PlanDto,
};
use crate::domain::Plan;

pub(super) fn event_to_dto(
    event: SolverEvent<Plan>,
    snapshot_score: Option<HardSoftScore>,
) -> JobLifecycleEventDto {
    match event {
        SolverEvent::Progress { metadata } => {
            dto_from_metadata("progress", metadata, None, snapshot_score)
        }
        SolverEvent::BestSolution { metadata, solution } => {
            let score = solverforge::analyze(&solution).score;
            dto_from_metadata(
                "best_solution",
                metadata,
                Some(PlanDto::from_plan(&solution)),
                Some(score),
            )
        }
        SolverEvent::PauseRequested { metadata } => {
            dto_from_metadata("pause_requested", metadata, None, snapshot_score)
        }
        SolverEvent::Paused { metadata } => {
            dto_from_metadata("paused", metadata, None, snapshot_score)
        }
        SolverEvent::Resumed { metadata } => {
            dto_from_metadata("resumed", metadata, None, snapshot_score)
        }
        SolverEvent::Completed { metadata, solution } => {
            let score = solverforge::analyze(&solution).score;
            dto_from_metadata("completed", metadata, None, Some(score))
        }
        SolverEvent::Cancelled { metadata } => {
            dto_from_metadata("cancelled", metadata, None, snapshot_score)
        }
        SolverEvent::Failed { metadata, error } => {
            let mut dto = dto_from_metadata("failed", metadata, None, snapshot_score);
            dto.error = Some(error);
            dto
        }
    }
}

fn dto_from_metadata(
    event_type: &'static str,
    metadata: SolverEventMetadata<HardSoftScore>,
    solution: Option<PlanDto>,
    display_score: Option<HardSoftScore>,
) -> JobLifecycleEventDto {
    JobLifecycleEventDto {
        event_type,
        job_id: metadata.job_id.to_string(),
        event_sequence: metadata.event_sequence,
        lifecycle_state: lifecycle_state_label(metadata.lifecycle_state),
        terminal_reason: metadata.terminal_reason.map(terminal_reason_label),
        snapshot_revision: metadata.snapshot_revision,
        current_score: format_score(display_score),
        best_score: format_score(display_score),
        telemetry: telemetry_to_dto(metadata.telemetry),
        solution,
        error: None,
    }
}
