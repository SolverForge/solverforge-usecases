use parking_lot::Mutex;
use solverforge::{
    HardSoftScore, SolverEvent, SolverEventMetadata, SolverLifecycleState, SolverManager,
    SolverManagerError, SolverSnapshot, SolverSnapshotAnalysis, SolverStatus,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc};

use crate::api::{
    lifecycle_state_label, telemetry_to_dto, terminal_reason_label, JobLifecycleEventDto, PlanDto,
};
use crate::domain::Plan;

mod events;
#[cfg(test)]
mod tests;

use events::event_to_dto;

static MANAGER: SolverManager<Plan> = SolverManager::new();
const BEST_SOLUTION_STREAM_INTERVAL: Duration = Duration::from_millis(1_000);

#[derive(Default)]
pub struct SolverService {
    job_streams: Mutex<HashMap<usize, broadcast::Sender<JobLifecycleEventDto>>>,
}

impl SolverService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_job(&self, plan: Plan) -> Result<String, SolverManagerError> {
        let (job_id, receiver) = MANAGER.solve(plan)?;
        self.attach_job_stream(job_id, receiver);
        Ok(job_id.to_string())
    }

    pub fn subscribe(
        &self,
        id: &str,
    ) -> Result<broadcast::Receiver<JobLifecycleEventDto>, SolverManagerError> {
        let job_id = parse_job_id(id)?;
        self.job_streams
            .lock()
            .get(&job_id)
            .map(broadcast::Sender::subscribe)
            .ok_or(SolverManagerError::JobNotFound { job_id })
    }

    pub fn bootstrap_event(&self, id: &str) -> Result<JobLifecycleEventDto, SolverManagerError> {
        let job_id = parse_job_id(id)?;
        let status = MANAGER.get_status(job_id)?;
        if let Some(revision) = status.latest_snapshot_revision {
            let snapshot = MANAGER.get_snapshot(job_id, Some(revision))?;
            return Ok(snapshot_status_event_dto(
                bootstrap_snapshot_event_type(status.lifecycle_state),
                status,
                snapshot,
            ));
        }
        Ok(status_event_dto(
            bootstrap_event_type(status.lifecycle_state),
            status,
        ))
    }

    pub fn get_status(&self, id: &str) -> Result<SolverStatus<HardSoftScore>, SolverManagerError> {
        MANAGER.get_status(parse_job_id(id)?)
    }

    pub fn get_snapshot(
        &self,
        id: &str,
        snapshot_revision: Option<u64>,
    ) -> Result<SolverSnapshot<Plan>, SolverManagerError> {
        MANAGER.get_snapshot(parse_job_id(id)?, snapshot_revision)
    }

    pub fn analyze_snapshot(
        &self,
        id: &str,
        snapshot_revision: Option<u64>,
    ) -> Result<SolverSnapshotAnalysis<HardSoftScore>, SolverManagerError> {
        MANAGER.analyze_snapshot(parse_job_id(id)?, snapshot_revision)
    }

    pub fn pause(&self, id: &str) -> Result<(), SolverManagerError> {
        MANAGER.pause(parse_job_id(id)?)
    }

    pub fn resume(&self, id: &str) -> Result<(), SolverManagerError> {
        MANAGER.resume(parse_job_id(id)?)
    }

    pub fn cancel(&self, id: &str) -> Result<(), SolverManagerError> {
        MANAGER.cancel(parse_job_id(id)?)
    }

    pub fn delete(&self, id: &str) -> Result<(), SolverManagerError> {
        let job_id = parse_job_id(id)?;
        MANAGER.delete(job_id)?;
        self.remove_job_stream(job_id);
        Ok(())
    }

    fn attach_job_stream(
        &self,
        job_id: usize,
        receiver: mpsc::UnboundedReceiver<SolverEvent<Plan>>,
    ) {
        let (sender, _) = broadcast::channel(64);
        self.job_streams.lock().insert(job_id, sender.clone());
        tokio::spawn(bridge_solver_events(receiver, sender));
    }

    fn remove_job_stream(&self, job_id: usize) {
        self.job_streams.lock().remove(&job_id);
    }
}

