use super::shared::*;

/// Prevents day-only operators from being assigned to night shifts.
pub(super) fn day_only_night() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .operator_shift_assignments()
        .filter(|assignment: &OperatorShiftAssignment| {
            assignment.day_only && assignment.shift_type_enum() == Some(ShiftType::Night)
        })
        .penalize(hard_weight(|_: &OperatorShiftAssignment| {
            HardSoftScore::of_hard(1_200)
        }))
        .named("dayOnlyNight")
}

/// Requires enough rest time between two shifts worked by the same operator.
pub(super) fn minimum_rest() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .operator_shift_assignments()
        .join(equal(|assignment: &OperatorShiftAssignment| {
            assignment.operator_idx
        }))
        .filter(
            |previous: &OperatorShiftAssignment, current: &OperatorShiftAssignment| {
                has_insufficient_rest(previous, current)
            },
        )
        .penalize(hard_weight(
            |previous: &OperatorShiftAssignment, current: &OperatorShiftAssignment| {
                HardSoftScore::of_hard(
                    (MINIMUM_REST_MINUTES - shift_rest_minutes(previous, current).unwrap()) as i64,
                )
            },
        ))
        .named("minimumRest")
}

/// Caps the number of visible roster shifts an operator may work in the week.
pub(super) fn visible_shift_limit() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .operator_shift_assignments()
        .filter(|assignment: &OperatorShiftAssignment| {
            assignment.roster_day >= 0 && assignment.is_working()
        })
        .group_by(
            |assignment: &OperatorShiftAssignment| assignment.operator_idx,
            count(),
        )
        .penalize(hard_weight(
            |_operator_idx: &usize, visible_shifts: &usize| {
                HardSoftScore::of_hard(
                    visible_shifts.saturating_sub(MAX_VISIBLE_SHIFTS_PER_WEEK) as i64 * 900,
                )
            },
        ))
        .named("visibleShiftLimit")
}

/// Caps consecutive night-shift windows for each operator.
pub(super) fn consecutive_nights() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .operator_shift_assignments()
        .project(OperatorNightWindowEntries)
        .group_by(
            |entry: &NightWindowEntry| (entry.operator_id, entry.window_start_day),
            sum(|entry: &NightWindowEntry| entry.delta),
        )
        .penalize(hard_weight(|_window: &(usize, i32), night_shifts: &i64| {
            HardSoftScore::of_hard((*night_shifts - MAX_CONSECUTIVE_NIGHTS as i64).max(0) * 900)
        }))
        .named("consecutiveNights")
}

#[cfg(test)]
mod tests {
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

        assert_eq!(hard_score(&plan, "consecutiveNights"), 0);
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

        assert!(hard_score(&plan, "consecutiveNights") < 0);
    }
}
