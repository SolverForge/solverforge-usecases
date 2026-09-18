use serde::Deserialize;
use solverforge::{
    HardSoftScore, SolverConfig, SolverEvent, SolverEventMetadata, SolverLifecycleState,
    SolverStatus, SolverTerminalReason,
};
use std::time::Duration;

use crate::data::build_demo_problem;
use crate::domain::Plan;

pub(super) fn load_plan_solver_config() -> SolverConfig {
    SolverConfig::from_toml_str(include_str!("../../../../solver.toml"))
        .expect("repo solver.toml should load as the final Plan solver config")
}

pub(super) fn sample_status() -> SolverStatus<HardSoftScore> {
    SolverStatus {
        job_id: 7,
        lifecycle_state: SolverLifecycleState::Solving,
        terminal_reason: None,
        checkpoint_available: true,
        event_sequence: 1,
        latest_snapshot_revision: Some(2),
        current_score: Some(HardSoftScore::of(-5, -10)),
        best_score: Some(HardSoftScore::of(-4, -8)),
        telemetry: solverforge::SolverTelemetry {
            elapsed: Duration::from_millis(10),
            step_count: 1,
            moves_generated: 4,
            moves_evaluated: 3,
            moves_accepted: 1,
            moves_applied: 1,
            score_calculations: 2,
            construction_slots_assigned: 0,
            construction_slots_kept: 0,
            construction_slots_no_doable: 0,
            generation_time: Duration::from_millis(3),
            evaluation_time: Duration::from_millis(4),
            selector_telemetry: Vec::new(),
            ..solverforge::SolverTelemetry::default()
        },
    }
}

pub(super) fn sample_terminal_event() -> SolverEvent<Plan> {
    SolverEvent::Cancelled {
        metadata: SolverEventMetadata {
            job_id: 7,
            event_sequence: 3,
            lifecycle_state: SolverLifecycleState::Cancelled,
            terminal_reason: Some(SolverTerminalReason::Cancelled),
            telemetry: solverforge::SolverTelemetry {
                elapsed: Duration::from_millis(12),
                step_count: 2,
                moves_generated: 5,
                moves_evaluated: 4,
                moves_accepted: 1,
                moves_applied: 1,
                score_calculations: 3,
                construction_slots_assigned: 0,
                construction_slots_kept: 0,
                construction_slots_no_doable: 0,
                generation_time: Duration::from_millis(4),
                evaluation_time: Duration::from_millis(5),
                selector_telemetry: Vec::new(),
                ..solverforge::SolverTelemetry::default()
            },
            current_score: None,
            best_score: None,
            snapshot_revision: Some(2),
        },
    }
}

pub(super) fn sample_progress_event() -> SolverEvent<Plan> {
    SolverEvent::Progress {
        metadata: SolverEventMetadata {
            job_id: 7,
            event_sequence: 2,
            lifecycle_state: SolverLifecycleState::Solving,
            terminal_reason: None,
            telemetry: solverforge::SolverTelemetry {
                elapsed: Duration::from_millis(20),
                step_count: 4,
                moves_generated: 12,
                moves_evaluated: 10,
                moves_accepted: 2,
                moves_applied: 2,
                score_calculations: 9,
                construction_slots_assigned: 0,
                construction_slots_kept: 0,
                construction_slots_no_doable: 0,
                generation_time: Duration::from_millis(7),
                evaluation_time: Duration::from_millis(8),
                selector_telemetry: Vec::new(),
                ..solverforge::SolverTelemetry::default()
            },
            current_score: Some(HardSoftScore::of(-4, -8)),
            best_score: Some(HardSoftScore::of(-4, -8)),
            snapshot_revision: Some(3),
        },
    }
}

pub(super) fn sample_completed_event() -> SolverEvent<Plan> {
    SolverEvent::Completed {
        metadata: SolverEventMetadata {
            job_id: 7,
            event_sequence: 4,
            lifecycle_state: SolverLifecycleState::Completed,
            terminal_reason: Some(SolverTerminalReason::Completed),
            telemetry: sample_status().telemetry,
            current_score: Some(HardSoftScore::of(0, -1)),
            best_score: Some(HardSoftScore::of(0, -1)),
            snapshot_revision: Some(4),
        },
        solution: build_demo_problem(),
    }
}

pub(super) fn sample_failed_event() -> SolverEvent<Plan> {
    SolverEvent::Failed {
        metadata: SolverEventMetadata {
            job_id: 7,
            event_sequence: 5,
            lifecycle_state: SolverLifecycleState::Failed,
            terminal_reason: Some(SolverTerminalReason::Failed),
            telemetry: sample_status().telemetry,
            current_score: None,
            best_score: None,
            snapshot_revision: Some(4),
        },
        error: "canonical solve failed".to_string(),
    }
}

#[derive(Deserialize)]
pub(super) struct AppSpecContract {
    pub app: AppSpecSection,
    pub runtime: RuntimeSection,
    pub demo: DemoSection,
    pub solution: SolutionSection,
}

#[derive(Deserialize)]
pub(super) struct AppSpecSection {
    pub name: String,
    pub starter: String,
    pub cli_version: String,
    #[serde(default)]
    pub shell: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct RuntimeSection {
    pub target: String,
    pub runtime_source: String,
    pub ui_source: String,
}

#[derive(Deserialize)]
pub(super) struct DemoSection {
    pub default_size: String,
    pub available_sizes: Vec<String>,
}

#[derive(Deserialize)]
pub(super) struct SolutionSection {
    pub name: String,
    pub score: String,
}
