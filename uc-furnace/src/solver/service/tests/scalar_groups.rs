use solverforge::{ConstraintSet, ScalarCandidate, ScalarEdit, ScalarGroupLimits};

use crate::constraints::create_constraints;
use crate::data::build_demo_problem;
use crate::domain::{furnace_assignment_local_search_value_order_key, Plan, OFF_SHIFT_VALUE};
use crate::solver::conflict_repair::assignment_value_is_hard_clean;
use crate::solver::scalar_groups;

type ScalarGroupProvider =
    fn(&crate::domain::Plan, ScalarGroupLimits) -> Vec<ScalarCandidate<Plan>>;

fn limits(
    value_candidate_limit: Option<usize>,
    group_candidate_limit: Option<usize>,
) -> ScalarGroupLimits {
    ScalarGroupLimits {
        value_candidate_limit,
        group_candidate_limit,
        max_moves_per_step: None,
        ..ScalarGroupLimits::default()
    }
}

#[test]
fn pristine_demo_returns_furnace_schedule_candidates() {
    let plan = build_demo_problem();
    let candidates = scalar_groups::furnace_assignment_candidates(&plan, limits(Some(2), Some(8)));

    assert!(!candidates.is_empty());
    assert!(candidates.iter().all(|candidate| {
        candidate.reason() == "furnace_assignment"
            && candidate.edits().len() == 1
            && candidate.edits()[0].variable_name() == "assignment"
            && candidate.construction_slot_key().is_some()
            && candidate.construction_entity_order_key().is_some()
            && candidate.construction_value_order_key().is_some()
    }));
}

#[test]
fn furnace_provider_advances_past_scheduled_slots_under_total_cap() {
    let mut plan = build_demo_problem();
    let first = scalar_groups::furnace_assignment_candidates(&plan, limits(None, Some(16)));
    let completed_slots = first
        .iter()
        .filter_map(|candidate| candidate.construction_slot_key())
        .collect::<std::collections::HashSet<_>>();
    assert!(!completed_slots.is_empty());

    for slot in &completed_slots {
        let value = plan.assignments[*slot].compatible_assignments[0];
        plan.assignments[*slot].assignment = Some(value);
    }

    let second = scalar_groups::furnace_assignment_candidates(&plan, limits(None, Some(16)));

    assert!(!second.is_empty());
    assert!(second.iter().all(|candidate| {
        candidate
            .construction_slot_key()
            .is_some_and(|slot| !completed_slots.contains(&slot))
    }));
}

#[test]
fn furnace_provider_emits_hard_clean_construction_values() {
    let mut plan = build_demo_problem();
    let first = scalar_groups::furnace_assignment_candidates(&plan, limits(None, Some(24)));
    apply_candidate(&mut plan, first.first().expect("first furnace candidate"));

    let candidates = scalar_groups::furnace_assignment_candidates(&plan, limits(None, Some(64)));

    assert!(!candidates.is_empty());
    for candidate in candidates {
        let mut candidate_plan = plan.clone();
        apply_candidate(&mut candidate_plan, &candidate);
        assert_eq!(hard_score(&candidate_plan, "furnaceOverlap"), 0);
        assert_eq!(hard_score(&candidate_plan, "changeoverGap"), 0);
    }
}

#[test]
fn roster_provider_respects_value_candidate_limit() {
    let plan = build_demo_problem();
    let candidates =
        scalar_groups::roster_shift_assignment_candidates(&plan, limits(Some(1), Some(64)));

    assert!(!candidates.is_empty());
    assert!(candidates.iter().all(|candidate| {
        candidate.reason() == "roster_shift_assignment"
            && candidate.edits().len() == 1
            && candidate.edits()[0].variable_name() == "shift_type"
            && candidate.construction_slot_key().is_some()
            && candidate.construction_entity_order_key().is_some()
            && candidate.construction_value_order_key().is_some()
    }));
    assert_at_most_one_candidate_per_slot(&candidates);
}

