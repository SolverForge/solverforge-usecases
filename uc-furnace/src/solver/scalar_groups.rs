use std::collections::HashSet;

use solverforge::prelude::*;

use crate::domain::{
    furnace_assignment_construction_entity_order_key,
    furnace_assignment_construction_value_order_key,
    load_build_operator_construction_value_order_key, operator_shift_construction_entity_order_key,
    operator_shift_construction_value_order_key, program_operator_construction_value_order_key,
    quench_operator_construction_value_order_key, task_operator_construction_entity_order_key,
    unload_operator_construction_value_order_key, FurnaceAssignment, ManualTaskKind, Plan,
    ShiftIdentity, OFF_SHIFT_VALUE,
};
use crate::solver::conflict_repair::assignment_value_is_hard_clean;

mod schedule_task;

pub(crate) use schedule_task::furnace_schedule_task_assignment_candidates;

pub(crate) fn groups() -> Vec<ScalarGroup<Plan>> {
    let assignments = Plan::assignments();
    let roster_assignments = Plan::operator_shift_assignments();
    vec![
        ScalarGroup::candidates(
            "furnace_assignment",
            vec![assignments.scalar("assignment")],
            furnace_assignment_candidates,
        ),
        ScalarGroup::candidates(
            "roster_shift_assignment",
            vec![roster_assignments.scalar("shift_type")],
            roster_shift_assignment_candidates,
        ),
        ScalarGroup::candidates(
            "task_operator_assignment",
            vec![
                assignments.scalar("load_build_operator_id"),
                assignments.scalar("program_operator_id"),
                assignments.scalar("quench_operator_id"),
                assignments.scalar("unload_operator_id"),
            ],
            task_operator_assignment_candidates,
        ),
        ScalarGroup::candidates(
            "furnace_schedule_task_assignment",
            vec![
                assignments.scalar("assignment"),
                assignments.scalar("load_build_operator_id"),
                assignments.scalar("program_operator_id"),
                assignments.scalar("quench_operator_id"),
                assignments.scalar("unload_operator_id"),
            ],
            furnace_schedule_task_assignment_candidates,
        ),
    ]
}

pub(crate) fn furnace_assignment_candidates(
    plan: &Plan,
    limits: ScalarGroupLimits,
) -> Vec<ScalarCandidate<Plan>> {
    let mut candidates = Vec::new();
    let max_total = total_candidate_limit(limits);
    let values_per_assignment = limits.value_candidate_limit.unwrap_or(usize::MAX);

    for (assignment_index, assignment) in plan.assignments.iter().enumerate() {
        if is_construction_pass(limits) && assignment.assignment.is_some() {
            continue;
        }
        let entity_key = furnace_assignment_construction_entity_order_key(plan, assignment);
        let mut emitted_for_assignment = 0usize;
        for value in assignment.compatible_assignments.iter().copied() {
            if candidates.len() >= max_total {
                return candidates;
            }
            if emitted_for_assignment >= values_per_assignment {
                break;
            }
            if assignment.assignment == Some(value) {
                continue;
            }
            if is_construction_pass(limits)
                && !assignment_value_is_hard_clean(plan, assignment_index, value)
            {
                continue;
            }

            candidates.push(furnace_assignment_candidate(
                plan,
                assignment_index,
                value,
                entity_key,
            ));
            emitted_for_assignment += 1;
        }
    }

    candidates
}

fn furnace_assignment_candidate(
    plan: &Plan,
    assignment_index: usize,
    value: usize,
    entity_key: i64,
) -> ScalarCandidate<Plan> {
    let assignment = &plan.assignments[assignment_index];
    let value_key = furnace_assignment_construction_value_order_key(plan, assignment, value);
    ScalarCandidate::new(
        "furnace_assignment",
        vec![Plan::assignments()
            .scalar("assignment")
            .set(assignment_index, Some(value))],
    )
    .with_construction_slot_key(assignment_index)
    .with_construction_entity_order_key(entity_key)
    .with_construction_value_order_key(value_key)
}

pub(crate) fn roster_shift_assignment_candidates(
    plan: &Plan,
    limits: ScalarGroupLimits,
) -> Vec<ScalarCandidate<Plan>> {
    let mut candidates = Vec::new();
    let max_total = total_candidate_limit(limits);
    let values_per_assignment = limits.value_candidate_limit.unwrap_or(usize::MAX);

    for (assignment_index, assignment) in plan.operator_shift_assignments.iter().enumerate() {
        if is_construction_pass(limits) && assignment.shift_type.is_some() {
            continue;
        }
        let entity_key = operator_shift_construction_entity_order_key(plan, assignment);
        let mut emitted_for_assignment = 0usize;
        for value in assignment.allowed_shift_types.iter().copied() {
            if candidates.len() >= max_total {
                return candidates;
            }
            if emitted_for_assignment >= values_per_assignment {
                break;
            }
            if assignment.shift_type == Some(value) {
                continue;
            }
            let value_key = operator_shift_construction_value_order_key(plan, assignment, value);
            candidates.push(
                ScalarCandidate::new(
                    "roster_shift_assignment",
                    vec![Plan::operator_shift_assignments()
                        .scalar("shift_type")
                        .set(assignment_index, Some(value))],
                )
                .with_construction_slot_key(assignment_index)
                .with_construction_entity_order_key(entity_key)
                .with_construction_value_order_key(value_key),
            );
            emitted_for_assignment += 1;
        }
    }

    candidates
}

