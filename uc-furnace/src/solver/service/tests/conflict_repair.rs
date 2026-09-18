use solverforge::{ConstraintSet, RepairCandidate, RepairLimits, ScalarEdit};

use crate::constraints::create_constraints;
use crate::data::build_demo_problem;
use crate::domain::{OperatorRole, Plan, ShiftType, OFF_SHIFT_VALUE};
use crate::solver::conflict_repair;

#[test]
fn visible_shift_repair_keeps_hard_improving_partial_roster_moves() {
    let mut plan = build_demo_problem();
    for assignment in &mut plan.operator_shift_assignments {
        assignment.shift_type = Some(OFF_SHIFT_VALUE);
    }

    let operator = operator_with_all_visible_morning_days(&plan);
    for day in 0..=6 {
        let assignment = plan
            .operator_shift_assignments
            .iter_mut()
            .find(|assignment| assignment.operator_idx == operator && assignment.roster_day == day)
            .expect("visible roster assignment");
        assignment.shift_type = Some(ShiftType::Morning.index());
    }

    let role = plan
        .operator_shift_assignments
        .iter()
        .find(|assignment| assignment.operator_idx == operator)
        .expect("operator role")
        .role;
    mark_replacement_pool_rest_dirty(&mut plan, role, operator);

    let baseline_hard = hard_score(&plan);
    let repairs = conflict_repair::providers()
        .into_iter()
        .find(|entry| entry.constraint_name() == "visibleShiftLimit")
        .expect("visible shift repair provider")
        .provider();

    let candidates = repairs(&plan, limits());

    assert!(
        candidates.iter().any(|candidate| {
            candidate.reason() == "visibleShiftLimit"
                && candidate_hard_score(&plan, candidate) > baseline_hard
        }),
        "visible shift repair should leave hard-improvement filtering to scoring"
    );
}

fn operator_with_all_visible_morning_days(plan: &Plan) -> usize {
    plan.operator_shift_assignments
        .iter()
        .filter(|assignment| assignment.roster_day == 0)
        .find(|assignment| {
            (0..=6).all(|day| {
                plan.operator_shift_assignments.iter().any(|candidate| {
                    candidate.operator_idx == assignment.operator_idx
                        && candidate.roster_day == day
                        && candidate
                            .allowed_shift_types
                            .contains(&ShiftType::Morning.index())
                })
            })
        })
        .expect("operator with visible morning coverage")
        .operator_idx
}

fn mark_replacement_pool_rest_dirty(plan: &mut Plan, role: OperatorRole, excluded_operator: usize) {
    let replacement_operators = plan
        .operator_shift_assignments
        .iter()
        .filter(|assignment| {
            assignment.role == role
                && assignment.operator_idx != excluded_operator
                && assignment.roster_day == 6
                && assignment
                    .allowed_shift_types
                    .contains(&ShiftType::Morning.index())
        })
        .map(|assignment| assignment.operator_idx)
        .collect::<std::collections::HashSet<_>>();

    assert!(
        !replacement_operators.is_empty(),
        "demo data should expose same-role replacement operators"
    );

    for operator in replacement_operators {
        set_operator_day(plan, operator, 0, ShiftType::Afternoon);
        set_operator_day(plan, operator, 1, ShiftType::Morning);
    }
}

fn set_operator_day(plan: &mut Plan, operator: usize, day: i32, shift_type: ShiftType) {
    let assignment = plan
        .operator_shift_assignments
        .iter_mut()
        .find(|assignment| assignment.operator_idx == operator && assignment.roster_day == day)
        .expect("operator day assignment");
    assert!(
        assignment.allowed_shift_types.contains(&shift_type.index()),
        "test selected an unsupported shift type"
    );
    assignment.shift_type = Some(shift_type.index());
}

fn candidate_hard_score(plan: &Plan, candidate: &RepairCandidate<Plan>) -> i64 {
    let mut candidate_plan = plan.clone();
    for edit in candidate.edits() {
        apply_edit(&mut candidate_plan, edit);
    }
    hard_score(&candidate_plan)
}

fn apply_edit(plan: &mut Plan, edit: &ScalarEdit<Plan>) {
    match edit.variable_name() {
        "shift_type" => {
            plan.operator_shift_assignments[edit.entity_index()].shift_type = edit.to_value();
        }
        "assignment" => {
            plan.assignments[edit.entity_index()].assignment = edit.to_value();
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
        _ => panic!("unsupported conflict repair edit"),
    }
}

fn hard_score(plan: &Plan) -> i64 {
    create_constraints()
        .evaluate_each(plan)
        .into_iter()
        .map(|result| result.score.hard())
        .sum()
}

fn limits() -> RepairLimits {
    RepairLimits {
        max_matches_per_step: 16,
        max_repairs_per_match: 32,
        max_moves_per_step: 256,
    }
}
