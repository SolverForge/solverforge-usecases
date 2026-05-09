//! Retained-job runtime orchestration and event translation.
//!
//! The app delegates actual solving to `SolverManager<Plan>`. This file exists
//! to do the app-specific glue around that stock runtime:
//! - create/delete jobs
//! - expose snapshots and analysis
//! - translate stock runtime events into the JSON payload expected by the UI

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

use solverforge::{
    HardSoftDecimalScore, SolverEvent, SolverManager, SolverManagerError, SolverSnapshot,
    SolverSnapshotAnalysis, SolverStatus,
};

use crate::domain::Plan;

mod payload;

use payload::{
    bootstrap_event_type, bootstrap_snapshot_event_type, event_payload,
    snapshot_status_event_payload, status_event_payload,
};

static MANAGER: SolverManager<Plan> = SolverManager::new();

/// In-memory state we keep for each live or retained job.
struct JobState {
    sse_tx: broadcast::Sender<String>,
}

/// Small application facade over the global `SolverManager`.
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

    /// Starts a solve and registers the broadcaster used by SSE subscribers.
    pub fn start_job(&self, plan: Plan) -> Result<String, SolverManagerError> {
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

    /// Builds the first SSE payload a client should see after connecting.
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

    /// Thin pass-through to the runtime's job summary API.
    pub fn get_status(
        &self,
        id: &str,
    ) -> Result<SolverStatus<HardSoftDecimalScore>, SolverManagerError> {
        MANAGER.get_status(parse_job_id(id)?)
    }

    /// Requests an exact retained-runtime pause.
    pub fn pause(&self, id: &str) -> Result<(), SolverManagerError> {
        MANAGER.pause(parse_job_id(id)?)
    }

    /// Resumes a previously paused job from its checkpoint.
    pub fn resume(&self, id: &str) -> Result<(), SolverManagerError> {
        MANAGER.resume(parse_job_id(id)?)
    }

    /// Cancels a live or paused retained job.
    pub fn cancel(&self, id: &str) -> Result<(), SolverManagerError> {
        MANAGER.cancel(parse_job_id(id)?)
    }

    /// Deletes a terminal job from both the runtime and the local SSE cache.
    pub fn delete(&self, id: &str) -> Result<(), SolverManagerError> {
        let job_id = parse_job_id(id)?;
        MANAGER.delete(job_id)?;
        self.jobs.write().remove(&job_id);
        Ok(())
    }

    /// Fetches a retained snapshot, optionally by explicit revision.
    pub fn get_snapshot(
        &self,
        id: &str,
        snapshot_revision: Option<u64>,
    ) -> Result<SolverSnapshot<Plan>, SolverManagerError> {
        MANAGER.get_snapshot(parse_job_id(id)?, snapshot_revision)
    }

    /// Runs exact constraint analysis against a retained snapshot revision.
    pub fn analyze_snapshot(
        &self,
        id: &str,
        snapshot_revision: Option<u64>,
    ) -> Result<SolverSnapshotAnalysis<HardSoftDecimalScore>, SolverManagerError> {
        MANAGER.analyze_snapshot(parse_job_id(id)?, snapshot_revision)
    }
}

/// Background task that converts runtime events into serialized SSE payloads.
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

        if !jobs.read().contains_key(&job_id) {
            return;
        }

        let _ = sse_tx.send(payload);
    }
}

/// Parses the string job id used in HTTP routes into the runtime's numeric key.
fn parse_job_id(id: &str) -> Result<usize, SolverManagerError> {
    id.parse::<usize>()
        .map_err(|_| SolverManagerError::JobNotFound { job_id: usize::MAX })
}

impl Default for SolverService {
    fn default() -> Self {
        Self::new()
    }
}
