use super::shared::*;

/// Penalizes urgent work that finishes after its due time.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .filter(|assignment: &FurnaceAssignment| assignment.priority == PriorityBand::Urgent)
        .filter(|assignment: &FurnaceAssignment| {
            assignment
                .end_minutes()
                .is_some_and(|end| end > assignment.due_datetime_minutes)
        })
        .penalize(|assignment: &FurnaceAssignment| {
            priority_lateness_score(
                assignment.end_minutes().unwrap(),
                assignment.due_datetime_minutes,
                20,
            )
            .unwrap()
        })
        .named("urgent_lateness")
}