#[test]
fn task_operator_provider_waits_for_scheduled_furnace_assignment() {
    let mut plan = build_demo_problem();
    let task_provider: ScalarGroupProvider = scalar_groups::task_operator_assignment_candidates;

    assert!(task_provider(&plan, limits(None, Some(16))).is_empty());

    let scheduled_index = plan
        .assignments
        .iter()
        .position(|assignment| !assignment.compatible_assignments.is_empty())
        .expect("schedulable furnace assignment");
    let scheduled_value = plan.assignments[scheduled_index].compatible_assignments[0];
    plan.assignments[scheduled_index].assignment = Some(scheduled_value);

    let candidates = task_provider(&plan, limits(None, Some(16)));

    assert!(!candidates.is_empty());
    assert!(candidates.iter().all(|candidate| {
        candidate.reason() == "task_operator_assignment"
            && !candidate.edits().is_empty()
            && candidate
                .edits()
                .iter()
                .all(|edit| edit.variable_name().ends_with("_operator_id"))
            && candidate.construction_slot_key().is_some()
            && candidate.construction_entity_order_key().is_some()
            && candidate.construction_value_order_key().is_some()
    }));
}

#[test]
fn task_operator_provider_treats_off_roster_slots_as_coverable_during_construction() {
    let mut plan = build_demo_problem();
    for assignment in &mut plan.operator_shift_assignments {
        assignment.shift_type = Some(OFF_SHIFT_VALUE);
    }
    let scheduled_index = plan
        .assignments
        .iter()
        .position(|assignment| !assignment.compatible_assignments.is_empty())
        .expect("schedulable furnace assignment");
    let scheduled_value = plan.assignments[scheduled_index].compatible_assignments[0];
    plan.assignments[scheduled_index].assignment = Some(scheduled_value);

    let candidates =
        scalar_groups::task_operator_assignment_candidates(&plan, limits(None, Some(16)));

    assert!(!candidates.is_empty());
}

#[test]
fn task_operator_provider_requires_real_roster_coverage_during_local_search() {
    let mut plan = build_demo_problem();
    for assignment in &mut plan.operator_shift_assignments {
        assignment.shift_type = Some(OFF_SHIFT_VALUE);
    }
    let scheduled_index = plan
        .assignments
        .iter()
        .position(|assignment| !assignment.compatible_assignments.is_empty())
        .expect("schedulable furnace assignment");
    let scheduled_value = plan.assignments[scheduled_index].compatible_assignments[0];
    plan.assignments[scheduled_index].assignment = Some(scheduled_value);

    let candidates = scalar_groups::task_operator_assignment_candidates(
        &plan,
        ScalarGroupLimits {
            value_candidate_limit: None,
            group_candidate_limit: None,
            max_moves_per_step: Some(16),
            ..ScalarGroupLimits::default()
        },
    );

    assert!(candidates.is_empty());
}

#[test]
fn provider_total_caps_are_enforced() {
    let plan = build_demo_problem();

    let candidates = scalar_groups::furnace_assignment_candidates(&plan, limits(None, Some(3)));

    assert_eq!(candidates.len(), 3);
}

#[test]
fn schedule_task_provider_is_local_search_only() {
    let plan = build_demo_problem();

    let candidates =
        scalar_groups::furnace_schedule_task_assignment_candidates(&plan, limits(None, Some(16)));

    assert!(candidates.is_empty());
}

