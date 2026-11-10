use super::shared::*;

/// Penalizes completing work earlier than necessary to reduce early-start pressure.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .filter(|assignment: &FurnaceAssignment| {
            assignment
                .end_minutes()
                .is_some_and(|end| end < assignment.due_datetime_minutes)
        })
        .penalize(|assignment: &FurnaceAssignment| {
            early_start_pressure_score(
                assignment.end_minutes().unwrap(),
                assignment.due_datetime_minutes,
            )
            .unwrap()
        })
        .named("early_start_pressure")
}
