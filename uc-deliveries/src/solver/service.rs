use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

use solverforge::{
    HardSoftScore, SolverEvent, SolverManager, SolverManagerError, SolverSnapshot,
    SolverSnapshotAnalysis, SolverStatus,
};

use crate::domain::Plan;

mod runtime_payload;

use runtime_payload::{bootstrap_event_type, event_payload, status_event_payload};

// Static manager — must be 'static for retained job execution.
static MANAGER: SolverManager<Plan> = SolverManager::new();

struct JobState {
    sse_tx: broadcast::Sender<String>,
    last_event: String,
}

/// Manages retained solving jobs and broadcasts lifecycle-complete SSE payloads.
pub struct SolverService {
    jobs: Arc<RwLock<HashMap<usize, JobState>>>,
}

impl SolverService {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn start_job(&self, plan: Plan) -> Result<String, SolverManagerError> {
        let (job_id, receiver) = MANAGER.solve(plan)?;
        let status = MANAGER.get_status(job_id)?;
        let initial_event = status_event_payload(
            job_id,
            bootstrap_event_type(status.lifecycle_state),
            &status,
        );
        let (sse_tx, _) = broadcast::channel(64);

        self.jobs.write().insert(
            job_id,
            JobState {
                sse_tx: sse_tx.clone(),
                last_event: initial_event,
            },
        );

        let jobs = Arc::clone(&self.jobs);
        tokio::spawn(async move {
            drain_receiver(jobs, job_id, sse_tx, receiver).await;
        });

        Ok(job_id.to_string())
    }

    pub fn subscribe(&self, id: &str) -> Option<broadcast::Receiver<String>> {
        let job_id = parse_job_id(id).ok()?;
        self.jobs
            .read()
            .get(&job_id)
            .map(|state| state.sse_tx.subscribe())
    }

    pub fn sse_snapshot(&self, id: &str) -> Option<String> {
        let job_id = parse_job_id(id).ok()?;
        self.jobs
            .read()
            .get(&job_id)
            .map(|state| state.last_event.clone())
    }

    pub fn bootstrap_event(&self, id: &str) -> Result<String, SolverManagerError> {
        if let Some(snapshot) = self.sse_snapshot(id) {
            return Ok(snapshot);
        }

        let job_id = parse_job_id(id)?;
        let status = MANAGER.get_status(job_id)?;
        Ok(status_event_payload(
            job_id,
            bootstrap_event_type(status.lifecycle_state),
            &status,
        ))
    }

    pub fn get_status(&self, id: &str) -> Result<SolverStatus<HardSoftScore>, SolverManagerError> {
        let job_id = parse_job_id(id)?;
        MANAGER.get_status(job_id)
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
        self.jobs.write().remove(&job_id);
        Ok(())
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
}

async fn drain_receiver(
    jobs: Arc<RwLock<HashMap<usize, JobState>>>,
    job_id: usize,
    sse_tx: broadcast::Sender<String>,
    mut receiver: mpsc::UnboundedReceiver<SolverEvent<Plan>>,
) {
    while let Some(event) = receiver.recv().await {
        let payload = match &event {
            SolverEvent::Progress { metadata } => {
                event_payload(job_id, "progress", metadata, None, None)
            }
            SolverEvent::BestSolution { metadata, solution } => {
                event_payload(job_id, "best_solution", metadata, Some(solution), None)
            }
            SolverEvent::PauseRequested { metadata } => {
                event_payload(job_id, "pause_requested", metadata, None, None)
            }
            SolverEvent::Paused { metadata } => {
                event_payload(job_id, "paused", metadata, None, None)
            }
            SolverEvent::Resumed { metadata } => {
                event_payload(job_id, "resumed", metadata, None, None)
            }
            SolverEvent::Completed { metadata, solution } => {
                event_payload(job_id, "completed", metadata, Some(solution), None)
            }
            SolverEvent::Cancelled { metadata } => {
                event_payload(job_id, "cancelled", metadata, None, None)
            }
            SolverEvent::Failed { metadata, error } => {
                event_payload(job_id, "failed", metadata, None, Some(error.as_str()))
            }
        };

        let mut jobs = jobs.write();
        if let Some(state) = jobs.get_mut(&job_id) {
            state.last_event = payload.clone();
        } else {
            return;
        }
        drop(jobs);

        let _ = sse_tx.send(payload);
    }
}

fn parse_job_id(id: &str) -> Result<usize, SolverManagerError> {
    id.parse::<usize>()
        .map_err(|_| SolverManagerError::JobNotFound { job_id: usize::MAX })
}

impl Default for SolverService {
    fn default() -> Self {
        Self::new()
    }
}
