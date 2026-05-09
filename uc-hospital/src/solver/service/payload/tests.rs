//! Unit tests for the retained-job SSE payload contract.

use super::*;
use crate::domain::{Employee, Shift};
use serde_json::Value;
use solverforge::{
    SolverLifecycleState, SolverSnapshot, SolverStatus, SolverTelemetry, SolverTerminalReason,
};

#[test]
fn failed_events_use_stock_payload_fields() {
    let payload: Value = serde_json::from_str(&event_payload(
        7,
        "failed",
        &SolverEventMetadata {
            job_id: 7,
            event_sequence: 4,
            lifecycle_state: SolverLifecycleState::Failed,
            terminal_reason: Some(SolverTerminalReason::Failed),
            snapshot_revision: Some(3),
            current_score: None,
            best_score: None,
            telemetry: SolverTelemetry::default(),
        },
        None,
        Some("boom"),
    ))
    .unwrap();

    assert_eq!(payload["id"], "7");
    assert_eq!(payload["jobId"], "7");
    assert_eq!(payload["eventType"], "failed");
    assert_eq!(payload["lifecycleState"], "FAILED");
    assert_eq!(payload["terminalReason"], "failed");
    assert_eq!(payload["error"], "boom");
}

#[test]
fn event_payload_derives_stock_telemetry_fields_from_exact_runtime_telemetry() {
    let payload: Value = serde_json::from_str(&event_payload(
        5,
        "progress",
        &SolverEventMetadata {
            job_id: 5,
            event_sequence: 2,
            lifecycle_state: SolverLifecycleState::Solving,
            terminal_reason: None,
            snapshot_revision: Some(1),
            current_score: Some(HardSoftDecimalScore::ZERO),
            best_score: Some(HardSoftDecimalScore::ZERO),
            telemetry: SolverTelemetry {
                elapsed: std::time::Duration::from_millis(2_500),
                step_count: 9,
                moves_generated: 300,
                moves_evaluated: 200,
                moves_accepted: 50,
                score_calculations: 80,
                generation_time: std::time::Duration::from_millis(400),
                evaluation_time: std::time::Duration::from_millis(900),
                ..SolverTelemetry::default()
            },
        },
        None,
        None,
    ))
    .unwrap();

    assert_eq!(payload["telemetry"]["elapsedMs"], 2500);
    assert_eq!(payload["telemetry"]["stepCount"], 9);
    assert_eq!(payload["telemetry"]["movesGenerated"], 300);
    assert_eq!(payload["telemetry"]["movesEvaluated"], 200);
    assert_eq!(payload["telemetry"]["movesAccepted"], 50);
    assert_eq!(payload["telemetry"]["scoreCalculations"], 80);
    assert_eq!(payload["telemetry"]["generationMs"], 400);
    assert_eq!(payload["telemetry"]["evaluationMs"], 900);
    assert_eq!(payload["telemetry"]["movesPerSecond"], 80);
    assert_eq!(payload["telemetry"]["acceptanceRate"], 0.25);
}

#[test]
fn snapshot_bootstrap_payload_exposes_live_solution_and_ui_score_fields() {
    let mut solution = Plan::new(
        vec![Employee::new(0, "Alex").with_skill("Doctor")],
        vec![{
            let mut shift = Shift::new(
                "shift-1",
                chrono::NaiveDate::from_ymd_opt(2024, 1, 1)
                    .unwrap()
                    .and_hms_opt(8, 0, 0)
                    .unwrap(),
                chrono::NaiveDate::from_ymd_opt(2024, 1, 1)
                    .unwrap()
                    .and_hms_opt(16, 0, 0)
                    .unwrap(),
                "ER",
                "Doctor",
            );
            shift.employee_idx = Some(0);
            shift
        }],
    );
    solution.score = Some(HardSoftDecimalScore::ZERO);

    let status = SolverStatus {
        job_id: 11,
        lifecycle_state: SolverLifecycleState::Solving,
        terminal_reason: None,
        checkpoint_available: true,
        event_sequence: 9,
        latest_snapshot_revision: Some(4),
        current_score: None,
        best_score: None,
        telemetry: SolverTelemetry::default(),
    };
    let snapshot = SolverSnapshot {
        job_id: 11,
        snapshot_revision: 4,
        lifecycle_state: SolverLifecycleState::Solving,
        terminal_reason: None,
        current_score: Some(HardSoftDecimalScore::ZERO),
        best_score: Some(HardSoftDecimalScore::ZERO),
        telemetry: SolverTelemetry::default(),
        solution,
    };

    let payload: Value = serde_json::from_str(&snapshot_status_event_payload(
        11,
        bootstrap_snapshot_event_type(status.lifecycle_state),
        &status,
        &snapshot,
    ))
    .unwrap();

    assert_eq!(payload["eventType"], "best_solution");
    assert_eq!(payload["lifecycleState"], "SOLVING");
    assert_eq!(payload["snapshotRevision"], 4);
    assert_eq!(payload["currentScore"], "0hard/0soft");
    assert_eq!(payload["bestScore"], "0hard/0soft");
    assert!(payload["solution"].is_object());
    assert_eq!(payload["solution"]["shifts"][0]["employeeIdx"], 0);
    assert_eq!(payload["solution"]["score"], "0hard/0soft");
}