async fn bridge_solver_events(
    mut receiver: mpsc::UnboundedReceiver<SolverEvent<Plan>>,
    sender: broadcast::Sender<JobLifecycleEventDto>,
) {
    let mut last_best_solution_sent_at = None;
    while let Some(event) = receiver.recv().await {
        if matches!(event, SolverEvent::BestSolution { .. }) {
            let now = Instant::now();
            if !should_publish_best_solution(last_best_solution_sent_at, now) {
                continue;
            }
            last_best_solution_sent_at = Some(now);
        }

        let snapshot_score = if event_carries_solution(&event) {
            None
        } else {
            exact_snapshot_score(event.metadata())
        };
        let dto = event_to_dto(event, snapshot_score);
        let is_terminal = dto.is_terminal();
        let _ = sender.send(dto);
        if is_terminal {
            break;
        }
    }
}

fn should_publish_best_solution(last_sent_at: Option<Instant>, now: Instant) -> bool {
    last_sent_at.is_none_or(|last_sent_at| {
        now.duration_since(last_sent_at) >= BEST_SOLUTION_STREAM_INTERVAL
    })
}

fn parse_job_id(id: &str) -> Result<usize, SolverManagerError> {
    id.parse::<usize>()
        .map_err(|_| SolverManagerError::JobNotFound { job_id: usize::MAX })
}

fn exact_snapshot_score(metadata: &SolverEventMetadata<HardSoftScore>) -> Option<HardSoftScore> {
    metadata
        .snapshot_revision
        .and_then(|revision| {
            MANAGER
                .analyze_snapshot(metadata.job_id, Some(revision))
                .ok()
        })
        .map(|snapshot_analysis| snapshot_analysis.analysis.score)
}

fn event_carries_solution(event: &SolverEvent<Plan>) -> bool {
    matches!(
        event,
        SolverEvent::BestSolution { .. } | SolverEvent::Completed { .. }
    )
}

fn status_event_dto(
    event_type: &'static str,
    status: SolverStatus<HardSoftScore>,
) -> JobLifecycleEventDto {
    JobLifecycleEventDto {
        event_type,
        job_id: status.job_id.to_string(),
        event_sequence: status.event_sequence,
        lifecycle_state: lifecycle_state_label(status.lifecycle_state),
        terminal_reason: status.terminal_reason.map(terminal_reason_label),
        snapshot_revision: status.latest_snapshot_revision,
        current_score: None,
        best_score: None,
        telemetry: telemetry_to_dto(status.telemetry),
        solution: None,
        error: None,
    }
}

fn snapshot_status_event_dto(
    event_type: &'static str,
    status: SolverStatus<HardSoftScore>,
    snapshot: SolverSnapshot<Plan>,
) -> JobLifecycleEventDto {
    let score = solverforge::analyze(&snapshot.solution).score;
    let solution = if event_type == "best_solution" {
        Some(PlanDto::from_plan(&snapshot.solution))
    } else {
        None
    };
    JobLifecycleEventDto {
        event_type,
        job_id: status.job_id.to_string(),
        event_sequence: status.event_sequence,
        lifecycle_state: lifecycle_state_label(status.lifecycle_state),
        terminal_reason: status.terminal_reason.map(terminal_reason_label),
        snapshot_revision: Some(snapshot.snapshot_revision),
        current_score: Some(score.to_string()),
        best_score: Some(score.to_string()),
        telemetry: telemetry_to_dto(status.telemetry),
        solution,
        error: None,
    }
}

fn bootstrap_event_type(state: SolverLifecycleState) -> &'static str {
    match state {
        SolverLifecycleState::Solving => "progress",
        SolverLifecycleState::PauseRequested => "pause_requested",
        SolverLifecycleState::Paused => "paused",
        SolverLifecycleState::Cancelled => "cancelled",
        SolverLifecycleState::Failed => "failed",
        SolverLifecycleState::Completed => "completed",
    }
}

fn bootstrap_snapshot_event_type(state: SolverLifecycleState) -> &'static str {
    match state {
        SolverLifecycleState::Solving => "best_solution",
        other => bootstrap_event_type(other),
    }
}