pub(crate) fn task_operator_assignment_candidates(
    plan: &Plan,
    limits: ScalarGroupLimits,
) -> Vec<ScalarCandidate<Plan>> {
    let mut candidates = Vec::new();
    let max_total = total_candidate_limit(limits);
    let assignments_per_step = limits.value_candidate_limit.unwrap_or(usize::MAX);

    let mut emitted_assignments = 0usize;
    for (assignment_index, assignment) in plan.assignments.iter().enumerate() {
        if candidates.len() >= max_total {
            return candidates;
        }
        if emitted_assignments >= assignments_per_step {
            break;
        }
        if !assignment.is_scheduled() {
            continue;
        }
        let Some(candidate) =
            task_operator_assignment_candidate_for_index(plan, limits, assignment_index)
        else {
            continue;
        };
        candidates.push(candidate);
        emitted_assignments += 1;
    }

    candidates
}

pub(crate) fn task_operator_assignment_candidate_for_index(
    plan: &Plan,
    limits: ScalarGroupLimits,
    assignment_index: usize,
) -> Option<ScalarCandidate<Plan>> {
    let assignment = plan.assignments.get(assignment_index)?;
    if !assignment.is_scheduled() {
        return None;
    }

    let mut edits = Vec::new();
    let value_key = append_task_operator_edits(
        plan,
        assignment_index,
        assignment,
        limits.max_moves_per_step.is_none(),
        &mut edits,
    )?;
    if edits.is_empty() {
        return None;
    }

    Some(
        ScalarCandidate::new("task_operator_assignment", edits)
            .with_construction_slot_key(assignment_index)
            .with_construction_entity_order_key(task_operator_construction_entity_order_key(
                plan, assignment,
            ))
            .with_construction_value_order_key(value_key),
    )
}

fn append_task_operator_edits(
    plan: &Plan,
    assignment_index: usize,
    scheduled: &FurnaceAssignment,
    allow_placeholder_roster_coverage: bool,
    edits: &mut Vec<ScalarEdit<Plan>>,
) -> Option<i64> {
    let mut variables = HashSet::new();
    let mut value_key = 0_i64;
    for kind in scheduled.required_task_kinds() {
        let variable_name = task_variable_name(kind);
        if !variables.insert(variable_name) {
            continue;
        }
        let shift = scheduled.task_shift_identity(kind)?;
        let current = plan.assignments[assignment_index].task_operator_id(kind);
        let (operator_id, operator_key) = best_operator_for_task_shift(
            plan,
            scheduled,
            kind,
            shift,
            allow_placeholder_roster_coverage,
        )?;
        value_key = value_key.saturating_add(operator_key);
        if current == Some(operator_id) {
            continue;
        }
        edits.push(
            Plan::assignments()
                .scalar(variable_name)
                .set(assignment_index, Some(operator_id)),
        );
    }
    Some(value_key)
}

fn best_operator_for_task_shift(
    plan: &Plan,
    assignment: &FurnaceAssignment,
    kind: ManualTaskKind,
    shift: ShiftIdentity,
    allow_placeholder_roster_coverage: bool,
) -> Option<(usize, i64)> {
    assignment
        .task_eligible_operator_ids(kind)
        .iter()
        .copied()
        .filter(|operator_id| {
            operator_can_cover_shift(plan, *operator_id, shift, allow_placeholder_roster_coverage)
        })
        .map(|operator_id| {
            (
                operator_id,
                task_operator_value_order_key(plan, assignment, kind, operator_id),
            )
        })
        .min_by_key(|(operator_id, key)| (*key, *operator_id))
}

fn operator_can_cover_shift(
    plan: &Plan,
    operator_id: usize,
    shift: ShiftIdentity,
    allow_placeholder_roster_coverage: bool,
) -> bool {
    plan.operator_shift_assignments.iter().any(|assignment| {
        assignment.operator_idx == operator_id
            && assignment.roster_day == shift.roster_day
            && assignment.shift_type.is_none_or(|shift_type| {
                shift_type == shift.shift_type.index()
                    || (allow_placeholder_roster_coverage && shift_type == OFF_SHIFT_VALUE)
            })
            && assignment
                .allowed_shift_types
                .contains(&shift.shift_type.index())
    })
}

fn task_operator_value_order_key(
    plan: &Plan,
    assignment: &FurnaceAssignment,
    kind: ManualTaskKind,
    operator_id: usize,
) -> i64 {
    match kind {
        ManualTaskKind::LoadBuild => {
            load_build_operator_construction_value_order_key(plan, assignment, operator_id)
        }
        ManualTaskKind::Program => {
            program_operator_construction_value_order_key(plan, assignment, operator_id)
        }
        ManualTaskKind::Quench => {
            quench_operator_construction_value_order_key(plan, assignment, operator_id)
        }
        ManualTaskKind::Unload | ManualTaskKind::HeavyUnload => {
            unload_operator_construction_value_order_key(plan, assignment, operator_id)
        }
    }
}

fn total_candidate_limit(limits: ScalarGroupLimits) -> usize {
    limits
        .group_candidate_limit
        .or(limits.max_moves_per_step)
        .unwrap_or(usize::MAX)
}

fn is_construction_pass(limits: ScalarGroupLimits) -> bool {
    limits.max_moves_per_step.is_none()
}

fn task_variable_name(kind: ManualTaskKind) -> &'static str {
    match kind {
        ManualTaskKind::LoadBuild => "load_build_operator_id",
        ManualTaskKind::Program => "program_operator_id",
        ManualTaskKind::Quench => "quench_operator_id",
        ManualTaskKind::Unload | ManualTaskKind::HeavyUnload => "unload_operator_id",
    }
}
