use super::shared::*;

/// Requires quench work to be owned by the shift covering that task interval.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .assignments()
        .filter(|assignment: &FurnaceAssignment| assignment.assignment.is_some())
        .filter(|assignment: &FurnaceAssignment| {
            assignment.quench_interval().is_some() && assignment.quench_shift_identity().is_none()
        })
        .penalize(hard_weight(|_: &FurnaceAssignment| {
            HardSoftScore::of_hard(900)
        }))
        .named("quench_shift_ownership")
}
