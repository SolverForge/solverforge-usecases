//! Retained-job orchestration for field-service solves.
//!
//! SolverForge owns search and scoring. This service owns app-level concerns:
//! registering SSE broadcasters, translating public string ids to runtime job
//! ids, and exposing pause/resume/cancel/delete to the API layer.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

use solverforge::{
    HardSoftScore, SolverEvent, SolverManager, SolverManagerError, SolverSnapshot,
    SolverSnapshotAnalysis, SolverStatus,
};

use super::event_payload::{
    bootstrap_event_type, bootstrap_snapshot_event_type, event_payload,
    snapshot_status_event_payload, status_event_payload,
};
use crate::domain::FieldServicePlan;

// The retained runtime needs a manager with `'static` lifetime because jobs can
// continue after the HTTP handler that started them has returned.
static MANAGER: SolverManager<FieldServicePlan> = SolverManager::new();

struct JobState {
    sse_tx: broadcast::Sender<String>,
}

/// Manages retained solving jobs and broadcasts lifecycle-complete SSE payloads.
pub struct SolverService {
    jobs: Arc<RwLock<HashMap<usize, JobState>>>,
}

impl SolverService {
    /// Creates an empty job registry. The underlying runtime itself is global.
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Starts a retained solve and registers the SSE broadcaster for that job.
    pub fn start_job(&self, plan: FieldServicePlan) -> Result<String, SolverManagerError> {
        let (job_id, receiver) = MANAGER.solve(plan)?;
        let (sse_tx, _) = broadcast::channel(64);

        self.jobs.write().insert(
            job_id,
            JobState {
                sse_tx: sse_tx.clone(),
            },
        );

        let jobs = Arc::clone(&self.jobs);
        tokio::spawn(async move {
            drain_receiver(jobs, job_id, sse_tx, receiver).await;
        });

        Ok(job_id.to_string())
    }

    /// Subscribes a browser client to future live events for a retained job.
    pub fn subscribe(&self, id: &str) -> Option<broadcast::Receiver<String>> {
        let job_id = parse_job_id(id).ok()?;
        self.jobs
            .read()
            .get(&job_id)
            .map(|state| state.sse_tx.subscribe())
    }

    /// Builds the first SSE payload a client should receive after connecting.
    pub fn bootstrap_event(&self, id: &str) -> Result<String, SolverManagerError> {
        let job_id = parse_job_id(id)?;
        let status = MANAGER.get_status(job_id)?;
        if let Some(revision) = status.latest_snapshot_revision {
            let snapshot = MANAGER.get_snapshot(job_id, Some(revision))?;
            return Ok(snapshot_status_event_payload(
                job_id,
                bootstrap_snapshot_event_type(status.lifecycle_state),
                &status,
                &snapshot,
            ));
        }

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
    ) -> Result<SolverSnapshot<FieldServicePlan>, SolverManagerError> {
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
    mut receiver: mpsc::UnboundedReceiver<SolverEvent<FieldServicePlan>>,
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

        if !jobs.read().contains_key(&job_id) {
            return;
        }

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
