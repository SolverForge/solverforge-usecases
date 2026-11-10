use crate::constraints::create_constraints;
use crate::constraints::shared::changeover_shortage;
use crate::data::build_demo_problem;
use crate::domain::calculate_changeover_cost_for_sequence;
use crate::domain::Plan;
use solverforge::ConstraintSet;
use std::collections::HashMap;

fn hard_score(plan: &Plan, constraint_name: &str) -> i64 {
    create_constraints()
        .evaluate_each(plan)
        .into_iter()
        .find(|result| result.name == constraint_name)
        .expect("constraint result")
        .score
        .hard()
}

fn first_compatible_assignment_plan() -> Plan {
    let mut plan = build_demo_problem();
    for assignment in &mut plan.assignments {
        assignment.assignment = assignment.compatible_assignments.first().copied();
    }
    plan
}

fn furnace_and_start(value: usize) -> (usize, usize) {
    (
        value / crate::domain::NUM_TIME_SLOTS,
        (value % crate::domain::NUM_TIME_SLOTS) * crate::domain::TIME_STEP,
    )
}

fn adjacent_pair(plan: &Plan) -> (usize, usize, usize, usize) {
    let by_furnace_start = assignment_values_by_furnace_start(plan);
    for left_index in 0..plan.assignments.len() {
        for left_value in &plan.assignments[left_index].compatible_assignments {
            let (furnace_id, left_start) = furnace_and_start(*left_value);
            let right_start = left_start + plan.assignments[left_index].duration_minutes();
            if let Some(candidates) = by_furnace_start.get(&(furnace_id, right_start)) {
                if let Some((right_index, right_value)) = candidates
                    .iter()
                    .copied()
                    .find(|(right_index, _)| *right_index != left_index)
                {
                    return (left_index, *left_value, right_index, right_value);
                }
            }
        }
    }
    panic!("adjacent compatible assignment pair");
}

fn same_start_pair(plan: &Plan, same_furnace: bool) -> (usize, usize, usize, usize) {
    let by_furnace_start = assignment_values_by_furnace_start(plan);
    for entries in by_furnace_start.values() {
        if same_furnace && entries.len() >= 2 {
            return (entries[0].0, entries[0].1, entries[1].0, entries[1].1);
        }
    }
    if !same_furnace {
        let mut by_start: HashMap<usize, Vec<(usize, usize, usize)>> = HashMap::new();
        for ((furnace_id, start), entries) in by_furnace_start {
            for (assignment_index, value) in entries {
                by_start
                    .entry(start)
                    .or_default()
                    .push((furnace_id, assignment_index, value));
            }
        }
        for entries in by_start.values() {
            for left in entries {
                for right in entries {
                    if left.0 != right.0 && left.1 != right.1 {
                        return (left.1, left.2, right.1, right.2);
                    }
                }
            }
        }
    }
    panic!("same-start compatible assignment pair");
}

fn assignment_values_by_furnace_start(plan: &Plan) -> HashMap<(usize, usize), Vec<(usize, usize)>> {
    let mut values = HashMap::new();
    for (assignment_index, assignment) in plan.assignments.iter().enumerate() {
        for value in &assignment.compatible_assignments {
            values
                .entry(furnace_and_start(*value))
                .or_insert_with(Vec::new)
                .push((assignment_index, *value));
        }
    }
    values
}

fn reversed_work_order_changeover_shortage_pair(
    plan: &Plan,
) -> (usize, usize, usize, usize, usize) {
    for left_index in 0..plan.assignments.len() {
        for right_index in 0..plan.assignments.len() {
            if left_index <= right_index {
                continue;
            }
            for left_value in &plan.assignments[left_index].compatible_assignments {
                let (left_furnace, left_start) = furnace_and_start(*left_value);
                let left_end = left_start + plan.assignments[left_index].duration_minutes();
                for right_value in &plan.assignments[right_index].compatible_assignments {
                    let (right_furnace, right_start) = furnace_and_start(*right_value);
                    if left_furnace != right_furnace || left_end > right_start {
                        continue;
                    }

                    let mut candidate = plan.clone();
                    candidate.assignments[left_index].assignment = Some(*left_value);
                    candidate.assignments[right_index].assignment = Some(*right_value);
                    let shortage = changeover_shortage(
                        candidate.assignments[left_index].end_minutes().unwrap(),
                        candidate.assignments[right_index].start_minutes().unwrap(),
                        calculate_changeover_cost_for_sequence(
                            &candidate.assignments[left_index],
                            &candidate.assignments[right_index],
                        ) as usize,
                    );
                    if let Some(shortage) = shortage {
                        return (left_index, *left_value, right_index, *right_value, shortage);
                    }
                }
            }
        }
    }
    panic!("reversed work-order changeover shortage pair");
}

#[test]
fn generated_assignment_values_do_not_create_incompatible_process_matches() {
    let plan = first_compatible_assignment_plan();

    assert_eq!(hard_score(&plan, "incompatible_process"), 0);
}

#[test]
fn generated_assignment_values_do_not_create_excessive_temperature_matches() {
    let plan = first_compatible_assignment_plan();

    assert_eq!(hard_score(&plan, "excessive_temperature"), 0);
}

#[test]
fn generated_assignment_values_do_not_create_excessive_load_matches() {
    let plan = first_compatible_assignment_plan();

    assert_eq!(hard_score(&plan, "excessive_load"), 0);
}

#[test]
fn adjacent_same_furnace_intervals_are_allowed() {
    let mut plan = build_demo_problem();
    let (left_index, left_value, right_index, right_value) = adjacent_pair(&plan);
    plan.assignments[left_index].assignment = Some(left_value);
    plan.assignments[right_index].assignment = Some(right_value);

    assert_eq!(hard_score(&plan, "furnace_overlap"), 0);
}

#[test]
fn true_same_furnace_overlap_penalizes_overlap_minutes() {
    let mut plan = build_demo_problem();
    let (left_index, left_value, right_index, right_value) = same_start_pair(&plan, true);
    plan.assignments[left_index].assignment = Some(left_value);
    plan.assignments[right_index].assignment = Some(right_value);

    assert!(hard_score(&plan, "furnace_overlap") < 0);
}

#[test]
fn different_furnaces_never_overlap_match() {
    let mut plan = build_demo_problem();
    let (left_index, left_value, right_index, right_value) = same_start_pair(&plan, false);
    plan.assignments[left_index].assignment = Some(left_value);
    plan.assignments[right_index].assignment = Some(right_value);

    assert_eq!(hard_score(&plan, "furnace_overlap"), 0);
}

#[test]
fn changeover_gap_uses_temporal_order_not_work_order_order() {
    let mut plan = build_demo_problem();
    let (left_index, left_value, right_index, right_value, shortage) =
        reversed_work_order_changeover_shortage_pair(&plan);
    plan.assignments[left_index].assignment = Some(left_value);
    plan.assignments[right_index].assignment = Some(right_value);

    assert_eq!(hard_score(&plan, "changeover_gap"), -(shortage as i64) * 10);
}
