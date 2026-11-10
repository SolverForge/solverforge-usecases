use crate::constraints::create_constraints;
use crate::data::build_demo_plan;
use crate::domain::{Plan, ShiftType};
use solverforge::ConstraintSet;

fn hard_score(plan: &Plan, constraint_name: &str) -> i64 {
    create_constraints()
        .evaluate_each(plan)
        .into_iter()
        .find(|result| result.name == constraint_name)
        .expect("constraint result")
        .score
        .hard()
}

#[test]
fn adjacent_night_run_within_policy_is_allowed() {
    let mut plan = build_demo_plan();
    for day in 1..=3 {
        let assignment = plan
            .operator_shift_assignments
            .iter_mut()
            .find(|assignment| assignment.operator_idx == 0 && assignment.roster_day == day)
            .expect("operator day assignment");
        assignment.shift_type = Some(ShiftType::Night.index());
    }

    assert_eq!(hard_score(&plan, "consecutive_nights"), 0);
}

#[test]
fn night_count_beyond_policy_is_hard_penalized() {
    let mut plan = build_demo_plan();
    for day in 1..=4 {
        let assignment = plan
            .operator_shift_assignments
            .iter_mut()
            .find(|assignment| assignment.operator_idx == 0 && assignment.roster_day == day)
            .expect("operator day assignment");
        assignment.shift_type = Some(ShiftType::Night.index());
    }

    assert!(hard_score(&plan, "consecutive_nights") < 0);
}
