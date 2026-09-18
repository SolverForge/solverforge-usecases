use super::*;
use crate::data::{baseline_plan, baseline_solution_plan};
use solverforge::IncrementalConstraint;

#[test]
fn baseline_problem_starts_completely_unassigned() {
    let plan = baseline_plan();

    assert!(plan
        .work_packages
        .iter()
        .all(|package| package.dock_idx.is_none() && package.start_day_idx.is_none()));
    assert!(plan
        .inspection_assignments
        .iter()
        .all(|assignment| assignment.day_idx.is_none()));
    assert!(plan
        .training_assignments
        .iter()
        .all(|assignment| assignment.day_idx.is_none()));
}

#[test]
fn baseline_solution_satisfies_native_hard_constraints() {
    let plan = baseline_solution_plan();

    assert_eq!(count_required_assignments(&plan), 0);
    assert_eq!(count_work_window(&plan), 0);
    assert_eq!(count_dock_compatibility(&plan), 0);
    assert_eq!(count_dock_capacity(&plan), 0);
    assert_eq!(count_inspection_training_precedence(&plan), 0);
    assert_eq!(count_technician_capacity(&plan), 0);
    assert_eq!(count_parts_availability(&plan), 0);
    assert_eq!(count_weekly_readiness_floor(&plan), 0);
    assert_eq!(count_inspection_lateness(&plan), 0);
}

#[test]
fn readiness_floor_is_native_and_detects_policy_pressure() {
    let mut plan = baseline_solution_plan();
    apply_readiness_floor(&mut plan, 20);

    assert!(count_weekly_readiness_floor(&plan) > 0);
    assert!(score_weekly_readiness_floor(&plan).hard() < 0);
    assert!(score_minimize_readiness_shortfall(&plan).soft() < 0);
}

#[test]
fn inspection_lateness_is_a_named_soft_objective() {
    let mut plan = baseline_solution_plan();
    let late_day_idx = plan.days.iter().position(|day| day.index == 56);
    plan.inspection_assignments[0].day_idx = late_day_idx;

    assert!(count_inspection_lateness(&plan) > 0);
    assert!(score_minimize_inspection_lateness(&plan).soft() < 0);
}

#[test]
fn dock_capacity_detects_overlapping_assignments() {
    let mut plan = baseline_solution_plan();
    let first_start = plan.work_packages[0].start_day_idx;
    let first_dock = plan.work_packages[0].dock_idx;
    plan.work_packages[2].start_day_idx = first_start;
    plan.work_packages[2].dock_idx = first_dock;

    assert!(count_dock_capacity(&plan) > 0);
}

#[test]
fn fleet_constraint_reports_real_full_recompute_deltas() {
    let mut plan = baseline_solution_plan();
    let mut constraint = FleetConstraint::new(
        "minimize_churn",
        false,
        solverforge::HardSoftScore::of_soft(1),
        score_minimize_churn,
        count_churn,
    );
    let mut tracked = constraint.initialize(&plan);

    tracked = tracked + constraint.on_retract(&plan, 0, 0);
    tracked = tracked + constraint.on_retract(&plan, 1, 0);
    plan.work_packages[0].start_day_idx = plan.days.iter().position(|day| day.index == 3);
    plan.work_packages[1].start_day_idx = plan.days.iter().position(|day| day.index == 4);
    tracked = tracked + constraint.on_insert(&plan, 0, 0);
    tracked = tracked + constraint.on_insert(&plan, 1, 0);

    assert_eq!(tracked, score_minimize_churn(&plan));
}

#[test]
fn dock_outage_is_persisted_and_scored_as_hard_state() {
    let mut plan = baseline_solution_plan();
    let dock_idx = plan.docks.iter().position(|dock| dock.id == "D2");
    plan.work_packages[0].dock_idx = dock_idx;
    plan.work_packages[0].start_day_idx = plan.days.iter().position(|day| day.index == 22);

    apply_dock_outage(&mut plan, "D2", 22, 31);

    assert_eq!(plan.dock_outages.len(), 1);
    plan.work_packages[0].start_day_idx = plan.days.iter().position(|day| day.index == 22);
    assert!(count_dock_outages(&plan) > 0);
    assert!(score_dock_outages(&plan).hard() < 0);
}

#[test]
fn dated_technician_shortage_does_not_rewrite_global_capacity() {
    let mut plan = baseline_solution_plan();
    let original_capacity = plan
        .technician_pools
        .iter()
        .find(|pool| pool.id == "ELEC")
        .map(|pool| pool.capacity_per_day)
        .expect("ELEC pool");

    apply_technician_shortage_window(&mut plan, "ELEC", -1, 15, 21);

    let elec = plan
        .technician_pools
        .iter()
        .find(|pool| pool.id == "ELEC")
        .expect("ELEC pool");
    assert_eq!(elec.capacity_per_day, original_capacity);
    assert!(plan
        .technician_capacity_overrides
        .iter()
        .any(|override_| override_.pool_id == "ELEC"
            && override_.start_day == 15
            && override_.end_day == 21
            && override_.delta == -1));
}

#[test]
fn repair_seed_moves_dependent_inspection_with_shifted_work() {
    let mut plan = baseline_solution_plan();
    let dock_idx = plan.docks.iter().position(|dock| dock.id == "D2");
    plan.work_packages[0].dock_idx = dock_idx;
    plan.work_packages[0].start_day_idx = plan.days.iter().position(|day| day.index == 22);
    let package_id = plan.work_packages[0].id.clone();
    let duration = plan.work_packages[0].duration_days;

    apply_dock_outage(&mut plan, "D2", 22, 31);

    let package = plan
        .work_packages
        .iter()
        .find(|package| package.id == package_id)
        .expect("package");
    let shifted_start = assigned_start(&plan, package).expect("shifted start");
    let inspection = plan
        .inspection_assignments
        .iter()
        .find(|assignment| assignment.work_package_id == package_id)
        .expect("inspection");
    assert_eq!(shifted_start, 32);
    assert_eq!(
        day_index(&plan, inspection.day_idx),
        Some(shifted_start + duration)
    );
}

#[test]
fn parts_availability_detects_delayed_replenishment() {
    let mut raw_delay = baseline_solution_plan();
    for delivery in &mut raw_delay.deliveries {
        if delivery.id == "DELIV-01" {
            delivery.arrival_day = 29;
        }
    }

    assert!(count_parts_availability(&raw_delay) > 0);

    let mut repaired = baseline_solution_plan();
    apply_parts_delay(&mut repaired, "DELIV-01", 29);

    assert_eq!(count_parts_availability(&repaired), 0);
    assert!(revision_diff(&baseline_solution_plan(), &repaired).moved_assignments > 0);
}

#[test]
fn repair_diff_reports_lineage_changes() {
    let before = baseline_solution_plan();
    let mut after = baseline_solution_plan();
    after.work_packages[0].start_day_idx = after.days.iter().position(|day| day.index == 3);

    let diff = revision_diff(&before, &after);

    assert_eq!(diff.moved_assignments, 1);
    assert!(diff.human_summary.contains("1 assignments moved"));
}
