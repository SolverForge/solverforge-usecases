use super::shared::*;

/// Penalizes operators who exceed the target number of visible weekly shifts.
pub(super) fn overtime() -> impl IncrementalConstraint<Plan, HardSoftScore> {
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

/// Penalizes uneven distribution of working shifts across operators.
pub(super) fn shift_load_balance() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .operator_shift_assignments()
        .filter(|assignment: &OperatorShiftAssignment| {
            assignment.roster_day >= 0 && assignment.is_working()
        })
        .balance(|assignment: &OperatorShiftAssignment| Some(assignment.operator_idx))
        .penalize(HardSoftScore::of_soft(120))
        .named("shiftLoadBalance")
}
