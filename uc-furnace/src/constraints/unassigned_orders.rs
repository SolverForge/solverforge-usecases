use super::shared::*;

/// Penalizes work orders that have not been assigned to a furnace and start time.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .filter(|assignment: &FurnaceAssignment| assignment.assignment.is_none())
        .penalize(hard_weight(|_: &FurnaceAssignment| {
            HardSoftScore::of_hard(30_000)
        }))
        .named("unassigned_orders")
}