#[test]
fn schedule_task_provider_emits_atomic_schedule_and_task_candidate() {
    let mut plan = build_demo_problem();
    let assignment_index = 0;
    let current_value = plan.assignments[assignment_index].compatible_assignments[0];
    plan.assignments[assignment_index].assignment = Some(current_value);
    let next_value = plan.assignments[assignment_index]
        .compatible_assignments
        .iter()
        .copied()
        .filter(|value| *value != current_value)
        .filter(|value| assignment_value_is_hard_clean(&plan, assignment_index, *value))
        .min_by_key(|value| {
            furnace_assignment_local_search_value_order_key(
                &plan,
                &plan.assignments[assignment_index],
                *value,
            )
        })
        .expect("alternate hard-clean assignment value");
    cover_task_shifts_for_value(&mut plan, assignment_index, next_value);
    plan.assignments[assignment_index].assignment = Some(current_value);

    let candidates = scalar_groups::furnace_schedule_task_assignment_candidates(
        &plan,
        ScalarGroupLimits {
            value_candidate_limit: Some(1),
            group_candidate_limit: None,
            max_moves_per_step: Some(32),
            ..ScalarGroupLimits::default()
        },
    );
    let candidate = candidates
        .iter()
        .find(|candidate| {
            candidate.construction_slot_key() == Some(assignment_index)
                && candidate.edits().iter().any(|edit| {
                    edit.variable_name() == "assignment" && edit.to_value() == Some(next_value)
                })
        })
        .expect("schedule+task candidate for covered assignment");

    assert!(candidate
        .edits()
        .iter()
        .any(|edit| edit.variable_name() == "assignment"));
    assert!(
        candidate
            .edits()
            .iter()
            .any(|edit| edit.variable_name().ends_with("_operator_id")),
        "candidate should repair task operators atomically"
    );
}

fn assert_at_most_one_candidate_per_slot(candidates: &[ScalarCandidate<Plan>]) {
    let mut seen = std::collections::HashSet::new();
    for candidate in candidates {
        assert!(
            seen.insert(candidate.construction_slot_key()),
            "duplicate slot candidate: {candidate:?}"
        );
    }
}

fn apply_candidate(plan: &mut Plan, candidate: &ScalarCandidate<Plan>) {
    for edit in candidate.edits() {
        apply_edit(plan, edit);
    }
}

fn apply_edit(plan: &mut Plan, edit: &ScalarEdit<Plan>) {
    match edit.variable_name() {
        "assignment" => plan.assignments[edit.entity_index()].assignment = edit.to_value(),
        "shift_type" => {
            plan.operator_shift_assignments[edit.entity_index()].shift_type = edit.to_value()
        }
        "load_build_operator_id" => {
            plan.assignments[edit.entity_index()].load_build_operator_id = edit.to_value();
        }
        "program_operator_id" => {
            plan.assignments[edit.entity_index()].program_operator_id = edit.to_value();
        }
        "quench_operator_id" => {
            plan.assignments[edit.entity_index()].quench_operator_id = edit.to_value();
        }
        "unload_operator_id" => {
            plan.assignments[edit.entity_index()].unload_operator_id = edit.to_value();
        }
        variable_name => panic!("unsupported test edit variable {variable_name}"),
    }
}

fn cover_task_shifts_for_value(plan: &mut Plan, assignment_index: usize, value: usize) {
    plan.assignments[assignment_index].assignment = Some(value);
    for kind in plan.assignments[assignment_index].required_task_kinds() {
        let shift = plan.assignments[assignment_index]
            .task_shift_identity(kind)
            .expect("task shift");
        let operator_id = *plan.assignments[assignment_index]
            .task_eligible_operator_ids(kind)
            .first()
            .expect("eligible operator");
        let roster = plan
            .operator_shift_assignments
            .iter_mut()
            .find(|assignment| {
                assignment.operator_idx == operator_id
                    && assignment.roster_day == shift.roster_day
                    && assignment
                        .allowed_shift_types
                        .contains(&shift.shift_type.index())
            })
            .expect("operator roster slot for task shift");
        roster.shift_type = Some(shift.shift_type.index());
    }
}

fn hard_score(plan: &Plan, constraint_name: &str) -> i64 {
    create_constraints()
        .evaluate_each(plan)
        .into_iter()
        .find(|result| result.name == constraint_name)
        .expect("constraint result")
        .score
        .hard()
}
