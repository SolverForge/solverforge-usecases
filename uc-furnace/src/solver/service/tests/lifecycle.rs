use solverforge::SolverManagerError;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use super::super::events::event_to_dto;
use super::super::{should_publish_best_solution, SolverService};
use super::support::{
    sample_completed_event, sample_failed_event, sample_progress_event, sample_terminal_event,
};
use crate::data::build_demo_problem;

#[tokio::test]
async fn subscriptions_receive_live_events_and_delete_cleanup_works() {
    let service = SolverService::new();
    let (tx, rx) = mpsc::unbounded_channel();
    service.attach_job_stream(7, rx);

    let mut live_rx = service
        .subscribe("7")
        .expect("job stream should be present");

    tx.send(sample_terminal_event())
        .expect("terminal event should bridge");
    let live = live_rx
        .recv()
        .await
        .expect("live event should be delivered");
    assert_eq!(live.event_type, "cancelled");
    assert_eq!(live.lifecycle_state, "CANCELLED");

    service.remove_job_stream(7);
    assert!(matches!(
        service.subscribe("7"),
        Err(SolverManagerError::JobNotFound { .. })
    ));
}

#[tokio::test]
async fn progress_events_without_exact_snapshot_score_do_not_serialize_metadata_score() {
    let service = SolverService::new();
    let (tx, rx) = mpsc::unbounded_channel();
    service.attach_job_stream(11, rx);

    let mut live_rx = service
        .subscribe("11")
        .expect("job stream should be present");
    tx.send(sample_progress_event())
        .expect("progress event should bridge");

    let live = live_rx
        .recv()
        .await
        .expect("progress event should be delivered");
    assert_eq!(live.event_type, "progress");
    assert_eq!(live.snapshot_revision, Some(3));
    assert_eq!(live.current_score, None);
    assert_eq!(live.best_score, None);
}

#[test]
fn best_solution_stream_cadence_keeps_first_and_spaced_updates() {
    let now = Instant::now();
    let previous = now
        .checked_sub(Duration::from_secs(1))
        .expect("test instant should support subtraction");

    assert!(should_publish_best_solution(None, now));
    assert!(!should_publish_best_solution(Some(now), now));
    assert!(should_publish_best_solution(Some(previous), now));
}

#[test]
fn progress_events_can_use_public_snapshot_score() {
    let dto = event_to_dto(
        sample_progress_event(),
        Some(solverforge::HardSoftScore::of(0, -3)),
    );

    assert_eq!(dto.event_type, "progress");
    assert_eq!(dto.current_score.as_deref(), Some("0hard/-3soft"));
    assert_eq!(dto.best_score.as_deref(), Some("0hard/-3soft"));
}

#[test]
fn best_solution_events_use_public_exact_solution_score() {
    let event = solverforge::SolverEvent::BestSolution {
        metadata: match sample_progress_event() {
            solverforge::SolverEvent::Progress { metadata } => metadata,
            _ => unreachable!("sample should be progress"),
        },
        solution: build_demo_problem(),
    };

    let dto = event_to_dto(event, None);
    let analyzed_score = solverforge::analyze(&build_demo_problem())
        .score
        .to_string();

    assert_eq!(dto.event_type, "best_solution");
    assert_eq!(dto.current_score.as_deref(), Some(analyzed_score.as_str()));
    assert_eq!(dto.best_score.as_deref(), Some(analyzed_score.as_str()));
    assert!(dto.solution.is_some());
}

#[test]
fn completed_cancelled_and_failed_events_are_terminal() {
    let completed = event_to_dto(sample_completed_event(), None);
    let cancelled = event_to_dto(sample_terminal_event(), None);
    let failed = event_to_dto(sample_failed_event(), None);

    assert!(completed.is_terminal());
    assert_eq!(completed.event_type, "completed");
    assert_eq!(completed.terminal_reason, Some("completed"));

    assert!(cancelled.is_terminal());
    assert_eq!(cancelled.event_type, "cancelled");
    assert_eq!(cancelled.terminal_reason, Some("cancelled"));

    assert!(failed.is_terminal());
    assert_eq!(failed.event_type, "failed");
    assert_eq!(failed.error.as_deref(), Some("canonical solve failed"));
}
