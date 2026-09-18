use super::plan::AssignmentDto;
use super::*;
use crate::data::build_demo_plan;
use crate::domain::canonical_minute_label;
use solverforge::{
    SelectorTelemetry, SolverLifecycleState, SolverSnapshot, SolverStatus, SolverTelemetry,
    SolverTerminalReason,
};
use std::time::Duration;

#[test]
fn dto_labels_cover_visible_and_boundary_minutes() {
    assert_eq!(canonical_minute_label(0), "Mon 00:00");
    assert_eq!(canonical_minute_label(360), "Mon 06:00");
    assert_eq!(canonical_minute_label(1440), "Tue 00:00");
    assert_eq!(canonical_minute_label(10079), "Sun 23:59");
    assert_eq!(canonical_minute_label(10080), "Sun 24:00");
    assert_eq!(canonical_minute_label(10095), "S+1 Mon 00:15");
}

#[test]
fn solution_dto_uses_backend_owned_assignment_labels() {
    let mut plan = build_demo_plan();
    let value = 23 * 60 / crate::domain::TIME_STEP;
    let assignment = plan
        .assignments
        .iter_mut()
        .find(|assignment| assignment.compatible_assignments.contains(&value))
        .expect("demo plan should expose a scheduled assignment");
    assignment.assignment = Some(value);
    assignment.soak_time_minutes = 120;

    let dto = solution_to_dto(&plan);
    let rendered = dto
        .assignments
        .first()
        .expect("dto should contain the injected assignment");

    assert_eq!(rendered.start_minutes, 1380);
    assert_eq!(rendered.end_minutes, 1500);
    assert_eq!(rendered.start_label, "Mon 23:00");
    assert_eq!(rendered.end_label, "Tue 01:00");
    assert_eq!(rendered.window_label, "Mon 23:00 -> Tue 01:00");
}

#[test]
fn solution_dto_exposes_boundary_shift_labels() {
    let dto = solution_to_dto(&build_demo_plan());
    let carry_in = dto
        .shifts
        .iter()
        .find(|shift| shift.roster_day == -1)
        .expect("carry-in shift should exist");
    let carry_out = dto
        .shifts
        .iter()
        .find(|shift| shift.roster_day == 6 && shift.shift_type == "Night")
        .expect("Sunday night shift should exist");

    assert_eq!(carry_in.label, "S-1 Sun Night");
    assert_eq!(carry_in.time_label, "S-1 Sun 22:00 -> Mon 06:00");
    assert_eq!(carry_out.time_label, "Sun 22:00 -> S+1 Mon 06:00");
}

#[test]
fn solution_dto_exposes_only_physically_compatible_furnaces() {
    let dto = solution_to_dto(&build_demo_plan());
    let target = dto
        .work_orders
        .iter()
        .find(|work_order| work_order.order_code == "WO-2026-T1400")
        .expect("demo work order");

    let compatible_names = target
        .compatible_furnaces
        .iter()
        .map(|furnace| furnace.name.as_str())
        .collect::<Vec<_>>();

    assert!(!compatible_names.contains(&"Chamber Furnace 4"));
    assert!(!compatible_names.contains(&"Chamber Furnace 5"));
}

#[test]
fn to_plan_ignores_render_assignments_and_uses_canonical_furnace_assignments() {
    let mut dto = solution_to_dto(&build_demo_plan());
    let value = dto.furnace_assignments[0].compatible_assignments[0];
    dto.assignments.push(AssignmentDto {
        work_order_id: 999,
        order_code: "render-only".to_string(),
        furnace_id: 999,
        furnace_name: "not canonical".to_string(),
        start_minutes: 0,
        end_minutes: 15,
        start_label: "x".to_string(),
        end_label: "y".to_string(),
        window_label: "x -> y".to_string(),
        temperature_celsius: 1,
        process: "Quenching".to_string(),
        customer: "render".to_string(),
        priority: "Standard".to_string(),
        late: false,
        late_minutes: 0,
        load_weight_kg: 1,
        requires_quench: false,
    });
    dto.furnace_assignments[0].assignment = Some(value);

    let plan = dto.to_plan().expect("canonical dto should decode");

    assert_eq!(plan.assignments[0].assignment, Some(value));
    assert_eq!(
        plan.assignments
            .iter()
            .filter(|assignment| assignment.assignment.is_some())
            .count(),
        1
    );
}

