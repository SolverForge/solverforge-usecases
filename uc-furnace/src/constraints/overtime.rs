use super::shared::*;

/// Penalizes operators who exceed the target number of visible weekly shifts.
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
        .penalize(|_operator_idx: &usize, visible_shifts: &usize| {
            HardSoftScore::of_soft(
                visible_shifts.saturating_sub(TARGET_VISIBLE_SHIFTS_PER_WEEK) as i64 * 240,
            )
        })
        .named("overtime")
}
