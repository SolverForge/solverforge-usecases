use super::shared::*;

/// Caps the number of visible roster shifts an operator may work in the week.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
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
        .named("visible_shift_limit")
}