#[test]
fn to_plan_rejects_values_outside_canonical_ranges() {
    let mut dto = solution_to_dto(&build_demo_plan());
    dto.furnace_assignments[0].assignment = Some(usize::MAX);

    let error = dto.to_plan().expect_err("invalid assignment should fail");

    assert!(error.contains("outside its value range"));
}

#[test]
fn snapshot_dto_uses_current_job_status_lifecycle_fields() {
    let snapshot = SolverSnapshot {
        job_id: 7,
        snapshot_revision: 3,
        lifecycle_state: SolverLifecycleState::Solving,
        terminal_reason: None,
        current_score: None,
        best_score: None,
        telemetry: Default::default(),
        solution: build_demo_plan(),
    };
    let status = SolverStatus {
        job_id: 7,
        lifecycle_state: SolverLifecycleState::Cancelled,
        terminal_reason: Some(SolverTerminalReason::Cancelled),
        checkpoint_available: true,
        event_sequence: 12,
        latest_snapshot_revision: Some(3),
        current_score: None,
        best_score: None,
        telemetry: Default::default(),
    };

    let dto = snapshot_to_dto(snapshot, &status);

    assert_eq!(dto.lifecycle_state, "CANCELLED");
    assert_eq!(dto.terminal_reason, Some("cancelled"));
}

#[test]
fn telemetry_dto_preserves_solver_rejection_and_hard_delta_fields() {
    let dto = telemetry_to_dto(SolverTelemetry {
        elapsed: Duration::from_millis(100),
        moves_generated: 20,
        moves_evaluated: 18,
        moves_accepted: 5,
        moves_applied: 3,
        moves_not_doable: 2,
        moves_acceptor_rejected: 4,
        moves_forager_ignored: 6,
        moves_hard_improving: 7,
        moves_hard_neutral: 8,
        moves_hard_worse: 9,
        conflict_repair_provider_generated: 10,
        conflict_repair_duplicate_filtered: 11,
        conflict_repair_illegal_filtered: 12,
        conflict_repair_not_doable_filtered: 13,
        conflict_repair_hard_improving: 14,
        conflict_repair_exposed: 15,
        selector_telemetry: vec![SelectorTelemetry {
            selector_index: 2,
            selector_label: "compound_scalar".to_string(),
            moves_not_doable: 16,
            moves_acceptor_rejected: 17,
            moves_forager_ignored: 18,
            moves_hard_improving: 19,
            moves_hard_neutral: 20,
            moves_hard_worse: 21,
            conflict_repair_provider_generated: 22,
            conflict_repair_duplicate_filtered: 23,
            conflict_repair_illegal_filtered: 24,
            conflict_repair_not_doable_filtered: 25,
            conflict_repair_hard_improving: 26,
            conflict_repair_exposed: 27,
            ..SelectorTelemetry::default()
        }],
        ..SolverTelemetry::default()
    });

    assert_eq!(dto.moves_not_doable, 2);
    assert_eq!(dto.moves_acceptor_rejected, 4);
    assert_eq!(dto.moves_forager_ignored, 6);
    assert_eq!(dto.moves_hard_improving, 7);
    assert_eq!(dto.moves_hard_neutral, 8);
    assert_eq!(dto.moves_hard_worse, 9);
    assert_eq!(dto.conflict_repair_provider_generated, 10);
    assert_eq!(dto.conflict_repair_exposed, 15);
    assert_eq!(dto.selectors[0].moves_not_doable, 16);
    assert_eq!(dto.selectors[0].moves_hard_worse, 21);
    assert_eq!(dto.selectors[0].conflict_repair_exposed, 27);
}
