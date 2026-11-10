use super::shared::*;

/// Penalizes uneven distribution of working shifts across operators.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .operator_shift_assignments()
        .filter(|assignment: &OperatorShiftAssignment| {
            assignment.roster_day >= 0 && assignment.is_working()
        })
        .balance(|assignment: &OperatorShiftAssignment| Some(assignment.operator_idx))
        .penalize(HardSoftScore::of_soft(120))
        .named("shift_load_balance")
}
