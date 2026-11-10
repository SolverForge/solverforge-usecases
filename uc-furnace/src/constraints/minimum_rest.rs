use super::shared::*;

/// Requires enough rest time between two shifts worked by the same operator.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
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
        .named("minimum_rest")
}
