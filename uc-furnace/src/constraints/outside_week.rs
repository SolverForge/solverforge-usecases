use super::shared::*;

/// Penalizes scheduled work that finishes beyond the weekly planning horizon.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .filter(|assignment: &FurnaceAssignment| {
            assignment
                .end_minutes()
                .is_some_and(|end| end > HORIZON_MINUTES)
        })
        .penalize(hard_weight(|assignment: &FurnaceAssignment| {
            HardSoftScore::of_hard((assignment.end_minutes().unwrap() - HORIZON_MINUTES) as i64 * 5)
        }))
        .named("outside_week")
}
