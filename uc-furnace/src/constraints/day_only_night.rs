use super::shared::*;

/// Prevents day-only operators from being assigned to night shifts.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .operator_shift_assignments()
        .filter(|assignment: &OperatorShiftAssignment| {
            assignment.day_only && assignment.shift_type_enum() == Some(ShiftType::Night)
        })
        .penalize(hard_weight(|_: &OperatorShiftAssignment| {
            HardSoftScore::of_hard(1_200)
        }))
        .named("day_only_night")
}
