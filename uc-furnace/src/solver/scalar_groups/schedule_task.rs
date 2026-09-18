use solverforge::prelude::*;

use crate::domain::{
    furnace_assignment_construction_entity_order_key,
    furnace_assignment_local_search_value_order_key, Plan,
};
use crate::solver::conflict_repair::assignment_value_is_hard_clean;

pub(crate) fn furnace_schedule_task_assignment_candidates(
    plan: &Plan,
    limits: ScalarGroupLimits,
) -> Vec<ScalarCandidate<Plan>> {
    if super::is_construction_pass(limits) {
        return Vec::new();
    }

    let max_total = super::total_candidate_limit(limits);
    let values_per_assignment = limits.value_candidate_limit.unwrap_or(max_total);
    let mut per_assignment = Vec::new();

    for assignment_index in 0..plan.assignments.len() {
        let mut values = schedule_task_values_for_assignment(plan, assignment_index);
        values.truncate(values_per_assignment);
        per_assignment.push((assignment_index, values));
    }

    let mut candidates = Vec::new();
    let max_depth = per_assignment
        .iter()
        .map(|(_, values)| values.len())
        .max()
        .unwrap_or(0);
    for depth in 0..max_depth {
        for (assignment_index, values) in &per_assignment {
            if candidates.len() >= max_total {
                return candidates;
            }
            let Some(value) = values.get(depth).copied() else {
                continue;
            };
            if let Some(candidate) = schedule_task_candidate(plan, *assignment_index, value) {
                candidates.push(candidate);
            }
        }
    }

    candidates
}

fn schedule_task_values_for_assignment(plan: &Plan, assignment_index: usize) -> Vec<usize> {
    let assignment = &plan.assignments[assignment_index];
    let mut values = assignment
        .compatible_assignments
        .iter()
        .copied()
        .filter(|value| assignment.assignment != Some(*value))
        .filter(|value| assignment_value_is_hard_clean(plan, assignment_index, *value))
        .collect::<Vec<_>>();
    values.sort_by_key(|value| {
        furnace_assignment_local_search_value_order_key(plan, assignment, *value)
    });
    values
}

fn schedule_task_candidate(
    plan: &Plan,
    assignment_index: usize,
    value: usize,
) -> Option<ScalarCandidate<Plan>> {
    let assignment = &plan.assignments[assignment_index];
    let mut scheduled = assignment.clone();
    scheduled.assignment = Some(value);
    let mut task_edits = Vec::new();
    super::append_task_operator_edits(plan, assignment_index, &scheduled, false, &mut task_edits)?;
    if task_edits.is_empty() {
        return None;
    }

    let mut edits = Vec::with_capacity(task_edits.len() + 1);
    edits.push(
        Plan::assignments()
            .scalar("assignment")
            .set(assignment_index, Some(value)),
    );
    edits.extend(task_edits);

    Some(
        ScalarCandidate::new("furnace_schedule_task_assignment", edits)
            .with_construction_slot_key(assignment_index)
            .with_construction_entity_order_key(furnace_assignment_construction_entity_order_key(
                plan, assignment,
            ))
            .with_construction_value_order_key(furnace_assignment_local_search_value_order_key(
                plan, assignment, value,
            )),
    )
}
